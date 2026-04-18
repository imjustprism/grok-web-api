use bytes::Bytes;
use futures::{Stream, StreamExt};
use memchr::memchr;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::{GrokError, Result};
use crate::types::common::ConversationId;

const MAX_LINE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum StreamChunk {
    ConversationCreated {
        conversation_id: ConversationId,
    },

    Token {
        text: String,
        is_soft_stop: bool,
    },

    ThinkingToken {
        text: String,
    },

    WebSearch {
        query: Option<String>,
        results: Vec<WebSearchResult>,
        raw: serde_json::Value,
    },
    ImageGenerated {
        url: Option<String>,
        raw: serde_json::Value,
    },

    Error {
        message: String,
    },

    Done,
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebSearchResult {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

pin_project! {
    pub struct GrokStream<S> {
        #[pin]
        inner: S,
        buffer: Vec<u8>,
        done: bool,
    }
}

impl<S> GrokStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, wreq::Error>>,
{
    #[must_use]
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: Vec::with_capacity(4096),
            done: false,
        }
    }
}

impl<S> GrokStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, wreq::Error>> + Unpin,
{
    pub async fn collect_text(&mut self) -> Result<String> {
        let mut text = String::new();
        while let Some(chunk) = self.next().await {
            match chunk? {
                StreamChunk::Token { text: t, .. } => text.push_str(&t),
                StreamChunk::Done => break,
                StreamChunk::Error { message } => {
                    return Err(GrokError::StreamParse(message));
                }
                _ => {}
            }
        }
        Ok(text)
    }

    pub async fn collect_full(&mut self) -> Result<CollectedResponse> {
        let mut response = CollectedResponse::default();
        while let Some(chunk) = self.next().await {
            match chunk? {
                StreamChunk::ConversationCreated { conversation_id } => {
                    response.conversation_id = Some(conversation_id);
                }
                StreamChunk::Token { text, .. } => response.text.push_str(&text),
                StreamChunk::ThinkingToken { text } => response.thinking.push_str(&text),
                StreamChunk::Done => break,
                StreamChunk::Error { message } => {
                    return Err(GrokError::StreamParse(message));
                }
                _ => {}
            }
        }
        Ok(response)
    }
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct CollectedResponse {
    pub conversation_id: Option<ConversationId>,
    pub text: String,
    pub thinking: String,
}

impl<S> Stream for GrokStream<S>
where
    S: Stream<Item = std::result::Result<Bytes, wreq::Error>>,
{
    type Item = Result<StreamChunk>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if *this.done {
            return Poll::Ready(None);
        }

        loop {
            if let Some(newline_pos) = memchr(b'\n', this.buffer) {
                let line_end = newline_pos + 1;
                let line_bytes = &this.buffer[..newline_pos];

                let result = match std::str::from_utf8(line_bytes) {
                    Ok(s) => {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            this.buffer.drain(..line_end);
                            continue;
                        }
                        parse_ndjson_line(trimmed)
                    }
                    Err(e) => Err(GrokError::StreamParse(format!(
                        "invalid UTF-8 in NDJSON line: {e}"
                    ))),
                };
                this.buffer.drain(..line_end);

                match result {
                    Ok(Some(chunk)) => return Poll::Ready(Some(Ok(chunk))),
                    Ok(None) => continue,
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }

            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    if this.buffer.len().saturating_add(bytes.len()) > MAX_LINE_BYTES {
                        *this.done = true;
                        return Poll::Ready(Some(Err(GrokError::StreamParse(format!(
                            "NDJSON line exceeds {MAX_LINE_BYTES} bytes without newline"
                        )))));
                    }
                    this.buffer.extend_from_slice(&bytes);
                }
                Poll::Ready(Some(Err(e))) => {
                    *this.done = true;
                    return Poll::Ready(Some(Err(GrokError::Request(e))));
                }
                Poll::Ready(None) => {
                    *this.done = true;
                    if this.buffer.is_empty() {
                        return Poll::Ready(None);
                    }
                    let parsed = std::str::from_utf8(this.buffer)
                        .map_err(|e| {
                            GrokError::StreamParse(format!("invalid UTF-8 at end of stream: {e}"))
                        })
                        .and_then(|s| {
                            let trimmed = s.trim();
                            if trimmed.is_empty() {
                                Ok(None)
                            } else {
                                parse_ndjson_line(trimmed)
                            }
                        });
                    this.buffer.clear();
                    return match parsed {
                        Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
                        Ok(None) => Poll::Ready(None),
                        Err(e) => Poll::Ready(Some(Err(e))),
                    };
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLine {
    #[serde(default)]
    result: Option<RawResult>,
    #[serde(default)]
    error: Option<RawError>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResult {
    #[serde(default)]
    conversation: Option<RawConversation>,
    #[serde(default)]
    response: Option<RawResponse>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    is_thinking: bool,
    #[serde(default)]
    is_soft_stop: bool,
    #[serde(default)]
    web_search_results: Option<Vec<WebSearchResult>>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    generated_image_url: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawConversation {
    #[serde(default)]
    conversation_id: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResponse {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    is_thinking: bool,
    #[serde(default)]
    is_soft_stop: bool,
    #[serde(default)]
    web_search_results: Option<Vec<WebSearchResult>>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    generated_image_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawError {
    #[serde(default)]
    message: Option<String>,
}

#[inline]
fn parse_ndjson_line(line: &str) -> Result<Option<StreamChunk>> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| GrokError::StreamParse(format!("failed to parse NDJSON: {e}")))?;
    let raw: RawLine = serde_json::from_value(value.clone())
        .map_err(|e| GrokError::StreamParse(format!("unexpected NDJSON shape: {e}")))?;

    if let Some(err) = raw.error {
        if let Some(msg) = err.message {
            return Ok(Some(StreamChunk::Error { message: msg }));
        }
    }

    let Some(result) = raw.result else {
        return Ok(None);
    };

    if let Some(conv) = &result.conversation {
        if let Some(id) = &conv.conversation_id {
            return Ok(Some(StreamChunk::ConversationCreated {
                conversation_id: ConversationId::new(id),
            }));
        }
    }

    let (token, is_thinking, is_soft_stop, search, query, image) =
        if let Some(ref resp) = result.response {
            (
                resp.token.as_deref(),
                resp.is_thinking,
                resp.is_soft_stop,
                resp.web_search_results.as_ref(),
                resp.query.as_ref(),
                resp.generated_image_url.as_ref(),
            )
        } else {
            (
                result.token.as_deref(),
                result.is_thinking,
                result.is_soft_stop,
                result.web_search_results.as_ref(),
                result.query.as_ref(),
                result.generated_image_url.as_ref(),
            )
        };

    if let Some(tok) = token {
        if is_soft_stop && tok.is_empty() {
            return Ok(Some(StreamChunk::Done));
        }
        return if is_thinking {
            Ok(Some(StreamChunk::ThinkingToken {
                text: tok.to_owned(),
            }))
        } else {
            Ok(Some(StreamChunk::Token {
                text: tok.to_owned(),
                is_soft_stop,
            }))
        };
    }

    if let Some(results) = search {
        return Ok(Some(StreamChunk::WebSearch {
            query: query.cloned(),
            results: results.clone(),
            raw: value,
        }));
    }

    if let Some(url) = image {
        return Ok(Some(StreamChunk::ImageGenerated {
            url: Some(url.clone()),
            raw: value,
        }));
    }

    Ok(Some(StreamChunk::Unknown(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::stream;

    fn parse(line: &str) -> Result<Option<StreamChunk>> {
        parse_ndjson_line(line)
    }

    #[test]
    fn parses_token_chunk() {
        let chunk = parse(r#"{"result":{"token":"hi","isThinking":false,"isSoftStop":false}}"#)
            .unwrap()
            .unwrap();
        match chunk {
            StreamChunk::Token { text, is_soft_stop } => {
                assert_eq!(text, "hi");
                assert!(!is_soft_stop);
            }
            _ => panic!("expected Token, got {chunk:?}"),
        }
    }

    #[test]
    fn parses_thinking_token() {
        let chunk = parse(r#"{"result":{"token":"th","isThinking":true}}"#)
            .unwrap()
            .unwrap();
        assert!(matches!(chunk, StreamChunk::ThinkingToken { .. }));
    }

    #[test]
    fn empty_soft_stop_token_is_done() {
        let chunk = parse(r#"{"result":{"token":"","isSoftStop":true}}"#)
            .unwrap()
            .unwrap();
        assert!(matches!(chunk, StreamChunk::Done));
    }

    #[test]
    fn nested_response_token_extracted() {
        let chunk = parse(r#"{"result":{"response":{"token":"hi"}}}"#)
            .unwrap()
            .unwrap();
        assert!(matches!(chunk, StreamChunk::Token { .. }));
    }

    #[test]
    fn conversation_id_extracted() {
        let chunk = parse(r#"{"result":{"conversation":{"conversationId":"abc-123"}}}"#)
            .unwrap()
            .unwrap();
        match chunk {
            StreamChunk::ConversationCreated { conversation_id } => {
                assert_eq!(conversation_id.as_ref(), "abc-123");
            }
            _ => panic!("expected ConversationCreated"),
        }
    }

    #[test]
    fn error_propagated() {
        let chunk = parse(r#"{"error":{"message":"boom"}}"#).unwrap().unwrap();
        match chunk {
            StreamChunk::Error { message } => assert_eq!(message, "boom"),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn invalid_json_returns_err() {
        assert!(parse("not json").is_err());
    }

    #[test]
    fn empty_result_returns_none() {
        let result = parse(r#"{}"#).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn stream_handles_split_chunks() {
        let parts = vec![
            Ok::<_, wreq::Error>(Bytes::from_static(b"{\"result\":{\"token\":\"")),
            Ok(Bytes::from_static(
                b"hello\"}}\n{\"result\":{\"token\":\"world\"}}\n",
            )),
        ];
        let mut s = GrokStream::new(stream::iter(parts));
        let mut tokens = Vec::new();
        while let Some(item) = s.next().await {
            if let StreamChunk::Token { text, .. } = item.unwrap() {
                tokens.push(text);
            }
        }
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[tokio::test]
    async fn stream_handles_blank_lines() {
        let parts = vec![Ok::<_, wreq::Error>(Bytes::from_static(
            b"\n\n{\"result\":{\"token\":\"x\"}}\n\n",
        ))];
        let mut s = GrokStream::new(stream::iter(parts));
        let mut count = 0;
        while let Some(item) = s.next().await {
            if let StreamChunk::Token { .. } = item.unwrap() {
                count += 1;
            }
        }
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn stream_handles_trailing_no_newline() {
        let parts = vec![Ok::<_, wreq::Error>(Bytes::from_static(
            b"{\"result\":{\"token\":\"final\"}}",
        ))];
        let mut s = GrokStream::new(stream::iter(parts));
        let mut got = None;
        while let Some(item) = s.next().await {
            if let StreamChunk::Token { text, .. } = item.unwrap() {
                got = Some(text);
            }
        }
        assert_eq!(got.as_deref(), Some("final"));
    }

    #[tokio::test]
    async fn stream_rejects_oversized_line() {
        let big = vec![b'x'; MAX_LINE_BYTES + 1];
        let parts = vec![Ok::<_, wreq::Error>(Bytes::from(big))];
        let mut s = GrokStream::new(stream::iter(parts));
        let item = s.next().await.expect("expected one item");
        assert!(matches!(item, Err(GrokError::StreamParse(_))));
    }

    #[tokio::test]
    async fn stream_handles_utf8_split_across_chunks() {
        let emoji = "😀".as_bytes();
        assert_eq!(emoji.len(), 4);
        let mut prefix = b"{\"result\":{\"token\":\"".to_vec();
        prefix.extend_from_slice(&emoji[..2]);
        let mut suffix = emoji[2..].to_vec();
        suffix.extend_from_slice(b"\"}}\n");

        let parts = vec![
            Ok::<_, wreq::Error>(Bytes::from(prefix)),
            Ok(Bytes::from(suffix)),
        ];
        let mut s = GrokStream::new(stream::iter(parts));
        let mut got = None;
        while let Some(item) = s.next().await {
            if let StreamChunk::Token { text, .. } = item.unwrap() {
                got = Some(text);
            }
        }
        assert_eq!(got.as_deref(), Some("😀"));
    }
}

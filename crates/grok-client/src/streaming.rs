use bytes::Bytes;
use futures::{Stream, StreamExt};
use memchr::memchr;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::{GrokError, Result};
use crate::types::common::ConversationId;

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
        buffer: String,
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
            buffer: String::with_capacity(4096),
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
            if let Some(newline_pos) = memchr(b'\n', this.buffer.as_bytes()) {
                let line_end = newline_pos + 1;
                let trimmed = this.buffer[..newline_pos].trim();

                if trimmed.is_empty() {
                    this.buffer.drain(..line_end);
                    continue;
                }

                let result = parse_ndjson_line(trimmed);
                this.buffer.drain(..line_end);

                match result {
                    Ok(Some(chunk)) => return Poll::Ready(Some(Ok(chunk))),
                    Ok(None) => continue,
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }

            match this.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => match std::str::from_utf8(&bytes) {
                    Ok(text) => this.buffer.push_str(text),
                    Err(e) => {
                        return Poll::Ready(Some(Err(GrokError::StreamParse(format!(
                            "invalid UTF-8 in stream: {e}"
                        )))));
                    }
                },
                Poll::Ready(Some(Err(e))) => {
                    *this.done = true;
                    return Poll::Ready(Some(Err(GrokError::Request(e))));
                }
                Poll::Ready(None) => {
                    *this.done = true;
                    let remaining = this.buffer.trim();
                    if !remaining.is_empty() {
                        let result = parse_ndjson_line(remaining);
                        this.buffer.clear();
                        match result {
                            Ok(Some(chunk)) => return Poll::Ready(Some(Ok(chunk))),
                            Ok(None) => return Poll::Ready(None),
                            Err(e) => return Poll::Ready(Some(Err(e))),
                        }
                    }

                    return Poll::Ready(None);
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
    let raw: RawLine = serde_json::from_str(line)
        .map_err(|e| GrokError::StreamParse(format!("failed to parse NDJSON: {e}")))?;

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
            raw: serde_json::from_str(line).unwrap_or_default(),
        }));
    }

    if let Some(url) = image {
        return Ok(Some(StreamChunk::ImageGenerated {
            url: Some(url.clone()),
            raw: serde_json::from_str(line).unwrap_or_default(),
        }));
    }

    Ok(Some(StreamChunk::Unknown(
        serde_json::from_str(line).unwrap_or_default(),
    )))
}

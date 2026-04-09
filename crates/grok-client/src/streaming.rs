use bytes::Bytes;
use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::{GrokError, Result};
use crate::types::common::ConversationId;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum StreamChunk {
    ConversationCreated { conversation_id: ConversationId },

    Token { text: String, is_soft_stop: bool },

    ThinkingToken { text: String },

    WebSearch(serde_json::Value),
    ImageGenerated(serde_json::Value),
    Progress { category: String, message: String },

    Error { message: String },

    Done,
    Unknown(serde_json::Value),
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
            if let Some(newline_pos) = this.buffer.find('\n') {
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

fn parse_ndjson_line(line: &str) -> Result<Option<StreamChunk>> {
    let value: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| GrokError::StreamParse(format!("failed to parse NDJSON line: {e}: {line}")))?;

    let result = &value["result"];

    if let Some(conv) = result.get("conversation") {
        if let Some(id) = conv.get("conversationId").and_then(|v| v.as_str()) {
            return Ok(Some(StreamChunk::ConversationCreated {
                conversation_id: ConversationId::new(id),
            }));
        }
    }

    let token_source = result.get("response").unwrap_or(result);

    if let Some(token) = token_source.get("token").and_then(|v| v.as_str()) {
        let is_thinking = token_source
            .get("isThinking")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_soft_stop = token_source
            .get("isSoftStop")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if is_soft_stop && token.is_empty() {
            return Ok(Some(StreamChunk::Done));
        }

        if is_thinking {
            return Ok(Some(StreamChunk::ThinkingToken {
                text: token.to_owned(),
            }));
        }

        return Ok(Some(StreamChunk::Token {
            text: token.to_owned(),
            is_soft_stop,
        }));
    }

    if let Some(error) = value.get("error") {
        if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
            return Ok(Some(StreamChunk::Error {
                message: msg.to_owned(),
            }));
        }
    }

    Ok(Some(StreamChunk::Unknown(value)))
}

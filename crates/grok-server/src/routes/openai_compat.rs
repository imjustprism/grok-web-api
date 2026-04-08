use axum::Json;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;
use grok_client::streaming::StreamChunk;
use grok_client::types::chat::NewConversationRequest;
use grok_client::types::models::ModelName;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: u64,
    pub model: String,
    pub choices: [Choice; 1],
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Serialize)]
struct ChunkWrapper<'a> {
    id: &'a str,
    object: &'static str,
    created: u64,
    model: &'a str,
    choices: [ChunkChoice<'a>; 1],
}

#[derive(Serialize)]
struct ChunkChoice<'a> {
    index: u32,
    delta: Delta<'a>,
    finish_reason: Option<&'static str>,
}

#[derive(Serialize)]
struct Delta<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
}

#[derive(Serialize)]
struct SseError<'a> {
    error: SseErrorInner<'a>,
}

#[derive(Serialize)]
struct SseErrorInner<'a> {
    message: &'a str,
    r#type: &'static str,
}

fn sse_line(json: &impl Serialize) -> bytes::Bytes {
    let mut buf = Vec::with_capacity(128);
    buf.extend_from_slice(b"data: ");
    serde_json::to_writer(&mut buf, json).unwrap_or_default();
    buf.extend_from_slice(b"\n\n");
    bytes::Bytes::from(buf)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, ApiError> {
    let user_message = request
        .messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let mut grok_req = NewConversationRequest::new(&user_message);
    grok_req.temporary = Some(true);
    grok_req.options.custom_instructions = request
        .messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.clone())
        .reduce(|mut acc, s| {
            acc.push('\n');
            acc.push_str(&s);
            acc
        });

    if let Some(ref model) = request.model {
        grok_req.options.model_name = Some(match model.as_str() {
            "grok-2" => ModelName::Grok2,
            "grok-3" => ModelName::Grok3,
            "grok-3-mini" => ModelName::Grok3Mini,
            "grok-4" => ModelName::Grok4,
            "grok-4-mini" => ModelName::Grok4Mini,
            other => ModelName::Other(other.to_owned()),
        });
    }

    let model_str = request.model.unwrap_or_else(|| "grok-3".into());

    if request.stream {
        stream_response(state, grok_req, model_str.clone()).await
    } else {
        non_stream_response(state, grok_req, model_str).await
    }
}

async fn stream_response(
    state: AppState,
    grok_req: NewConversationRequest,
    model: String,
) -> Result<Response, ApiError> {
    let grok_stream = state.client.create_conversation(&grok_req).await?;
    let completion_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = now_secs();

    let sse_stream = grok_stream.filter_map(move |chunk_result| {
        let id = completion_id.clone();
        let model = model.clone();

        async move {
            match chunk_result {
                Ok(StreamChunk::Token { text, is_soft_stop }) => {
                    if is_soft_stop && text.is_empty() {
                        return None;
                    }
                    let chunk = ChunkWrapper {
                        id: &id,
                        object: "chat.completion.chunk",
                        created,
                        model: &model,
                        choices: [ChunkChoice {
                            index: 0,
                            delta: Delta {
                                content: Some(&text),
                            },
                            finish_reason: None,
                        }],
                    };
                    Some(Ok::<_, std::io::Error>(sse_line(&chunk)))
                }
                Ok(StreamChunk::Done) => {
                    let chunk = ChunkWrapper {
                        id: &id,
                        object: "chat.completion.chunk",
                        created,
                        model: &model,
                        choices: [ChunkChoice {
                            index: 0,
                            delta: Delta { content: None },
                            finish_reason: Some("stop"),
                        }],
                    };
                    let mut buf = sse_line(&chunk).to_vec();
                    buf.extend_from_slice(b"data: [DONE]\n\n");
                    Some(Ok(bytes::Bytes::from(buf)))
                }
                Ok(StreamChunk::ThinkingToken { .. }) => None,
                Ok(StreamChunk::Error { message }) => {
                    let err = SseError {
                        error: SseErrorInner {
                            message: &message,
                            r#type: "upstream_error",
                        },
                    };
                    Some(Ok(sse_line(&err)))
                }
                Ok(_) => None,
                Err(e) => {
                    let msg = e.to_string();
                    let err = SseError {
                        error: SseErrorInner {
                            message: &msg,
                            r#type: "stream_error",
                        },
                    };
                    Some(Ok(sse_line(&err)))
                }
            }
        }
    });

    Ok((
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONNECTION, "keep-alive"),
        ],
        axum::body::Body::from_stream(sse_stream),
    )
        .into_response())
}

async fn non_stream_response(
    state: AppState,
    grok_req: NewConversationRequest,
    model: String,
) -> Result<Response, ApiError> {
    let mut grok_stream = state.client.create_conversation(&grok_req).await?;

    let mut full_text = String::new();
    while let Some(chunk) = grok_stream.next().await {
        match chunk {
            Ok(StreamChunk::Token { text, .. }) => full_text.push_str(&text),
            Ok(StreamChunk::Done) => break,
            Err(e) => return Err(ApiError::from(e)),
            _ => {}
        }
    }

    Ok(Json(ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion",
        created: now_secs(),
        model,
        choices: [Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant",
                content: full_text,
            },
            finish_reason: "stop",
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    })
    .into_response())
}

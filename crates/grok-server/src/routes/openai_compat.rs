use axum::Json;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::error::{ApiError, AppJson};
use crate::routes::models::MODE_IDS;
use crate::state::AppState;
use grok_client::streaming::StreamChunk;
use grok_client::types::chat::NewConversationRequest;
use grok_client::types::models::{ModelMode, ModelName};

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub max_completion_tokens: Option<u64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub tools: Option<serde_json::Value>,
    #[serde(default)]
    pub response_format: Option<serde_json::Value>,
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
    role: Option<&'static str>,
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

fn parse_model_string(model: &str) -> (&str, Option<ModelMode>) {
    for suffix in ["-auto", "-fast", "-expert", "-heavy"] {
        if let Some(base) = model.strip_suffix(suffix) {
            let mode = match suffix {
                "-auto" => ModelMode::Auto,
                "-fast" => ModelMode::Fast,
                "-expert" => ModelMode::Expert,
                "-heavy" => ModelMode::Heavy,
                _ => unreachable!(),
            };
            return (base, Some(mode));
        }
    }
    (model, None)
}

fn sse_line(json: &impl Serialize) -> bytes::Bytes {
    let mut buf = Vec::with_capacity(256);
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
    AppJson(request): AppJson<ChatCompletionRequest>,
) -> Result<Response, ApiError> {
    if request.temperature.is_some()
        || request.top_p.is_some()
        || request.max_tokens.is_some()
        || request.max_completion_tokens.is_some()
    {
        tracing::debug!(
            "OpenAI params temperature/top_p/max_tokens ignored — Grok web API does not support these"
        );
    }
    if request.tools.is_some() {
        tracing::warn!("tools/functions not supported — tool calls will be ignored");
    }
    if request.response_format.is_some() {
        tracing::debug!("response_format ignored — Grok web API does not support JSON mode");
    }

    let system_prompt = request
        .messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>();

    let non_system: Vec<_> = request
        .messages
        .iter()
        .filter(|m| m.role != "system")
        .collect();

    let prompt = if non_system.len() <= 1 {
        non_system
            .first()
            .map(|m| m.content.clone())
            .unwrap_or_default()
    } else {
        let history = non_system[..non_system.len() - 1]
            .iter()
            .map(|m| format!("[{}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let last = &non_system[non_system.len() - 1].content;
        format!("{history}\n\n[user]: {last}")
    };

    let mut grok_req = NewConversationRequest::new(&prompt);
    grok_req.temporary = Some(true);
    if !system_prompt.is_empty() {
        grok_req.options.custom_instructions = Some(system_prompt.join("\n"));
    }

    let model_str = request.model.unwrap_or_else(|| "auto".into());

    if MODE_IDS.contains(&model_str.as_str()) {
        grok_req.options.mode_id = Some(model_str.clone().into());
    } else {
        let (base_model, mode) = parse_model_string(&model_str);

        grok_req.options.model_name = Some(match base_model {
            "grok-2" => ModelName::Grok2,
            "grok-3" => ModelName::Grok3,
            "grok-3-mini" => ModelName::Grok3Mini,
            "grok-4" => ModelName::Grok4,
            "grok-4-mini" => ModelName::Grok4Mini,
            other => ModelName::Other(other.to_owned()),
        });

        if let Some(m) = mode {
            grok_req.options.model_mode = Some(m);
        }
    }

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

    let mut sent_role = false;

    let sse_stream = grok_stream.filter_map(move |chunk_result| {
        let id = completion_id.clone();
        let model = model.clone();
        let needs_role = !sent_role;
        if needs_role {
            sent_role = true;
        }

        async move {
            match chunk_result {
                Ok(StreamChunk::Token { text, is_soft_stop }) => {
                    if is_soft_stop && text.is_empty() {
                        return None;
                    }
                    if needs_role {
                        let role_chunk = ChunkWrapper {
                            id: &id,
                            object: "chat.completion.chunk",
                            created,
                            model: &model,
                            choices: [ChunkChoice {
                                index: 0,
                                delta: Delta {
                                    role: Some("assistant"),
                                    content: None,
                                },
                                finish_reason: None,
                            }],
                        };
                        let mut buf = sse_line(&role_chunk).to_vec();
                        let content_chunk = ChunkWrapper {
                            id: &id,
                            object: "chat.completion.chunk",
                            created,
                            model: &model,
                            choices: [ChunkChoice {
                                index: 0,
                                delta: Delta {
                                    role: None,
                                    content: Some(&text),
                                },
                                finish_reason: None,
                            }],
                        };
                        buf.extend_from_slice(&sse_line(&content_chunk));
                        return Some(Ok::<_, std::io::Error>(bytes::Bytes::from(buf)));
                    }
                    let chunk = ChunkWrapper {
                        id: &id,
                        object: "chat.completion.chunk",
                        created,
                        model: &model,
                        choices: [ChunkChoice {
                            index: 0,
                            delta: Delta {
                                role: None,
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
                            delta: Delta {
                                role: None,
                                content: None,
                            },
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
    let full_text = grok_stream.collect_text().await?;

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

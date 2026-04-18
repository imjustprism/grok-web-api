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

const TOOL_CALL_OPEN: &str = "<tool_call>";
const TOOL_CALL_CLOSE: &str = "</tool_call>";

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
    pub tools: Option<Vec<ToolDef>>,
    #[serde(default)]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(default)]
    pub response_format: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(default, deserialize_with = "deserialize_content")]
    pub content: String,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<InboundToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct InboundToolCall {
    pub id: String,
    pub function: InboundFunctionCall,
}

#[derive(Debug, Deserialize)]
pub struct InboundFunctionCall {
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolDef {
    pub function: FunctionDef,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

fn deserialize_content<'de, D>(de: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(de)?;
    match value {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Array(parts) => {
            let mut out = String::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                }
            }
            Ok(out)
        }
        other => Err(D::Error::custom(format!(
            "unsupported content type: {other}"
        ))),
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OutboundToolCall>>,
}

#[derive(Debug, Serialize)]
pub struct OutboundToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub function: OutboundFunctionCall,
}

#[derive(Debug, Serialize)]
pub struct OutboundFunctionCall {
    pub name: String,
    pub arguments: String,
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

#[derive(Serialize, Default)]
struct Delta<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCallDelta<'a>>>,
}

#[derive(Serialize)]
struct ToolCallDelta<'a> {
    index: u32,
    id: &'a str,
    #[serde(rename = "type")]
    kind: &'static str,
    function: FunctionDelta<'a>,
}

#[derive(Serialize)]
struct FunctionDelta<'a> {
    name: &'a str,
    arguments: &'a str,
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
    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(b"data: ");
    if let Err(e) = serde_json::to_writer(&mut buf, json) {
        tracing::error!("sse_line serialize failed: {e}");
        buf.truncate(b"data: ".len());
        buf.extend_from_slice(br#"{"error":{"message":"serialization error","type":"internal"}}"#);
    }
    buf.extend_from_slice(b"\n\n");
    bytes::Bytes::from(buf)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn alias_map(tools: &[ToolDef]) -> std::collections::HashMap<String, String> {
    tools
        .iter()
        .enumerate()
        .map(|(i, t)| (format!("fn_{i}"), t.function.name.clone()))
        .collect()
}

fn aliased_specs(tools: &[ToolDef], map: &std::collections::HashMap<String, String>) -> String {
    let reverse: std::collections::HashMap<&str, &str> =
        map.iter().map(|(a, o)| (o.as_str(), a.as_str())).collect();
    let specs: Vec<FunctionDef> = tools
        .iter()
        .map(|t| FunctionDef {
            name: reverse
                .get(t.function.name.as_str())
                .copied()
                .unwrap_or(&t.function.name)
                .to_owned(),
            description: t.function.description.clone(),
            parameters: t.function.parameters.clone(),
        })
        .collect();
    serde_json::to_string_pretty(&specs).unwrap_or_else(|_| "[]".into())
}

fn build_tool_system_block(
    tools: &[ToolDef],
    tool_choice: Option<&serde_json::Value>,
    alias: &std::collections::HashMap<String, String>,
) -> String {
    let schema = aliased_specs(tools, alias);
    let reverse: std::collections::HashMap<&str, &str> = alias
        .iter()
        .map(|(a, o)| (o.as_str(), a.as_str()))
        .collect();
    let directive = match tool_choice {
        Some(serde_json::Value::String(s)) if s == "required" => {
            "You MUST call at least one function. Do not answer in natural language."
        }
        Some(serde_json::Value::Object(obj))
            if obj.get("type").and_then(|v| v.as_str()) == Some("function") =>
        {
            if let Some(name) = obj
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
            {
                let aliased = reverse.get(name).copied().unwrap_or(name);
                return format!(
                    "You are connected to a programmatic function-calling interface. To satisfy this request, call the function \"{aliased}\" exactly once by emitting one line:\n\
                     {TOOL_CALL_OPEN}{{\"name\":\"{aliased}\",\"arguments\":{{...}}}}{TOOL_CALL_CLOSE}\n\
                     Arguments must be a JSON object matching the schema.\n\n\
                     Available functions:\n{schema}"
                );
            }
            "Call a function when appropriate."
        }
        _ => "Call a function only when needed to satisfy the user's request.",
    };
    format!(
        "You are connected to a programmatic function-calling interface. Instead of answering directly, you may request that the host program call one of the available functions on your behalf and return the result to you on the next turn.\n\n\
         To request a function call, include exactly one line with this format per call:\n\
         {TOOL_CALL_OPEN}{{\"name\":\"<fn_id>\",\"arguments\":{{...}}}}{TOOL_CALL_CLOSE}\n\
         Arguments must be a JSON object matching the function's parameter schema. After emitting function-call lines, stop — results will arrive on the next turn. Do not wrap the line in code fences.\n\
         {directive}\n\n\
         Example:\n\
         {TOOL_CALL_OPEN}{{\"name\":\"fn_0\",\"arguments\":{{\"x\":1}}}}{TOOL_CALL_CLOSE}\n\n\
         Available functions:\n{schema}"
    )
}

fn render_history<'a, I>(messages: I, alias: &std::collections::HashMap<String, String>) -> String
where
    I: IntoIterator<Item = &'a Message>,
{
    use std::fmt::Write;
    let real_to_alias: std::collections::HashMap<&str, &str> = alias
        .iter()
        .map(|(a, o)| (o.as_str(), a.as_str()))
        .collect();
    let mut out = String::new();
    for m in messages {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        match m.role.as_str() {
            "tool" => {
                let id = m.tool_call_id.as_deref().unwrap_or("unknown");
                let _ = write!(
                    &mut out,
                    "[tool_result id={id}]\n{}\n[/tool_result]",
                    m.content
                );
            }
            "assistant" => {
                out.push_str("[assistant]: ");
                out.push_str(&m.content);
                if let Some(calls) = &m.tool_calls {
                    for c in calls {
                        let args = if c.function.arguments.is_empty() {
                            "{}"
                        } else {
                            c.function.arguments.as_str()
                        };
                        let name = real_to_alias
                            .get(c.function.name.as_str())
                            .copied()
                            .unwrap_or(c.function.name.as_str());
                        let _ = write!(
                            &mut out,
                            "\n{TOOL_CALL_OPEN}{{\"id\":\"{}\",\"name\":\"{}\",\"arguments\":{args}}}{TOOL_CALL_CLOSE}",
                            c.id, name,
                        );
                    }
                }
            }
            role => {
                let _ = write!(&mut out, "[{role}]: {}", m.content);
            }
        }
    }
    out
}

#[derive(Debug, Clone)]
struct ParsedToolCall {
    id: String,
    name: String,
    arguments: String,
}

fn parse_bare_json_calls(
    text: &str,
    known: &std::collections::HashSet<String>,
) -> Vec<ParsedToolCall> {
    let mut calls = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = json_object_end(&text[i..]) {
                let blob = &text[i..i + end];
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(blob) {
                    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    if !name.is_empty() && known.contains(name) && val.get("arguments").is_some() {
                        let arguments = val
                            .get("arguments")
                            .map(|a| {
                                if a.is_string() {
                                    a.as_str().unwrap_or("").to_owned()
                                } else {
                                    serde_json::to_string(a).unwrap_or_else(|_| "{}".into())
                                }
                            })
                            .unwrap_or_else(|| "{}".into());
                        let id = val
                            .get("id")
                            .and_then(|v| v.as_str())
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4().simple()));
                        calls.push(ParsedToolCall {
                            id,
                            name: name.to_owned(),
                            arguments,
                        });
                        i += end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    calls
}

fn strip_json_fences(s: &str) -> String {
    let mut out = s.to_owned();
    for fence in ["```json", "```JSON", "```"] {
        out = out.replace(fence, "");
    }
    out
}

fn reverse_alias(calls: &mut [ParsedToolCall], alias: &std::collections::HashMap<String, String>) {
    for c in calls {
        if let Some(real) = alias.get(&c.name) {
            c.name = real.clone();
        }
    }
}

fn parse_tool_calls(text: &str) -> (String, Vec<ParsedToolCall>) {
    let mut content = String::with_capacity(text.len());
    let mut calls = Vec::new();
    let mut cursor = 0;
    while let Some(open_rel) = text[cursor..].find(TOOL_CALL_OPEN) {
        let open = cursor + open_rel;
        content.push_str(&text[cursor..open]);
        let body_start = open + TOOL_CALL_OPEN.len();
        let close_rel = text[body_start..].find(TOOL_CALL_CLOSE);
        let next_open_rel = text[body_start..].find(TOOL_CALL_OPEN);
        let body_end = match (close_rel, next_open_rel) {
            (Some(c), Some(n)) if c < n => c,
            (Some(c), None) => c,
            _ => match json_object_end(&text[body_start..]) {
                Some(n) => n,
                None => {
                    content.push_str(&text[open..]);
                    return (content, calls);
                }
            },
        };
        let advance = body_end
            + if close_rel == Some(body_end) {
                TOOL_CALL_CLOSE.len()
            } else {
                0
            };
        let body = text[body_start..body_start + body_end].trim();
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
            let name = val
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned();
            let id = val
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4().simple()));
            let arguments = val
                .get("arguments")
                .map(|a| {
                    if a.is_string() {
                        a.as_str().unwrap_or("").to_owned()
                    } else {
                        serde_json::to_string(a).unwrap_or_else(|_| "{}".into())
                    }
                })
                .unwrap_or_else(|| "{}".into());
            if !name.is_empty() {
                calls.push(ParsedToolCall {
                    id,
                    name,
                    arguments,
                });
            }
        }
        cursor = body_start + advance;
    }
    content.push_str(&text[cursor..]);
    (content.trim().to_owned(), calls)
}

fn json_object_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(|&b| b == b'{')?;
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
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
    if request.response_format.is_some() {
        tracing::debug!("response_format ignored — Grok web API does not support JSON mode");
    }
    let tool_choice_none = request
        .tool_choice
        .as_ref()
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "none");

    let system_parts: Vec<String> = request
        .messages
        .iter()
        .filter(|m| m.role == "system")
        .map(|m| m.content.clone())
        .collect();

    let tools_enabled = !tool_choice_none && request.tools.as_ref().is_some_and(|t| !t.is_empty());

    let alias: std::collections::HashMap<String, String> = if tools_enabled {
        alias_map(request.tools.as_deref().unwrap_or(&[]))
    } else {
        std::collections::HashMap::new()
    };

    let tool_block = if tools_enabled && let Some(tools) = request.tools.as_ref() {
        Some(build_tool_system_block(
            tools,
            request.tool_choice.as_ref(),
            &alias,
        ))
    } else {
        None
    };

    let non_system: Vec<&Message> = request
        .messages
        .iter()
        .filter(|m| m.role != "system")
        .collect();

    if non_system.is_empty() {
        return Err(ApiError::bad_request(
            "messages must contain at least one non-system message".into(),
        ));
    }

    let history = if non_system.len() == 1
        && non_system[0].tool_calls.is_none()
        && non_system[0].role != "tool"
    {
        non_system[0].content.clone()
    } else {
        render_history(non_system.iter().copied(), &alias)
    };

    let prompt = match &tool_block {
        Some(block) => format!("{block}\n\n---\n\n{history}"),
        None => history,
    };

    let mut grok_req = NewConversationRequest::new(&prompt);
    grok_req.temporary = Some(true);
    if !system_parts.is_empty() {
        grok_req.options.custom_instructions = Some(system_parts.join("\n\n"));
    }

    let model_str = request.model.unwrap_or_else(|| "auto".into());
    let resolved = if MODE_IDS.contains(&model_str.as_str()) {
        model_str.clone()
    } else {
        tracing::debug!(
            requested = %model_str,
            "unknown model id, falling back to 'auto' (supported: auto, fast, expert, heavy, grok-4-3)"
        );
        "auto".to_owned()
    };
    grok_req.options.mode_id = Some(resolved.into());

    if request.stream {
        stream_response(state, grok_req, model_str, tools_enabled, alias).await
    } else {
        non_stream_response(state, grok_req, model_str, tools_enabled, alias).await
    }
}

async fn stream_response(
    state: AppState,
    grok_req: NewConversationRequest,
    model: String,
    tools_enabled: bool,
    alias: std::collections::HashMap<String, String>,
) -> Result<Response, ApiError> {
    let known_aliases: std::collections::HashSet<String> = alias.keys().cloned().collect();
    let mut grok_stream = state.client.create_conversation(&grok_req).await?;
    let completion_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = now_secs();

    let id_for_stream = completion_id.clone();
    let model_for_stream = model.clone();

    let sse_stream = async_stream::try_stream! {
        let role_chunk = ChunkWrapper {
            id: &id_for_stream,
            object: "chat.completion.chunk",
            created,
            model: &model_for_stream,
            choices: [ChunkChoice {
                index: 0,
                delta: Delta { role: Some("assistant"), ..Delta::default() },
                finish_reason: None,
            }],
        };
        yield sse_line(&role_chunk);

        let mut buffer = String::new();
        let mut emitted_tool = false;

        while let Some(chunk_result) = grok_stream.next().await {
            match chunk_result {
                Ok(StreamChunk::Token { text, is_soft_stop }) => {
                    if is_soft_stop && text.is_empty() { continue; }
                    buffer.push_str(&text);

                    if tools_enabled {
                        let flush_to = safe_flush_boundary(&buffer);
                        if flush_to > 0 {
                            let piece: String = buffer.drain(..flush_to).collect();
                            let chunk = ChunkWrapper {
                                id: &id_for_stream,
                                object: "chat.completion.chunk",
                                created,
                                model: &model_for_stream,
                                choices: [ChunkChoice {
                                    index: 0,
                                    delta: Delta { content: Some(&piece), ..Delta::default() },
                                    finish_reason: None,
                                }],
                            };
                            yield sse_line(&chunk);
                        }
                    } else {
                        let chunk = ChunkWrapper {
                            id: &id_for_stream,
                            object: "chat.completion.chunk",
                            created,
                            model: &model_for_stream,
                            choices: [ChunkChoice {
                                index: 0,
                                delta: Delta { content: Some(&text), ..Delta::default() },
                                finish_reason: None,
                            }],
                        };
                        buffer.clear();
                        yield sse_line(&chunk);
                    }
                }
                Ok(StreamChunk::Done) => break,
                Ok(StreamChunk::ThinkingToken { .. }) => {}
                Ok(StreamChunk::Error { message }) => {
                    tracing::warn!("upstream error chunk (suppressed): {message}");
                }
                Ok(_) => {}
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("unexpected NDJSON shape") || msg.contains("parse error") {
                        tracing::debug!("transient grok NDJSON parse error (suppressed): {msg}");
                        continue;
                    }
                    let err = SseError {
                        error: SseErrorInner { message: &msg, r#type: "stream_error" },
                    };
                    yield sse_line(&err);
                }
            }
        }

        let (remaining_content, mut calls) = if tools_enabled {
            let cleaned = strip_json_fences(&buffer);
            let (mut content, mut parsed) = parse_tool_calls(&cleaned);
            if parsed.is_empty() {
                parsed = parse_bare_json_calls(&cleaned, &known_aliases);
                if !parsed.is_empty() {
                    content.clear();
                }
            }
            (content, parsed)
        } else {
            (std::mem::take(&mut buffer), Vec::new())
        };
        reverse_alias(&mut calls, &alias);

        if !remaining_content.is_empty() {
            let chunk = ChunkWrapper {
                id: &id_for_stream,
                object: "chat.completion.chunk",
                created,
                model: &model_for_stream,
                choices: [ChunkChoice {
                    index: 0,
                    delta: Delta { content: Some(&remaining_content), ..Delta::default() },
                    finish_reason: None,
                }],
            };
            yield sse_line(&chunk);
        }

        if !calls.is_empty() {
            emitted_tool = true;
            let deltas: Vec<ToolCallDelta<'_>> = calls
                .iter()
                .enumerate()
                .map(|(i, c)| ToolCallDelta {
                    index: i as u32,
                    id: &c.id,
                    kind: "function",
                    function: FunctionDelta { name: &c.name, arguments: &c.arguments },
                })
                .collect();
            let chunk = ChunkWrapper {
                id: &id_for_stream,
                object: "chat.completion.chunk",
                created,
                model: &model_for_stream,
                choices: [ChunkChoice {
                    index: 0,
                    delta: Delta { tool_calls: Some(deltas), ..Delta::default() },
                    finish_reason: None,
                }],
            };
            yield sse_line(&chunk);
        }

        let finish = if emitted_tool { "tool_calls" } else { "stop" };
        let final_chunk = ChunkWrapper {
            id: &id_for_stream,
            object: "chat.completion.chunk",
            created,
            model: &model_for_stream,
            choices: [ChunkChoice {
                index: 0,
                delta: Delta::default(),
                finish_reason: Some(finish),
            }],
        };
        yield sse_line(&final_chunk);
        yield bytes::Bytes::from_static(b"data: [DONE]\n\n");
    };

    let boxed: std::pin::Pin<
        Box<dyn futures::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>> + Send>,
    > = Box::pin(sse_stream);

    Ok((
        [
            (header::CONTENT_TYPE, "text/event-stream"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::CONNECTION, "keep-alive"),
        ],
        axum::body::Body::from_stream(boxed),
    )
        .into_response())
}

fn safe_flush_boundary(buf: &str) -> usize {
    if let Some(open) = buf.find(TOOL_CALL_OPEN) {
        return open;
    }
    let bytes = buf.as_bytes();
    let open_bytes = TOOL_CALL_OPEN.as_bytes();
    let max_partial = open_bytes.len() - 1;
    let start = bytes.len().saturating_sub(max_partial);
    for i in start..bytes.len() {
        if !buf.is_char_boundary(i) {
            continue;
        }
        let tail = &bytes[i..];
        if open_bytes.starts_with(tail) {
            return i;
        }
    }
    bytes.len()
}

async fn non_stream_response(
    state: AppState,
    grok_req: NewConversationRequest,
    model: String,
    tools_enabled: bool,
    alias: std::collections::HashMap<String, String>,
) -> Result<Response, ApiError> {
    let mut grok_stream = state.client.create_conversation(&grok_req).await?;
    let full_text = grok_stream.collect_text().await?;

    let known_aliases: std::collections::HashSet<String> = alias.keys().cloned().collect();
    let (content, mut calls) = if tools_enabled {
        let cleaned = strip_json_fences(&full_text);
        let (mut content, mut parsed) = parse_tool_calls(&cleaned);
        if parsed.is_empty() {
            parsed = parse_bare_json_calls(&cleaned, &known_aliases);
            if !parsed.is_empty() {
                content.clear();
            }
        }
        (content, parsed)
    } else {
        (full_text, Vec::new())
    };
    reverse_alias(&mut calls, &alias);

    let has_calls = !calls.is_empty();
    let tool_calls = has_calls.then(|| {
        calls
            .into_iter()
            .map(|c| OutboundToolCall {
                id: c.id,
                kind: "function",
                function: OutboundFunctionCall {
                    name: c.name,
                    arguments: c.arguments,
                },
            })
            .collect()
    });

    Ok(Json(ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion",
        created: now_secs(),
        model,
        choices: [Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant",
                content: if content.is_empty() {
                    None
                } else {
                    Some(content)
                },
                tool_calls,
            },
            finish_reason: if has_calls { "tool_calls" } else { "stop" },
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    })
    .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_call() {
        let s = "ok, calling\n<tool_call>{\"name\":\"foo\",\"arguments\":{\"x\":1}}</tool_call>";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(content, "ok, calling");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "foo");
        assert_eq!(calls[0].arguments, "{\"x\":1}");
    }

    #[test]
    fn parses_multiple_calls() {
        let s = "<tool_call>{\"name\":\"a\",\"arguments\":{}}</tool_call><tool_call>{\"name\":\"b\",\"arguments\":{}}</tool_call>";
        let (_, calls) = parse_tool_calls(s);
        assert_eq!(calls.len(), 2);
    }

    #[test]
    fn flush_boundary_holds_partial_open() {
        let b = "hello <tool";
        let n = safe_flush_boundary(b);
        assert_eq!(&b[..n], "hello ");
    }

    #[test]
    fn flush_boundary_holds_full_open() {
        let b = "hello <tool_call>{partial";
        let n = safe_flush_boundary(b);
        assert_eq!(&b[..n], "hello ");
    }

    #[test]
    fn flush_boundary_no_marker() {
        let b = "just text no tags";
        assert_eq!(safe_flush_boundary(b), b.len());
    }

    #[test]
    fn unclosed_tag_kept_as_content() {
        let s = "hello <tool_call>{\"name\":\"a\"";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(content, s.trim());
        assert!(calls.is_empty());
    }

    #[test]
    fn invalid_json_body_skipped() {
        let s = "<tool_call>not json at all</tool_call>keep";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(content, "keep");
        assert!(calls.is_empty());
    }

    #[test]
    fn missing_name_skipped() {
        let s = "<tool_call>{\"arguments\":{}}</tool_call>after";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(content, "after");
        assert!(calls.is_empty());
    }

    #[test]
    fn mixed_content_and_calls() {
        let s = "thinking...\n<tool_call>{\"name\":\"x\",\"arguments\":{\"a\":1}}</tool_call>\ntrailing";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(content, "thinking...\n\ntrailing");
        assert_eq!(calls.len(), 1);
    }

    #[test]
    fn string_arguments_preserved() {
        let s = "<tool_call>{\"name\":\"x\",\"arguments\":\"{\\\"a\\\":1}\"}</tool_call>";
        let (_, calls) = parse_tool_calls(s);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments, "{\"a\":1}");
    }

    #[test]
    fn id_passed_through_when_present() {
        let s = "<tool_call>{\"id\":\"call_abc\",\"name\":\"x\",\"arguments\":{}}</tool_call>";
        let (_, calls) = parse_tool_calls(s);
        assert_eq!(calls[0].id, "call_abc");
    }

    #[test]
    fn id_auto_generated_when_missing() {
        let s = "<tool_call>{\"name\":\"x\",\"arguments\":{}}</tool_call>";
        let (_, calls) = parse_tool_calls(s);
        assert!(calls[0].id.starts_with("call_"));
    }

    #[test]
    fn render_tool_result_message() {
        let m = Message {
            role: "tool".into(),
            content: "result data".into(),
            tool_call_id: Some("call_123".into()),
            tool_calls: None,
        };
        let out = render_history([&m], &std::collections::HashMap::new());
        assert!(out.contains("[tool_result id=call_123]"));
        assert!(out.contains("result data"));
    }

    #[test]
    fn render_assistant_tool_calls() {
        let m = Message {
            role: "assistant".into(),
            content: "calling".into(),
            tool_call_id: None,
            tool_calls: Some(vec![InboundToolCall {
                id: "c1".into(),
                function: InboundFunctionCall {
                    name: "foo".into(),
                    arguments: "{\"x\":1}".into(),
                },
            }]),
        };
        let out = render_history([&m], &std::collections::HashMap::new());
        assert!(out.contains("[assistant]: calling"));
        assert!(out.contains(r#"{"id":"c1","name":"foo","arguments":{"x":1}}"#));
    }

    #[test]
    fn render_history_separates_with_blank_line() {
        let a = Message {
            role: "user".into(),
            content: "hi".into(),
            tool_call_id: None,
            tool_calls: None,
        };
        let b = Message {
            role: "assistant".into(),
            content: "hello".into(),
            tool_call_id: None,
            tool_calls: None,
        };
        let out = render_history([&a, &b], &std::collections::HashMap::new());
        assert_eq!(out, "[user]: hi\n\n[assistant]: hello");
    }

    #[test]
    fn tool_block_forces_specific_function() {
        let tools = vec![ToolDef {
            function: FunctionDef {
                name: "my_fn".into(),
                description: None,
                parameters: None,
            },
        }];
        let choice = serde_json::json!({"type": "function", "function": {"name": "my_fn"}});
        let alias = alias_map(&tools);
        let block = build_tool_system_block(&tools, Some(&choice), &alias);
        assert!(block.contains("fn_0"));
    }

    #[test]
    fn tool_block_required_adds_mandate() {
        let tools = vec![ToolDef {
            function: FunctionDef {
                name: "x".into(),
                description: None,
                parameters: None,
            },
        }];
        let choice = serde_json::Value::String("required".into());
        let alias = alias_map(&tools);
        let block = build_tool_system_block(&tools, Some(&choice), &alias);
        assert!(block.contains("MUST call at least one function"));
    }

    #[test]
    fn parses_without_close_tags_back_to_back() {
        let s = "<tool_call>{\"name\":\"a\",\"arguments\":{\"x\":1}}\n<tool_call>{\"name\":\"b\",\"arguments\":{}}";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "a");
        assert_eq!(calls[1].name, "b");
        assert_eq!(content, "");
    }

    #[test]
    fn parses_without_close_tag_at_eof() {
        let s = "thinking\n<tool_call>{\"name\":\"a\",\"arguments\":{\"x\":1}}";
        let (content, calls) = parse_tool_calls(s);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "a");
        assert_eq!(content, "thinking");
    }

    #[test]
    fn json_object_end_handles_nested_and_strings() {
        assert_eq!(json_object_end("{\"a\":{\"b\":1}}rest"), Some(13));
        assert_eq!(json_object_end("{\"s\":\"has } brace\"}tail"), Some(19));
        assert_eq!(json_object_end("{unterminated"), None);
    }

    #[test]
    fn flush_boundary_utf8_safe() {
        let b = "héllo <to";
        let n = safe_flush_boundary(b);
        assert!(b.is_char_boundary(n));
        assert_eq!(&b[..n], "héllo ");
    }
}

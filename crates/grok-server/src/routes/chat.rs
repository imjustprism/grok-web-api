use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;

use crate::error::ApiError;
use crate::state::AppState;
use grok_client::types::chat::{AddResponseRequest, NewConversationRequest};
use grok_client::types::common::ConversationId;

fn ndjson_stream(response: grok_client::wreq::Response) -> Response {
    let stream = response
        .bytes_stream()
        .map(|chunk| chunk.map_err(std::io::Error::other));
    (
        [(header::CONTENT_TYPE, "application/x-ndjson")],
        axum::body::Body::from_stream(stream),
    )
        .into_response()
}

pub async fn create_chat(
    State(state): State<AppState>,
    Json(request): Json<NewConversationRequest>,
) -> Result<Response, ApiError> {
    let response = state.client.create_conversation_raw(&request).await?;
    Ok(ndjson_stream(response))
}

pub async fn continue_chat(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    Json(request): Json<AddResponseRequest>,
) -> Result<Response, ApiError> {
    let response = state
        .client
        .add_response_raw(&ConversationId::new(conversation_id), &request)
        .await?;
    Ok(ndjson_stream(response))
}

pub async fn quick_answer(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, ApiError> {
    let query = body
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let response = state.client.quick_answer(query).await?;
    Ok(ndjson_stream(response))
}

pub async fn stop_chat(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state
        .client
        .stop_responses(&ConversationId::new(conversation_id))
        .await?;
    Ok(Json(serde_json::json!({ "status": "stopped" })))
}

use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::{CompanionId, MemoryId};
use grok_client::types::memory::EditMemoryRequest;

pub async fn get_memory_blurb(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.get_memory_blurb().await?;
    Ok(Json(result))
}

pub async fn fetch_memories(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .fetch_memories_v2(&CompanionId::new(id))
        .await?;
    Ok(Json(result))
}

pub async fn edit_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AppJson(request): AppJson<EditMemoryRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .client
        .edit_memory_v2(&MemoryId::new(id), &request)
        .await?;
    Ok(Json(serde_json::json!({ "status": "updated" })))
}

pub async fn delete_memory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state.client.delete_memory_v2(&MemoryId::new(id)).await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

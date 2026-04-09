use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::{CompanionId, MemoryId};
use grok_client::types::memory::EditMemoryRequest;

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub soft: Option<bool>,
}

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
    Query(query): Query<DeleteQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let mid = MemoryId::new(id);
    if query.soft.unwrap_or(false) {
        state.client.soft_delete_memory_v2(&mid).await?;
    } else {
        state.client.delete_memory_v2(&mid).await?;
    }
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn delete_all_memories(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let cid = CompanionId::new(id);
    if query.soft.unwrap_or(false) {
        state.client.soft_delete_all_memories_v2(&cid).await?;
    } else {
        state.client.delete_all_memories_v2(&cid).await?;
    }
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

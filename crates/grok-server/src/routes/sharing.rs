use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::{ConversationId, ShareLinkId};
use grok_client::types::sharing::{ShareArtifactRequest, ShareConversationRequest};

pub async fn share_conversation(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    AppJson(request): AppJson<ShareConversationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .share_conversation(&ConversationId::new(conversation_id), &request)
        .await?;
    Ok(Json(result))
}

pub async fn list_share_links(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.list_share_links().await?;
    Ok(Json(result))
}

pub async fn get_share_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.get_share_link(&ShareLinkId::new(id)).await?;
    Ok(Json(result))
}

pub async fn clone_share_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.clone_share_link(&ShareLinkId::new(id)).await?;
    Ok(Json(result))
}

pub async fn share_artifact(
    State(state): State<AppState>,
    Path(conversation_id): Path<String>,
    AppJson(request): AppJson<ShareArtifactRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .share_artifact(&ConversationId::new(conversation_id), &request)
        .await?;
    Ok(Json(result))
}

pub async fn get_shared_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.get_shared_artifact(&id).await?;
    Ok(Json(result))
}

pub async fn delete_share_link(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .client
        .delete_share_link(&ShareLinkId::new(id))
        .await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

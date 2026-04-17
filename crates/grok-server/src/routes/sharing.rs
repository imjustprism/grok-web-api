use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::{ConversationId, ShareLinkId, SharedArtifactId};
use grok_client::types::sharing::{ShareArtifactRequest, ShareConversationRequest};

#[derive(Debug, Deserialize)]
pub struct ListShareLinksQuery {
    pub page_size: Option<u32>,
    pub page_token: Option<String>,
}

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
    Query(q): Query<ListShareLinksQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .list_share_links(q.page_size, q.page_token.as_deref())
        .await?;
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
    let result = state
        .client
        .get_shared_artifact(&SharedArtifactId::new(id))
        .await?;
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

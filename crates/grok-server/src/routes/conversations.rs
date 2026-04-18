use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::endpoints::conversations::ListConversationsQuery;
use grok_client::types::common::ConversationId;
use grok_client::types::conversation::UpdateConversationRequest;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page_size: Option<u32>,
    pub page_token: Option<String>,
    pub starred: Option<bool>,
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let mut q = ListConversationsQuery::new();
    if let Some(size) = query.page_size {
        q = q.page_size(size);
    }
    if let Some(token) = query.page_token {
        q = q.page_token(token);
    }
    if let Some(starred) = query.starred {
        q = q.starred(starred);
    }
    let result = state.client.list_conversations(&q).await?;
    Ok(Json(result))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .get_conversation(&ConversationId::new(id))
        .await?;
    Ok(Json(result))
}

pub async fn update_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AppJson(request): AppJson<UpdateConversationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .update_conversation(&ConversationId::new(id), &request)
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    pub soft: Option<bool>,
    pub delete_media: Option<bool>,
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let cid = ConversationId::new(id);
    if query.soft.unwrap_or(false) {
        state.client.soft_delete_conversation(&cid).await?;
    } else {
        state
            .client
            .delete_conversation(&cid, query.delete_media.unwrap_or(false))
            .await?;
    }
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn delete_all_conversations(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    state.client.delete_all_conversations().await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn conversation_exists(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let exists = state
        .client
        .conversation_exists(&ConversationId::new(id))
        .await?;
    Ok(Json(serde_json::json!({ "exists": exists })))
}

pub async fn list_deleted_conversations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .list_deleted_conversations(query.page_size, query.page_token.as_deref())
        .await?;
    Ok(Json(result))
}

pub async fn restore_conversation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state
        .client
        .restore_conversation(&ConversationId::new(id))
        .await?;
    Ok(Json(serde_json::json!({ "status": "restored" })))
}

pub async fn generate_title(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .generate_title(&ConversationId::new(id))
        .await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ResponsesQuery {
    pub include_threads: Option<bool>,
}

pub async fn list_responses(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ResponsesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .list_responses(
            &ConversationId::new(id),
            query.include_threads.unwrap_or(false),
        )
        .await?;
    Ok(Json(result))
}

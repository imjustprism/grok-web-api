use axum::Json;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SuggestionsQuery {
    pub query: Option<String>,
    pub count: Option<u32>,
}

pub async fn get_suggestions(
    State(state): State<AppState>,
    Query(q): Query<SuggestionsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .get_suggestions(q.query.as_deref(), q.count)
        .await?;
    Ok(Json(result))
}

pub async fn get_starters(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.get_conversation_starters().await?;
    Ok(Json(result))
}

pub async fn fetch_follow_up_suggestions(
    State(state): State<AppState>,
    AppJson(body): AppJson<serde_json::Value>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.fetch_follow_up_suggestions(&body).await?;
    Ok(Json(result))
}

pub async fn list_image_generations(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.list_image_generations().await?;
    Ok(Json(result))
}

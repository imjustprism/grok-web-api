use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use grok_client::types::common::CodeLanguage;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RunCodeBody {
    pub language: CodeLanguage,
    pub code: String,
}

pub async fn run_code(
    State(state): State<AppState>,
    AppJson(body): AppJson<RunCodeBody>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.run_code(&body.language, &body.code).await?;
    Ok(Json(result))
}

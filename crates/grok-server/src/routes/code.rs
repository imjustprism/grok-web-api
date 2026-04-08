use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RunCodeBody {
    pub language: String,
    pub code: String,
}

pub async fn run_code(
    State(state): State<AppState>,
    Json(body): Json<RunCodeBody>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.run_code(&body.language, &body.code).await?;
    Ok(Json(result))
}

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DriveQuery {
    pub query: Option<String>,
}

pub async fn list_files(
    State(state): State<AppState>,
    Query(q): Query<DriveQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .list_google_drive_files(q.query.as_deref())
        .await?;
    Ok(Json(result))
}

pub async fn read_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.read_google_drive_file(&id).await?;
    Ok(Json(result))
}

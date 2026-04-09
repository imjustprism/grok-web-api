use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::FileMetadataId;
use grok_client::types::files::UploadFileRequest;

pub async fn upload_file(
    State(state): State<AppState>,
    AppJson(request): AppJson<UploadFileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.upload_file(&request).await?;
    Ok(Json(result))
}

pub async fn get_file_metadata(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .get_file_metadata(&FileMetadataId::new(id))
        .await?;
    Ok(Json(result))
}

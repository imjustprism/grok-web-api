use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;

use crate::error::ApiError;
use crate::state::AppState;
use grok_client::types::artifacts::UpdateArtifactRequest;
use grok_client::types::common::{ArtifactId, ArtifactVersionId};

pub async fn get_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.client.get_artifact(&ArtifactId::new(id)).await?;
    Ok(Json(result))
}

pub async fn get_artifact_content(
    State(state): State<AppState>,
    Path((_, version_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .get_artifact_content(&ArtifactVersionId::new(version_id))
        .await?;
    Ok(Json(result))
}

pub async fn update_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateArtifactRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .client
        .update_artifact(&ArtifactId::new(id), &request)
        .await?;
    Ok(Json(result))
}

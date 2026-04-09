use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use serde::Deserialize;

use crate::error::{ApiError, AppJson};
use crate::state::AppState;
use grok_client::types::common::{ResponseId, VoiceId};
use grok_client::types::voice::TtsRequest;

#[derive(Debug, Deserialize)]
pub struct VoiceQuery {
    pub voice_id: Option<String>,
}

fn stream_binary(response: grok_client::wreq::Response, fallback_ct: &str) -> Response {
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or(fallback_ct)
        .to_owned();
    let stream = response
        .bytes_stream()
        .map(|chunk| chunk.map_err(std::io::Error::other));
    (
        [(header::CONTENT_TYPE, content_type)],
        axum::body::Body::from_stream(stream),
    )
        .into_response()
}

pub async fn read_response(
    State(state): State<AppState>,
    Path(response_id): Path<String>,
    Query(query): Query<VoiceQuery>,
) -> Result<Response, ApiError> {
    let response = state
        .client
        .read_response(
            &ResponseId::new(response_id),
            query.voice_id.map(VoiceId::new).as_ref(),
        )
        .await?;
    Ok(stream_binary(response, "application/octet-stream"))
}

pub async fn read_response_audio(
    State(state): State<AppState>,
    Path(response_id): Path<String>,
    Query(query): Query<VoiceQuery>,
) -> Result<Response, ApiError> {
    let response = state
        .client
        .read_response_audio(
            &ResponseId::new(response_id),
            query.voice_id.map(VoiceId::new).as_ref(),
        )
        .await?;
    Ok(stream_binary(response, "audio/mpeg"))
}

pub async fn tts(
    State(state): State<AppState>,
    AppJson(request): AppJson<TtsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let response = state.client.tts(&request).await?;
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(grok_client::error::GrokError::Request)?;
    Ok(Json(body))
}

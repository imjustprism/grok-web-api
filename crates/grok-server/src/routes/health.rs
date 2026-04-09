use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::state::AppState;

pub async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "grok-web-api"
    }))
}

pub async fn session_health(State(state): State<AppState>) -> impl IntoResponse {
    match state.client.check_session().await {
        Ok(true) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "session": "valid" })),
        ),
        Ok(false) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(
                json!({ "status": "error", "session": "expired", "message": "Grok SSO cookies expired" }),
            ),
        ),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "status": "error", "session": "unknown", "message": e.to_string() })),
        ),
    }
}

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.started_at.elapsed();
    let hours = uptime.as_secs() / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    let session_valid = state.client.check_session().await.ok();

    let challenge_loaded =
        state.config.challenge_header_hex.is_some() && state.config.challenge_suffix.is_some();

    let request_count = state.request_count.load(Ordering::Relaxed);
    let last_request_at = state.last_request_at.load(Ordering::Relaxed);

    Json(json!({
        "uptime_seconds": uptime.as_secs(),
        "uptime": format!("{hours}h {minutes}m {seconds}s"),
        "session_valid": session_valid,
        "challenge_loaded": challenge_loaded,
        "requests_served": request_count,
        "last_request_at": if last_request_at > 0 { Some(last_request_at) } else { None },
    }))
}

const SETUP_SCRIPT: &str = concat!(
    "Open grok.com in your browser, then paste this into the developer console (F12):\n\n",
    "(async () => {\n",
    "  const r = await fetch('/rest/app-chat/conversations?pageSize=1', { credentials: 'include' });\n",
    "  const id = r.headers.get('x-statsig-id') || '';\n",
    "  const reqId = r.headers.get('x-xai-request-id') || '';\n",
    "  console.log('Set these environment variables:\\n' +\n",
    "    'CHALLENGE_HEADER_HEX=' + id + '\\n' +\n",
    "    'CHALLENGE_SUFFIX=' + reqId);\n",
    "})();"
);

pub async fn setup() -> impl IntoResponse {
    Json(json!({
        "instructions": SETUP_SCRIPT
    }))
}

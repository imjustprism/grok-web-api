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

    let session_valid = state.client.auth().is_valid();

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
    "Install Void (https://github.com/imjustprism/Void), open grok.com, then paste into the devtools console (F12).\n",
    "Top-level await + var declarations — safe to re-run, no IIFE wrapping to mangle on paste:\n\n",
    "var m=Void.findByProps(\"chatApi\"),p=m.chatApi.configuration.middleware[0].pre,",
    "r=Math.random,d=Date.now,g=crypto.subtle.digest.bind(crypto.subtle),h;",
    "Math.random=()=>0;Date.now=()=>1e12;",
    "crypto.subtle.digest=async(a,b)=>{h=new TextDecoder().decode(b);return g(a,b)};",
    "var s=await p({url:\"https://grok.com/rest/app-chat/x\",init:{method:\"POST\",headers:{}}});",
    "Math.random=r;Date.now=d;crypto.subtle.digest=g;",
    "var t=new Uint8Array([...atob(s.init.headers[\"x-statsig-id\"])].map(c=>c.charCodeAt(0)));",
    "console.log(`CHALLENGE_HEADER_HEX=${[...t.slice(0,49)].map(b=>b.toString(16).padStart(2,\"0\")).join(\"\")}\\n",
    "CHALLENGE_SUFFIX=${h.split(\"!\").slice(2).join(\"!\").replace(/^-?\\d+/,\"\")}\\n",
    "CHALLENGE_TRAILER=${t[69]}`)"
);

pub async fn setup() -> impl IntoResponse {
    Json(json!({
        "instructions": SETUP_SCRIPT
    }))
}

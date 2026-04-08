use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub async fn api_key_auth(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let Some(expected) = state.config.api_key.as_deref().filter(|k| !k.is_empty()) else {
        return next.run(request).await;
    };

    if request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .is_some_and(|token| constant_time_eq(token.as_bytes(), expected.as_bytes()))
    {
        return next.run(request).await;
    }

    (
        StatusCode::UNAUTHORIZED,
        [("content-type", "application/json")],
        r#"{"error":"unauthorized","message":"Missing or invalid API key"}"#,
    )
        .into_response()
}

use axum::extract::{Request, State};
use axum::http::{Method, header};
use axum::response::IntoResponse;

use crate::error::ApiError;
use crate::routes::stream_body;
use crate::state::AppState;

const MAX_RAW_BODY_BYTES: usize = 10 * 1024 * 1024;

pub async fn raw_proxy(
    State(state): State<AppState>,
    request: Request,
) -> Result<impl IntoResponse, ApiError> {
    let uri = request.uri().clone();
    let raw_path = uri.path().strip_prefix("/raw/").unwrap_or(uri.path());

    if raw_path
        .split('/')
        .any(|seg| seg == ".." || seg == "." || seg.is_empty())
    {
        return Err(ApiError::bad_request(
            "path must not contain empty, '.' or '..' segments".into(),
        ));
    }

    let path = match uri.query() {
        Some(q) if !q.is_empty() => format!("{raw_path}?{q}"),
        _ => raw_path.to_owned(),
    };

    let method = request.method().clone();

    let body_bytes = axum::body::to_bytes(request.into_body(), MAX_RAW_BODY_BYTES)
        .await
        .map_err(|e| ApiError::bad_request(format!("Failed to read body: {e}")))?;

    let parse_json = |bytes: &[u8]| -> Result<serde_json::Value, ApiError> {
        serde_json::from_slice(bytes)
            .map_err(|e| ApiError::bad_request(format!("Invalid JSON: {e}")))
    };

    let response = match (&method, body_bytes.is_empty()) {
        (m, true) if *m == Method::GET => state.client.get(&path).await?,
        (m, true) if *m == Method::DELETE => state.client.delete(&path).await?,
        (m, false) if *m == Method::PUT => {
            state.client.put(&path, &parse_json(&body_bytes)?).await?
        }
        _ => {
            let json = if body_bytes.is_empty() {
                serde_json::Value::Null
            } else {
                parse_json(&body_bytes)?
            };
            state.client.post(&path, &json).await?
        }
    };

    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
        .to_owned();

    Ok((
        status,
        [(header::CONTENT_TYPE, content_type)],
        stream_body(response),
    ))
}

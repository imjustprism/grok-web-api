use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub error_type: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<u64>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let retry_after = self.retry_after_seconds;
        let body = serde_json::to_string(&self).unwrap_or_else(|_| {
            r#"{"type":"internal","title":"Internal Server Error","status":500,"detail":"Failed to serialize error"}"#.into()
        });

        let mut response =
            (status, [("content-type", "application/problem+json")], body).into_response();
        if let Some(secs) = retry_after {
            if let Ok(val) = axum::http::HeaderValue::from_str(&secs.to_string()) {
                response.headers_mut().insert("retry-after", val);
            }
        }
        response
    }
}

impl ApiError {
    fn new(error_type: &'static str, title: &'static str, status: u16, detail: String) -> Self {
        Self {
            error_type,
            title,
            status,
            detail,
            retry_after_seconds: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            "unauthorized",
            "Unauthorized",
            401,
            "Missing or invalid API key".into(),
        )
    }

    pub fn bad_request(detail: String) -> Self {
        Self::new("bad_request", "Bad Request", 400, detail)
    }
}

impl From<grok_client::error::GrokError> for ApiError {
    fn from(err: grok_client::error::GrokError) -> Self {
        use grok_client::error::GrokError;

        match err {
            GrokError::AuthExpired => Self::new(
                "auth_expired",
                "Session Expired",
                503,
                "Grok session cookies expired".into(),
            ),
            GrokError::RateLimited {
                message,
                wait_seconds,
                ..
            } => Self {
                retry_after_seconds: wait_seconds,
                ..Self::new("rate_limited", "Rate Limited", 429, message)
            },
            GrokError::NotFound(detail) => Self::new("not_found", "Not Found", 404, detail),
            GrokError::Upstream { status, ref body } if body.contains("anti-bot") => Self::new(
                "anti_bot",
                "Anti-Bot Rejected",
                403,
                format!(
                    "Anti-bot rejected (HTTP {status}). Challenge values may be expired. \
                     Re-extract from browser: GET /setup"
                ),
            ),
            GrokError::Upstream { status, body } => Self::new(
                "upstream_error",
                "Upstream Error",
                502,
                format!("Grok returned HTTP {status}: {body}"),
            ),
            GrokError::StreamParse(detail) => {
                Self::new("stream_error", "Stream Parse Error", 502, detail)
            }
            GrokError::Request(ref e) if e.is_timeout() => {
                Self::new("gateway_timeout", "Gateway Timeout", 504, err.to_string())
            }
            GrokError::Request(ref e) if e.is_connect() => Self::new(
                "upstream_unavailable",
                "Upstream Unavailable",
                503,
                err.to_string(),
            ),
            GrokError::Request(e) => {
                Self::new("request_error", "Request Error", 502, e.to_string())
            }
            GrokError::Json(e) => Self::new("json_error", "JSON Error", 500, e.to_string()),
            GrokError::Config(detail) => {
                Self::new("config_error", "Configuration Error", 500, detail)
            }
            _ => Self::new("unknown", "Unknown Error", 500, err.to_string()),
        }
    }
}

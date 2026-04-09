use std::fmt;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GrokError {
    #[error("authentication failed: cookies expired or invalid")]
    AuthExpired,

    #[error("rate limited ({limit_type}): {message}")]
    RateLimited {
        message: String,
        wait_seconds: Option<u64>,
        limit_type: RateLimitType,
    },

    #[error("upstream error ({status}): {body}")]
    Upstream { status: u16, body: String },

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("stream parse error: {0}")]
    StreamParse(String),

    #[error("request error: {0}")]
    Request(#[from] wreq::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid configuration: {0}")]
    Config(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RateLimitType {
    User,
    Global,
    Model,
    Other(String),
}

impl fmt::Display for RateLimitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Global => write!(f, "global"),
            Self::Model => write!(f, "model"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, GrokError>;

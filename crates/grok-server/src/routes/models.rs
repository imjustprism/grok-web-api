use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::error::ApiError;

pub const MODE_IDS: &[&str] = &["auto", "fast", "expert", "heavy", "grok-4-3"];

pub const GROK_4_3_UPSTREAM: &str = "grok-420-computer-use-sa";

#[derive(Serialize)]
struct ModelObject {
    id: &'static str,
    object: &'static str,
    created: u64,
    owned_by: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
}

#[derive(Serialize)]
struct ModelList {
    object: &'static str,
    data: &'static [ModelObject],
}

const MODELS: &[ModelObject] = &[
    ModelObject {
        id: "auto",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: Some("Chooses Fast or Expert based on query complexity"),
    },
    ModelObject {
        id: "fast",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: Some("Quick responses"),
    },
    ModelObject {
        id: "expert",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: Some("Thinks hard"),
    },
    ModelObject {
        id: "heavy",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
        description: Some("Team of experts — multi-agent orchestration"),
    },
    ModelObject {
        id: "grok-4-3",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
        description: Some("Grok 4.3 (beta) — early access"),
    },
];

pub async fn list_models() -> impl IntoResponse {
    Json(ModelList {
        object: "list",
        data: MODELS,
    })
}

pub async fn get_model(Path(id): Path<String>) -> Result<impl IntoResponse, ApiError> {
    MODELS.iter().find(|m| m.id == id).map(Json).ok_or_else(|| {
        ApiError::not_found(format!(
            "Model '{id}' not found. Use GET /v1/models for available models."
        ))
    })
}

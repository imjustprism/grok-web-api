use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::error::ApiError;

const CREATED_GROK_BASE: u64 = 1_710_000_000;
const CREATED_GROK_HEAVY: u64 = 1_720_000_000;
const CREATED_GROK_43: u64 = 1_740_000_000;

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
        created: CREATED_GROK_BASE,
        owned_by: "xai",
        description: Some("Chooses Fast or Expert based on query complexity"),
    },
    ModelObject {
        id: "fast",
        object: "model",
        created: CREATED_GROK_BASE,
        owned_by: "xai",
        description: Some("Quick responses"),
    },
    ModelObject {
        id: "expert",
        object: "model",
        created: CREATED_GROK_BASE,
        owned_by: "xai",
        description: Some("Thinks hard"),
    },
    ModelObject {
        id: "heavy",
        object: "model",
        created: CREATED_GROK_HEAVY,
        owned_by: "xai",
        description: Some("Team of experts — multi-agent orchestration"),
    },
    ModelObject {
        id: "grok-43",
        object: "model",
        created: CREATED_GROK_43,
        owned_by: "xai",
        description: Some("Grok 4.3 — early access, may require account access"),
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

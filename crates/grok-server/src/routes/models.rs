use axum::Json;
use axum::extract::Path;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::error::ApiError;

pub const MODE_IDS: &[&str] = &[
    "auto",
    "fast",
    "expert",
    "heavy",
    "grok-420",
    "grok-4-mini-thinking",
    "grok-4-1",
    "grok-4-1-thinking",
    "grok-4-1-nightly",
];

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
        description: Some("Quick responses — Grok 4.20"),
    },
    ModelObject {
        id: "expert",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: Some("Thinks hard — Grok 4.20"),
    },
    ModelObject {
        id: "heavy",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
        description: Some("Multi-agent orchestration — Grok 4.20"),
    },
    ModelObject {
        id: "grok-420",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
        description: Some("Multi-agent orchestration"),
    },
    ModelObject {
        id: "grok-4-mini-thinking",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
        description: Some("Fast reasoning model"),
    },
    ModelObject {
        id: "grok-4-1",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
        description: Some("Latest non-thinking model"),
    },
    ModelObject {
        id: "grok-4-1-thinking",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
        description: Some("Latest reasoning model"),
    },
    ModelObject {
        id: "grok-4-1-nightly",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
        description: Some("Experimental nightly build"),
    },
    ModelObject {
        id: "grok-2",
        object: "model",
        created: 1700000000,
        owned_by: "xai",
        description: Some("Legacy model"),
    },
    ModelObject {
        id: "grok-3",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: None,
    },
    ModelObject {
        id: "grok-3-mini",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
        description: None,
    },
    ModelObject {
        id: "grok-4",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
        description: None,
    },
    ModelObject {
        id: "grok-4-mini",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
        description: None,
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

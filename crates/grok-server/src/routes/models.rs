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
    },
    ModelObject {
        id: "fast",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "expert",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "heavy",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-420",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4-mini-thinking",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4-1",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4-1-thinking",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4-1-nightly",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-2",
        object: "model",
        created: 1700000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-3",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-3-mini",
        object: "model",
        created: 1710000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4-mini",
        object: "model",
        created: 1720000000,
        owned_by: "xai",
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
        ApiError::bad_request(format!(
            "Model '{id}' not found. Use GET /v1/models for available models."
        ))
    })
}

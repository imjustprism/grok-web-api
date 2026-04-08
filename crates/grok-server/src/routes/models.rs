use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;

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
        id: "grok-3-mini-fast",
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
    ModelObject {
        id: "grok-4.1-fast-reasoning",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4.1-fast-non-reasoning",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4.20-0309-reasoning",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4.20-0309-non-reasoning",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-4.20-multi-agent-0309",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-code-fast-1",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-imagine-image",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-imagine-image-pro",
        object: "model",
        created: 1730000000,
        owned_by: "xai",
    },
    ModelObject {
        id: "grok-imagine-video",
        object: "model",
        created: 1740000000,
        owned_by: "xai",
    },
    ModelObject { id: "grok-420", object: "model", created: 1740000000, owned_by: "xai" },
    ModelObject { id: "grok-3-mini-companion", object: "model", created: 1730000000, owned_by: "xai" },
    ModelObject { id: "grok-3-auto", object: "model", created: 1710000000, owned_by: "xai" },
    ModelObject { id: "grok-3-fast", object: "model", created: 1710000000, owned_by: "xai" },
    ModelObject { id: "grok-3-expert", object: "model", created: 1710000000, owned_by: "xai" },
    ModelObject { id: "grok-3-heavy", object: "model", created: 1710000000, owned_by: "xai" },
    ModelObject { id: "grok-4-auto", object: "model", created: 1720000000, owned_by: "xai" },
    ModelObject { id: "grok-4-fast", object: "model", created: 1720000000, owned_by: "xai" },
    ModelObject { id: "grok-4-expert", object: "model", created: 1720000000, owned_by: "xai" },
    ModelObject { id: "grok-4-heavy", object: "model", created: 1720000000, owned_by: "xai" },
];

pub async fn list_models() -> impl IntoResponse {
    Json(ModelList {
        object: "list",
        data: MODELS,
    })
}

use axum::Router;
use axum::extract::{Request, State};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, post};
use serde_json::json;

use crate::auth::api_key_auth;
use crate::state::AppState;

pub mod artifacts;
pub mod chat;
pub mod code;
pub mod conversations;
pub mod files;
pub mod health;
pub mod memory;
pub mod models;
pub mod openai_compat;
pub mod raw;
pub mod sharing;
pub mod suggestions;
pub mod voice;

async fn request_tracking(State(state): State<AppState>, request: Request, next: Next) -> Response {
    state.record_request();
    let request_id = uuid::Uuid::new_v4().to_string();
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let start = std::time::Instant::now();

    let mut response = next.run(request).await;

    let elapsed = start.elapsed();
    let status = response.status().as_u16();
    tracing::info!("{method} {path} {status} {}ms", elapsed.as_millis());

    if let Ok(val) = axum::http::HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", val);
    }
    if let Ok(val) = axum::http::HeaderValue::from_str(&format!("{}ms", elapsed.as_millis())) {
        response.headers_mut().insert("x-response-time", val);
    }
    response
}

async fn chat_completions_get() -> impl IntoResponse {
    axum::Json(json!({
        "message": "POST a ChatCompletion request to this endpoint. See /setup for configuration help.",
        "example": {
            "model": "auto",
            "messages": [{"role": "user", "content": "Hello"}],
            "stream": false
        }
    }))
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route(
            "/v1/chat/completions",
            post(openai_compat::chat_completions).get(chat_completions_get),
        )
        .route("/v1/models", get(models::list_models))
        .route("/v1/models/{id}", get(models::get_model))
        .route("/v1/chat", post(chat::create_chat))
        .route("/v1/chat/quick", post(chat::quick_answer))
        .route(
            "/v1/chat/{conversation_id}/message",
            post(chat::continue_chat),
        )
        .route("/v1/chat/{conversation_id}/stop", post(chat::stop_chat))
        .route(
            "/v1/conversations",
            get(conversations::list_conversations).delete(conversations::delete_all_conversations),
        )
        .route(
            "/v1/conversations/{id}",
            get(conversations::get_conversation)
                .put(conversations::update_conversation)
                .delete(conversations::delete_conversation),
        )
        .route(
            "/v1/conversations/{id}/restore",
            post(conversations::restore_conversation),
        )
        .route(
            "/v1/conversations/{id}/title",
            post(conversations::generate_title),
        )
        .route(
            "/v1/conversations/{id}/responses",
            get(conversations::list_responses),
        )
        .route("/v1/files", post(files::upload_file))
        .route("/v1/files/{id}/metadata", get(files::get_file_metadata))
        .route("/v1/code/run", post(code::run_code))
        .route("/v1/memory/blurb", get(memory::get_memory_blurb))
        .route(
            "/v1/memory/v2/{id}",
            get(memory::fetch_memories)
                .put(memory::edit_memory)
                .delete(memory::delete_memory),
        )
        .route("/v1/voice/read/{response_id}", get(voice::read_response))
        .route(
            "/v1/voice/audio/{response_id}",
            get(voice::read_response_audio),
        )
        .route("/v1/voice/tts", post(voice::tts))
        .route(
            "/v1/artifacts/{id}",
            get(artifacts::get_artifact).put(artifacts::update_artifact),
        )
        .route(
            "/v1/artifacts/{id}/content/{version_id}",
            get(artifacts::get_artifact_content),
        )
        .route(
            "/v1/sharing/{conversation_id}",
            post(sharing::share_conversation),
        )
        .route("/v1/sharing/links", get(sharing::list_share_links))
        .route(
            "/v1/sharing/links/{id}",
            get(sharing::get_share_link).delete(sharing::delete_share_link),
        )
        .route(
            "/v1/sharing/links/{id}/clone",
            post(sharing::clone_share_link),
        )
        .route("/v1/suggestions", get(suggestions::get_suggestions))
        .route("/v1/suggestions/starters", get(suggestions::get_starters))
        .route("/v1/images", get(suggestions::list_image_generations))
        .layer(middleware::from_fn_with_state(state.clone(), api_key_auth));

    let raw = Router::new()
        .route("/raw/{*path}", any(raw::raw_proxy))
        .layer(middleware::from_fn_with_state(state.clone(), api_key_auth));

    let public = Router::new()
        .route("/health", get(health::health))
        .route("/health/session", get(health::session_health))
        .route("/status", get(health::status))
        .route("/setup", get(health::setup));

    public
        .merge(api)
        .merge(raw)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            request_tracking,
        ))
        .with_state(state)
}

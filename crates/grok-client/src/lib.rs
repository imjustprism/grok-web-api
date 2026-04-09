pub mod auth;
pub mod challenge;
pub mod client;
pub mod endpoints;
pub mod error;
pub mod streaming;
pub mod types;

pub use auth::GrokAuth;
pub use challenge::ChallengeConfig;
pub use client::{GrokClient, HttpTokenProvider, TokenPair, TokenProvider};
pub use error::{GrokError, Result};
pub use streaming::{CollectedResponse, GrokStream, StreamChunk, WebSearchResult};
pub use types::chat::{
    AddResponseRequest, AddResponseRequestBuilder, ChatOptions, NewConversationRequest,
    NewConversationRequestBuilder,
};
pub use types::common::{ConversationId, ResponseId};
pub use types::models::{DeepsearchPreset, ModelMode, ModelName};
pub use wreq;

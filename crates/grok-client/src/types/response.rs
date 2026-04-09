use serde::{Deserialize, Serialize};

use super::common::{ConversationId, ResponseId, Sender, Timestamp};
use super::conversation::Conversation;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GrokEnvelope<T> {
    #[serde(default)]
    pub result: Option<T>,

    #[serde(default)]
    pub error: Option<GrokApiError>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GrokApiError {
    #[serde(default)]
    pub message: Option<String>,

    #[serde(default)]
    pub code: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NewConversationResult {
    #[serde(default)]
    pub response: Option<GrokResponse>,

    #[serde(default)]
    pub conversation: Option<Conversation>,

    #[serde(default)]
    pub title: Option<GeneratedTitle>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GrokResponse {
    #[serde(default)]
    pub response_id: Option<ResponseId>,

    #[serde(default)]
    pub conversation_id: Option<ConversationId>,

    #[serde(default)]
    pub message: Option<String>,

    #[serde(default)]
    pub sender: Option<Sender>,

    #[serde(default)]
    pub create_time: Option<Timestamp>,

    #[serde(default)]
    pub is_soft_stop: Option<bool>,

    #[serde(default)]
    pub token: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GeneratedTitle {
    #[serde(default)]
    pub title: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

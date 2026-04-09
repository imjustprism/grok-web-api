use serde::{Deserialize, Serialize};

use super::common::ConversationId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Conversation {
    pub conversation_id: ConversationId,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub create_time: Option<String>,

    #[serde(default)]
    pub update_time: Option<String>,

    #[serde(default)]
    pub starred: Option<bool>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdateConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConversationList {
    #[serde(default)]
    pub conversations: Vec<Conversation>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

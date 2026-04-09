use serde::{Deserialize, Serialize};

use super::common::VoiceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TtsRequest {
    pub articles: Vec<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitize: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<VoiceId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_alignment: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TtsResponse {
    #[serde(default)]
    pub result: Option<TtsResult>,

    #[serde(default)]
    pub error: Option<serde_json::Value>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TtsResult {
    #[serde(default)]
    pub data: Option<String>,

    #[serde(default)]
    pub content_type: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShareVoiceChatRequest {
    pub video_bytes: String,
    pub text: String,
}

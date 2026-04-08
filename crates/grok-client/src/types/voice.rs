use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsRequest {
    pub articles: Vec<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitize: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_alignment: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsResponse {
    #[serde(default)]
    pub result: Option<TtsResult>,

    #[serde(default)]
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsResult {
    #[serde(default)]
    pub data: Option<String>,

    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareVoiceChatRequest {
    pub video_bytes: String,
    pub text: String,
}

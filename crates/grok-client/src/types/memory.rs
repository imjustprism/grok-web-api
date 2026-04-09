use serde::{Deserialize, Serialize};

use super::common::MemoryId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Memory {
    #[serde(default)]
    pub id: Option<MemoryId>,

    #[serde(default)]
    pub content: Option<String>,

    #[serde(default)]
    pub create_time: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MemoryList {
    #[serde(default)]
    pub memories: Vec<Memory>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EditMemoryRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MemoryBlurb {
    #[serde(default)]
    pub memory_content: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

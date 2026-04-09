use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SuggestionList {
    #[serde(default)]
    pub suggestions: Option<Vec<Suggestion>>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Suggestion {
    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub query: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConversationStarters {
    #[serde(default)]
    pub starters: Option<Vec<serde_json::Value>>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageGenerationList {
    #[serde(default)]
    pub generations: Vec<serde_json::Value>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

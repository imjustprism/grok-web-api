use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunCodeRequest {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunCodeResponse {
    #[serde(default)]
    pub success: Option<bool>,

    #[serde(default)]
    pub stdout: Option<String>,

    #[serde(default)]
    pub stderr: Option<String>,

    #[serde(default)]
    pub output_files: Option<Vec<serde_json::Value>>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

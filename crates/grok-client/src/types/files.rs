use serde::{Deserialize, Serialize};

use super::common::FileMetadataId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFileRequest {
    pub file_name: String,
    pub file_mime_type: String,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub make_public: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_source: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub third_party_file_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UploadFileResponse {
    #[serde(default)]
    pub file_metadata_id: Option<FileMetadataId>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FileMetadata {
    #[serde(default)]
    pub file_metadata_id: Option<FileMetadataId>,

    #[serde(default)]
    pub file_name: Option<String>,

    #[serde(default)]
    pub file_mime_type: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

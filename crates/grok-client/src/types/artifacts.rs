use serde::{Deserialize, Serialize};

use super::common::{ArtifactId, ArtifactVersionId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Artifact {
    #[serde(default)]
    pub artifact_id: Option<ArtifactId>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub artifact_type: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ArtifactContent {
    #[serde(default)]
    pub content: Option<String>,

    #[serde(default)]
    pub artifact_version_id: Option<ArtifactVersionId>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArtifactRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_artifact: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_diff: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_version_id: Option<ArtifactVersionId>,
}

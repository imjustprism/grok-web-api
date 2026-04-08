use serde::{Deserialize, Serialize};

use super::common::{ArtifactId, ArtifactVersionId, ResponseId, ShareLinkId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareConversationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_id: Option<ResponseId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_members_to_share: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_with_team_members: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_publicly: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_indexing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareConversationResponse {
    pub share_link_id: Option<ShareLinkId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareArtifactRequest {
    pub response_id: ResponseId,
    pub artifact_id: ArtifactId,
    pub artifact_version_id: ArtifactVersionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShareLink {
    #[serde(default)]
    pub share_link_id: Option<ShareLinkId>,

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub create_time: Option<String>,

    #[serde(default)]
    pub view_count: Option<u64>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkList {
    #[serde(default)]
    pub share_links: Vec<ShareLink>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

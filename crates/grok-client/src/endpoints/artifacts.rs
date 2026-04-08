use crate::client::GrokClient;
use crate::error::Result;
use crate::types::artifacts::UpdateArtifactRequest;
use crate::types::common::{ArtifactId, ArtifactVersionId, ConversationId};

impl GrokClient {
    pub async fn get_artifact(&self, id: &ArtifactId) -> Result<serde_json::Value> {
        self.post_json(&format!("artifacts/{id}"), &serde_json::json!({}))
            .await
    }

    pub async fn get_artifact_content(
        &self,
        version_id: &ArtifactVersionId,
    ) -> Result<serde_json::Value> {
        self.get_json(&format!("artifact_content/{version_id}"))
            .await
    }

    pub async fn update_artifact(
        &self,
        id: &ArtifactId,
        request: &UpdateArtifactRequest,
    ) -> Result<serde_json::Value> {
        self.post_json(&format!("artifacts/{id}/update"), request)
            .await
    }

    pub async fn get_artifacts_metadata(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<serde_json::Value> {
        self.post_json(
            &format!("conversations/{conversation_id}/artifacts_metadata"),
            &serde_json::json!({}),
        )
        .await
    }
}

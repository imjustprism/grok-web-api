use crate::client::GrokClient;
use crate::error::Result;
use crate::types::common::{ConversationId, ShareLinkId, SharedArtifactId};
use crate::types::sharing::{ShareArtifactRequest, ShareConversationRequest};

impl GrokClient {
    pub async fn share_conversation(
        &self,
        conversation_id: &ConversationId,
        request: &ShareConversationRequest,
    ) -> Result<serde_json::Value> {
        self.post_json(&format!("conversations/{conversation_id}/share"), request)
            .await
    }

    pub async fn share_artifact(
        &self,
        conversation_id: &ConversationId,
        request: &ShareArtifactRequest,
    ) -> Result<serde_json::Value> {
        self.post_json(
            &format!("conversations/{conversation_id}/share_artifact"),
            request,
        )
        .await
    }

    pub async fn get_share_link(&self, id: &ShareLinkId) -> Result<serde_json::Value> {
        self.get_json(&format!("share_links_data/{id}")).await
    }

    pub async fn list_share_links(
        &self,
        page_size: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<serde_json::Value> {
        #[derive(serde::Serialize)]
        struct Q<'a> {
            #[serde(rename = "pageSize", skip_serializing_if = "Option::is_none")]
            page_size: Option<u32>,
            #[serde(rename = "pageToken", skip_serializing_if = "Option::is_none")]
            page_token: Option<&'a str>,
        }
        self.get_query_json(
            "share_links",
            &Q {
                page_size,
                page_token,
            },
        )
        .await
    }

    pub async fn clone_share_link(&self, id: &ShareLinkId) -> Result<serde_json::Value> {
        self.post_json(&format!("share_links/{id}/clone"), &serde_json::json!({}))
            .await
    }

    pub async fn delete_share_link(&self, id: &ShareLinkId) -> Result<()> {
        self.delete(&format!("share_links/{id}")).await?;
        Ok(())
    }

    pub async fn get_shared_artifact(&self, id: &SharedArtifactId) -> Result<serde_json::Value> {
        self.get_json(&format!("shared_artifacts/{id}")).await
    }
}

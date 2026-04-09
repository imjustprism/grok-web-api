use crate::client::GrokClient;
use crate::error::Result;
use crate::types::common::ConversationId;
use crate::types::conversation::{ConversationList, UpdateConversationRequest};

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListConversationsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_is_starred: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl GrokClient {
    pub async fn list_conversations(
        &self,
        query: &ListConversationsQuery,
    ) -> Result<ConversationList> {
        self.get_query_json("conversations", query).await
    }

    pub async fn get_conversation(&self, id: &ConversationId) -> Result<serde_json::Value> {
        self.get_json(&format!("conversations/{id}")).await
    }

    pub async fn conversation_exists(&self, id: &ConversationId) -> Result<bool> {
        match self.get(&format!("conversations/exists/{id}")).await {
            Ok(_) => Ok(true),
            Err(crate::error::GrokError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn update_conversation(
        &self,
        id: &ConversationId,
        request: &UpdateConversationRequest,
    ) -> Result<serde_json::Value> {
        self.put_json(&format!("conversations/{id}"), request).await
    }

    pub async fn delete_conversation(&self, id: &ConversationId, delete_media: bool) -> Result<()> {
        let path = format!("conversations/{id}");
        if delete_media {
            #[derive(serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            struct Q {
                delete_media: bool,
            }
            self.delete_with_query(&path, &Q { delete_media: true })
                .await?;
        } else {
            self.delete(&path).await?;
        }
        Ok(())
    }

    pub async fn soft_delete_conversation(&self, id: &ConversationId) -> Result<()> {
        self.delete(&format!("conversations/soft/{id}")).await?;
        Ok(())
    }

    pub async fn restore_conversation(&self, id: &ConversationId) -> Result<()> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            conversation_id: String,
        }
        self.post(
            "conversations/restore",
            &Body {
                conversation_id: id.to_string(),
            },
        )
        .await?;
        Ok(())
    }

    pub async fn generate_title(&self, id: &ConversationId) -> Result<serde_json::Value> {
        self.post_json(&format!("conversations/{id}/title"), &serde_json::json!({}))
            .await
    }

    pub async fn list_responses(
        &self,
        id: &ConversationId,
        include_threads: bool,
    ) -> Result<serde_json::Value> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Q {
            #[serde(skip_serializing_if = "std::ops::Not::not")]
            include_threads: bool,
        }
        self.get_query_json(
            &format!("conversations/{id}/responses"),
            &Q { include_threads },
        )
        .await
    }

    pub async fn list_deleted_conversations(
        &self,
        page_size: Option<u32>,
        page_token: Option<&str>,
    ) -> Result<serde_json::Value> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Q<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            page_size: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            page_token: Option<&'a str>,
        }
        self.get_query_json(
            "conversations/deleted",
            &Q {
                page_size,
                page_token,
            },
        )
        .await
    }

    pub async fn delete_all_conversations(&self) -> Result<()> {
        self.delete("conversations").await?;
        Ok(())
    }
}

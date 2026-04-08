use crate::client::GrokClient;
use crate::error::Result;
use crate::types::suggestions::{ConversationStarters, ImageGenerationList, SuggestionList};

impl GrokClient {
    pub async fn get_suggestions(
        &self,
        query: Option<&str>,
        count: Option<u32>,
    ) -> Result<SuggestionList> {
        #[derive(serde::Serialize)]
        struct Q<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            query: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            count: Option<u32>,
        }
        self.get_query_json("suggestions", &Q { query, count })
            .await
    }

    pub async fn get_conversation_starters(&self) -> Result<ConversationStarters> {
        self.get_json("conversation-starters").await
    }

    pub async fn fetch_follow_up_suggestions(
        &self,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.post_json("fetch-suggestions", body).await
    }

    pub async fn list_image_generations(&self) -> Result<ImageGenerationList> {
        self.get_json("image-generations").await
    }
}

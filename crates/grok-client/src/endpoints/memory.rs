use crate::client::GrokClient;
use crate::error::Result;
use crate::types::common::{CompanionId, MemoryId};
use crate::types::memory::{EditMemoryRequest, MemoryBlurb};

impl GrokClient {
    pub async fn get_memory(&self, conversation_ids: &[String]) -> Result<serde_json::Value> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            conversation_ids: &'a [String],
        }
        self.post_json("memory", &Body { conversation_ids }).await
    }

    pub async fn delete_memory(&self, conversation_ids: &[String]) -> Result<()> {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Q<'a> {
            conversation_ids: &'a [String],
        }
        self.delete_with_query("memory", &Q { conversation_ids })
            .await?;
        Ok(())
    }

    pub async fn fetch_memories_v2(&self, companion_id: &CompanionId) -> Result<serde_json::Value> {
        self.get_json(&format!("memories_v2/{companion_id}")).await
    }

    pub async fn delete_all_memories_v2(&self, companion_id: &CompanionId) -> Result<()> {
        self.delete(&format!("memories_v2/{companion_id}")).await?;
        Ok(())
    }

    pub async fn soft_delete_all_memories_v2(&self, companion_id: &CompanionId) -> Result<()> {
        self.delete(&format!("memories_v2/soft/{companion_id}"))
            .await?;
        Ok(())
    }

    pub async fn edit_memory_v2(&self, id: &MemoryId, request: &EditMemoryRequest) -> Result<()> {
        self.put(&format!("memory_v2/{id}"), request).await?;
        Ok(())
    }

    pub async fn delete_memory_v2(&self, id: &MemoryId) -> Result<()> {
        self.delete(&format!("memory_v2/{id}")).await?;
        Ok(())
    }

    pub async fn soft_delete_memory_v2(&self, id: &MemoryId) -> Result<()> {
        self.delete(&format!("memory_v2/soft/{id}")).await?;
        Ok(())
    }

    pub async fn get_memory_blurb(&self) -> Result<MemoryBlurb> {
        self.get_json("user-memory-blurb").await
    }
}

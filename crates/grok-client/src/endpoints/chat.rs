use std::pin::Pin;
use wreq::Response;

use crate::client::GrokClient;
use crate::error::Result;
use crate::streaming::GrokStream;
use crate::types::chat::{AddResponseRequest, NewConversationRequest, QuickAnswerRequest};
use crate::types::common::ConversationId;

type BoxByteStream =
    Pin<Box<dyn futures::Stream<Item = std::result::Result<bytes::Bytes, wreq::Error>> + Send>>;

impl GrokClient {
    pub async fn create_conversation_raw(
        &self,
        request: &NewConversationRequest,
    ) -> Result<Response> {
        self.post_stream("conversations/new", request).await
    }

    pub async fn create_conversation(
        &self,
        request: &NewConversationRequest,
    ) -> Result<GrokStream<BoxByteStream>> {
        let response = self.create_conversation_raw(request).await?;
        Ok(GrokStream::new(Box::pin(response.bytes_stream())))
    }

    pub async fn add_response_raw(
        &self,
        conversation_id: &ConversationId,
        request: &AddResponseRequest,
    ) -> Result<Response> {
        self.post_stream(&format!("conversations/{conversation_id}/responses"), request)
            .await
    }

    pub async fn add_response(
        &self,
        conversation_id: &ConversationId,
        request: &AddResponseRequest,
    ) -> Result<GrokStream<BoxByteStream>> {
        let response = self.add_response_raw(conversation_id, request).await?;
        Ok(GrokStream::new(Box::pin(response.bytes_stream())))
    }

    pub async fn quick_answer(&self, query: impl Into<String>) -> Result<Response> {
        self.post_stream(
            "quick-answer",
            &QuickAnswerRequest {
                query: query.into(),
            },
        )
        .await
    }

    pub async fn stop_responses(&self, conversation_id: &ConversationId) -> Result<()> {
        self.post(
            &format!("conversations/{conversation_id}/stop-inflight-responses"),
            &serde_json::Value::Null,
        )
        .await?;
        Ok(())
    }

    pub async fn cancel_response(&self, response_id: &str) -> Result<()> {
        self.delete(&format!("conversations/inflight-response/{response_id}"))
            .await?;
        Ok(())
    }

    pub async fn reconnect_response(&self, response_id: &str) -> Result<Response> {
        self.get(&format!("conversations/reconnect-response-v2/{response_id}"))
            .await
    }
}

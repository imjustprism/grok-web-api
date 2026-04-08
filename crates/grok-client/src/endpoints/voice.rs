use wreq::Response;

use crate::client::GrokClient;
use crate::error::Result;
use crate::types::voice::{ShareVoiceChatRequest, TtsRequest};

impl GrokClient {
    async fn get_voice_response(&self, path: &str, voice_id: Option<&str>) -> Result<Response> {
        match voice_id {
            Some(vid) => {
                #[derive(serde::Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Q<'a> {
                    voice_id: &'a str,
                }
                self.get_with_query(path, &Q { voice_id: vid }).await
            }
            None => self.get(path).await,
        }
    }

    pub async fn read_response(
        &self,
        response_id: &str,
        voice_id: Option<&str>,
    ) -> Result<Response> {
        self.get_voice_response(&format!("read-response/{response_id}"), voice_id).await
    }

    pub async fn read_response_audio(
        &self,
        response_id: &str,
        voice_id: Option<&str>,
    ) -> Result<Response> {
        self.get_voice_response(&format!("read-response-audio/{response_id}"), voice_id).await
    }

    pub async fn tts(&self, request: &TtsRequest) -> Result<Response> {
        self.post("tts", request).await
    }

    pub async fn post_voice_recording(&self, body: &serde_json::Value) -> Result<Response> {
        self.post("voice/post-recording", body).await
    }

    pub async fn share_voice_chat(&self, request: &ShareVoiceChatRequest) -> Result<Response> {
        self.post("voice/share", request).await
    }
}

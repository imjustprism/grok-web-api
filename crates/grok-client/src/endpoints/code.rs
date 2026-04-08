use crate::client::GrokClient;
use crate::error::Result;
use crate::types::code::RunCodeRequest;

impl GrokClient {
    pub async fn run_code(&self, language: &str, code: &str) -> Result<serde_json::Value> {
        self.post_json(
            "run-code",
            &RunCodeRequest {
                language: language.to_owned(),
                code: code.to_owned(),
            },
        )
        .await
    }
}

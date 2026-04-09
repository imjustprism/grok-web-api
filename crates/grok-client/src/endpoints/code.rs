use crate::client::GrokClient;
use crate::error::Result;
use crate::types::code::RunCodeRequest;
use crate::types::common::CodeLanguage;

impl GrokClient {
    pub async fn run_code(&self, language: &CodeLanguage, code: &str) -> Result<serde_json::Value> {
        self.post_json(
            "run-code",
            &RunCodeRequest {
                language: language.clone(),
                code: code.to_owned(),
            },
        )
        .await
    }
}

use crate::client::GrokClient;
use crate::error::Result;

impl GrokClient {
    pub async fn list_google_drive_files(&self, query: Option<&str>) -> Result<serde_json::Value> {
        #[derive(serde::Serialize)]
        struct Q<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            query: Option<&'a str>,
        }
        self.get_query_json("google-drive/files", &Q { query })
            .await
    }

    pub async fn read_google_drive_file(&self, id: &str) -> Result<serde_json::Value> {
        self.get_json(&format!("google-drive/files/{id}")).await
    }
}

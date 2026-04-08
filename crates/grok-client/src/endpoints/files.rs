use crate::client::GrokClient;
use crate::error::Result;
use crate::types::common::FileMetadataId;
use crate::types::files::{UploadFileRequest, UploadFileResponse};

impl GrokClient {
    pub async fn upload_file(&self, request: &UploadFileRequest) -> Result<UploadFileResponse> {
        self.post_json("upload-file", request).await
    }

    pub async fn get_file_metadata(&self, id: &FileMetadataId) -> Result<serde_json::Value> {
        self.post_json(&format!("file-metadata/{id}"), &serde_json::json!({}))
            .await
    }
}

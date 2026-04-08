use serde::{Deserialize, Serialize};

use super::common::{CompanionId, WorkspaceId};
use super::models::{DeepsearchPreset, ModelMode, ModelName};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<ModelName>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_attachments: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_attachments: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_search: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_image_generation: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_image_bytes: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_image_streaming: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_generation_count: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_concise: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_overrides: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_side_by_side: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_personality: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deepsearch_preset: Option<DeepsearchPreset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reasoning: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webpage_urls: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_edit_uris: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_artifact: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_memory: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_parent_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_mode: Option<ModelMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_response_cache: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_nsfw: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_regen_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub companion_id: Option<CompanionId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_override_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_skills: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NewConversationRequest {
    pub message: String,

    #[serde(flatten)]
    pub options: ChatOptions,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_raw_grok_in_xai_request: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_final_metadata: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_edit_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_ids: Option<Vec<WorkspaceId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_text_follow_ups: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_artifact_diff: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_force_trigger_artifact: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_from_grok_files: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_file_text_content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_file_text_content_start_position: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_file_text_content_end_position: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_side_by_side: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub models_user_can_use: Option<Vec<ModelName>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async_chat: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_kids_mode: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_fast_tools: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_self_harm_short_circuit: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectors: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_all_connectors: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_env_info: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_geo_location: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights_story_requests: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_personalization: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub imagine_project_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_cancel_current_inflight_requests: Option<bool>,
}

impl NewConversationRequest {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }

    #[must_use]
    pub fn builder(message: impl Into<String>) -> NewConversationRequestBuilder {
        NewConversationRequestBuilder(Self::new(message))
    }
}

pub struct NewConversationRequestBuilder(NewConversationRequest);

impl NewConversationRequestBuilder {
    #[must_use]
    pub fn model(mut self, model: ModelName) -> Self {
        self.0.options.model_name = Some(model);
        self
    }

    #[must_use]
    pub fn custom_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.0.options.custom_instructions = Some(instructions.into());
        self
    }

    #[must_use]
    pub fn temporary(mut self, temp: bool) -> Self {
        self.0.temporary = Some(temp);
        self
    }

    #[must_use]
    pub fn reasoning(mut self, enabled: bool) -> Self {
        self.0.options.is_reasoning = Some(enabled);
        self
    }

    #[must_use]
    pub fn image_generation(mut self, count: u32) -> Self {
        self.0.options.enable_image_generation = Some(true);
        self.0.options.image_generation_count = Some(count);
        self
    }

    #[must_use]
    pub fn disable_search(mut self) -> Self {
        self.0.options.disable_search = Some(true);
        self
    }

    #[must_use]
    pub fn disable_memory(mut self) -> Self {
        self.0.options.disable_memory = Some(true);
        self
    }

    #[must_use]
    pub fn deepsearch(mut self, preset: DeepsearchPreset) -> Self {
        self.0.options.deepsearch_preset = Some(preset);
        self
    }

    #[must_use]
    pub fn model_mode(mut self, mode: ModelMode) -> Self {
        self.0.options.model_mode = Some(mode);
        self
    }

    #[must_use]
    pub fn force_concise(mut self) -> Self {
        self.0.options.force_concise = Some(true);
        self
    }

    #[must_use]
    pub fn webpage_urls(mut self, urls: Vec<String>) -> Self {
        self.0.options.webpage_urls = Some(urls);
        self
    }

    #[must_use]
    pub fn nsfw(mut self, enabled: bool) -> Self {
        self.0.options.enable_nsfw = Some(enabled);
        self
    }

    #[must_use]
    pub fn build(self) -> NewConversationRequest {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickAnswerRequest {
    pub query: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AddResponseRequest {
    pub message: String,

    #[serde(flatten)]
    pub options: ChatOptions,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_response_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_quoted_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_response_id: Option<String>,

    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Map<String, serde_json::Value>>,
}

impl AddResponseRequest {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ..Default::default()
        }
    }
}

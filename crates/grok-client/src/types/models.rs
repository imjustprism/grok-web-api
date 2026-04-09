use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModelName {
    #[serde(rename = "grok-2")]
    Grok2,
    #[default]
    #[serde(rename = "grok-3")]
    Grok3,
    #[serde(rename = "grok-3-mini")]
    Grok3Mini,
    #[serde(rename = "grok-3-mini-fast")]
    Grok3MiniFast,
    #[serde(rename = "grok-4")]
    Grok4,
    #[serde(rename = "grok-4-mini")]
    Grok4Mini,
    #[serde(rename = "grok-4.1-fast-reasoning")]
    Grok41FastReasoning,
    #[serde(rename = "grok-4.1-fast-non-reasoning")]
    Grok41FastNonReasoning,
    #[serde(rename = "grok-4.20-0309-reasoning")]
    Grok420Reasoning,
    #[serde(rename = "grok-4.20-0309-non-reasoning")]
    Grok420NonReasoning,
    #[serde(rename = "grok-4.20-multi-agent-0309")]
    Grok420MultiAgent,
    #[serde(rename = "grok-code-fast-1")]
    GrokCodeFast1,
    #[serde(rename = "grok-imagine-image")]
    GrokImagineImage,
    #[serde(rename = "grok-imagine-image-pro")]
    GrokImagineImagePro,
    #[serde(rename = "grok-imagine-video")]
    GrokImagineVideo,
    #[serde(untagged)]
    Other(String),
}

impl ModelName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Grok2 => "grok-2",
            Self::Grok3 => "grok-3",
            Self::Grok3Mini => "grok-3-mini",
            Self::Grok3MiniFast => "grok-3-mini-fast",
            Self::Grok4 => "grok-4",
            Self::Grok4Mini => "grok-4-mini",
            Self::Grok41FastReasoning => "grok-4.1-fast-reasoning",
            Self::Grok41FastNonReasoning => "grok-4.1-fast-non-reasoning",
            Self::Grok420Reasoning => "grok-4.20-0309-reasoning",
            Self::Grok420NonReasoning => "grok-4.20-0309-non-reasoning",
            Self::Grok420MultiAgent => "grok-4.20-multi-agent-0309",
            Self::GrokCodeFast1 => "grok-code-fast-1",
            Self::GrokImagineImage => "grok-imagine-image",
            Self::GrokImagineImagePro => "grok-imagine-image-pro",
            Self::GrokImagineVideo => "grok-imagine-video",
            Self::Other(s) => s,
        }
    }
}

impl std::fmt::Display for ModelName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModelMode {
    #[serde(rename = "fast")]
    Fast,
    #[serde(rename = "expert")]
    Expert,
    #[serde(rename = "heavy")]
    Heavy,
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "grok-4-mini-thinking")]
    Grok4MiniThinking,
    #[serde(rename = "grok-4-1")]
    Grok41,
    #[serde(rename = "grok-4-1-thinking")]
    Grok41Thinking,
    #[serde(rename = "grok-4-1-nightly")]
    Grok41Nightly,
    #[serde(rename = "grok-420")]
    Grok420,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepsearchPreset {
    #[serde(rename = "deepsearch")]
    Deepsearch,
    #[serde(rename = "deepersearch")]
    Deepersearch,
    #[serde(rename = "think")]
    Think,
    #[serde(untagged)]
    Other(String),
}

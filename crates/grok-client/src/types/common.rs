use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

define_id!(ConversationId);

define_id!(ResponseId);

define_id!(ShareLinkId);

define_id!(ArtifactId);

define_id!(ArtifactVersionId);

define_id!(FileMetadataId);

define_id!(CompanionId);

define_id!(MemoryId);

define_id!(WorkspaceId);

define_id!(TemplateId);

define_id!(CollectionId);

define_id!(ConnectorId);

define_id!(ImagineProjectId);

define_id!(ModeId);

define_id!(VoiceId);

define_id!(GoogleDriveFileId);

define_id!(SharedArtifactId);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Sender {
    #[serde(rename = "human")]
    Human,
    #[serde(rename = "ASSISTANT")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ArtifactType {
    #[serde(rename = "code")]
    Code,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "mermaid")]
    Mermaid,
    #[serde(rename = "react")]
    React,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FollowUpType {
    #[serde(rename = "suggested")]
    Suggested,
    #[serde(rename = "manual")]
    Manual,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CodeLanguage {
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "javascript")]
    JavaScript,
    #[serde(rename = "typescript")]
    TypeScript,
    #[serde(rename = "rust")]
    Rust,
    #[serde(rename = "go")]
    Go,
    #[serde(rename = "bash")]
    Bash,
    #[serde(untagged)]
    Other(String),
}

impl fmt::Display for CodeLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => f.write_str("python"),
            Self::JavaScript => f.write_str("javascript"),
            Self::TypeScript => f.write_str("typescript"),
            Self::Rust => f.write_str("rust"),
            Self::Go => f.write_str("go"),
            Self::Bash => f.write_str("bash"),
            Self::Other(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    #[must_use]
    pub fn new(ts: impl Into<String>) -> Self {
        Self(ts.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Timestamp {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Timestamp {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

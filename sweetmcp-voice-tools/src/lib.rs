//! MCP Voice Tools - Tool definitions for voice operations
//!
//! This crate provides the MCP tool definitions for text-to-speech (TTS)
//! and speech-to-text (STT) operations, enabling LLMs to interact with
//! voice capabilities through a clean, intuitive interface.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod error;
pub mod protocol;
pub mod tools;
pub mod types;

// Re-export commonly used types
pub use error::{VoiceError, VoiceResult};
pub use protocol::{VoiceRequest, VoiceResponse};
pub use tools::{listen_tool, speak_tool};
pub use types::{ListenParams, ListenResult, SpeakParams, VoiceConfig};

/// MCP Tool definition structure (matching sweetmcp-axum types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: ToolInputSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub properties: HashMap<String, ToolInputSchemaProperty>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchemaProperty {
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub description: Option<String>,
}

/// Voice service trait that implementations must provide
#[async_trait::async_trait]
pub trait VoiceService: Send + Sync {
    /// Synthesize speech from text
    async fn speak(&self, params: SpeakParams) -> VoiceResult<()>;

    /// Listen for speech and transcribe to text
    async fn listen(&self, params: ListenParams) -> VoiceResult<ListenResult>;

    /// Get available voice IDs
    async fn list_voices(&self) -> VoiceResult<Vec<String>>;

    /// Get available microphone devices
    async fn list_microphones(&self) -> VoiceResult<Vec<String>>;
}

/// Tool registry helper
pub fn register_voice_tools() -> Vec<Tool> {
    vec![speak_tool(), listen_tool()]
}

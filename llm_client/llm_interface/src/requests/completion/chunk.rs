use super::response::CompletionFinishReason; // Import from sibling module
use serde::{Deserialize, Serialize};

/// Represents a chunk of a streaming completion response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionResponseChunk {
    /// The ID of the completion stream.
    pub id: String,
    /// The index of the choice (usually 0 for non-batched requests).
    pub index: u32,
    /// The text delta for this chunk, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_delta: Option<String>,
    /// The ID of the tool call, if this chunk relates to a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// The name of the tool being called, if this chunk relates to a tool call start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_name: Option<String>,
    /// The input delta for the tool call, if this chunk provides tool input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_input_delta: Option<String>,
    /// The reason the completion finished, if this is the final chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<CompletionFinishReason>,
    // Usage stats are typically aggregated at the end of the stream, not per chunk.
}

// Add Default implementation for easier state management in streams
impl Default for CompletionResponseChunk {
    fn default() -> Self {
        Self {
            id: String::new(),
            index: 0,
            text_delta: None,
            tool_call_id: None,
            tool_call_name: None,
            tool_call_input_delta: None,
            finish_reason: None,
        }
    }
}

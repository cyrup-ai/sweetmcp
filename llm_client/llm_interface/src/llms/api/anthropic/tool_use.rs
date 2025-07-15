use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Definition of a tool that Claude can use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// The name of the tool. Must match the regex `^[a-zA-Z0-9_-]{1,64}$`.
    pub name: String,
    /// A detailed plaintext description of what the tool does, when it should be used,
    /// and how it behaves.
    pub description: String,
    /// A JSON Schema object defining the expected parameters for the tool.
    pub input_schema: JsonValue,
}

/// Controls how Claude decides which tool(s) to use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// Claude decides whether to call any provided tools or not. (Default when tools are provided).
    Auto,
    /// Claude must use one of the provided tools.
    Any,
    /// Claude must use the specified tool.
    Tool { name: String },
    // Note: `none` type is represented by omitting the `tool_choice` field in the request.
}

/// Represents Claude's request to use a specific tool. Found in assistant messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolUseBlock {
    /// The type of content block.
    #[serde(rename = "type")]
    pub block_type: String, // Should always be "tool_use"
    /// A unique identifier for this specific tool use block.
    pub id: String,
    /// The name of the tool being used.
    pub name: String,
    /// An object containing the input being passed to the tool, conforming to the tool's `input_schema`.
    pub input: JsonValue,
}

/// Represents the result of a tool execution. Sent in user messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResultBlock {
    /// The type of content block.
    #[serde(rename = "type")]
    pub block_type: String, // Should always be "tool_result"
    /// The `id` of the tool use request this is a result for.
    pub tool_use_id: String,
    /// The result of the tool, as a string.
    /// Anthropic also supports list of content blocks (text/image) here, using String for now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Set to `true` if the tool execution resulted in an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolResultBlock {
    /// Creates a new successful tool result block.
    pub fn success(tool_use_id: String, content: String) -> Self {
        Self {
            block_type: "tool_result".to_string(),
            tool_use_id,
            content: Some(content),
            is_error: None,
        }
    }

    /// Creates a new error tool result block.
    pub fn error(tool_use_id: String, error_message: String) -> Self {
        Self {
            block_type: "tool_result".to_string(),
            tool_use_id,
            content: Some(error_message),
            is_error: Some(true),
        }
    }

    /// Creates a new empty tool result block (valid according to docs).
    pub fn empty(tool_use_id: String) -> Self {
        Self {
            block_type: "tool_result".to_string(),
            tool_use_id,
            content: None,
            is_error: None,
        }
    }
}

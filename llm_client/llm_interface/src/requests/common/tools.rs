use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Errors that can occur during tool operations
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid tool name: {0}")]
    InvalidName(String),
    
    #[error("Invalid tool schema: {0}")]
    InvalidSchema(String),
    
    #[error("Provider doesn't support tool calling: {0}")]
    UnsupportedProvider(String),
    
    #[error("Tool execution error: {0}")]
    ExecutionError(String),
    
    #[error("Tool validation error: {0}")]
    ValidationError(String),
}

/// Represents the definition of a tool that the LLM can call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    /// The name of the tool. Should be specific and follow backend naming constraints
    /// (e.g., `^[a-zA-Z0-9_-]{1,64}$` for Anthropic).
    pub name: String,
    /// A detailed description of the tool's purpose, parameters, and when to use it.
    pub description: String,
    /// The schema (e.g., JSON Schema) defining the input parameters for the tool.
    /// Using `serde_json::Value` for flexibility across backends.
    pub input_schema: JsonValue,
}

impl ToolDefinition {
    /// Creates a new tool definition with validation.
    pub fn new(
        name: String,
        description: String,
        input_schema: JsonValue,
    ) -> Result<Self, ToolError> {
        // Validate tool name (alphanumeric, underscore, hyphen)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ToolError::InvalidName(
                "Tool name must contain only alphanumeric characters, underscores, or hyphens"
                    .to_string(),
            ));
        }

        // Ensure description is not empty
        if description.trim().is_empty() {
            return Err(ToolError::ValidationError(
                "Tool description cannot be empty".to_string(),
            ));
        }

        // Basic schema validation - ensure it's an object
        if !input_schema.is_object() {
            return Err(ToolError::InvalidSchema(
                "Tool schema must be a valid JSON object".to_string(),
            ));
        }

        Ok(Self {
            name,
            description,
            input_schema,
        })
    }

    /// Create a simple function tool with parameters
    pub fn function<S: Into<String>>(
        name: S,
        description: S,
        parameters: HashMap<String, JsonValue>,
        required: Vec<String>,
    ) -> Result<Self, ToolError> {
        let name = name.into();
        let description = description.into();

        // Build JSON Schema for function parameters
        let mut schema = serde_json::Map::new();
        schema.insert("type".to_string(), JsonValue::String("object".to_string()));
        schema.insert("properties".to_string(), JsonValue::Object(parameters.into_iter().collect()));
        schema.insert("required".to_string(), JsonValue::Array(required.into_iter().map(JsonValue::String).collect()));

        Self::new(name, description, JsonValue::Object(schema))
    }
}

/// Controls how the model decides which tool(s) to use, if any.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// The model decides whether to call a tool or not (default behavior when tools are present).
    Auto,
    /// The model *must* call one of the provided tools.
    Any,
    /// The model *must* call the specified tool.
    Tool { name: String },
    /// The model must *not* call any tool (default behavior when no tools are present).
    None,
}

/// Represents a request from the model to call a specific tool.
/// This might be part of the `CompletionResponse` in the future.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// A unique identifier for this specific tool call instance.
    pub id: String,
    /// The name of the tool being called.
    pub name: String,
    /// The arguments (input) for the tool, typically as a JSON string or object.
    /// Using `serde_json::Value` for flexibility.
    pub arguments: JsonValue,
}

/// A summarized version of a tool call for use in completion responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallSummary {
    /// The name of the tool that was called
    pub name: String,
    /// The input provided to the tool
    pub input: JsonValue,
}

/// Represents the result of executing a tool, to be sent back to the model.
/// This might be part of the `PromptMessage` structure in the future.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// The unique identifier of the tool call this result corresponds to.
    pub tool_call_id: String,
    /// The output/result from the tool execution.
    pub content: String, // Or potentially richer content like JSON/images
    /// Flag indicating if the tool execution resulted in an error.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_error: bool,
}

impl ToolResult {
    /// Create a successful tool result
    pub fn success<S: Into<String>>(tool_call_id: S, content: S) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error<S: Into<String>>(tool_call_id: S, error_message: S) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: error_message.into(),
            is_error: true,
        }
    }

    /// Convert a Result to a ToolResult
    pub fn from_result<T, E: std::fmt::Display>(tool_call_id: String, result: Result<T, E>) -> Self 
    where
        T: serde::Serialize,
    {
        match result {
            Ok(value) => Self::success(
                tool_call_id,
                serde_json::to_string(&value).unwrap_or_else(|e| format!("{{\"error\":\"Serialization error: {}\"}}", e)),
            ),
            Err(e) => Self::error(tool_call_id, e.to_string()),
        }
    }
}

// Helper for serde default skip
fn is_false(v: &bool) -> bool {
    !*v
}

/// Represents a collection of tools and tool calling configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Tools {
    /// The list of tool definitions the LLM can use.
    pub definitions: Vec<ToolDefinition>,
    /// Controls how the model decides which tool(s) to use, if any.
    pub choice: Option<ToolChoice>,
}

impl Tools {
    /// Create a new empty tools collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tool definition to the collection
    pub fn add_tool(&mut self, tool: ToolDefinition) {
        self.definitions.push(tool);
    }

    /// Set the tool choice mode
    pub fn set_choice(&mut self, choice: ToolChoice) {
        self.choice = Some(choice);
    }

    /// Validates all tool definitions against provider-specific constraints.
    /// `provider` parameter identifies which provider's rules to check against
    pub fn validate_for_provider(&self, provider: &str) -> Result<(), ToolError> {
        match provider.to_lowercase().as_str() {
            "openai" | "generic_openai" => {
                // OpenAI specific validation
                for tool in &self.definitions {
                    if tool.name.len() > 64 {
                        return Err(ToolError::InvalidName(
                            format!("OpenAI tool name exceeds 64 characters: {}", tool.name)
                        ));
                    }
                }
            }
            "anthropic" => {
                // Anthropic specific validation
                for tool in &self.definitions {
                    if tool.name.len() > 64 {
                        return Err(ToolError::InvalidName(
                            format!("Anthropic tool name exceeds 64 characters: {}", tool.name)
                        ));
                    }
                    
                    // Check for Anthropic's regex pattern requirement
                    if !tool.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                        return Err(ToolError::InvalidName(
                            format!("Anthropic tool name must match regex ^[a-zA-Z0-9_-]{{1,64}}$: {}", tool.name)
                        ));
                    }
                }
            }
            "perplexity" => {
                // Perplexity doesn't support tool calling
                if !self.definitions.is_empty() {
                    return Err(ToolError::UnsupportedProvider(
                        "Perplexity API does not support tool calling".to_string(),
                    ));
                }
            }
            _ => { /* No specific validation for other providers */ }
        }

        Ok(())
    }

    /// Check if tools are being used
    pub fn has_tools(&self) -> bool {
        !self.definitions.is_empty()
    }
}

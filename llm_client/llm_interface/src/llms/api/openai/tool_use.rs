use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Definition of a tool that OpenAI models can use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// The type of the tool. Currently only "function" is supported.
    pub r#type: ToolType,
    /// The function definition.
    pub function: FunctionDefinition,
}

/// Type of the tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// A tool that calls a function
    Function,
}

/// Definition of a function that OpenAI can call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDefinition {
    /// The name of the function. Must be alphanumeric characters, periods, underscores, or hyphens.
    pub name: String,
    /// A description of what the function does. The model uses this to decide when and how to call the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The parameters the function accepts, described as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<JsonValue>,
}

/// Controls how the model uses tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolChoice {
    /// The model will automatically decide whether to use tools or respond directly.
    Auto,
    /// The model will not use any tools, even if specified.
    None,
    /// The model will use any of the available tools.
    Any,
    /// The model will use a specific tool.
    #[serde(untagged)]
    Tool {
        /// Specify that the model should use a particular tool.
        #[serde(rename = "type")]
        tool_type: String, // always "function"
        /// The specific function to use.
        function: FunctionChoiceObject,
    },
}

/// Specifies which function to use when tool_choice is set to a specific function.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionChoiceObject {
    /// The name of the function to call.
    pub name: String,
}

/// Represents a tool call made by the model in a response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// The ID of the tool call.
    pub id: String,
    /// The type of tool call. Currently only "function" is supported.
    #[serde(rename = "type")]
    pub call_type: String, // always "function"
    /// The function that was called.
    pub function: FunctionCall,
}

/// Represents a function call made by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// The arguments to call the function with, as a JSON string.
    pub arguments: String,
}

/// Represents a tool result to be sent back to the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// The type of message, which should be "tool" for tool results.
    pub role: String, // always "tool"
    /// The content of the tool result.
    pub content: String,
    /// The ID of the tool call this result is responding to.
    pub tool_call_id: String,
}

// Conversion functions between common tool interface and OpenAI-specific formats
use crate::requests::common::tools::{ToolCall as CommonToolCall, ToolDefinition, ToolChoice as CommonToolChoice, ToolResult as CommonToolResult};

impl From<ToolDefinition> for Tool {
    fn from(tool_def: ToolDefinition) -> Self {
        Self {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: tool_def.name,
                description: Some(tool_def.description),
                parameters: Some(tool_def.input_schema),
            },
        }
    }
}

impl From<CommonToolChoice> for ToolChoice {
    fn from(choice: CommonToolChoice) -> Self {
        match choice {
            CommonToolChoice::Auto => ToolChoice::Auto,
            CommonToolChoice::None => ToolChoice::None,
            CommonToolChoice::Any => ToolChoice::Any,
            CommonToolChoice::Tool { name } => ToolChoice::Tool {
                tool_type: "function".to_string(),
                function: FunctionChoiceObject { name },
            },
        }
    }
}

impl From<ToolCall> for CommonToolCall {
    fn from(tool_call: ToolCall) -> Self {
        // Parse the arguments string to a JsonValue
        let arguments = serde_json::from_str(&tool_call.function.arguments)
            .unwrap_or_else(|_| JsonValue::Object(serde_json::Map::new()));

        CommonToolCall {
            id: tool_call.id,
            name: tool_call.function.name,
            arguments,
        }
    }
}

impl From<CommonToolResult> for ToolResult {
    fn from(result: CommonToolResult) -> Self {
        Self {
            role: "tool".to_string(),
            content: result.content,
            tool_call_id: result.tool_call_id,
        }
    }
}

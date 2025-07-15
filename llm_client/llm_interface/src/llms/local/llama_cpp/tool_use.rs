use crate::requests::common::tools::{ToolCall, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;


/// Represents a function tool for llama_cpp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlamaCppTool {
    /// Name of the tool
    pub name: String,
    /// Description of the tool
    pub description: String,
    /// Parameters for the tool in JSON Schema format
    pub parameters: JsonValue,
    /// Whether this tool can be called by the model
    #[serde(default)]
    pub enabled: bool,
}

/// Represents a tool call in the llama_cpp response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlamaCppToolCall {
    /// The name of the tool that was called
    pub name: String,
    /// The parameters passed to the tool
    pub arguments: JsonValue,
    /// A unique identifier for this tool call
    #[serde(default = "generate_id")]
    pub id: String,
}

// Helper function to generate a simple random ID
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("call_{}", now)
}

impl From<&ToolDefinition> for LlamaCppTool {
    fn from(tool: &ToolDefinition) -> Self {
        Self {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.input_schema.clone(),
            enabled: true,
        }
    }
}

impl From<&LlamaCppToolCall> for ToolCall {
    fn from(tool_call: &LlamaCppToolCall) -> Self {
        Self {
            id: tool_call.id.clone(),
            name: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
        }
    }
}

/// Convert a list of common ToolDefinition objects to LlamaCppTool objects
pub fn convert_tools(tools: &[ToolDefinition]) -> Vec<LlamaCppTool> {
    tools.iter().map(LlamaCppTool::from).collect()
}

/// Parse tool calls from a string, extracting valid JSON objects representing tool calls
pub fn parse_tool_calls(content: &str) -> Vec<LlamaCppToolCall> {
    // Simple regex-based extraction for tool call format
    // This is a basic implementation - in a production environment, you might want
    // a more robust parsing method based on the specific llama_cpp output format
    
    let mut tool_calls = Vec::new();
    
    // Look for patterns like: <tool:name>{"param": "value"}</tool>
    // or function call format like: name({"param": "value"})
    if let Some(tool_start) = content.find("<tool:") {
        if let Some(tool_end) = content[tool_start..].find("</tool>") {
            let tool_content = &content[tool_start..tool_start + tool_end + 7];
            
            // Extract name and arguments
            if let (Some(name_start), Some(args_start)) = 
                (tool_content.find("<tool:"), tool_content.find(">")) {
                let name = tool_content[name_start + 6..args_start].trim().to_string();
                let args_str = tool_content[args_start + 1..tool_content.len() - 7].trim();
                
                if let Ok(args) = serde_json::from_str::<JsonValue>(args_str) {
                    tool_calls.push(LlamaCppToolCall {
                        name,
                        arguments: args,
                        id: generate_id(),
                    });
                }
            }
        }
    }
    
    // Check for function call format
    let re = regex::Regex::new(r#"(\w+)\s*\(\s*(\{.*?\})\s*\)"#).unwrap();
    for capture in re.captures_iter(content) {
        if let (Some(name_match), Some(args_match)) = (capture.get(1), capture.get(2)) {
            let name = name_match.as_str().to_string();
            let args_str = args_match.as_str();
            
            if let Ok(args) = serde_json::from_str::<JsonValue>(args_str) {
                tool_calls.push(LlamaCppToolCall {
                    name,
                    arguments: args,
                    id: generate_id(),
                });
            }
        }
    }
    
    tool_calls
}

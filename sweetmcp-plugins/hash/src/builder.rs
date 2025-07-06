// MCP Plugin Builder - Creates the call() and describe() functions fluently

use crate::plugin::types::*;
use extism_pdk::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Fluent builder for MCP plugin definition
pub struct PluginBuilder {
    tools: HashMap<String, ToolDefinition>,
}

/// Definition of a single tool
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub handler: Box<dyn Fn(Value) -> Result<CallToolResult, Error> + Send + Sync>,
}

impl PluginBuilder {
    /// Start building a new plugin
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Add a tool with fluent interface
    pub fn tool<F>(mut self, name: &str, description: &str, schema: Value, handler: F) -> Self 
    where
        F: Fn(Value) -> Result<CallToolResult, Error> + Send + Sync + 'static,
    {
        self.tools.insert(name.to_string(), ToolDefinition {
            name: name.to_string(),
            description: description.to_string(),
            input_schema: schema,
            handler: Box::new(handler),
        });
        self
    }

    /// Generate the call() function
    pub fn build_call_handler(&self) -> impl Fn(CallToolRequest) -> Result<CallToolResult, Error> + '_ {
        move |input: CallToolRequest| {
            let tool_name = &input.params.name;
            let args = input.params.arguments.unwrap_or_default();

            match self.tools.get(tool_name) {
                Some(tool) => (tool.handler)(Value::Object(args)),
                None => Err(Error::msg(format!("Unknown tool: {}", tool_name))),
            }
        }
    }

    /// Generate the describe() function
    pub fn build_describe_handler(&self) -> impl Fn() -> Result<ListToolsResult, Error> + '_ {
        move || {
            let tools = self.tools.values().map(|tool| {
                ToolDescription {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.as_object()
                        .expect("Schema should be object")
                        .clone(),
                }
            }).collect();

            Ok(ListToolsResult { tools })
        }
    }
}

/// Helper to create text content
pub fn text_content(text: impl Into<String>) -> CallToolResult {
    CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text.into()),
            mime_type: Some("text/plain".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

/// Helper to create error content
pub fn error_content(error: impl Into<String>) -> CallToolResult {
    CallToolResult {
        is_error: Some(true),
        content: vec![Content {
            annotations: None,
            text: Some(error.into()),
            mime_type: Some("text/plain".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    }
}

/// Macro to make schema definition easier
#[macro_export]
macro_rules! schema {
    (
        properties: {
            $($prop:ident: {
                type: $prop_type:expr,
                description: $prop_desc:expr
                $(, enum: [$($enum_val:expr),*])?
                $(,)?
            }),* $(,)?
        }
        $(, required: [$($req:ident),*])?
    ) => {
        json!({
            "type": "object",
            "properties": {
                $(
                    stringify!($prop): {
                        "type": $prop_type,
                        "description": $prop_desc
                        $(, "enum": [$($enum_val),*])?
                    }
                ),*
            }
            $(, "required": [$(stringify!($req)),*])?
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_macro() {
        let schema = schema! {
            properties: {
                data: {
                    type: "string",
                    description: "Input data"
                },
                algorithm: {
                    type: "string", 
                    description: "Hash algorithm",
                    enum: ["sha256", "md5"]
                }
            },
            required: [data, algorithm]
        };

        assert!(schema.is_object());
        assert!(schema["properties"]["data"]["type"] == "string");
    }
}
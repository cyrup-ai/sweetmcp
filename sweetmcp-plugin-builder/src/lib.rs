//! Sexy fluent builder for MCP plugins
//!
//! No `new()`, no boilerplate, just pure fluent chaining with closures

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::marker::PhantomData;

pub mod prelude {
    pub use super::{
        ContentBuilder, DescriptionBuilder, McpPlugin, McpTool, SchemaBuilder, mcp_plugin,
    };
}

// MCP protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub params: CallToolParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    pub arguments: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub r#type: ContentType,
    pub text: Option<String>,
    pub mime_type: Option<String>,
    pub data: Option<String>,
    pub annotations: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ContentType {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<ToolDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Type states for compile-time safety
pub struct Empty;
pub struct Named;
pub struct Described;
pub struct Ready;

/// The sexy MCP plugin builder
pub struct McpPlugin<State = Empty> {
    name: Option<String>,
    description: Option<String>,
    tools: Vec<ToolDef>,
    _state: PhantomData<State>,
}

struct ToolDef {
    name: String,
    description: String,
    schema: Value,
    handler: Box<dyn Fn(Value) -> Result<CallToolResult, Error> + Send + Sync>,
}

/// Entry point - no `new()` needed!
pub fn mcp_plugin(name: impl Into<String>) -> McpPlugin<Named> {
    McpPlugin {
        name: Some(name.into()),
        description: None,
        tools: Vec::new(),
        _state: PhantomData,
    }
}

impl McpPlugin<Named> {
    /// Describe the plugin fluently
    pub fn description(self, desc: impl Into<String>) -> McpPlugin<Described> {
        McpPlugin {
            name: self.name,
            description: Some(desc.into()),
            tools: self.tools,
            _state: PhantomData,
        }
    }
}

impl McpPlugin<Described> {
    /// Register a tool with const-generic type
    pub fn tool<T: McpTool>(mut self) -> Self {
        let description = T::description(DescriptionBuilder::default());
        self.tools.push(ToolDef {
            name: T::NAME.to_string(),
            description: description.build(),
            schema: T::schema(SchemaBuilder::default()),
            handler: Box::new(T::execute),
        });
        self
    }

    /// Ready to serve MCP clients
    pub fn serve(self) -> McpPlugin<Ready> {
        McpPlugin {
            name: self.name,
            description: self.description,
            tools: self.tools,
            _state: PhantomData,
        }
    }
}

impl McpPlugin<Ready> {
    /// Handle incoming MCP calls
    pub fn call(&self, request: CallToolRequest) -> Result<CallToolResult, Error> {
        let tool_name = &request.params.name;
        let args = request.params.arguments.unwrap_or_default();

        for tool in &self.tools {
            if tool.name == *tool_name {
                return (tool.handler)(Value::Object(args));
            }
        }

        Err(Error::msg(format!("Tool '{}' not found", tool_name)))
    }

    /// Describe available tools
    pub fn describe(&self) -> Result<ListToolsResult, Error> {
        let tools = self
            .tools
            .iter()
            .map(|tool| ToolDescription {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.schema.clone(),
            })
            .collect();

        Ok(ListToolsResult { tools })
    }
}

/// Tool trait with fluent description
pub trait McpTool: Send + Sync + 'static {
    const NAME: &'static str;

    fn description(builder: DescriptionBuilder) -> DescriptionBuilder;
    fn schema(builder: SchemaBuilder) -> Value;
    fn execute(args: Value) -> Result<CallToolResult, Error>;
}

/// Fluent description builder
#[derive(Default)]
pub struct DescriptionBuilder {
    primary_function: Option<String>,
    use_cases: Vec<String>,
    perfect_for: Vec<String>,
    operations: Vec<(String, String)>,
    prerequisites: Vec<String>,
    limitations: Vec<String>,
    always_use_for: Vec<String>,
}

impl DescriptionBuilder {
    /// Primary function - starts with verb
    pub fn does(mut self, action: impl Into<String>) -> Self {
        self.primary_function = Some(action.into());
        self
    }

    /// When to use this tool
    pub fn when(mut self, use_case: impl Into<String>) -> Self {
        self.use_cases.push(use_case.into());
        self
    }

    /// Perfect for these scenarios
    pub fn perfect_for(mut self, scenario: impl Into<String>) -> Self {
        self.perfect_for.push(scenario.into());
        self
    }

    /// Multi-operation tools
    pub fn operation(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.operations.push((name.into(), desc.into()));
        self
    }

    /// Prerequisites
    pub fn requires(mut self, prereq: impl Into<String>) -> Self {
        self.prerequisites.push(prereq.into());
        self
    }

    /// When NOT to use
    pub fn not_for(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self
    }

    /// Always use for
    pub fn always_for(mut self, scenario: impl Into<String>) -> Self {
        self.always_use_for.push(scenario.into());
        self
    }

    /// Build the description following MCP best practices
    pub fn build(self) -> String {
        let mut parts = Vec::new();

        if let Some(primary) = self.primary_function {
            parts.push(format!("{}.", primary));
        }

        if !self.operations.is_empty() {
            let ops = self
                .operations
                .iter()
                .map(|(name, desc)| format!("- `{}`: {}", name, desc))
                .collect::<Vec<_>>()
                .join("\n");
            parts.push(format!("It provides the following operations:\n{}", ops));
        }

        if !self.use_cases.is_empty() {
            parts.push(format!(
                "Use this tool when you need to:\n- {}",
                self.use_cases.join("\n- ")
            ));
        }

        if !self.always_use_for.is_empty() {
            parts.push(format!(
                "Always use this tool to {}",
                self.always_use_for.join(", ")
            ));
        }

        if !self.prerequisites.is_empty() {
            parts.push(format!("NOTE: {}", self.prerequisites.join(". ")));
        }

        if !self.limitations.is_empty() {
            parts.push(format!("Not suitable for: {}", self.limitations.join("; ")));
        }

        if !self.perfect_for.is_empty() {
            parts.push(format!("Perfect for {}.", self.perfect_for.join(", ")));
        }

        parts.join(" ")
    }
}

/// Fluent schema builder
#[derive(Default)]
pub struct SchemaBuilder {
    properties: serde_json::Map<String, Value>,
    required: Vec<String>,
}

impl SchemaBuilder {
    /// Required string parameter
    pub fn required_string(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            serde_json::json!({
                "type": "string",
                "description": desc.into()
            }),
        );
        self.required.push(name);
        self
    }

    /// Optional string parameter
    pub fn optional_string(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.properties.insert(
            name.into(),
            serde_json::json!({
                "type": "string",
                "description": desc.into()
            }),
        );
        self
    }

    /// Required enum parameter
    pub fn required_enum(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
        options: &[&str],
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            serde_json::json!({
                "type": "string",
                "description": desc.into(),
                "enum": options
            }),
        );
        self.required.push(name);
        self
    }

    /// Optional enum parameter
    pub fn optional_enum(
        mut self,
        name: impl Into<String>,
        desc: impl Into<String>,
        options: &[&str],
    ) -> Self {
        self.properties.insert(
            name.into(),
            serde_json::json!({
                "type": "string",
                "description": desc.into(),
                "enum": options
            }),
        );
        self
    }

    /// Optional boolean parameter
    pub fn optional_bool(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.properties.insert(
            name.into(),
            serde_json::json!({
                "type": "boolean",
                "description": desc.into()
            }),
        );
        self
    }

    /// Optional number parameter
    pub fn optional_number(mut self, name: impl Into<String>, desc: impl Into<String>) -> Self {
        self.properties.insert(
            name.into(),
            serde_json::json!({
                "type": "number",
                "description": desc.into()
            }),
        );
        self
    }

    /// Build the schema
    pub fn build(self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": self.properties,
            "required": self.required
        })
    }
}

/// Content builder for responses
pub struct ContentBuilder;

impl ContentBuilder {
    /// Successful text response
    pub fn text(content: impl Into<String>) -> CallToolResult {
        CallToolResult {
            is_error: Some(false),
            content: vec![Content {
                r#type: ContentType::Text,
                text: Some(content.into()),
                mime_type: Some("text/plain".into()),
                data: None,
                annotations: None,
            }],
        }
    }

    /// Error response
    pub fn error(message: impl Into<String>) -> CallToolResult {
        CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                r#type: ContentType::Text,
                text: Some(message.into()),
                mime_type: Some("text/plain".into()),
                data: None,
                annotations: None,
            }],
        }
    }

    /// Base64 data response
    pub fn data(data: impl Into<String>, mime_type: impl Into<String>) -> CallToolResult {
        CallToolResult {
            is_error: Some(false),
            content: vec![Content {
                r#type: ContentType::Image,
                text: None,
                data: Some(data.into()),
                mime_type: Some(mime_type.into()),
                annotations: None,
            }],
        }
    }
}

/// Generate standard MCP entry points for your plugin
#[macro_export]
macro_rules! generate_mcp_functions {
    ($plugin_fn:ident) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn call() -> i32 {
            let input: $crate::CallToolRequest = $crate::try_input_json!();
            let result = $plugin_fn().call(input);
            match result.and_then(|x| ::extism_pdk::output(::extism_pdk::Json(x))) {
                Ok(()) => 0,
                Err(e) => {
                    let err = format!("{:?}", e);
                    if let Ok(mem) = ::extism_pdk::Memory::from_bytes(err.as_bytes()) {
                        unsafe {
                            ::extism_pdk::extism::error_set(mem.offset());
                        }
                    }
                    -1
                }
            }
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn describe() -> i32 {
            let result = $plugin_fn().describe();
            match result.and_then(|x| ::extism_pdk::output(::extism_pdk::Json(x))) {
                Ok(()) => 0,
                Err(e) => {
                    let err = format!("{:?}", e);
                    if let Ok(mem) = ::extism_pdk::Memory::from_bytes(err.as_bytes()) {
                        unsafe {
                            ::extism_pdk::extism::error_set(mem.offset());
                        }
                    }
                    -1
                }
            }
        }
    };
}

#[macro_export]
macro_rules! try_input_json {
    () => {{
        let x = ::extism_pdk::input();
        match x {
            Ok(::extism_pdk::Json(x)) => x,
            Err(e) => {
                let err = format!("{:?}", e);
                if let Ok(mem) = ::extism_pdk::Memory::from_bytes(err.as_bytes()) {
                    unsafe {
                        ::extism_pdk::extism::error_set(mem.offset());
                    }
                }
                return -1;
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    impl McpTool for TestTool {
        const NAME: &'static str = "test";

        fn description(builder: DescriptionBuilder) -> DescriptionBuilder {
            builder
                .does("Test tool functionality")
                .when("Running tests")
                .perfect_for("testing")
        }

        fn schema(builder: SchemaBuilder) -> Value {
            builder.required_string("input", "Test input").build()
        }

        fn execute(args: Value) -> Result<CallToolResult, Error> {
            Ok(ContentBuilder::text("Test result"))
        }
    }

    #[test]
    fn test_fluent_plugin_builder() {
        let plugin = mcp_plugin("test-plugin")
            .description("A test plugin")
            .tool::<TestTool>()
            .serve();

        let tools = plugin.describe().unwrap();
        assert_eq!(tools.tools.len(), 1);
        assert_eq!(tools.tools[0].name, "test");
    }
}

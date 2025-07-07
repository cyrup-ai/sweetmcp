// Sexy fluent builder with proper semantics and state progression

use crate::plugin::types::*;
use extism_pdk::Error;
use serde_json::Value;
use std::marker::PhantomData;

/// State markers for type-safe progression
pub struct Empty;
pub struct Named;
pub struct Described;
pub struct Ready;

/// MCP Plugin fluent builder with state progression
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

/// Tool trait for const-generic registration
pub trait McpTool: Send + Sync + 'static {
    const NAME: &'static str;

    fn description() -> String;
    fn schema() -> Value;
    fn execute(args: Value) -> Result<CallToolResult, Error>;
}

impl McpPlugin<Empty> {
    /// Start defining an MCP plugin
    pub fn named(name: impl Into<String>) -> McpPlugin<Named> {
        McpPlugin {
            name: Some(name.into()),
            description: None,
            tools: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl McpPlugin<Named> {
    /// Describe what this plugin does
    pub fn described(mut self, description: impl Into<String>) -> McpPlugin<Described> {
        self.description = Some(description.into());
        McpPlugin {
            name: self.name,
            description: self.description,
            tools: self.tools,
            _state: PhantomData,
        }
    }
}

impl McpPlugin<Described> {
    /// Register a tool using const-generic type registration
    pub fn provides<T: McpTool>(mut self) -> Self {
        self.tools.push(ToolDef {
            name: T::NAME.to_string(),
            description: T::DESCRIPTION.to_string(),
            schema: T::schema(),
            handler: Box::new(T::execute),
        });
        self
    }

    /// Expose this plugin to MCP clients
    pub fn expose(self) -> McpPlugin<Ready> {
        McpPlugin {
            name: self.name,
            description: self.description,
            tools: self.tools,
            _state: PhantomData,
        }
    }
}

impl McpPlugin<Ready> {
    /// Handle incoming tool calls
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

    /// Describe available tools to MCP clients
    pub fn describe(&self) -> Result<ListToolsResult, Error> {
        let tools = self
            .tools
            .iter()
            .map(|tool| ToolDescription {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool
                    .schema
                    .as_object()
                    .expect("Tool schema must be object")
                    .clone(),
            })
            .collect();

        Ok(ListToolsResult { tools })
    }
}

/// Description builder with semantic methods following MCP best practices
pub struct DescriptionBuilder {
    primary_function: Option<String>,
    use_cases: Vec<String>,
    perfect_for: Vec<String>,
    operations: Vec<(String, String)>, // For multi-operation tools
    prerequisites: Vec<String>,
    limitations: Vec<String>,
    always_use_for: Vec<String>,
}

impl DescriptionBuilder {
    pub fn new() -> Self {
        Self {
            primary_function: None,
            use_cases: Vec::new(),
            perfect_for: Vec::new(),
            operations: Vec::new(),
            prerequisites: Vec::new(),
            limitations: Vec::new(),
            always_use_for: Vec::new(),
        }
    }

    /// Primary function - starts with verb (Generate, Validate, Retrieve, etc.)
    pub fn does(mut self, action: impl Into<String>) -> Self {
        self.primary_function = Some(action.into());
        self
    }

    /// When you should use this tool (best practice: 3-5 specific use cases)
    pub fn when(mut self, use_case: impl Into<String>) -> Self {
        self.use_cases.push(use_case.into());
        self
    }

    /// What this tool is perfect for (value proposition)
    pub fn perfect_for(mut self, scenario: impl Into<String>) -> Self {
        self.perfect_for.push(scenario.into());
        self
    }

    /// For multi-operation tools, describe each operation
    pub fn operation(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.operations.push((name.into(), description.into()));
        self
    }

    /// Prerequisites or requirements
    pub fn requires(mut self, prerequisite: impl Into<String>) -> Self {
        self.prerequisites.push(prerequisite.into());
        self
    }

    /// Limitations or when NOT to use
    pub fn not_for(mut self, limitation: impl Into<String>) -> Self {
        self.limitations.push(limitation.into());
        self
    }

    /// Always use this tool for these scenarios
    pub fn always_for(mut self, scenario: impl Into<String>) -> Self {
        self.always_use_for.push(scenario.into());
        self
    }

    /// Build following MCP best practices format
    pub fn build(self) -> String {
        let mut parts = Vec::new();

        // Primary function (required)
        if let Some(primary) = self.primary_function {
            parts.push(format!("{}.", primary));
        }

        // Multi-operation tools get special treatment
        if !self.operations.is_empty() {
            let ops = self
                .operations
                .iter()
                .map(|(name, desc)| format!("- `{}`: {}", name, desc))
                .collect::<Vec<_>>()
                .join("\n");
            parts.push(format!("It provides the following operations:\n{}", ops));
        }

        // Use cases (best practice: always include)
        if !self.use_cases.is_empty() {
            parts.push(format!(
                "Use this tool when you need to:\n- {}",
                self.use_cases.join("\n- ")
            ));
        }

        // Always use for (when specified)
        if !self.always_use_for.is_empty() {
            parts.push(format!(
                "Always use this tool to {}",
                self.always_use_for.join(", ")
            ));
        }

        // Prerequisites (if any)
        if !self.prerequisites.is_empty() {
            parts.push(format!("NOTE: {}", self.prerequisites.join(". ")));
        }

        // Limitations (if any)
        if !self.limitations.is_empty() {
            parts.push(format!("Not suitable for: {}", self.limitations.join("; ")));
        }

        // Perfect for (value proposition - always end with this)
        if !self.perfect_for.is_empty() {
            parts.push(format!("Perfect for {}.", self.perfect_for.join(", ")));
        }

        parts.join(" ")
    }
}

/// Content builders with proper semantics
pub struct ContentBuilder;

impl ContentBuilder {
    /// Return successful text result
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

    /// Return error result
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

    /// Return base64 encoded data
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

/// Schema builder with fluent semantics
pub struct SchemaBuilder {
    properties: serde_json::Map<String, Value>,
    required: Vec<String>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            properties: serde_json::Map::new(),
            required: Vec::new(),
        }
    }

    /// Add a required string parameter
    pub fn requires_string(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            serde_json::json!({
                "type": "string",
                "description": description.into()
            }),
        );
        self.required.push(name);
        self
    }

    /// Add an optional string parameter
    pub fn accepts_string(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.properties.insert(
            name.into(),
            serde_json::json!({
                "type": "string",
                "description": description.into()
            }),
        );
        self
    }

    /// Add a required enum parameter
    pub fn requires_enum(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        options: &[&str],
    ) -> Self {
        let name = name.into();
        self.properties.insert(
            name.clone(),
            serde_json::json!({
                "type": "string",
                "description": description.into(),
                "enum": options
            }),
        );
        self.required.push(name);
        self
    }

    /// Build the JSON schema
    pub fn build(self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": self.properties,
            "required": self.required
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    impl McpTool for TestTool {
        const NAME: &'static str = "test";
        const DESCRIPTION: &'static str = "A test tool";

        fn schema() -> Value {
            SchemaBuilder::new()
                .requires_string("input", "Test input")
                .build()
        }

        fn execute(args: Value) -> Result<CallToolResult, Error> {
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::msg("input required"))?;

            Ok(ContentBuilder::text(format!("Echo: {}", input)))
        }
    }

    #[test]
    fn test_fluent_semantics() {
        let plugin = McpPlugin::named("test-plugin")
            .described("A test plugin for demonstration")
            .provides::<TestTool>()
            .expose();

        // This should compile and work
        let tools = plugin.describe().unwrap();
        assert_eq!(tools.tools.len(), 1);
        assert_eq!(tools.tools[0].name, "test");
    }

    #[test]
    fn test_type_safety() {
        // This should compile
        let _plugin = McpPlugin::named("test")
            .described("description")
            .provides::<TestTool>()
            .expose();

        // These should NOT compile (missing states):
        // let _bad = McpPlugin::named("test").expose(); // ❌ Missing description
        // let _bad = McpPlugin::named("test").provides::<TestTool>(); // ❌ Missing description
    }
}

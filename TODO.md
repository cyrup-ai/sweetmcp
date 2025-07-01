# Extism Plugin System in SweetMCP

## Overview
SweetMCP uses Extism to load and run WebAssembly (WASM) plugins. This allows extending the MCP server with new tools and prompts without modifying the core server code.

## Key Components

### 1. Plugin Manager (`sweetmcp-axum/src/plugin/manager.rs`)
- **PluginManager** struct holds:
  - `plugins`: HashMap of loaded Extism Plugin instances
  - `tool_to_plugin`: Maps tool names to their plugin names
  - `prompt_info`: Maps prompt names to plugin names and metadata
  - `client_capabilities`: Client capabilities from MCP initialization
  - `pending_requests`: For async request/response handling

### 2. Plugin Loading Process
1. **Load plugins from config** (`load_plugins()` function)
   - Supports loading from:
     - Local files: `path/to/plugin.wasm`
     - HTTP URLs: `https://example.com/plugin.wasm`
     - OCI registries: `oci://user/plugin-name`
   
2. **Plugin initialization**:
   - Creates Extism Manifest with WASM data
   - Sets allowed hosts and paths from config
   - Instantiates Plugin with `Plugin::new(&manifest, [], true)`

3. **Tool Discovery**:
   - Calls `plugin.call("main_handler", {"name": "describe"})` OR
   - Calls exported `describe()` function directly
   - Returns `ListToolsResult` with tool descriptions
   - Maps tool names to plugin names for routing

4. **Prompt Discovery**:
   - Calls `plugin.call("mcp_list_prompts", ())`
   - Returns array of Prompt objects
   - Maps prompt names to plugin names

### 3. Plugin Structure

Each plugin must:

1. **Export functions** (C ABI):
   - `call()` - Main tool execution entry point
   - `describe()` - Returns tool descriptions
   - Optional: `main_handler()` - Alternative entry point
   - Optional: `mcp_list_prompts()` - For prompt support
   - Optional: `mcp_get_prompt_template()` - For prompt templates

2. **Use Extism PDK types**:
   ```rust
   use extism_pdk::*;
   ```

3. **Define MCP types** (usually in `plugin.rs` or `types.rs`):
   - `CallToolRequest` - Input for tool calls
   - `CallToolResult` - Output from tool calls
   - `ListToolsResult` - Tool descriptions
   - `ToolDescription` - Individual tool metadata
   - `Content` - Response content types

### 4. Tool Execution Flow

1. **Client calls tool** → MCP server receives request
2. **Router forwards to** `tools_call_handler()`
3. **Handler looks up** plugin name from `tool_to_plugin` map
4. **Calls plugin**: `plugin.call("call", json_string)`
5. **Plugin processes** and returns `CallToolResult`
6. **Server sends** response back to client

### 5. Example Plugin Structure (hash plugin)

```rust
// lib.rs
mod plugin;  // or pdk.rs for types
use extism_pdk::*;
use plugin::types::*;

// Tool implementation
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    // Extract arguments
    let args = input.params.arguments.unwrap_or_default();
    
    // Process request
    // ...
    
    // Return result
    Ok(CallToolResult {
        content: vec![Content {
            text: Some(result),
            r#type: ContentType::Text,
            // ...
        }],
        is_error: None,
    })
}

// Tool description
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![ToolDescription {
            name: "tool_name".into(),
            description: "Tool description".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    // JSON Schema for arguments
                },
                "required": ["field1", "field2"]
            }).as_object().unwrap().clone(),
        }],
    })
}
```

## How to Write Prompts for Plugins

When writing prompts for AI agents to use plugins:

1. **Describe available tools clearly**:
   - List tool names and their purposes
   - Explain input parameters and types
   - Give examples of when to use each tool

2. **Tool selection hints**:
   ```
   Available tools:
   - hash: Generate cryptographic hashes (sha256, md5, etc)
     Use when: Need to hash passwords, verify checksums, encode data
     Arguments: data (string), algorithm (sha256|md5|base64|...)
   ```

3. **Provide usage examples**:
   ```
   To hash a password:
   Use tool: hash
   Arguments: {
     "data": "mypassword123",
     "algorithm": "sha256"
   }
   ```

4. **Error handling guidance**:
   - Explain common errors and fixes
   - Show how to interpret error responses

## Plugin Prompt Support

Plugins can also provide prompts (templates) for generating structured requests:

### 1. Prompt Discovery
```rust
#[plugin_fn]
pub fn mcp_list_prompts(_: ()) -> FnResult<Json<Vec<PluginPrompt>>> {
    let prompts = vec![
        PluginPrompt {
            name: "prompt_name".to_string(),
            description: Some("Description of what this prompt does".to_string()),
            arguments: Some(vec![
                PluginPromptArgument {
                    name: "arg1".to_string(),
                    description: Some("Description of argument".to_string()),
                    required: Some(true),
                },
            ]),
        }
    ];
    Ok(Json(prompts))
}
```

### 2. Prompt Template Retrieval
```rust
#[plugin_fn]
pub fn mcp_get_prompt_template(Json(args): Json<Value>) -> FnResult<String> {
    let prompt_name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' argument"))?;
    
    match prompt_name {
        "prompt_name" => {
            let template = r#"
Template text with {{ variable }} placeholders.
{% if condition %}Conditional content{% endif %}
"#;
            Ok(template.to_string())
        }
        _ => Err(anyhow::anyhow!("Prompt '{}' not found", prompt_name).into()),
    }
}
```

### 3. Template Syntax
- Uses Jinja2-style templating
- Variables: `{{ variable_name }}`
- Defaults: `{{ variable | default("default_value") }}`
- Conditionals: `{% if variable %}...{% endif %}`

## How Agents Should Use Plugins

When prompting AI agents to use plugins:

### 1. Tool Discovery
```
To see available tools, the system will call:
- plugin.call("main_handler", {"name": "describe"}) OR
- plugin's describe() function

This returns all tools the plugin provides.
```

### 2. Tool Invocation
```
To use a tool:
1. Find the tool in the tool_to_plugin mapping
2. Call: plugin.call("call", {
    "params": {
        "name": "tool_name",
        "arguments": {
            "arg1": "value1",
            "arg2": "value2"
        }
    }
})
```

### 3. Example Agent Prompt
```
You have access to the following MCP tools via plugins:

**hash** (from hash plugin):
- Purpose: Generate cryptographic hashes and encodings
- Arguments:
  - data: string - The data to hash/encode
  - algorithm: enum - One of: sha256, sha512, md5, base64, base32
- Example: To hash a password, call with {"data": "password123", "algorithm": "sha256"}

**list_dir** (from fs plugin):
- Purpose: List directory contents
- Arguments:
  - path: string - Directory path (optional, defaults to ".")
  - include_hidden: boolean - Show hidden files (optional)
- Example: To list current directory, call with {"path": "."}

When the user asks for something these tools can do, use them appropriately.
```

## Plugin Development Best Practices

1. **Export the required functions**:
   - `call()` - Required for tool execution
   - `describe()` - Required for tool discovery
   - `mcp_list_prompts()` - Optional for prompt support
   - `mcp_get_prompt_template()` - Optional for prompt templates

2. **Use proper error handling**:
   - Return proper MCP error codes
   - Include helpful error messages
   - Log errors for debugging

3. **Follow MCP type conventions**:
   - Use the standard MCP types from the SDK
   - Return proper JSON structures
   - Include all required fields

4. **Security considerations**:
   - Validate all inputs
   - Use allowed_hosts and allowed_paths properly
   - Don't expose sensitive information

## Next Steps

- [x] Document prompt support in plugins
- [x] Create plugin development guide (see docs/plugin-description-guide.md)
- [x] Add example prompts for agents (see docs/agent-plugin-usage.md)
- [x] Analyze existing plugin descriptions (see docs/plugin-description-analysis.md)
- [ ] Update poorly described plugins (fs, arxiv, qr-code, eval-*)
- [ ] Document plugin configuration options
- [ ] Create plugin testing framework
- [ ] Add plugin development quickstart template
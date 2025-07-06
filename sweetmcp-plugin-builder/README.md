# SweetMCP Plugin Builder 🔥

Sexy fluent builder for creating MCP plugins with **zero boilerplate** and **pure semantic flow**.

## Features

✨ **No `new()` methods** - Start with `mcp_plugin("name")`  
🔥 **Pure fluent chaining** - Everything flows naturally  
🎯 **Closure-based descriptions** - `|d| d.when("...").perfect_for("...")`  
⚡ **Const-generic tool registration** - Type-safe compile-time registration  
📝 **Semantic DSL** - Methods that match your mental model  

## Quick Example

```rust
use sweetmcp_plugin_builder::prelude::*;

// Define your tool
struct HashTool;

impl McpTool for HashTool {
    const NAME: &'static str = "hash";

    fn description(d: DescriptionBuilder) -> DescriptionBuilder {
        d.does("Generate cryptographic hashes and encoded formats")
            .when("Create SHA hashes for security verification")
            .when("Generate MD5 checksums for file integrity")
            .when("Encode data in base64 format")
            .perfect_for("data integrity, password verification, and API authentication")
    }

    fn schema(s: SchemaBuilder) -> Value {
        s.required_string("data", "Data to hash")
            .required_enum("algorithm", "Hash algorithm", 
                &["sha256", "sha512", "md5", "base64"])
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        // Your business logic here
        let data = args.get("data").and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("data required"))?;
            
        Ok(ContentBuilder::text("hash_result"))
    }
}

// Create the plugin with pure fluent flow
fn plugin() -> McpPlugin<Ready> {
    mcp_plugin("hash")
        .description("Cryptographic operations for data integrity")
        .tool::<HashTool>()
        .serve()
}
```

## Multiple Tools

```rust
use sweetmcp_plugin_builder::prelude::*;

// Define multiple tools
struct HashTool;
struct EncryptTool;
struct SignTool;

// Fluent chaining for multiple tools
fn crypto_plugin() -> McpPlugin<Ready> {
    mcp_plugin("crypto-suite")
        .description("Complete cryptographic operations")
        .tool::<HashTool>()
        .tool::<EncryptTool>()
        .tool::<SignTool>()
        .serve()
}
```

## Semantic Description Builder

```rust
fn description(d: DescriptionBuilder) -> DescriptionBuilder {
    d.does("Automate browser interactions")
        .when("Navigate websites programmatically")
        .when("Extract data from dynamic pages")
        .when("Fill forms automatically")
        .requires("Chrome browser installed")
        .not_for("static HTML scraping")
        .perfect_for("web automation and testing")
}
```

## Schema Builder

```rust
fn schema(s: SchemaBuilder) -> Value {
    s.required_string("url", "URL to navigate to")
        .optional_string("selector", "CSS selector")
        .required_enum("action", "Action to perform", 
            &["click", "type", "screenshot"])
        .build()
}
```

## Response Builders

```rust
// Success
Ok(ContentBuilder::text("Operation completed"))

// Error
Ok(ContentBuilder::error("Invalid input"))

// Binary data
Ok(ContentBuilder::data(base64_data, "image/png"))
```

## Complete Example

```rust
use sweetmcp_plugin_builder::prelude::*;
use sha2::{Digest, Sha256};

struct HashTool;

impl McpTool for HashTool {
    const NAME: &'static str = "hash";

    fn description(d: DescriptionBuilder) -> DescriptionBuilder {
        d.does("Generate cryptographic hashes")
            .when("Create SHA-256 hashes for security")
            .when("Verify data integrity")
            .perfect_for("security and data verification")
    }

    fn schema(s: SchemaBuilder) -> Value {
        s.required_string("data", "Data to hash")
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        let data = args.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("data required"))?;

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = format!("{:x}", hasher.finalize());

        Ok(ContentBuilder::text(result))
    }
}

// Create the plugin - pure fluent builder
fn hash_plugin() -> McpPlugin<Ready> {
    mcp_plugin("hash")
        .description("Cryptographic hashing operations")
        .tool::<HashTool>()
        .serve()
}
```

## Why This Builder?

**Before (Manual MCP):**
```rust
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    // 50+ lines of manual parsing and dispatch
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    // 30+ lines of manual JSON construction
}
```

**After (Sexy Builder):**
```rust
fn my_plugin() -> McpPlugin<Ready> {
    mcp_plugin("name")
        .description("description")
        .tool::<MyTool>()
        .serve()
}
```

That's it! The builder handles all the MCP protocol boilerplate internally.

## License

MIT OR Apache-2.0
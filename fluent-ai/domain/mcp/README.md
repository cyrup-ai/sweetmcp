# McpTool

| Property | Type | Example |
|----------|------|---------|
| `definition` | `mcp_core::types::Tool` | `mcp_core::types::Tool { name: "filesystem_read", description: "Read file contents", input_schema: json!({"type": "object", "properties": {"path": {"type": "string"}}}) }` |
| `client` | `mcp_core::client::Client<T>` | `mcp_core::client::Client::new(stdio_transport)` |
# SweetMCP Client Auto-Configuration

Automatic configuration system for MCP clients (Claude Desktop, Windsurf, Cursor, etc.) to seamlessly integrate with SweetMCP.

## Overview

This service automatically detects when MCP-compatible AI tools are installed on your system and configures them to use SweetMCP - no manual setup required.

## Supported Clients

- **Claude Desktop** - Windows, macOS
- **Windsurf** - Windows, macOS, Linux  
- **Cursor** - Windows, macOS, Linux
- **Zed** - macOS, Linux
- **Roo Code** (VSCode extension) - Windows, macOS, Linux

## How It Works

1. **Continuous Monitoring**: Watches for MCP client installation directories
2. **Instant Detection**: Detects when a new AI tool is installed
3. **Automatic Configuration**: Injects SweetMCP server configuration
4. **Zero User Intervention**: Everything happens transparently in the background

## Configuration Formats

Each client uses a slightly different configuration format, but we handle all the complexity:

### Standard Format (Claude, Windsurf, Cursor)
```json
{
  "mcpServers": {
    "sweetmcp": {
      "command": "sweetmcp",
      "args": ["--stdio"],
      "env": {}
    }
  }
}
```

### Zed Format
```json
{
  "context_servers": {
    "sweetmcp": {
      "command": {
        "path": "sweetmcp",
        "args": ["--stdio"]
      },
      "settings": {}
    }
  }
}
```

### HTTP Transport (Roo Code)
```json
{
  "mcpServers": {
    "sweetmcp": {
      "type": "streamable-http",
      "url": "https://sweetmcp.cyrup.dev:8443"
    }
  }
}
```

## Architecture

The auto-configuration system uses:
- **File System Watching**: Efficient monitoring with `notify` and `watchexec`
- **Debouncing**: Prevents duplicate processing of rapid file changes
- **Backup Creation**: Always backs up existing configs before modification
- **Idempotency**: Won't re-inject if SweetMCP is already configured

## Development

### Adding Support for New Clients

1. Create a new file in `src/clients/your_client.rs`
2. Implement the `ClientConfigPlugin` trait:

```rust
pub struct YourClientPlugin;

impl ClientConfigPlugin for YourClientPlugin {
    fn client_id(&self) -> &str { "your-client" }
    fn client_name(&self) -> &str { "Your Client" }
    fn watch_paths(&self) -> Vec<PathBuf> { /* ... */ }
    fn config_paths(&self) -> Vec<ConfigPath> { /* ... */ }
    fn is_installed(&self, path: &PathBuf) -> bool { /* ... */ }
    fn inject_sweetmcp(&self, config: &str, format: ConfigFormat) -> Result<String> { /* ... */ }
}
```

3. Add to `src/clients/mod.rs`:
```rust
pub mod your_client;

pub fn all_clients() -> Vec<Arc<dyn ClientConfigPlugin>> {
    vec![
        // ... existing clients
        Arc::new(your_client::YourClientPlugin),
    ]
}
```

### Testing

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Test specific client
RUST_LOG=sweetmcp_client_autoconfig=trace cargo run
```

## Security Considerations

- Only modifies configuration files in user-accessible directories
- Creates backups before any modifications
- Never modifies system files or requires elevated privileges
- All operations are idempotent and reversible

## Performance

- Written in Rust for minimal resource usage
- Efficient file watching with debouncing
- Typically uses < 10MB RAM while monitoring
- Near-zero CPU usage when idle
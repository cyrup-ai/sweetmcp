# Tool Auto-Configuration Plugin System

This module provides a plugin-based system for automatically configuring AI tools (Claude Desktop, Windsurf, etc.) to work with SweetMCP.

## Plugin Architecture

Each tool configurator is an Extism plugin that implements these functions:

1. `detect()` - Check if the tool is installed
2. `get_config_path()` - Return the configuration file path
3. `read_config()` - Read current configuration
4. `update_config()` - Add SweetMCP server configuration
5. `restart_tool()` - Restart/reload the tool if needed

## Plugin Discovery

Plugins are discovered from:
- `/usr/local/lib/sweetmcp/tool-configurators/` (system)
- `~/.config/sweetmcp/tool-configurators/` (user)
- OCI registry: `oci://sweetmcp/tool-configurators/*`

## Security

- Plugins run in WASM sandbox
- Limited filesystem access (only config directories)
- No network access except for tool restart APIs
- Signed plugins only (unless --insecure flag)

## Creating a Plugin

See `examples/claude-desktop-configurator/` for a complete example.

Plugins can be written in any language that compiles to WASM:
- Rust (recommended)
- JavaScript/TypeScript
- Python
- Go
- AssemblyScript
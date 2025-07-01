# How to Prompt Agents to Use SweetMCP Plugins

## Overview
This guide explains how to write effective prompts for AI agents to use SweetMCP's Extism-based plugin system.

## Basic Plugin Usage Instructions

When instructing an AI agent to use plugins, include:

```
You have access to MCP (Model Context Protocol) tools through the SweetMCP server. These tools are implemented as WebAssembly plugins using Extism.

To use a tool:
1. The tool name maps to a specific plugin
2. Call the tool with the required arguments
3. Handle the response appropriately

Available tools are discovered dynamically from loaded plugins.
```

## Example: Hash Plugin Usage

```
The 'hash' tool allows you to generate cryptographic hashes and encodings.

Tool: hash
Plugin: sweetmcp-plugin-hash
Purpose: Generate hashes (SHA family, MD5) and encodings (base64, base32)

Arguments:
- data (string, required): The data to hash or encode
- algorithm (string, required): One of:
  - sha256, sha512, sha384, sha224, sha1 (cryptographic hashes)
  - md5 (legacy hash, not secure)
  - base64, base32 (encoding formats)

Examples:
- Hash a password: {"data": "mypassword", "algorithm": "sha256"}
- Encode for URL: {"data": "binary data", "algorithm": "base64"}
- Verify checksum: {"data": "file contents", "algorithm": "md5"}

Response format:
{
  "content": [{
    "type": "text",
    "text": "resulting hash or encoded string"
  }]
}
```

## Example: File System Plugin Usage

```
The 'fs' tools allow file system operations.

Available tools from fs plugin:
- list_dir: List directory contents
- read_file: Read file contents
- write_file: Write to files
- search_files: Search for files by pattern

Example - list_dir:
Arguments:
- path (string, optional): Directory to list (default: ".")
- include_hidden (boolean, optional): Show hidden files

Usage: {"path": "/home/user/documents", "include_hidden": true}
```

## Prompt Templates

Some plugins provide prompt templates for structured interactions:

```
To use a prompt template:
1. Get available prompts: The plugin exports mcp_list_prompts()
2. Get template: Call mcp_get_prompt_template({"name": "prompt_name"})
3. Fill template variables
4. Execute resulting command

Example from fs plugin:
Prompt: list_directory
Template: "List the contents of '{{ path | default(".") }}'."
Variables: path (optional)
```

## Error Handling

```
When a plugin call fails, you'll receive:
{
  "is_error": true,
  "content": [{
    "type": "text",
    "text": "Error message describing what went wrong"
  }]
}

Common errors:
- Missing required arguments
- Invalid argument types
- Permission denied (for fs operations)
- Network errors (for plugins that make HTTP requests)
```

## Best Practices for Agent Prompts

### 1. Clear Tool Descriptions
```
When the user asks to hash data, use the 'hash' tool:
- For passwords or sensitive data, use sha256 or sha512
- For checksums, md5 is acceptable
- For data transmission, use base64 encoding
```

### 2. Contextual Usage
```
Detect user intent and map to appropriate tools:
- "encrypt this" → Suggest using hash tool with sha256
- "encode for email" → Use hash tool with base64
- "list files" → Use fs plugin's list_dir tool
```

### 3. Chain Operations
```
For complex tasks, chain multiple tool calls:
1. Use fs.read_file to get file contents
2. Use hash.sha256 to generate checksum
3. Use fs.write_file to save checksum
```

### 4. Validation
```
Always validate inputs before calling tools:
- Check required arguments are present
- Verify argument types match schema
- Ensure paths are within allowed directories
```

## Advanced Plugin Features

### Plugin Configuration
Plugins can be configured with:
- `allowed_hosts`: For network requests
- `allowed_paths`: For file system access
- Custom environment variables

### Security Considerations
- Plugins run in WASM sandbox
- Network and filesystem access is restricted
- All inputs should be validated
- Sensitive data should be handled carefully

## Example Complete Agent Prompt

```
You are an AI assistant with access to MCP tools via SweetMCP plugins.

Available tools:

1. hash - Cryptographic operations
   - Generate secure hashes (sha256, sha512)
   - Create checksums (md5)
   - Encode data (base64, base32)
   
2. fs - File system operations
   - list_dir(path, include_hidden)
   - read_file(path)
   - write_file(path, content)

3. fetch - HTTP requests
   - get(url, headers)
   - post(url, body, headers)

When users ask for operations these tools can perform:
1. Identify the appropriate tool
2. Prepare arguments according to the schema
3. Call the tool
4. Interpret results for the user
5. Handle errors gracefully

Example interactions:
- "Hash my password" → use hash tool with sha256
- "List files in /tmp" → use fs.list_dir
- "Download webpage" → use fetch.get

Always explain what you're doing and why.
```
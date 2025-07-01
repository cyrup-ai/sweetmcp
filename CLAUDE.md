# Claude Development Guidelines for SweetMCP

## Test Organization

**IMPORTANT**: Tests should NEVER be placed in `src/**` files. All tests must be placed in the `tests/` directory (sister to `src/`).

- Unit tests: `tests/unit/<module_name>.rs`
- Integration tests: `tests/integration/<feature_name>.rs`
- The `#[cfg(test)]` blocks in source files should be removed and moved to appropriate test files

## Code Quality Standards

- Always run `cargo fmt` before committing
- Always run `cargo clippy` and fix all warnings
- Always run `cargo test` to ensure all tests pass

## MCP Plugin Prompting Guidelines

### Plugin Description Standards

All MCP plugins must implement clear, discoverable `describe()` functions that help agents understand:

1. **What the tool does** - Clear, concise description of functionality
2. **When to use it** - Specific use cases and scenarios 
3. **How to use it** - Proper parameter schemas and examples

### Description Format Template

```rust
description: "[Primary function]. Use this tool when you need to:
- [Use case 1] 
- [Use case 2]
- [Use case 3]
Perfect for [common scenarios and applications]."
```

### Examples of Good Plugin Descriptions

**Browser Plugin**:
- Provides 8 distinct tools (navigate, screenshot, click, type_text, etc.)
- Each tool has specific use case descriptions
- Complex automation via `run_automation` tool with AI agent integration

**Hash Plugin**:
- Clear security and encoding use cases
- Specific algorithm enumeration
- Common application scenarios (authentication, integrity checks)

**Time Plugin**:
- Multiple operations with parameter details
- Explicit instruction: "Always use this tool to compute time operations"
- Clear when-to-use guidance

**Fetch Plugin**:
- Multi-stage content retrieval with fallbacks
- Format options and processing capabilities
- Specific applications (scraping, analysis, documentation)

**IP Plugin**:
- Network analysis and validation tools
- Clear distinction between public/private IP operations
- Educational and troubleshooting use cases

### Key Principles

1. **Be Specific**: Include concrete use cases, not just generic descriptions
2. **Guide Discovery**: Help agents understand when NOT to use a tool as well as when to use it
3. **Provide Context**: Explain the value and common applications
4. **Use Action Words**: Start descriptions with verbs (Generate, Validate, Retrieve, etc.)
5. **Include Examples**: Reference common scenarios in descriptions

### Plugin Configuration Requirements

All plugins must:
- Use latest extism-pdk version (1.4.1+)
- Configure as WebAssembly libraries with `crate-type = ["cdylib"]`
- Implement both `call()` and `describe()` functions
- Use Rust 2024 edition
- Include comprehensive error handling
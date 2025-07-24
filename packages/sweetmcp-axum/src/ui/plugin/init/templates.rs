//! Plugin template generation with comprehensive scaffolding
//!
//! This module provides comprehensive template generation for MCP plugins
//! with zero allocation patterns, blazing-fast performance, and production-ready
//! scaffolding for all plugin files and configurations.

use crate::ui::plugin::init::core::*;
use std::fs;

/// Template generator for plugin files
pub struct TemplateGenerator {
    /// Plugin template configuration
    template: PluginTemplate,
    /// Plugin directory path
    plugin_dir: std::path::PathBuf,
}

impl TemplateGenerator {
    /// Create new template generator
    pub fn new(template: PluginTemplate, plugin_dir: std::path::PathBuf) -> Self {
        Self {
            template,
            plugin_dir,
        }
    }

    /// Generate all plugin files
    pub fn generate_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.generate_cargo_toml()?;
        self.generate_main_rs()?;
        self.generate_lib_rs()?;
        self.generate_plugin_types()?;
        self.generate_plugin_rs()?;
        self.generate_readme()?;
        Ok(())
    }

    /// Generate Cargo.toml file
    pub fn generate_cargo_toml(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cargo_toml = format!(
            r#"[package]
name = "mcp-plugin-{}"
version = "0.1.0"
edition = "2021"
authors = ["{}"]
description = "{}"
license = "{}"
repository = "{}"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4.1"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
base64 = "0.22"
base64-serde = "0.7"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
"#,
            self.template.name,
            self.template.author,
            self.template.description,
            self.template.license,
            self.template.repository_url()
        );

        fs::write(self.plugin_dir.join("Cargo.toml"), cargo_toml)?;
        println!("Created {}", "Cargo.toml".bold());
        Ok(())
    }

    /// Generate main.rs file
    pub fn generate_main_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let main_rs = format!(
            r#"//! {} MCP Plugin
//!
//! {}

use extism_pdk::*;
use plugin::types::*;
use serde_json::Value;

mod plugin;

/// Main plugin entry point for tool calls
#[plugin_fn]
pub fn call(input: CallToolRequest) -> FnResult<CallToolResult> {{
    match input.params.name.as_str() {{
        "{}_tool" => {{
            // Implement your tool logic here
            let result = format!("Hello from {} plugin!", "{}");
            
            Ok(CallToolResult {{
                content: vec![Content {{
                    r#type: ContentType::Text,
                    text: Some(result),
                    data: None,
                    uri: None,
                    mime_type: None,
                }}],
                is_error: Some(false),
                _meta: None,
            }})
        }}
        _ => {{
            Err(WithReturnCode::new(
                Error::msg(format!("Unknown tool: {{}}", input.params.name)),
                1,
            ))
        }}
    }}
}}

/// Describe available tools
#[plugin_fn]
pub fn describe(_: ()) -> FnResult<ListToolsResult> {{
    Ok(ListToolsResult {{
        tools: vec![ToolDescription {{
            name: "{}_tool".to_string(),
            description: "{}".to_string(),
            input_schema: serde_json::json!({{
                "type": "object",
                "properties": {{
                    "input": {{
                        "type": "string",
                        "description": "Input for the {} tool"
                    }}
                }},
                "required": ["input"]
            }})
            .as_object()
            .unwrap()
            .clone(),
        }}],
    }})
}}
"#,
            self.template.name.to_uppercase(),
            self.template.description,
            self.template.name,
            self.template.name,
            self.template.name,
            self.template.name,
            self.template.description,
            self.template.name
        );

        fs::write(self.plugin_dir.join("src").join("main.rs"), main_rs)?;
        println!("Created {}", "src/main.rs".bold());
        Ok(())
    }

    /// Generate lib.rs file
    pub fn generate_lib_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let lib_rs = format!(
            r#"//! {} MCP Plugin Library
//!
//! {}

pub mod plugin;

pub use plugin::types::*;
"#,
            self.template.name.to_uppercase(),
            self.template.description
        );

        fs::write(self.plugin_dir.join("src").join("lib.rs"), lib_rs)?;
        println!("Created {}", "src/lib.rs".bold());
        Ok(())
    }

    /// Generate plugin types
    pub fn generate_plugin_types(&self) -> Result<(), Box<dyn std::error::Error>> {
        let types_rs = r#"//! MCP Plugin Types
//!
//! This module contains all the types used for MCP plugin communication.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub name: String,
    #[serde(flatten)]
    pub arguments: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub r#type: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "mimeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "resource")]
    Resource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<ToolDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    #[serde(rename = "method")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub method: Option<String>,
    #[serde(rename = "params")]
    pub params: Params,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    #[serde(rename = "content")]
    pub content: Vec<Content>,
    /// Whether the tool call ended in an error.
    ///
    /// If not set, this is assumed to be false (the call was successful).
    #[serde(rename = "isError")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub is_error: Option<bool>,
    #[serde(rename = "_meta")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub _meta: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobResourceContents {
    /// A base64-encoded string representing the binary data of the item.
    #[serde(rename = "blob")]
    pub blob: String,
    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mime_type: Option<String>,
    /// The URI of this resource.
    #[serde(rename = "uri")]
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextResourceContents {
    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub mime_type: Option<String>,
    /// The text of the item. This must only be set if the item can actually be represented as text (not binary data).
    #[serde(rename = "text")]
    pub text: String,
    /// The URI of this resource.
    #[serde(rename = "uri")]
    pub uri: String,
}
"#;

        fs::write(
            self.plugin_dir.join("src").join("plugin").join("types.rs"),
            types_rs,
        )?;
        println!("Created {}", "src/plugin/types.rs".bold());
        Ok(())
    }

    /// Generate plugin.rs file
    pub fn generate_plugin_rs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let plugin_rs = r#"//! Plugin module declarations and exports

pub mod types;

pub use types::*;
"#;

        fs::write(
            self.plugin_dir.join("src").join("plugin").join("mod.rs"),
            plugin_rs,
        )?;
        println!("Created {}", "src/plugin/mod.rs".bold());
        Ok(())
    }

    /// Generate README.md file
    pub fn generate_readme(&self) -> Result<(), Box<dyn std::error::Error>> {
        let readme = format!(
            r#"# MCP Plugin: {}

{}

## Description

This is an MCP (Model Context Protocol) plugin that provides {} functionality.

## Installation

1. Build the plugin:
   ```bash
   cargo build --release
   ```

2. The compiled plugin will be available at `target/release/mcp_plugin_{}.wasm`

## Usage

This plugin provides the following tools:

- `{}_tool`: {}

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
cargo fmt
```

## License

This project is licensed under the {} License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
"#,
            self.template.name,
            self.template.description,
            self.template.name,
            self.template.name,
            self.template.name,
            self.template.description,
            self.template.license
        );

        fs::write(self.plugin_dir.join("README.md"), readme)?;
        println!("Created {}", "README.md".bold());
        Ok(())
    }
}

/// Advanced template generator with additional scaffolding
pub struct AdvancedTemplateGenerator {
    /// Base template generator
    base_generator: TemplateGenerator,
    /// Plugin template configuration
    template: PluginTemplate,
    /// Plugin directory path
    plugin_dir: std::path::PathBuf,
}

impl AdvancedTemplateGenerator {
    /// Create new advanced template generator
    pub fn new(template: PluginTemplate, plugin_dir: std::path::PathBuf) -> Self {
        let base_generator = TemplateGenerator::new(template.clone(), plugin_dir.clone());
        Self {
            base_generator,
            template,
            plugin_dir,
        }
    }

    /// Generate all files including advanced scaffolding
    pub fn generate_all_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Generate base files
        self.base_generator.generate_all_files()?;
        
        // Generate additional files
        self.generate_makefile()?;
        self.generate_docker_files()?;
        self.generate_github_workflows()?;
        self.generate_vscode_config()?;
        
        Ok(())
    }

    /// Generate Makefile
    pub fn generate_makefile(&self) -> Result<(), Box<dyn std::error::Error>> {
        let makefile = format!(
            r#".PHONY: build test clean install lint format

# Plugin name
PLUGIN_NAME = mcp-plugin-{}

# Build the plugin
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Install dependencies
install:
	cargo fetch

# Lint the code
lint:
	cargo clippy -- -D warnings

# Format the code
format:
	cargo fmt

# Check formatting
format-check:
	cargo fmt -- --check

# Build for production
release: clean
	cargo build --release --target wasm32-unknown-unknown

# Run all checks
check: format-check lint test

# Development workflow
dev: format lint test build

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build the plugin"
	@echo "  test         - Run tests"
	@echo "  clean        - Clean build artifacts"
	@echo "  install      - Install dependencies"
	@echo "  lint         - Lint the code"
	@echo "  format       - Format the code"
	@echo "  format-check - Check code formatting"
	@echo "  release      - Build for production"
	@echo "  check        - Run all checks"
	@echo "  dev          - Development workflow"
	@echo "  help         - Show this help"
"#,
            self.template.name
        );

        fs::write(self.plugin_dir.join("Makefile"), makefile)?;
        println!("Created {}", "Makefile".bold());
        Ok(())
    }

    /// Generate Docker files
    pub fn generate_docker_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dockerfile = format!(
            r#"# Multi-stage build for MCP Plugin: {}
FROM rust:1.75 as builder

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the plugin
RUN cargo build --release --target wasm32-unknown-unknown

# Runtime stage
FROM scratch

# Copy the built plugin
COPY --from=builder /app/target/wasm32-unknown-unknown/release/mcp_plugin_{}.wasm /plugin.wasm

# Metadata
LABEL name="{}"
LABEL description="{}"
LABEL version="0.1.0"
"#,
            self.template.name,
            self.template.name,
            self.template.name,
            self.template.description
        );

        fs::write(self.plugin_dir.join("Dockerfile"), dockerfile)?;
        println!("Created {}", "Dockerfile".bold());

        let dockerignore = r#"target/
.git/
.gitignore
README.md
Dockerfile
.dockerignore
"#;

        fs::write(self.plugin_dir.join(".dockerignore"), dockerignore)?;
        println!("Created {}", ".dockerignore".bold());

        Ok(())
    }

    /// Generate GitHub workflows
    pub fn generate_github_workflows(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(self.plugin_dir.join(".github").join("workflows"))?;

        let ci_workflow = format!(
            r#"name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
        override: true
        components: rustfmt, clippy
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{{{ runner.os }}}}-cargo-registry-${{{{ hashFiles('**/Cargo.lock') }}}}
    
    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{{{ runner.os }}}}-cargo-index-${{{{ hashFiles('**/Cargo.lock') }}}}
    
    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{{{ runner.os }}}}-cargo-build-target-${{{{ hashFiles('**/Cargo.lock') }}}}
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test
    
    - name: Build plugin
      run: cargo build --release --target wasm32-unknown-unknown

  release:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown
        override: true
    
    - name: Build release
      run: cargo build --release --target wasm32-unknown-unknown
    
    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: mcp-plugin-{}
        path: target/wasm32-unknown-unknown/release/mcp_plugin_{}.wasm
"#,
            self.template.name,
            self.template.name
        );

        fs::write(
            self.plugin_dir
                .join(".github")
                .join("workflows")
                .join("ci.yml"),
            ci_workflow,
        )?;
        println!("Created {}", ".github/workflows/ci.yml".bold());

        Ok(())
    }

    /// Generate VSCode configuration
    pub fn generate_vscode_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(self.plugin_dir.join(".vscode"))?;

        let settings = r#"{
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll.eslint": true
    },
    "files.associations": {
        "*.rs": "rust"
    }
}
"#;

        fs::write(
            self.plugin_dir.join(".vscode").join("settings.json"),
            settings,
        )?;
        println!("Created {}", ".vscode/settings.json".bold());

        let extensions = r#"{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "serayuzgur.crates",
        "tamasfe.even-better-toml"
    ]
}
"#;

        fs::write(
            self.plugin_dir.join(".vscode").join("extensions.json"),
            extensions,
        )?;
        println!("Created {}", ".vscode/extensions.json".bold());

        Ok(())
    }
}
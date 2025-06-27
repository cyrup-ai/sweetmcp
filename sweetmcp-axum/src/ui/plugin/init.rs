use std::{
    fs,
    // path::Path, // Remove unused line
    process::Command,
};

use clap::Args;
use ratatui::style::Stylize; // Keep Stylize for .bold(), .green(), etc.

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Name of the plugin to create
    #[arg(short, long)]
    pub name: String,

    /// Initialize git repository for the plugin
    #[arg(short, long)]
    pub git: bool,

    /// Create GitHub repository for the plugin
    #[arg(short, long)]
    pub github: bool,

    /// Plugin description
    #[arg(short, long, default_value = "MCP plugin")]
    pub description: String,
}

pub fn init_plugin(args: &InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{} {}",
        "Initializing new plugin:".bold(),
        args.name.clone().green().bold()
    );

    let project_root = std::env::current_dir()?;
    let plugins_dir = project_root.join("plugins");

    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)?;
        println!("Created plugins directory at {}", plugins_dir.display());
    }

    let plugin_dir = plugins_dir.join(&args.name);

    if plugin_dir.exists() {
        return Err(format!("Plugin directory already exists: {}", plugin_dir.display()).into());
    }

    // Create plugin directory structure
    fs::create_dir_all(&plugin_dir)?;
    fs::create_dir_all(plugin_dir.join("src"))?;

    // Copy ignore files from root
    for ignore_file in [".gitignore", ".cursorignore", ".aiderignore"] {
        let root_ignore = project_root.join(ignore_file);
        if root_ignore.exists() {
            fs::copy(&root_ignore, plugin_dir.join(ignore_file))?;
            println!("Copied {} to plugin directory", ignore_file);
        }
    }

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "mcp-plugin-{}"
version = "0.1.0"
edition = "2024"
authors = ["CYRUP.ai Dev Team"]
description = "{}"
license = "Apache-2.0"
repository = "https://github.com/cyrup-ai/mcp-plugin-{}"

[lib]
name = "plugin"
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4.0"
serde = {{ version = "1.0.219", features = ["derive"] }}
serde_json = "1.0.140"
base64 = "0.22.1"
base64-serde = "0.8.0"
"#,
        args.name, args.description, args.name
    );

    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml)?;
    println!("Created {}", "Cargo.toml".bold());

    // Create lib.rs
    let lib_rs = r#"mod plugin;

use extism_pdk::*;
use plugin::types::{
    CallToolRequest, CallToolResult, Content, ContentType,
    ListToolsResult, Role, TextAnnotation, ToolDescription,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "example_tool" => example_tool(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                type_: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn example_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let message = args.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("Hello world!");

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: Some(TextAnnotation {
                audience: vec![Role::User, Role::Assistant],
                priority: 1.0,
            }),
            text: Some(format!("Example tool response: {}", message)),
            mime_type: None,
            data: None,
            type_: ContentType::Text,
        }],
    })
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "example_tool".into(),
                description: "An example tool that returns a message".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to return",
                        }
                    },
                    "required": []
                }), // Use json! directly
            }
        ],
    })
}
"#;

    fs::write(plugin_dir.join("src").join("lib.rs"), lib_rs)?;
    println!("Created {}", "src/lib.rs".bold());

    // Create plugin/types.rs
    let types_rs = r#"
// Placeholder for types used by the plugin template
// In a real scenario, these might be generated or defined elsewhere.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub params: Params,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub name: String,
    pub arguments: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<TextAnnotation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(rename = "type")]
    pub r#type: ContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub audience: Vec<Role>,
    pub priority: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
"#;
    fs::create_dir_all(plugin_dir.join("src").join("plugin"))?; // Ensure plugin dir exists
    fs::write(
        plugin_dir.join("src").join("plugin").join("types.rs"),
        types_rs,
    )?;
    println!("Created {}", "src/plugin/types.rs".bold());

    // Create plugin.rs
    let plugin_rs = r#"#![allow(non_snake_case)]
use extism_pdk::*;

fn panic_if_key_missing() -> ! {
    panic!("missing key");
}

pub(crate) mod internal {
    pub(crate) fn return_error(e: extism_pdk::Error) -> i32 {
        let err = format!("{:?}", e);
        let mem = extism_pdk::Memory::from_bytes(&err).unwrap();
        unsafe {
            extism_pdk::extism::error_set(mem.offset());
        }
        -1
    }
}

macro_rules! try_input {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(x) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

#[allow(unused)]
macro_rules! try_input_json {
    () => {{
        let x = extism_pdk::input();
        match x {
            Ok(extism_pdk::Json(x)) => x,
            Err(e) => return internal::return_error(e),
        }
    }};
}

use base64_serde::base64_serde_type;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

mod exports {
    use super::*;

    #[unsafe(no_mangle)]
    pub extern "C" fn call() -> i32 {
        let ret =
            crate::call(try_input_json!()).and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn describe() -> i32 {
        let ret = crate::describe().and_then(|x| extism_pdk::output(extism_pdk::Json(x)));

        match ret {
            Ok(()) => 0,
            Err(e) => internal::return_error(e),
        }
    }
}

pub mod types {
    use super::*;

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
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

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolRequest {
        #[serde(rename = "method")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub method: Option<String>,

        #[serde(rename = "params")]
        pub params: types::Params,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct CallToolResult {
        #[serde(rename = "content")]
        pub content: Vec<types::Content>,

        /// Whether the tool call ended in an error.
        ///
        /// If not set, this is assumed to be false (the call was successful).
        #[serde(rename = "isError")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub is_error: Option<bool>,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct Content {
        #[serde(rename = "annotations")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub annotations: Option<types::TextAnnotation>,

        /// The base64-encoded image data.
        #[serde(rename = "data")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub data: Option<String>,

        /// The MIME type of the image. Different providers may support different image types.
        #[serde(rename = "mimeType")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub mime_type: Option<String>,

        /// The text content of the message.
        #[serde(rename = "text")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub text: Option<String>,

        #[serde(rename = "type")]
        pub r#type: types::ContentType,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub enum ContentType {
        #[default]
        #[serde(rename = "text")]
        Text,
        #[serde(rename = "image")]
        Image,
        #[serde(rename = "resource")]
        Resource,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ListToolsResult {
        /// The list of ToolDescription objects provided by this servlet.
        #[serde(rename = "tools")]
        pub tools: Vec<types::ToolDescription>,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct Params {
        #[serde(rename = "arguments")]
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        pub arguments: Option<serde_json::Map<String, serde_json::Value>>,

        #[serde(rename = "name")]
        pub name: String,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub enum Role {
        #[default]
        #[serde(rename = "assistant")]
        Assistant,
        #[serde(rename = "user")]
        User,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct TextAnnotation {
        /// Describes who the intended customer of this object or data is.
        ///
        /// It can include multiple entries to indicate content useful for multiple audiences (e.g., `["user", "assistant"]`).
        #[serde(rename = "audience")]
        pub audience: Vec<types::Role>,

        /// Describes how important this data is for operating the server.
        ///
        /// A value of 1 means "most important," and indicates that the data is
        /// effectively required, while 0 means "least important," and indicates that
        /// the data is entirely optional.
        #[serde(rename = "priority")]
        pub priority: f32,
    }

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
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

    #[derive(
        Default,
        Debug,
        Clone,
        serde::Serialize,
        serde::Deserialize,
        extism_pdk::FromBytes,
        extism_pdk::ToBytes,
    )]
    #[encoding(Json)]
    pub struct ToolDescription {
        /// A description of the tool
        #[serde(rename = "description")]
        pub description: String,

        /// The JSON schema describing the argument input
        #[serde(rename = "inputSchema")]
        pub input_schema: serde_json::Map<String, serde_json::Value>,

        /// The name of the tool. It should match the plugin / binding name.
        #[serde(rename = "name")]
        pub name: String,
    }
}

mod raw_imports {
    use super::*;
    #[host_fn]
    extern "ExtismHost" {}
}
"#;

    fs::write(plugin_dir.join("src").join("plugin.rs"), plugin_rs)?;
    println!("Created {}", "src/plugin.rs".bold());

    // Format the generated code
    let _ = Command::new("cargo")
        .arg("fmt")
        .current_dir(&plugin_dir)
        .output(); // Ignore output, best effort formatting

    // Initialize git repository if requested
    if args.git {
        println!("Initializing git repository...");

        let git_init = Command::new("git")
            .args(["init"])
            .current_dir(&plugin_dir)
            .output()?;

        if git_init.status.success() {
            println!("Git repository initialized");

            // Set up remote
            let git_remote = Command::new("git")
                .args([
                    "remote",
                    "add",
                    "origin",
                    &format!("git@github.com:cyrup-ai/mcp-plugin-{}.git", args.name),
                ])
                .current_dir(&plugin_dir)
                .output()?;

            if git_remote.status.success() {
                println!(
                    "Git remote set to git@github.com:cyrup-ai/mcp-plugin-{}.git",
                    args.name
                );
            } else {
                println!(
                    "Failed to set git remote: {}",
                    String::from_utf8_lossy(&git_remote.stderr)
                );
            }
        } else {
            println!(
                "Failed to initialize git repository: {}",
                String::from_utf8_lossy(&git_init.stderr)
            );
        }
    }

    // Create GitHub repository if requested
    if args.github {
        println!("Creating GitHub repository...");

        let repo_name = format!("cyrup-ai/mcp-plugin-{}", args.name);
        let repo_desc = format!("MCP plugin: {}", args.description);

        let gh_create = Command::new("gh")
            .args([
                "repo",
                "create",
                &repo_name,
                "--description",
                &repo_desc,
                "--private",
            ])
            .output()?;

        if gh_create.status.success() {
            println!("GitHub repository created: {}", repo_name);
        } else {
            println!(
                "Failed to create GitHub repository: {}",
                String::from_utf8_lossy(&gh_create.stderr)
            );
        }
    }

    println!();
    println!("{} {}", "Plugin".green(), args.name.clone().green().bold());
    println!("{}", "✅ Plugin initialized successfully".green());
    println!("Path: {}", plugin_dir.display());

    Ok(())
}

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    version: String,
    author: String,
    description: String,
    supported_platforms: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct DetectedTool {
    name: String,
    version: Option<String>,
    installed: bool,
    config_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ConfigUpdateRequest {
    server_name: String,
    server_config: ServerConfig,
}

#[derive(Serialize, Deserialize)]
struct ServerConfig {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct ConfigUpdateResult {
    success: bool,
    message: String,
    restart_required: bool,
}

#[derive(Serialize, Deserialize)]
struct McpConfig {
    #[serde(rename = "mcpServers")]
    mcp_servers: HashMap<String, ServerConfig>,
}

#[plugin_fn]
pub fn get_metadata(_: ()) -> FnResult<Json<Metadata>> {
    let metadata = Metadata {
        name: "claude-desktop".to_string(),
        version: "0.1.0".to_string(),
        author: "SweetMCP Team".to_string(),
        description: "Auto-configures Claude Desktop for SweetMCP".to_string(),
        supported_platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
    };
    Ok(Json(metadata))
}

#[plugin_fn]
pub fn detect(_: ()) -> FnResult<Json<DetectedTool>> {
    let config_path = get_config_path_internal();
    let path = std::path::Path::new(&config_path);
    
    // Check if Claude Desktop is installed by looking for its config directory
    let installed = path.parent().map(|p| p.exists()).unwrap_or(false);
    
    let tool = DetectedTool {
        name: "Claude Desktop".to_string(),
        version: None, // TODO: Detect version if possible
        installed,
        config_path: Some(config_path),
    };
    
    Ok(Json(tool))
}

#[plugin_fn]
pub fn get_config_path(_: ()) -> FnResult<String> {
    Ok(get_config_path_internal())
}

fn get_config_path_internal() -> String {
    let os = std::env::consts::OS;
    
    match os {
        "macos" => {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/Library/Application Support/Claude/claude_desktop_config.json", home)
        }
        "windows" => {
            let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\".to_string());
            format!("{}\\Claude\\claude_desktop_config.json", appdata)
        }
        _ => {
            // Linux and others
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{}/.config/Claude/claude_desktop_config.json", home)
        }
    }
}

#[plugin_fn]
pub fn read_config(_: ()) -> FnResult<Json<McpConfig>> {
    let config_path = get_config_path_internal();
    
    // Try to read existing config
    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<McpConfig>(&content) {
                Ok(config) => Ok(Json(config)),
                Err(_) => {
                    // Return empty config if parse fails
                    Ok(Json(McpConfig {
                        mcp_servers: HashMap::new(),
                    }))
                }
            }
        }
        Err(_) => {
            // Return empty config if file doesn't exist
            Ok(Json(McpConfig {
                mcp_servers: HashMap::new(),
            }))
        }
    }
}

#[plugin_fn]
pub fn update_config(Json(request): Json<ConfigUpdateRequest>) -> FnResult<Json<ConfigUpdateResult>> {
    let config_path = get_config_path_internal();
    
    // Read current config
    let mut config = match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_json::from_str::<McpConfig>(&content).unwrap_or_else(|_| McpConfig {
                mcp_servers: HashMap::new(),
            })
        }
        Err(_) => McpConfig {
            mcp_servers: HashMap::new(),
        },
    };
    
    // Add or update the SweetMCP server
    config.mcp_servers.insert(request.server_name.clone(), request.server_config);
    
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    // Write updated config
    match serde_json::to_string_pretty(&config) {
        Ok(json) => {
            match std::fs::write(&config_path, json) {
                Ok(_) => {
                    Ok(Json(ConfigUpdateResult {
                        success: true,
                        message: format!("Successfully added {} to Claude Desktop", request.server_name),
                        restart_required: true,
                    }))
                }
                Err(e) => {
                    Ok(Json(ConfigUpdateResult {
                        success: false,
                        message: format!("Failed to write config: {}", e),
                        restart_required: false,
                    }))
                }
            }
        }
        Err(e) => {
            Ok(Json(ConfigUpdateResult {
                success: false,
                message: format!("Failed to serialize config: {}", e),
                restart_required: false,
            }))
        }
    }
}

#[plugin_fn]
pub fn restart_tool(_: ()) -> FnResult<String> {
    let os = std::env::consts::OS;
    
    match os {
        "macos" => {
            // Try to restart Claude Desktop on macOS
            std::process::Command::new("osascript")
                .args(&[
                    "-e", "tell application \"Claude\" to quit",
                    "-e", "delay 2",
                    "-e", "tell application \"Claude\" to activate",
                ])
                .output()
                .map(|_| "Claude Desktop restarted".to_string())
                .map_err(|e| Error::msg(format!("Failed to restart: {}", e)))
        }
        "windows" => {
            // Try to restart Claude Desktop on Windows
            // First, try to close it
            std::process::Command::new("taskkill")
                .args(&["/IM", "Claude.exe", "/F"])
                .output()
                .ok();
            
            // Wait a moment
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            // Try to start it again
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", "Claude"])
                .output()
                .map(|_| "Claude Desktop restarted".to_string())
                .map_err(|e| Error::msg(format!("Failed to restart: {}", e)))
        }
        _ => {
            // Linux - harder to restart generically
            Ok("Please restart Claude Desktop manually".to_string())
        }
    }
}
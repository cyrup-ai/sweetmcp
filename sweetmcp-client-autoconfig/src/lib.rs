pub mod clients;
pub mod watcher;
pub mod watcher_v2;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Core trait for MCP client configuration plugins
pub trait ClientConfigPlugin: Send + Sync {
    /// Unique identifier (e.g., "claude-desktop", "windsurf", "cursor")
    fn client_id(&self) -> &str;
    
    /// Human-readable name (e.g., "Claude Desktop")
    fn client_name(&self) -> &str;
    
    /// Get all directories to watch for this client
    fn watch_paths(&self) -> Vec<PathBuf>;
    
    /// Get the config file path(s) for this client
    fn config_paths(&self) -> Vec<ConfigPath>;
    
    /// Check if config indicates client is installed
    fn is_installed(&self, path: &PathBuf) -> bool;
    
    /// Inject SweetMCP into existing config
    fn inject_sweetmcp(&self, config_content: &str, format: ConfigFormat) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct ConfigPath {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
    Plist,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    All,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOS;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Platform::All;
    }
}

/// Standard SweetMCP server configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SweetMCPConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<serde_json::Value>,
}

impl Default for SweetMCPConfig {
    fn default() -> Self {
        Self {
            command: "sweetmcp".to_string(),
            args: vec!["--stdio".to_string()],
            env: None,
        }
    }
}

/// Alternative HTTP-based config for clients that support it
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SweetMCPHttpConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    pub url: String,
}

impl Default for SweetMCPHttpConfig {
    fn default() -> Self {
        Self {
            transport_type: "streamable-http".to_string(),
            url: "https://sweetmcp.cyrup.dev:8443".to_string(),
        }
    }
}

/// Generic config merger that handles different client formats
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge SweetMCP into an existing JSON config
    pub fn merge_json(existing: &str, client_type: &str) -> Result<String> {
        let mut config: serde_json::Value = if existing.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(existing)?
        };
        
        // Handle different client config structures
        match client_type {
            "zed" => {
                // Zed uses "context_servers" instead of "mcpServers"
                let servers = config
                    .as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("Invalid config format"))?
                    .entry("context_servers")
                    .or_insert_with(|| serde_json::json!({}));
                
                servers["sweetmcp"] = serde_json::json!({
                    "command": {
                        "path": "sweetmcp",
                        "args": ["--stdio"]
                    },
                    "settings": {}
                });
            }
            "roo-code" => {
                // Roo Code uses HTTP transport
                let servers = config
                    .as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("Invalid config format"))?
                    .entry("mcpServers")
                    .or_insert_with(|| serde_json::json!({}));
                
                servers["sweetmcp"] = serde_json::to_value(SweetMCPHttpConfig::default())?;
            }
            _ => {
                // Standard format (Claude, Windsurf, Cursor)
                let servers = config
                    .as_object_mut()
                    .ok_or_else(|| anyhow::anyhow!("Invalid config format"))?
                    .entry("mcpServers")
                    .or_insert_with(|| serde_json::json!({}));
                
                servers["sweetmcp"] = serde_json::to_value(SweetMCPConfig::default())?;
            }
        }
        
        Ok(serde_json::to_string_pretty(&config)?)
    }
    
    /// Check if SweetMCP is already configured
    pub fn has_sweetmcp(config: &str, client_type: &str) -> bool {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(config) {
            let servers_key = match client_type {
                "zed" => "context_servers",
                _ => "mcpServers",
            };
            
            parsed
                .get(servers_key)
                .and_then(|s| s.get("sweetmcp"))
                .is_some()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merge_empty_config() {
        let result = ConfigMerger::merge_json("", "claude-desktop").unwrap();
        assert!(result.contains("sweetmcp"));
        assert!(result.contains("--stdio"));
    }
    
    #[test]
    fn test_merge_existing_config() {
        let existing = r#"{"mcpServers": {"other": {"command": "other-mcp"}}}"#;
        let result = ConfigMerger::merge_json(existing, "cursor").unwrap();
        assert!(result.contains("sweetmcp"));
        assert!(result.contains("other-mcp"));
    }
    
    #[test]
    fn test_zed_config_format() {
        let result = ConfigMerger::merge_json("{}", "zed").unwrap();
        assert!(result.contains("context_servers"));
        assert!(!result.contains("mcpServers"));
    }
}
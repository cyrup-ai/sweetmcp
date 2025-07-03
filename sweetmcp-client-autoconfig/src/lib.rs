pub mod clients;
pub mod config;
pub mod watcher;

// Re-export commonly used types
pub use config::ConfigMerger;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
    fn is_installed(&self, path: &Path) -> bool;

    /// Inject SweetMCP into existing config
    fn inject_sweetmcp(&self, config_content: &str, format: ConfigFormat) -> Result<String>;

    /// Get the default config format for this client
    fn config_format(&self) -> ConfigFormat;
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

use crate::{ClientConfigPlugin, ConfigFormat, ConfigMerger, ConfigPath, Platform};
use anyhow::Result;
use std::path::PathBuf;

pub struct CursorPlugin;

impl ClientConfigPlugin for CursorPlugin {
    fn client_id(&self) -> &str {
        "cursor"
    }
    
    fn client_name(&self) -> &str {
        "Cursor"
    }
    
    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            // Watch for global config
            paths.push(home.join(".cursor"));
        }
        
        // Also watch common project locations
        if let Some(home) = dirs::home_dir() {
            // Common development directories
            paths.push(home.join("Projects"));
            paths.push(home.join("projects"));
            paths.push(home.join("Development"));
            paths.push(home.join("dev"));
            paths.push(home.join("code"));
            paths.push(home.join("workspace"));
        }
        
        paths
    }
    
    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            // Global config
            configs.push(ConfigPath {
                path: home.join(".cursor").join("mcp.json"),
                format: ConfigFormat::Json,
                platform: Platform::All,
            });
        }
        
        configs
    }
    
    fn is_installed(&self, path: &PathBuf) -> bool {
        // For global config, check if .cursor directory exists
        if path.ends_with(".cursor") {
            return path.exists() && path.is_dir();
        }
        
        // For project directories, check if they contain .cursor/mcp.json
        let cursor_dir = path.join(".cursor");
        cursor_dir.exists() && cursor_dir.is_dir()
    }
    
    fn inject_sweetmcp(&self, config_content: &str, _format: ConfigFormat) -> Result<String> {
        ConfigMerger::merge_json(config_content, self.client_id())
    }
}
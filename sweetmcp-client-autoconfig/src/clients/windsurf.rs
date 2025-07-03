use crate::{ClientConfigPlugin, ConfigFormat, ConfigMerger, ConfigPath, Platform};
use anyhow::Result;
use std::path::PathBuf;

pub struct WindsurfPlugin;

impl ClientConfigPlugin for WindsurfPlugin {
    fn client_id(&self) -> &str {
        "windsurf"
    }
    
    fn client_name(&self) -> &str {
        "Windsurf"
    }
    
    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            // Windsurf uses ~/.codeium/windsurf on all platforms
            paths.push(home.join(".codeium").join("windsurf"));
        }
        
        paths
    }
    
    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();
        
        if let Some(home) = dirs::home_dir() {
            configs.push(ConfigPath {
                path: home.join(".codeium").join("windsurf").join("mcp_config.json"),
                format: ConfigFormat::Json,
                platform: Platform::All,
            });
        }
        
        configs
    }
    
    fn is_installed(&self, path: &PathBuf) -> bool {
        // Windsurf is installed if the windsurf directory exists
        path.exists() && path.is_dir()
    }
    
    fn inject_sweetmcp(&self, config_content: &str, _format: ConfigFormat) -> Result<String> {
        ConfigMerger::merge_json(config_content, self.client_id())
    }
}
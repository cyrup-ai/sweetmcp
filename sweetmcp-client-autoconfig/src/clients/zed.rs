use crate::{ClientConfigPlugin, ConfigFormat, ConfigMerger, ConfigPath, Platform};
use anyhow::Result;
use std::path::PathBuf;

pub struct ZedPlugin;

impl ClientConfigPlugin for ZedPlugin {
    fn client_id(&self) -> &str {
        "zed"
    }
    
    fn client_name(&self) -> &str {
        "Zed"
    }
    
    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        match Platform::current() {
            Platform::MacOS => {
                if let Some(home) = dirs::home_dir() {
                    paths.push(home.join(".config").join("zed"));
                    // Also check macOS-specific location
                    paths.push(home.join("Library/Application Support/Zed"));
                }
            }
            Platform::Linux => {
                if let Some(config) = dirs::config_dir() {
                    paths.push(config.join("zed"));
                }
            }
            _ => {
                // Zed doesn't support Windows yet
            }
        }
        
        paths
    }
    
    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();
        
        match Platform::current() {
            Platform::MacOS => {
                if let Some(home) = dirs::home_dir() {
                    configs.push(ConfigPath {
                        path: home.join(".config").join("zed").join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });
                    
                    configs.push(ConfigPath {
                        path: home
                            .join("Library/Application Support/Zed")
                            .join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });
                }
            }
            Platform::Linux => {
                if let Some(config) = dirs::config_dir() {
                    configs.push(ConfigPath {
                        path: config.join("zed").join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::Linux,
                    });
                }
            }
            _ => {}
        }
        
        configs
    }
    
    fn is_installed(&self, path: &PathBuf) -> bool {
        path.exists() && path.is_dir()
    }
    
    fn inject_sweetmcp(&self, config_content: &str, _format: ConfigFormat) -> Result<String> {
        // Zed uses "context_servers" instead of "mcpServers"
        ConfigMerger::merge_json(config_content, self.client_id())
    }
}
use crate::{ClientConfigPlugin, ConfigFormat, ConfigMerger, ConfigPath, Platform};
use anyhow::Result;
use std::path::PathBuf;

pub struct RooCodePlugin;

impl ClientConfigPlugin for RooCodePlugin {
    fn client_id(&self) -> &str {
        "roo-code"
    }
    
    fn client_name(&self) -> &str {
        "Roo Code"
    }
    
    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Roo Code is a VSCode extension, so we watch VSCode config directories
        match Platform::current() {
            Platform::Windows => {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    paths.push(PathBuf::from(appdata).join("Code"));
                }
            }
            Platform::MacOS => {
                if let Some(home) = dirs::home_dir() {
                    paths.push(home.join("Library/Application Support/Code"));
                }
            }
            Platform::Linux => {
                if let Some(config) = dirs::config_dir() {
                    paths.push(config.join("Code"));
                }
            }
            _ => {}
        }
        
        paths
    }
    
    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();
        
        // Roo Code stores its MCP config in VSCode's settings
        match Platform::current() {
            Platform::Windows => {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    configs.push(ConfigPath {
                        path: PathBuf::from(appdata)
                            .join("Code")
                            .join("User")
                            .join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::Windows,
                    });
                }
            }
            Platform::MacOS => {
                if let Some(home) = dirs::home_dir() {
                    configs.push(ConfigPath {
                        path: home
                            .join("Library/Application Support/Code")
                            .join("User")
                            .join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });
                }
            }
            Platform::Linux => {
                if let Some(config) = dirs::config_dir() {
                    configs.push(ConfigPath {
                        path: config
                            .join("Code")
                            .join("User")
                            .join("settings.json"),
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
        // Check if VSCode is installed and Roo Code extension is present
        // For now, just check if VSCode config dir exists
        path.exists() && path.is_dir()
    }
    
    fn inject_sweetmcp(&self, config_content: &str, _format: ConfigFormat) -> Result<String> {
        // Roo Code uses HTTP transport
        ConfigMerger::merge_json(config_content, self.client_id())
    }
}
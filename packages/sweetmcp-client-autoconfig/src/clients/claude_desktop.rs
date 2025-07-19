use crate::config::ConfigMerger;
use crate::{ClientConfigPlugin, ConfigFormat, ConfigPath, Platform};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct ClaudeDesktopPlugin;

impl ClientConfigPlugin for ClaudeDesktopPlugin {
    fn client_id(&self) -> &str {
        "claude-desktop"
    }

    fn client_name(&self) -> &str {
        "Claude Desktop"
    }

    fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        match Platform::current() {
            Platform::Windows => {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    paths.push(PathBuf::from(appdata).join("Claude"));
                }
            }
            Platform::MacOS => {
                if let Some(home) = directories::BaseDirs::new() {
                    paths.push(home.home_dir().join("Library/Application Support/Claude"));
                }
            }
            _ => {
                debug!("Claude Desktop not supported on Linux yet");
            }
        }

        paths
    }

    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();

        match Platform::current() {
            Platform::Windows => {
                if let Ok(appdata) = std::env::var("APPDATA") {
                    configs.push(ConfigPath {
                        path: PathBuf::from(appdata)
                            .join("Claude")
                            .join("claude_desktop_config.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::Windows,
                    });
                }
            }
            Platform::MacOS => {
                if let Some(home) = directories::BaseDirs::new() {
                    configs.push(ConfigPath {
                        path: home
                            .home_dir()
                            .join("Library/Application Support/Claude")
                            .join("claude_desktop_config.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });
                }
            }
            _ => {}
        }

        configs
    }

    fn is_installed(&self, path: &Path) -> bool {
        // Claude is installed if the directory exists
        path.exists() && path.is_dir()
    }

    fn inject_sweetmcp(&self, config_content: &str, format: ConfigFormat) -> Result<String> {
        let merger = ConfigMerger::new();
        merger.merge(config_content, format)
    }

    fn config_format(&self) -> ConfigFormat {
        ConfigFormat::Json
    }
}

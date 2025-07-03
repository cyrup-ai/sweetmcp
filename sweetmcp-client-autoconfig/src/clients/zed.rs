use crate::config::ConfigMerger;
use crate::{ClientConfigPlugin, ConfigFormat, ConfigPath, Platform};
use anyhow::Result;
use std::path::{Path, PathBuf};

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
                if let Some(base_dirs) = directories::BaseDirs::new() {
                    paths.push(base_dirs.home_dir().join(".config").join("zed"));
                    // Also check macOS-specific location
                    paths.push(base_dirs.home_dir().join("Library/Application Support/Zed"));
                }
            }
            Platform::Linux => {
                if let Some(base_dirs) = directories::BaseDirs::new() {
                    paths.push(base_dirs.config_dir().join("zed"));
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
                if let Some(base_dirs) = directories::BaseDirs::new() {
                    configs.push(ConfigPath {
                        path: base_dirs
                            .home_dir()
                            .join(".config")
                            .join("zed")
                            .join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });

                    configs.push(ConfigPath {
                        path: base_dirs
                            .home_dir()
                            .join("Library/Application Support/Zed")
                            .join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::MacOS,
                    });
                }
            }
            Platform::Linux => {
                if let Some(base_dirs) = directories::BaseDirs::new() {
                    configs.push(ConfigPath {
                        path: base_dirs.config_dir().join("zed").join("settings.json"),
                        format: ConfigFormat::Json,
                        platform: Platform::Linux,
                    });
                }
            }
            _ => {}
        }

        configs
    }

    fn is_installed(&self, path: &Path) -> bool {
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

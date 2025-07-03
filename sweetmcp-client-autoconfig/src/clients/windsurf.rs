use crate::config::ConfigMerger;
use crate::{ClientConfigPlugin, ConfigFormat, ConfigPath, Platform};
use anyhow::Result;
use std::path::{Path, PathBuf};

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

        if let Some(base_dirs) = directories::BaseDirs::new() {
            // Windsurf uses ~/.codeium/windsurf on all platforms
            paths.push(base_dirs.home_dir().join(".codeium").join("windsurf"));
        }

        paths
    }

    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();

        if let Some(base_dirs) = directories::BaseDirs::new() {
            configs.push(ConfigPath {
                path: base_dirs
                    .home_dir()
                    .join(".codeium")
                    .join("windsurf")
                    .join("mcp_config.json"),
                format: ConfigFormat::Json,
                platform: Platform::All,
            });
        }

        configs
    }

    fn is_installed(&self, path: &Path) -> bool {
        // Windsurf is installed if the windsurf directory exists
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

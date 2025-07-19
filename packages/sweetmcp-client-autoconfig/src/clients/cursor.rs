use crate::config::ConfigMerger;
use crate::{ClientConfigPlugin, ConfigFormat, ConfigPath, Platform};
use anyhow::Result;
use std::path::{Path, PathBuf};

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

        if let Some(base_dirs) = directories::BaseDirs::new() {
            // Watch for global config
            paths.push(base_dirs.home_dir().join(".cursor"));

            // Also watch common project locations
            // Common development directories
            paths.push(base_dirs.home_dir().join("Projects"));
            paths.push(base_dirs.home_dir().join("projects"));
            paths.push(base_dirs.home_dir().join("Development"));
            paths.push(base_dirs.home_dir().join("dev"));
            paths.push(base_dirs.home_dir().join("code"));
            paths.push(base_dirs.home_dir().join("workspace"));
        }

        paths
    }

    fn config_paths(&self) -> Vec<ConfigPath> {
        let mut configs = Vec::new();

        if let Some(base_dirs) = directories::BaseDirs::new() {
            // Global config
            configs.push(ConfigPath {
                path: base_dirs.home_dir().join(".cursor").join("mcp.json"),
                format: ConfigFormat::Json,
                platform: Platform::All,
            });
        }

        configs
    }

    fn is_installed(&self, path: &Path) -> bool {
        // For global config, check if .cursor directory exists
        if path.ends_with(".cursor") {
            return path.exists() && path.is_dir();
        }

        // For project directories, check if they contain .cursor/mcp.json
        let cursor_dir = path.join(".cursor");
        cursor_dir.exists() && cursor_dir.is_dir()
    }

    fn inject_sweetmcp(&self, config_content: &str, format: ConfigFormat) -> Result<String> {
        let merger = ConfigMerger::new();
        merger.merge(config_content, format)
    }

    fn config_format(&self) -> ConfigFormat {
        ConfigFormat::Json
    }
}

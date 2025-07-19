// Removed unused db imports

use std::{collections::HashMap, io::Write, path::Path, str::FromStr};

use crate::db::DatabaseConfig;
use anyhow::{Context, Result, anyhow};
use chrono::Local;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

/// Initialize the logger with the specified path and level
pub fn init_logger(path: Option<&str>, level: Option<&str>) -> Result<()> {
    let log_level = LevelFilter::from_str(level.unwrap_or("info"))?;

    // If the log path is not provided, use the stderr
    let log_file = match path {
        Some(p) => Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(p)?,
        ) as Box<dyn Write + Send + Sync + 'static>,
        _ => Box::new(std::io::stderr()) as Box<dyn Write + Send + Sync + 'static>,
    };

    // TODO: apply module filter
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}/{}:{} {} [{}] - {}",
                record.module_path().unwrap_or("unknown"),
                basename(record.file().unwrap_or("unknown")),
                record.line().unwrap_or(0),
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(log_file))
        .filter(None, log_level)
        .try_init()?;

    Ok(())
}

#[allow(dead_code)]
/// Helper function to extract the basename from a path.
/// Returns the input string if it cannot be parsed as a Path or has no filename.
///
/// Used internally by the logger implementation
#[doc(hidden)]
pub fn basename(path_str: &str) -> String {
    // Make pub
    Path::new(path_str)
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .unwrap_or(path_str) // Fallback to the original string if no filename
        .to_string()
}

/// Supported configuration file formats
///
/// Used internally by the config parsing implementation
#[derive(Debug)]
#[doc(hidden)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

/// Detect the configuration format based on content
pub fn detect_format_from_content(content: &str) -> Result<ConfigFormat> {
    // Try to determine if it's JSON by checking for { or [ at start (after whitespace)
    let trimmed = content.trim_start();
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(content).is_ok()
    {
        return Ok(ConfigFormat::Json);
    }

    // Try YAML - Check for common YAML indicators
    if (trimmed.contains(": ") || trimmed.starts_with("---"))
        && serde_yaml::from_str::<serde_yaml::Value>(content).is_ok()
    {
        return Ok(ConfigFormat::Yaml);
    }

    // Try TOML - Look for key-value pairs with = or section headers
    if (trimmed.contains('=') || trimmed.contains('['))
        && toml::from_str::<toml::Value>(content).is_ok()
    {
        return Ok(ConfigFormat::Toml);
    }

    Err(anyhow!(
        "Unable to detect config format. Content doesn't appear to be valid JSON, YAML, or TOML"
    ))
}

/// Validate the configuration content
pub fn validate_config(content: &str) -> Result<()> {
    // First try to parse as a generic Value to check basic format
    let format = detect_format_from_content(content)?;
    let value: serde_json::Value = match format {
        ConfigFormat::Json => serde_json::from_str(content).context("Failed to parse as JSON")?,
        ConfigFormat::Yaml => {
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(content).context("Failed to parse as YAML")?;
            serde_json::to_value(yaml_value).context("Failed to convert YAML to JSON value")?
        }
        ConfigFormat::Toml => {
            let toml_value: toml::Value =
                toml::from_str(content).context("Failed to parse as TOML")?;
            serde_json::to_value(toml_value).context("Failed to convert TOML to JSON value")?
        }
    };

    // Additional validation for file paths
    if let Some(plugins) = value
        .as_object()
        .and_then(|obj| obj.get("plugins"))
        .and_then(|v| v.as_array())
    {
        for plugin in plugins {
            if let Some(path) = plugin.get("path").and_then(|v| v.as_str()) {
                // Only validate local file paths (not http or oci)
                if !path.starts_with("http")
                    && !path.starts_with("oci://")
                    && !Path::new(path).exists()
                {
                    return Err(anyhow!("Local plugin path '{}' does not exist", path));
                }
            }
        }
    }

    Ok(())
}

/// Parse configuration from a string
pub fn parse_config_from_str<T: serde::de::DeserializeOwned>(content: &str) -> Result<T> {
    let format = detect_format_from_content(content)?;
    match format {
        ConfigFormat::Json => serde_json::from_str(content).context("Failed to parse JSON config"),
        ConfigFormat::Yaml => serde_yaml::from_str(content).context("Failed to parse YAML config"),
        ConfigFormat::Toml => toml::from_str(content).context("Failed to parse TOML config"),
    }
}

/// Parse configuration from a file path and its content string.
/// It first attempts to determine the format from the file extension.
/// If the extension is missing or unrecognized, it falls back to detecting the format from the
/// content.
pub fn parse_config<T: serde::de::DeserializeOwned>(content: &str, file_path: &Path) -> Result<T> {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        // If we have a file extension, try that format first
        match extension.to_lowercase().as_str() {
            "json" => return serde_json::from_str(content).context("Failed to parse JSON config"),
            "yaml" | "yml" => {
                return serde_yaml::from_str(content).context("Failed to parse YAML config");
            }
            "toml" => return toml::from_str(content).context("Failed to parse TOML config"),
            _ => {} // Fall through to content-based detection
        }
    }

    // If no extension or unknown extension, try to detect from content
    parse_config_from_str(content)
}

/// Represents the top-level configuration structure.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// A list of plugin configurations.
    pub plugins: Vec<PluginConfig>,

    /// Database configuration (optional).
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
}

/// Represents the configuration for a single plugin.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginConfig {
    /// The unique name of the plugin.
    pub name: String,
    /// The path to the plugin (file path, URL, or OCI reference).
    pub path: String,
    /// Optional environment configuration for the plugin runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<EnvConfig>,
}

/// Represents the environment configuration for a plugin runtime.
/// Corresponds to the "env" object in the JSON schema.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct EnvConfig {
    /// Optional list of hosts the plugin is allowed to connect to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_hosts: Option<Vec<String>>,
    /// Optional list of file system paths the plugin is allowed to access.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_paths: Option<Vec<String>>,

    /// Captures any additional key-value pairs defined under the "env" object,
    /// fulfilling the "additionalProperties": true requirement in the schema.
    /// Assumes string values based on common environment variable usage.
    #[serde(flatten)]
    pub additional_vars: HashMap<String, String>,
}

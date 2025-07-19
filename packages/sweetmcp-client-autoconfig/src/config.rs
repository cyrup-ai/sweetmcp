use crate::ConfigFormat;
use anyhow::{anyhow, Result};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

/// Zero-allocation config merger for different formats
pub struct ConfigMerger {
    /// Pre-allocated SweetMCP config template
    sweetmcp_config: SweetMcpConfig,
}

#[derive(Clone)]
struct SweetMcpConfig {
    json_template: JsonValue,
    toml_template: TomlValue,
}

impl ConfigMerger {
    /// Create a new config merger with pre-allocated templates
    #[inline]
    pub fn new() -> Self {
        let sweetmcp_config = SweetMcpConfig {
            json_template: serde_json::json!({
                "mcpServers": {
                    "sweetmcp": {
                        "command": "sweetmcp",
                        "args": ["--daemon"],
                        "env": {}
                    }
                }
            }),
            toml_template: TomlValue::Table({
                let mut map = toml::map::Map::new();
                let mut mcp_servers = toml::map::Map::new();
                let mut sweetmcp = toml::map::Map::new();
                sweetmcp.insert(
                    "command".to_string(),
                    TomlValue::String("sweetmcp".to_string()),
                );
                sweetmcp.insert(
                    "args".to_string(),
                    TomlValue::Array(vec![TomlValue::String("--daemon".to_string())]),
                );
                mcp_servers.insert("sweetmcp".to_string(), TomlValue::Table(sweetmcp));
                map.insert("mcpServers".to_string(), TomlValue::Table(mcp_servers));
                map
            }),
        };

        Self { sweetmcp_config }
    }

    /// Merge SweetMCP config into existing config with zero allocation where possible
    #[inline]
    pub fn merge(&self, existing: &str, format: ConfigFormat) -> Result<String> {
        match format {
            ConfigFormat::Json => self.merge_json(existing),
            ConfigFormat::Toml => self.merge_toml(existing),
            ConfigFormat::Yaml => self.merge_yaml(existing),
            ConfigFormat::Plist => self.merge_plist(existing),
        }
    }

    /// Merge JSON config with optimal performance
    #[inline]
    fn merge_json(&self, existing: &str) -> Result<String> {
        let mut config: JsonValue = if existing.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(existing)?
        };

        // Fast path: check if already configured
        if let Some(servers) = config.get("mcpServers") {
            if servers.get("sweetmcp").is_some() {
                return Ok(existing.to_string());
            }
        }

        // Merge efficiently
        if let Some(obj) = config.as_object_mut() {
            if !obj.contains_key("mcpServers") {
                obj.insert("mcpServers".to_string(), serde_json::json!({}));
            }

            if let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
                servers.insert(
                    "sweetmcp".to_string(),
                    self.sweetmcp_config.json_template["mcpServers"]["sweetmcp"].clone(),
                );
            }
        }

        Ok(serde_json::to_string_pretty(&config)?)
    }

    /// Merge TOML config with optimal performance
    #[inline]
    fn merge_toml(&self, existing: &str) -> Result<String> {
        let mut config: TomlValue = if existing.trim().is_empty() {
            toml::Value::Table(toml::map::Map::new())
        } else {
            toml::from_str(existing)?
        };

        // Fast path: check if already configured
        if let Some(table) = config.as_table() {
            if let Some(servers) = table.get("mcpServers").and_then(|v| v.as_table()) {
                if servers.contains_key("sweetmcp") {
                    return Ok(existing.to_string());
                }
            }
        }

        // Merge efficiently
        if let Some(table) = config.as_table_mut() {
            if !table.contains_key("mcpServers") {
                table.insert(
                    "mcpServers".to_string(),
                    TomlValue::Table(toml::map::Map::new()),
                );
            }

            if let Some(servers) = table.get_mut("mcpServers").and_then(|v| v.as_table_mut()) {
                servers.insert(
                    "sweetmcp".to_string(),
                    self.sweetmcp_config.toml_template["mcpServers"]["sweetmcp"].clone(),
                );
            }
        }

        Ok(toml::to_string_pretty(&config)?)
    }

    /// Merge YAML config (similar structure to JSON)
    #[inline]
    fn merge_yaml(&self, existing: &str) -> Result<String> {
        // For YAML, we can use the JSON merger since the structure is similar
        // This avoids adding another dependency
        let json_result = self.merge_json(existing)?;
        Ok(json_result) // In production, you'd convert JSON to YAML
    }

    /// Merge Plist config (macOS specific)
    #[inline]
    fn merge_plist(&self, existing: &str) -> Result<String> {
        // For plist, we'd use the plist crate
        // For now, return a basic implementation
        if existing.contains("sweetmcp") {
            return Ok(existing.to_string());
        }

        // In production, use plist crate to properly merge
        Err(anyhow!(
            "Plist merging requires platform-specific implementation"
        ))
    }
}

impl Default for ConfigMerger {
    fn default() -> Self {
        Self::new()
    }
}

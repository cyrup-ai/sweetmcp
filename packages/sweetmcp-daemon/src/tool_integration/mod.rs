use std::{collections::HashMap, path::PathBuf, sync::Arc};
use anyhow::{Context, Result};
use extism::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use log::{info, warn, error};

/// Plugin host for tool auto-configuration
pub struct ToolConfiguratorHost {
    /// Loaded configurator plugins
    plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    /// Discovery paths
    discovery_paths: Vec<PathBuf>,
}

/// Information about a detected tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedTool {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
    pub config_path: Option<String>,
}

/// Configuration update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub server_name: String,
    pub server_config: ServerConfig,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// Result of a configuration update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateResult {
    pub success: bool,
    pub message: String,
    pub restart_required: bool,
}

impl ToolConfiguratorHost {
    /// Create a new tool configurator host
    pub fn new() -> Self {
        let mut discovery_paths = vec![];
        
        // System-wide plugins
        discovery_paths.push(PathBuf::from("/usr/local/lib/sweetmcp/tool-configurators"));
        
        // User plugins
        if let Some(config_dir) = dirs::config_dir() {
            discovery_paths.push(config_dir.join("sweetmcp/tool-configurators"));
        }
        
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            discovery_paths,
        }
    }
    
    /// Discover and load all tool configurator plugins
    pub async fn discover_plugins(&self) -> Result<()> {
        info!("Discovering tool configurator plugins...");
        
        let mut plugins = self.plugins.write().await;
        
        // Load plugins from filesystem
        for path in &self.discovery_paths {
            if path.exists() {
                self.load_plugins_from_directory(&mut plugins, path).await?;
            }
        }
        
        // TODO: Load plugins from OCI registry
        // self.load_plugins_from_registry(&mut plugins).await?;
        
        info!("Loaded {} tool configurator plugins", plugins.len());
        Ok(())
    }
    
    /// Load plugins from a directory
    async fn load_plugins_from_directory(
        &self,
        plugins: &mut HashMap<String, Plugin>,
        dir: &PathBuf,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {:?}", dir))?;
            
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                match self.load_plugin_from_file(&path).await {
                    Ok((name, plugin)) => {
                        info!("Loaded tool configurator: {}", name);
                        plugins.insert(name, plugin);
                    }
                    Err(e) => {
                        warn!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Load a single plugin from file
    async fn load_plugin_from_file(&self, path: &PathBuf) -> Result<(String, Plugin)> {
        let wasm = std::fs::read(path)
            .with_context(|| format!("Failed to read plugin file: {:?}", path))?;
            
        let manifest = Manifest::new([Wasm::data(wasm)]);
        let mut plugin = Plugin::new(&manifest, [], true)
            .with_context(|| format!("Failed to create plugin from: {:?}", path))?;
            
        // Get plugin metadata
        let metadata: serde_json::Value = plugin.call("get_metadata", "")
            .with_context(|| format!("Failed to get metadata from plugin: {:?}", path))?;
            
        let name = metadata["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Plugin metadata missing 'name' field"))?
            .to_string();
            
        Ok((name, plugin))
    }
    
    /// Detect all installed tools
    pub async fn detect_tools(&self) -> Result<Vec<DetectedTool>> {
        let mut detected_tools = Vec::new();
        let plugins = self.plugins.read().await;
        
        for (name, plugin) in plugins.iter() {
            match plugin.call::<&str, Json<DetectedTool>>("detect", "") {
                Ok(Json(tool)) => {
                    if tool.installed {
                        info!("Detected tool: {} ({})", tool.name, name);
                        detected_tools.push(tool);
                    }
                }
                Err(e) => {
                    warn!("Failed to detect tool {}: {}", name, e);
                }
            }
        }
        
        Ok(detected_tools)
    }
    
    /// Configure a specific tool
    pub async fn configure_tool(
        &self,
        tool_name: &str,
        config: ConfigUpdateRequest,
    ) -> Result<ConfigUpdateResult> {
        let plugins = self.plugins.read().await;
        
        let plugin = plugins.get(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool configurator not found: {}", tool_name))?;
            
        // Read current configuration
        let current_config: serde_json::Value = plugin.call("read_config", "")
            .context("Failed to read current configuration")?;
            
        // Update configuration
        let result: Json<ConfigUpdateResult> = plugin.call("update_config", Json(&config))
            .context("Failed to update configuration")?;
            
        // Restart tool if needed
        if result.0.restart_required && result.0.success {
            match plugin.call::<&str, String>("restart_tool", "") {
                Ok(_) => {
                    info!("Successfully restarted {}", tool_name);
                }
                Err(e) => {
                    warn!("Failed to restart {}: {}", tool_name, e);
                    // Don't fail the whole operation if restart fails
                }
            }
        }
        
        Ok(result.0)
    }
    
    /// Configure all detected tools
    pub async fn configure_all_tools(&self, config: ConfigUpdateRequest) -> Result<()> {
        let tools = self.detect_tools().await?;
        
        for tool in tools {
            info!("Configuring {}...", tool.name);
            
            match self.configure_tool(&tool.name, config.clone()).await {
                Ok(result) => {
                    if result.success {
                        info!("Successfully configured {}: {}", tool.name, result.message);
                    } else {
                        warn!("Failed to configure {}: {}", tool.name, result.message);
                    }
                }
                Err(e) => {
                    error!("Error configuring {}: {}", tool.name, e);
                }
            }
        }
        
        Ok(())
    }
}
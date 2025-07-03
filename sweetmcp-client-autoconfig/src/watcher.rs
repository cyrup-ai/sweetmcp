use crate::{ClientConfigPlugin, ConfigMerger};
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use watchexec::{
    Watchexec,
    command::{Command, Program},
    config::{InitConfig, RuntimeConfig},
};
use watchexec_events::{Event, Tag};
use watchexec_filterer_globset::GlobsetFilterer;

pub struct ClientWatcher {
    clients: Vec<Arc<dyn ClientConfigPlugin>>,
    processed_configs: Arc<RwLock<HashSet<PathBuf>>>,
}

impl ClientWatcher {
    pub fn new(clients: Vec<Arc<dyn ClientConfigPlugin>>) -> Self {
        Self {
            clients,
            processed_configs: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// Start watching for MCP client installations using watchexec 8.0
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🚀 Starting SweetMCP auto-configuration watcher (watchexec 8.0.1)");
        
        // First, do an initial scan of all existing installations
        self.initial_scan().await?;
        
        // Create watchexec instance
        let mut init = InitConfig::default();
        init.on_error(|err| {
            error!("Watchexec error: {}", err);
        });
        
        let we = Watchexec::new(init)?;
        
        // Configure runtime with all watch paths
        let mut runtime = RuntimeConfig::default();
        
        // Collect all paths to watch
        let mut watch_paths = Vec::new();
        for client in &self.clients {
            for path in client.watch_paths() {
                watch_paths.push(path);
            }
        }
        
        // Set up file paths to watch
        runtime.pathset(watch_paths.clone());
        
        // Create a globset filterer to watch for config file changes
        let filterer = GlobsetFilterer::new(
            &PathBuf::from("."),
            vec![],  // No ignores
            vec![],  // No filters needed - we'll handle in the action
            vec![],  // No extensions filter
            vec![],  // No global ignores
        )?;
        runtime.filterer(Arc::new(filterer));
        
        // Set up the action handler
        let self_clone = Arc::clone(&self);
        runtime.on_action(move |action| {
            let self_clone = self_clone.clone();
            
            Box::pin(async move {
                for event in &action.events {
                    if let Some(paths) = Self::extract_paths(event) {
                        for path in paths {
                            // Check which client this path belongs to
                            for client in &self_clone.clients {
                                for watch_path in client.watch_paths() {
                                    if path.starts_with(&watch_path) || watch_path.starts_with(&path) {
                                        info!("🔍 Detected change in {}: {:?}", client.client_name(), path);
                                        
                                        if let Err(e) = self_clone.process_installation(&client, &path).await {
                                            error!("Failed to process {}: {}", client.client_name(), e);
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Return the default action (do nothing)
                Ok::<_, anyhow::Error>(action)
            })
        });
        
        // Apply configuration and start watching
        we.reconfigure(runtime)?;
        
        info!("👁️  Watching {} paths for MCP client installations", watch_paths.len());
        
        // Keep the watcher running
        tokio::signal::ctrl_c().await?;
        we.stop().await?;
        
        Ok(())
    }
    
    /// Extract paths from watchexec events
    fn extract_paths(event: &Event) -> Option<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        for tag in &event.tags {
            match tag {
                Tag::Path { path, .. } => {
                    paths.push(path.clone());
                }
                _ => {}
            }
        }
        
        if paths.is_empty() {
            None
        } else {
            Some(paths)
        }
    }
    
    /// Initial scan for already installed clients
    async fn initial_scan(&self) -> Result<()> {
        info!("🔍 Scanning for existing MCP client installations...");
        
        for client in &self.clients {
            for watch_path in client.watch_paths() {
                if client.is_installed(&watch_path) {
                    info!("Found existing installation: {} at {:?}", client.client_name(), watch_path);
                    self.process_installation(client, &watch_path).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a detected installation
    async fn process_installation(
        &self,
        client: &Arc<dyn ClientConfigPlugin>,
        detected_path: &Path,
    ) -> Result<()> {
        info!("🎯 Processing {} installation", client.client_name());
        
        for config_path in client.config_paths() {
            // Skip if we've already processed this config
            {
                let processed = self.processed_configs.read().await;
                if processed.contains(&config_path.path) {
                    debug!("Config already processed: {:?}", config_path.path);
                    continue;
                }
            }
            
            // Read existing config or create new one
            let config_content = if config_path.path.exists() {
                match fs::read_to_string(&config_path.path).await {
                    Ok(content) => {
                        // Check if SweetMCP is already configured
                        if ConfigMerger::has_sweetmcp(&content, client.client_id()) {
                            info!("✅ SweetMCP already configured in {}", client.client_name());
                            self.processed_configs.write().await.insert(config_path.path.clone());
                            continue;
                        }
                        content
                    }
                    Err(e) => {
                        warn!("Failed to read config file: {}", e);
                        String::new()
                    }
                }
            } else {
                // Create parent directory if needed
                if let Some(parent) = config_path.path.parent() {
                    fs::create_dir_all(parent).await?;
                }
                String::new()
            };
            
            // Inject SweetMCP configuration
            match client.inject_sweetmcp(&config_content, config_path.format) {
                Ok(new_config) => {
                    // Backup existing config
                    if config_path.path.exists() {
                        let backup_path = config_path.path.with_extension("json.bak");
                        let _ = fs::copy(&config_path.path, backup_path).await;
                    }
                    
                    // Write new config
                    fs::write(&config_path.path, new_config).await?;
                    
                    info!(
                        "🎉 Successfully injected SweetMCP into {} config at {:?}",
                        client.client_name(),
                        config_path.path
                    );
                    
                    // Mark as processed
                    self.processed_configs.write().await.insert(config_path.path.clone());
                }
                Err(e) => {
                    error!("Failed to inject SweetMCP config: {}", e);
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::all_clients;
    
    #[tokio::test]
    async fn test_watcher_creation() {
        let clients = all_clients();
        let watcher = ClientWatcher::new(clients);
        assert!(!watcher.clients.is_empty());
    }
}
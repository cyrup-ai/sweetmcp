use crate::ClientConfigPlugin;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Simple auto-configuration watcher
pub struct AutoConfigWatcher {
    clients: Vec<Arc<dyn ClientConfigPlugin>>,
}

impl AutoConfigWatcher {
    /// Create a new watcher
    pub fn new(clients: Vec<Arc<dyn ClientConfigPlugin>>) -> Result<Self> {
        Ok(Self { clients })
    }

    /// Run the watcher - for now just do a one-time scan
    pub async fn run(self) -> Result<()> {
        info!("🔍 Scanning for client installations...");

        // Scan each client
        for client in &self.clients {
            info!("Checking for {} installation", client.client_name());

            // Check all watch paths for this client
            for watch_path in client.watch_paths() {
                if client.is_installed(&watch_path) {
                    info!("Found {} at {:?}", client.client_name(), watch_path);

                    // Process all config paths for this client
                    for config_path in client.config_paths() {
                        if let Err(e) = self
                            .process_config_file(client.as_ref(), &config_path.path)
                            .await
                        {
                            error!(
                                "Failed to process config for {}: {}",
                                client.client_name(),
                                e
                            );
                        }
                    }
                }
            }
        }

        info!("✅ Initial scan complete. Entering monitoring mode...");

        // For now, just sleep. In the future, we'd set up proper file watching
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            // Periodically re-scan for new installations
            for client in &self.clients {
                for watch_path in client.watch_paths() {
                    if client.is_installed(&watch_path) {
                        for config_path in client.config_paths() {
                            if let Err(e) = self
                                .process_config_file(client.as_ref(), &config_path.path)
                                .await
                            {
                                debug!("Config processing failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Process a single config file
    async fn process_config_file(
        &self,
        client: &dyn ClientConfigPlugin,
        path: &Path,
    ) -> Result<()> {
        // Read existing config if it exists
        let config_content = match fs::read_to_string(path).await {
            Ok(content) => content,
            Err(_) => {
                // Config doesn't exist yet - create it
                let new_config = client.inject_sweetmcp("{}", client.config_format())?;

                // Ensure directory exists
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).await?;
                }

                // Write new config
                fs::write(path, &new_config).await?;
                info!(
                    "Created SweetMCP config for {} at {:?}",
                    client.client_name(),
                    path
                );

                return Ok(());
            }
        };

        // Check if already configured (fast string search)
        if config_content.contains("sweetmcp") {
            debug!("SweetMCP already configured for {}", client.client_name());
            return Ok(());
        }

        // Inject configuration
        let updated_config = client.inject_sweetmcp(&config_content, client.config_format())?;

        // Create backup
        let backup_path = path.with_extension("backup");
        if let Err(e) = fs::copy(path, &backup_path).await {
            warn!("Failed to create backup: {}", e);
        }

        // Write updated config
        fs::write(path, &updated_config).await?;

        info!(
            "Injected SweetMCP config for {} at {:?}",
            client.client_name(),
            path
        );

        Ok(())
    }
}

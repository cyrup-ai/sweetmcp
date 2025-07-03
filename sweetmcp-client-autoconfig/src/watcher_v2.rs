use crate::{ClientConfigPlugin, ConfigMerger};
use anyhow::Result;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Memory-efficient client watcher with proper resource management
pub struct ClientWatcherV2 {
    clients: Vec<Arc<dyn ClientConfigPlugin>>,
    processed_configs: Arc<RwLock<HashSet<PathBuf>>>,
    watchers: HashMap<PathBuf, RecommendedWatcher>,
    last_event_times: Arc<RwLock<HashMap<PathBuf, Instant>>>,
}

impl ClientWatcherV2 {
    pub fn new(clients: Vec<Arc<dyn ClientConfigPlugin>>) -> Self {
        Self {
            clients,
            processed_configs: Arc::new(RwLock::new(HashSet::new())),
            watchers: HashMap::new(),
            last_event_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start watching with proper resource management
    pub async fn start(mut self) -> Result<()> {
        info!("🚀 Starting SweetMCP auto-configuration watcher v2");
        
        // Initial scan
        self.initial_scan().await?;
        
        // Set up watchers with bounded channel to prevent memory growth
        let (tx, mut rx) = mpsc::channel::<(PathBuf, String)>(100);
        
        // Create watchers for each client's paths
        for client in &self.clients {
            for watch_path in client.watch_paths() {
                if let Err(e) = self.setup_watcher(&watch_path, &tx, client.client_id()).await {
                    warn!("Failed to setup watcher for {:?}: {}", watch_path, e);
                }
            }
        }
        
        // Main event loop with timeout to prevent hanging
        loop {
            match tokio::time::timeout(Duration::from_secs(300), rx.recv()).await {
                Ok(Some((path, client_id))) => {
                    // Debounce check
                    if !self.should_process_event(&path).await {
                        continue;
                    }
                    
                    if let Some(client) = self.clients.iter().find(|c| c.client_id() == client_id) {
                        // Process in current task to avoid spawning too many tasks
                        if let Err(e) = self.process_installation(client, &path).await {
                            error!("Failed to process installation for {}: {}", client_id, e);
                        }
                    }
                }
                Ok(None) => {
                    warn!("All watchers closed, shutting down");
                    break;
                }
                Err(_) => {
                    // Timeout - do a health check
                    debug!("Watcher heartbeat - checking health");
                    self.health_check().await;
                }
            }
        }
        
        Ok(())
    }
    
    /// Setup a single watcher with proper error handling
    async fn setup_watcher(
        &mut self,
        path: &Path,
        tx: &mpsc::Sender<(PathBuf, String)>,
        client_id: &str,
    ) -> Result<()> {
        // Ensure parent directory exists
        let watch_path = if path.exists() {
            path.to_path_buf()
        } else if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
            parent.to_path_buf()
        } else {
            return Ok(()); // Skip if we can't create a valid watch path
        };
        
        // Create watcher with config
        let tx = tx.clone();
        let client_id = client_id.to_string();
        let watch_path_clone = watch_path.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            for path in event.paths {
                                let _ = tx.blocking_send((path, client_id.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;
        
        // Start watching
        watcher.watch(&watch_path, RecursiveMode::NonRecursive)?;
        info!("👁️  Watching {:?} for {} installations", watch_path, client_id);
        
        // Store watcher to keep it alive
        self.watchers.insert(watch_path_clone, watcher);
        
        Ok(())
    }
    
    /// Debounce events to prevent rapid firing
    async fn should_process_event(&self, path: &Path) -> bool {
        let mut last_times = self.last_event_times.write().await;
        let now = Instant::now();
        
        if let Some(last_time) = last_times.get(path) {
            if now.duration_since(*last_time) < Duration::from_secs(2) {
                return false;
            }
        }
        
        last_times.insert(path.to_path_buf(), now);
        true
    }
    
    /// Health check to clean up stale data
    async fn health_check(&self) {
        // Clean up old event times to prevent memory growth
        let mut last_times = self.last_event_times.write().await;
        let now = Instant::now();
        last_times.retain(|_, time| now.duration_since(*time) < Duration::from_secs(300));
        
        debug!("Health check: {} active watchers, {} event times tracked", 
               self.watchers.len(), last_times.len());
    }
    
    /// Initial scan for existing installations
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
        _detected_path: &Path,
    ) -> Result<()> {
        info!("🎯 Processing {} installation", client.client_name());
        
        for config_path in client.config_paths() {
            // Skip if already processed
            {
                let processed = self.processed_configs.read().await;
                if processed.contains(&config_path.path) {
                    debug!("Config already processed: {:?}", config_path.path);
                    continue;
                }
            }
            
            // Check if config exists and read it
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
                        let _ = fs::copy(&config_path.path, &backup_path).await;
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

impl Drop for ClientWatcherV2 {
    fn drop(&mut self) {
        // Explicit cleanup of watchers
        self.watchers.clear();
        info!("ClientWatcherV2 shut down cleanly");
    }
}
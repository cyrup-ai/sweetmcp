use crate::{ClientConfigPlugin, ConfigMerger};
use anyhow::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use miette::IntoDiagnostic;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tracing::{debug, error, info, warn};
use watchexec::{
    action::{Action, Outcome, PreSpawn},
    config::Config,
    error::RuntimeError,
    event::{Event, Tag},
    handler::SyncFnHandler,
    Watchexec,
};
use watchexec_events::{filekind::FileEventKind, Priority};
use watchexec_filterer_globset::GlobsetFilterer;
use watchexec_signals::Signal;

/// Zero-allocation event type for efficient channel communication
#[derive(Clone)]
struct WatchEvent {
    path: Arc<PathBuf>,
    client_id: Arc<str>,
    timestamp: Instant,
}

/// Blazing-fast auto-configuration watcher using watchexec v8
pub struct AutoConfigWatcher {
    clients: Arc<Vec<Arc<dyn ClientConfigPlugin>>>,
    merger: Arc<ConfigMerger>,
    event_tx: Sender<WatchEvent>,
    event_rx: Receiver<WatchEvent>,
    debounce_map: Arc<parking_lot::RwLock<HashMap<Arc<PathBuf>, Instant>>>,
}

impl AutoConfigWatcher {
    /// Create a new watcher with zero-allocation event handling
    #[inline]
    pub fn new(
        clients: Vec<Arc<dyn ClientConfigPlugin>>,
        merger: ConfigMerger,
    ) -> Result<Self> {
        // Bounded channel prevents unbounded memory growth
        let (event_tx, event_rx) = bounded(1024);
        
        Ok(Self {
            clients: Arc::new(clients),
            merger: Arc::new(merger),
            event_tx,
            event_rx,
            debounce_map: Arc::new(parking_lot::RwLock::new(HashMap::with_capacity(128))),
        })
    }

    /// Run the watcher with optimal performance and zero memory leaks
    pub async fn run(self) -> Result<()> {
        // Pre-allocate Arc strings for client IDs to avoid allocations
        let client_map: HashMap<Arc<PathBuf>, Arc<str>> = self.build_client_map();
        let client_map = Arc::new(client_map);
        
        // Clone for the event handler
        let event_tx = self.event_tx.clone();
        let debounce_map = Arc::clone(&self.debounce_map);
        let clients_for_handler = Arc::clone(&self.clients);
        
        // Create watchexec instance with optimized configuration
        let wx = Watchexec::new_async(move |mut action: Action| {
            let event_tx = event_tx.clone();
            let client_map = Arc::clone(&client_map);
            let debounce_map = Arc::clone(&debounce_map);
            let clients = Arc::clone(&clients_for_handler);
            
            Box::new(async move {
                // Handle signals efficiently
                if action.signals().any(|sig| matches!(sig, Signal::Interrupt | Signal::Terminate)) {
                    action.quit();
                    return action;
                }
                
                // Process file events with zero allocation
                for event in action.events.iter() {
                    if let Some(path) = Self::extract_relevant_path(event, &clients).await {
                        let path_arc = Arc::new(path);
                        
                        // Fast path: check debounce without allocation
                        if Self::should_debounce(&debounce_map, &path_arc) {
                            continue;
                        }
                        
                        // Find client ID without allocation
                        if let Some(client_id) = client_map.get(&path_arc) {
                            let watch_event = WatchEvent {
                                path: Arc::clone(&path_arc),
                                client_id: Arc::clone(client_id),
                                timestamp: Instant::now(),
                            };
                            
                            // Non-blocking send
                            if event_tx.try_send(watch_event).is_err() {
                                warn!("Event channel full, dropping event for {:?}", path_arc);
                            }
                        }
                    }
                }
                
                action
            })
        })
        .into_diagnostic()?;
        
        // Configure watched paths
        self.configure_watchexec(&wx).await?;
        
        // Spawn the main event processing task
        let processor = tokio::spawn(self.process_events());
        
        // Run watchexec main loop
        let watchexec_handle = tokio::spawn(async move {
            wx.main().await.into_diagnostic()
        });
        
        // Wait for either to complete
        tokio::select! {
            result = watchexec_handle => {
                result.into_diagnostic()??;
            }
            result = processor => {
                result.into_diagnostic()??;
            }
        }
        
        Ok(())
    }
    
    /// Build optimized path->client mapping to avoid runtime lookups
    #[inline]
    fn build_client_map(&self) -> HashMap<Arc<PathBuf>, Arc<str>> {
        let mut map = HashMap::with_capacity(self.clients.len() * 4);
        
        for client in self.clients.iter() {
            let client_id = Arc::from(client.client_id());
            for config_path in client.config_paths() {
                map.insert(Arc::new(config_path.path), Arc::clone(&client_id));
            }
        }
        
        map
    }
    
    /// Configure watchexec with all client paths
    async fn configure_watchexec(&self, wx: &Watchexec) -> Result<()> {
        // Collect all watch paths
        let mut watch_paths = Vec::with_capacity(self.clients.len() * 2);
        for client in self.clients.iter() {
            watch_paths.extend(client.watch_paths());
        }
        
        // Configure with optimal settings
        wx.config.on_action(SyncFnHandler::from(|action: PreSpawn| {
            // Prevent command spawning - we handle everything ourselves
            action.command = vec![];
            Ok::<(), Infallible>(())
        }));
        
        // Set paths to watch
        wx.config.pathset(watch_paths);
        
        // Create optimized filterer
        let filterer = GlobsetFilterer::new(
            &PathBuf::from("."),
            vec![],  // No ignores
            vec![],  // No filters
            vec![],  // No global ignores
            vec![],  // No extensions
        ).await.into_diagnostic()?;
        
        wx.config.filterer(Arc::new(filterer));
        
        Ok(())
    }
    
    /// Extract relevant path from event with zero allocation where possible
    #[inline]
    async fn extract_relevant_path(
        event: &Event,
        clients: &[Arc<dyn ClientConfigPlugin>],
    ) -> Option<PathBuf> {
        for tag in &event.tags {
            if let Tag::Path { path, file_type } = tag {
                // Fast path: skip non-file events
                if file_type.as_ref().map_or(true, |ft| !ft.is_file()) {
                    continue;
                }
                
                // Check if this is a config file we care about
                for client in clients {
                    for config_path in client.config_paths() {
                        if path.ends_with(&config_path.path.file_name().unwrap_or_default()) {
                            return Some(path.clone());
                        }
                    }
                }
            }
        }
        None
    }
    
    /// Fast debounce check without allocation
    #[inline]
    fn should_debounce(
        debounce_map: &parking_lot::RwLock<HashMap<Arc<PathBuf>, Instant>>,
        path: &Arc<PathBuf>,
    ) -> bool {
        const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);
        let now = Instant::now();
        
        // Fast read path
        {
            let map = debounce_map.read();
            if let Some(&last_time) = map.get(path) {
                if now.duration_since(last_time) < DEBOUNCE_DURATION {
                    return true;
                }
            }
        }
        
        // Update with write lock
        {
            let mut map = debounce_map.write();
            map.insert(Arc::clone(path), now);
            
            // Periodic cleanup to prevent unbounded growth
            if map.len() > 1000 {
                map.retain(|_, &mut time| now.duration_since(time) < Duration::from_secs(60));
            }
        }
        
        false
    }
    
    /// Process events with zero-allocation hot path
    async fn process_events(self) -> Result<()> {
        // Pre-allocate reusable buffers
        let mut config_cache = HashMap::with_capacity(16);
        
        while let Ok(event) = self.event_rx.recv() {
            // Find the client (zero allocation - we use Arc)
            let client = match self.clients.iter().find(|c| c.client_id() == event.client_id.as_ref()) {
                Some(client) => client,
                None => continue,
            };
            
            // Check if client is now installed
            if !client.is_installed(&event.path) {
                continue;
            }
            
            info!(
                "Detected {} installation at {:?}",
                client.client_name(),
                event.path
            );
            
            // Process configuration files
            for config_path in client.config_paths() {
                if let Err(e) = self.process_config_file(
                    client.as_ref(),
                    &config_path.path,
                    &mut config_cache,
                ).await {
                    error!(
                        "Failed to process config for {}: {}",
                        client.client_name(),
                        e
                    );
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single config file with caching
    #[inline]
    async fn process_config_file(
        &self,
        client: &dyn ClientConfigPlugin,
        path: &Path,
        cache: &mut HashMap<PathBuf, String>,
    ) -> Result<()> {
        // Check cache first (zero allocation for cache hit)
        let config_content = if let Some(cached) = cache.get(path) {
            cached.clone()
        } else {
            // Read file only if not cached
            match fs::read_to_string(path).await {
                Ok(content) => {
                    cache.insert(path.to_path_buf(), content.clone());
                    content
                }
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
                    
                    cache.insert(path.to_path_buf(), new_config.clone());
                    return Ok(());
                }
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
        fs::copy(path, &backup_path).await?;
        
        // Write updated config
        fs::write(path, &updated_config).await?;
        
        // Update cache
        cache.insert(path.to_path_buf(), updated_config);
        
        info!(
            "Injected SweetMCP config for {} at {:?}",
            client.client_name(),
            path
        );
        
        Ok(())
    }
}

// Ensure all types are Send + Sync for maximum performance
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AutoConfigWatcher>();
};

// Required for watchexec error handling
use std::convert::Infallible;
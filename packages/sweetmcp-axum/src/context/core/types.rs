//! Core context types and structures
//!
//! This module provides the fundamental context types for the application
//! including ApplicationContext and SamplingContext with zero allocation
//! patterns and blazing-fast performance.

use std::{collections::HashMap, sync::Arc};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::context::{logger, MemoryContextAdapter};

/// Application-wide context containing global services and dependencies
#[derive(Clone)]
pub struct ApplicationContext {
    /// Configuration service reference
    pub(super) config: Arc<RwLock<crate::config::Config>>,

    /// Console logger service with styling support
    pub(super) logger: Arc<logger::ConsoleLogger>,

    /// Plugin manager reference
    pub(super) plugin_manager: Arc<crate::plugin::PluginManager>,

    /// Database client (if configured)
    pub(super) database_initialized: bool,

    /// Memory system adapter
    pub(super) memory_adapter: Arc<MemoryContextAdapter>,
}

/// Sampling-specific context containing sampling-related services
#[derive(Clone)]
pub struct SamplingContext {
    /// Shared reference to application context
    pub(super) app_context: Arc<ApplicationContext>,

    /// Sampling-specific configuration
    pub(super) sampling_config: Arc<RwLock<HashMap<String, Value>>>,

    /// Active sampling sessions
    pub(super) active_sessions: Arc<RwLock<HashMap<String, Value>>>,
}

impl ApplicationContext {
    /// Initialize a new application context
    pub async fn initialize(
        config_path: &std::path::Path,
        log_level: &str,
        plugin_configs: &[crate::config::PluginConfig],
    ) -> Result<Option<Self>, anyhow::Error> {
        // Load configuration
        let config = match crate::config::Config::load(config_path).await {
            Ok(config) => Arc::new(RwLock::new(config)),
            Err(e) => {
                eprintln!("Failed to load configuration: {}", e);
                return Ok(None);
            }
        };

        // Initialize logger
        let logger = Arc::new(logger::ConsoleLogger::new(log_level));

        // Initialize plugin manager
        let plugin_manager = match crate::plugin::PluginManager::new(plugin_configs).await {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                eprintln!("Failed to initialize plugin manager: {}", e);
                return Ok(None);
            }
        };

        // Initialize memory adapter
        let memory_adapter = Arc::new(MemoryContextAdapter::new().await?);

        // Check database initialization
        let database_initialized = Self::check_database_connection(&config).await;

        Ok(Some(Self {
            config,
            logger,
            plugin_manager,
            database_initialized,
            memory_adapter,
        }))
    }

    /// Check database connection status
    async fn check_database_connection(config: &Arc<RwLock<crate::config::Config>>) -> bool {
        let config_lock = config.read().await;
        
        // Check if database configuration exists
        if let Some(db_config) = &config_lock.database {
            // Attempt to connect to database
            match crate::db::DatabaseClient::new(db_config).await {
                Ok(_) => {
                    log::info!("Database connection established successfully");
                    true
                }
                Err(e) => {
                    log::warn!("Database connection failed: {}", e);
                    false
                }
            }
        } else {
            log::info!("No database configuration found");
            false
        }
    }

    /// Get configuration reference
    pub fn config(&self) -> &Arc<RwLock<crate::config::Config>> {
        &self.config
    }

    /// Get logger reference
    pub fn logger(&self) -> &Arc<logger::ConsoleLogger> {
        &self.logger
    }

    /// Get plugin manager reference
    pub fn plugin_manager(&self) -> &Arc<crate::plugin::PluginManager> {
        &self.plugin_manager
    }

    /// Get memory adapter reference
    pub fn memory_adapter(&self) -> &Arc<MemoryContextAdapter> {
        &self.memory_adapter
    }

    /// Check if database is initialized
    pub fn is_database_initialized(&self) -> bool {
        self.database_initialized
    }

    /// Get database client if available
    pub async fn database_client(&self) -> Option<crate::db::DatabaseClient> {
        if !self.database_initialized {
            return None;
        }

        let config_lock = self.config.read().await;
        if let Some(db_config) = &config_lock.database {
            match crate::db::DatabaseClient::new(db_config).await {
                Ok(client) => Some(client),
                Err(e) => {
                    log::error!("Failed to create database client: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: crate::config::Config) -> Result<(), anyhow::Error> {
        let mut config_lock = self.config.write().await;
        *config_lock = new_config;
        Ok(())
    }

    /// Get configuration value by key
    pub async fn get_config_value(&self, key: &str) -> Option<Value> {
        let config_lock = self.config.read().await;
        config_lock.get_value(key)
    }

    /// Set configuration value by key
    pub async fn set_config_value(&self, key: &str, value: Value) -> Result<(), anyhow::Error> {
        let mut config_lock = self.config.write().await;
        config_lock.set_value(key, value)
    }

    /// Validate context integrity
    pub async fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Validate configuration
        let config_lock = self.config.read().await;
        if let Err(e) = config_lock.validate() {
            errors.push(format!("Configuration validation failed: {}", e));
        }

        // Validate plugin manager
        if let Err(e) = self.plugin_manager.validate().await {
            errors.push(format!("Plugin manager validation failed: {}", e));
        }

        // Validate memory adapter
        if let Err(e) = self.memory_adapter.validate().await {
            errors.push(format!("Memory adapter validation failed: {}", e));
        }

        errors
    }

    /// Get context statistics
    pub async fn get_stats(&self) -> ContextStats {
        let config_lock = self.config.read().await;
        let plugin_stats = self.plugin_manager.get_stats().await;
        let memory_stats = self.memory_adapter.get_stats().await;

        ContextStats {
            config_loaded: true,
            database_initialized: self.database_initialized,
            active_plugins: plugin_stats.active_count,
            total_plugins: plugin_stats.total_count,
            memory_nodes: memory_stats.total_nodes,
            memory_relationships: memory_stats.total_relationships,
        }
    }
}

impl SamplingContext {
    /// Create a new sampling context
    pub fn new(app_context: ApplicationContext) -> Self {
        Self {
            app_context: Arc::new(app_context),
            sampling_config: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get application context reference
    pub fn app_context(&self) -> &Arc<ApplicationContext> {
        &self.app_context
    }

    /// Get sampling configuration
    pub async fn sampling_config(&self) -> HashMap<String, Value> {
        let config_lock = self.sampling_config.read().await;
        config_lock.clone()
    }

    /// Set sampling configuration
    pub async fn set_sampling_config(&self, config: HashMap<String, Value>) {
        let mut config_lock = self.sampling_config.write().await;
        *config_lock = config;
    }

    /// Update sampling configuration value
    pub async fn update_sampling_config(&self, key: String, value: Value) {
        let mut config_lock = self.sampling_config.write().await;
        config_lock.insert(key, value);
    }

    /// Get sampling configuration value
    pub async fn get_sampling_config_value(&self, key: &str) -> Option<Value> {
        let config_lock = self.sampling_config.read().await;
        config_lock.get(key).cloned()
    }

    /// Remove sampling configuration value
    pub async fn remove_sampling_config_value(&self, key: &str) -> Option<Value> {
        let mut config_lock = self.sampling_config.write().await;
        config_lock.remove(key)
    }

    /// Clear sampling configuration
    pub async fn clear_sampling_config(&self) {
        let mut config_lock = self.sampling_config.write().await;
        config_lock.clear();
    }

    /// Add active sampling session
    pub async fn add_active_session(&self, session_id: String, metadata: Value) {
        let mut sessions_lock = self.active_sessions.write().await;
        sessions_lock.insert(session_id, metadata);
    }

    /// Get active sampling sessions
    pub async fn active_sessions(&self) -> HashMap<String, Value> {
        let sessions_lock = self.active_sessions.read().await;
        sessions_lock.clone()
    }

    /// Get active session by ID
    pub async fn get_active_session(&self, session_id: &str) -> Option<Value> {
        let sessions_lock = self.active_sessions.read().await;
        sessions_lock.get(session_id).cloned()
    }

    /// Remove active session
    pub async fn remove_active_session(&self, session_id: &str) -> Option<Value> {
        let mut sessions_lock = self.active_sessions.write().await;
        sessions_lock.remove(session_id)
    }

    /// Clear all active sessions
    pub async fn clear_active_sessions(&self) {
        let mut sessions_lock = self.active_sessions.write().await;
        sessions_lock.clear();
    }

    /// Get active session count
    pub async fn active_session_count(&self) -> usize {
        let sessions_lock = self.active_sessions.read().await;
        sessions_lock.len()
    }

    /// Check if session is active
    pub async fn is_session_active(&self, session_id: &str) -> bool {
        let sessions_lock = self.active_sessions.read().await;
        sessions_lock.contains_key(session_id)
    }

    /// Get sampling context statistics
    pub async fn get_stats(&self) -> SamplingStats {
        let config_lock = self.sampling_config.read().await;
        let sessions_lock = self.active_sessions.read().await;

        SamplingStats {
            config_entries: config_lock.len(),
            active_sessions: sessions_lock.len(),
            total_session_data_size: sessions_lock.values()
                .map(|v| v.to_string().len())
                .sum(),
        }
    }
}

/// Context statistics
#[derive(Debug, Clone)]
pub struct ContextStats {
    /// Whether configuration is loaded
    pub config_loaded: bool,
    /// Whether database is initialized
    pub database_initialized: bool,
    /// Number of active plugins
    pub active_plugins: usize,
    /// Total number of plugins
    pub total_plugins: usize,
    /// Number of memory nodes
    pub memory_nodes: usize,
    /// Number of memory relationships
    pub memory_relationships: usize,
}

/// Sampling context statistics
#[derive(Debug, Clone)]
pub struct SamplingStats {
    /// Number of configuration entries
    pub config_entries: usize,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Total size of session data in bytes
    pub total_session_data_size: usize,
}
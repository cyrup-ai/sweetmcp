use std::{collections::HashMap, sync::Arc};

use log;
use once_cell::sync::{Lazy, OnceCell};
use serde_json::Value;
use tokio::sync::RwLock;

// Import types from sibling modules/ Import items from submodules
pub mod logger;
pub mod memory_adapter;
pub mod rpc;

// Re-export key types/functions
pub use logger::ConsoleLogger;
pub use memory_adapter::MemoryContextAdapter;
pub use rpc::{
    ContextChangedNotification, ContextContent, ContextItem, GetContextRequest, GetContextResult,
    SubscribeContextRequest, SubscribeContextResult, context_get, context_subscribe,
};

// Add other necessary re-exports from submodules as needed

// Context subscription storage

#[derive(Debug, Clone)]
pub struct ContextSubscription {
    pub id: String,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub static CONTEXT_SUBSCRIPTIONS: Lazy<Arc<RwLock<HashMap<String, ContextSubscription>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Application-wide context containing global services and dependencies
#[derive(Clone)]
pub struct ApplicationContext {
    /// Configuration service reference
    config: Arc<RwLock<crate::config::Config>>,

    /// Console logger service with styling support
    logger: Arc<logger::ConsoleLogger>, // Use submodule path

    /// Plugin manager reference
    plugin_manager: Arc<crate::plugin::PluginManager>,

    /// Database client (if configured)
    database_initialized: bool,

    /// Memory system adapter
    memory_adapter: Arc<MemoryContextAdapter>,
}

/// Sampling-specific context containing sampling-related services
#[derive(Clone)]
pub struct SamplingContext {
    /// Shared reference to application context
    app_context: Arc<ApplicationContext>,

    /// Sampling-specific configuration
    sampling_config: Arc<RwLock<HashMap<String, Value>>>,

    /// Active sampling sessions
    active_sessions: Arc<RwLock<HashMap<String, Value>>>,
}

impl ApplicationContext {
    pub fn initialize(
        config_path: &std::path::Path,
        log_level: &str,
        _plugin_configs: &[crate::config::PluginConfig],
    ) -> crate::types::AsyncTask<Result<Option<Self>, anyhow::Error>> {
        // Clone these values for use in the async block
        let config_path = config_path.to_path_buf();
        let log_level = log_level.to_string();

        crate::types::AsyncTask::from_future(async move {
            // Read config file content
            let config_content = std::fs::read_to_string(&config_path)
                .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;

            // Parse configuration
            let config: crate::config::Config =
                crate::config::parse_config_from_str(&config_content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse config: {}", e))?;

            // Initialize logger
            let logger = logger::ConsoleLogger::new();
            crate::config::init_logger(None, Some(&log_level))
                .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))?;

            // Initialize plugin manager
            let plugin_manager = crate::plugin::PluginManager::new();

            // Initialize memory system
            let memory_config = sweetmcp_memory::MemoryConfig {
                database: sweetmcp_memory::utils::config::DatabaseConfig {
                    db_type: sweetmcp_memory::utils::config::DatabaseType::SurrealDB,
                    connection_string: "surrealkv://./data/mcp_context_memory.db".to_string(),
                    namespace: "mcp".to_string(),
                    database: "context_memory".to_string(),
                    username: None,
                    password: None,
                    pool_size: Some(10),
                    options: None,
                },
                vector_store: sweetmcp_memory::utils::config::VectorStoreConfig {
                    store_type: sweetmcp_memory::utils::config::VectorStoreType::SurrealDB,
                    embedding_model: sweetmcp_memory::utils::config::EmbeddingModelConfig {
                        model_type: sweetmcp_memory::utils::config::EmbeddingModelType::Custom,
                        model_name: "nomic-embed-text".to_string(),
                        api_key: None,
                        api_base: Some("http://localhost:11434/api/embeddings".to_string()),
                        options: None,
                    },
                    dimension: 768,
                    connection_string: None,
                    api_key: None,
                    options: None,
                },
                llm: sweetmcp_memory::utils::config::LLMConfig {
                    provider: sweetmcp_memory::utils::config::LLMProvider::Custom,
                    model_name: "llama2".to_string(),
                    api_key: None,
                    api_base: Some("http://localhost:11434/api/generate".to_string()),
                    temperature: Some(0.7),
                    max_tokens: Some(2048),
                    options: None,
                },
                cache: sweetmcp_memory::utils::config::CacheConfig {
                    enabled: true,
                    cache_type: sweetmcp_memory::utils::config::CacheType::Memory,
                    size: Some(10000),
                    ttl: Some(3600),
                    options: None,
                },
                logging: sweetmcp_memory::utils::config::LoggingConfig {
                    level: sweetmcp_memory::utils::config::LogLevel::Info,
                    file: Some("./logs/mcp_context_memory.log".to_string()),
                    console: true,
                    options: None,
                },
                api: None,
            };
            let memory_adapter = Arc::new(
                MemoryContextAdapter::new(memory_config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to initialize memory system: {}", e))?,
            );

            // Initialize database if configured
            let mut database_initialized = false;

            if let Some(app_db_config) = &config.database {
                // Extract values from the application database config structure
                let url = app_db_config
                    .url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Database URL missing"))?;
                let namespace = app_db_config.namespace.clone();
                let database = app_db_config.database.clone();
                let username = app_db_config.username.clone();
                let password = app_db_config.password.clone();

                // Validate WebSocket URL
                if !url.starts_with("ws://") && !url.starts_with("wss://") {
                    return Err(anyhow::anyhow!("Unsupported database URL type: {}", url));
                }
                log::info!("Initializing WebSocket database connection to {}", url);

                // Create our refined DatabaseConfig structure
                let db_config = crate::db::config::DatabaseConfig {
                    engine: crate::db::config::StorageEngine::Http,
                    path: None,
                    url: Some(url.clone()),
                    namespace,
                    database,
                    username,
                    password,
                };

                // Connect to the database using our client implementation
                let db_client = crate::db::client::connect_database(db_config).await;

                if let Ok(client) = db_client {
                    // Initialize CMS DAO if required
                    let cms_dao = crate::resource::cms::cms_dao::CmsDao::with_surrealdb(client);
                    if let Ok(_) = crate::resource::cms::cms_dao::init_cms_dao(cms_dao) {
                        database_initialized = true;
                        log::info!("CmsDao initialized successfully");
                    } else {
                        log::error!("Failed to initialize CmsDao");
                    }
                } else {
                    log::error!("Failed to connect to database: {:?}", db_client.err());
                }
            }

            // Return the initialized context
            Ok(Some(Self {
                config: Arc::new(RwLock::new(config)),
                logger: Arc::new(logger),
                plugin_manager: Arc::new(plugin_manager),
                database_initialized,
                memory_adapter,
            }))
        })
    }
    /// Access the global Configuration
    pub async fn config(&self) -> crate::config::Config {
        let config = self.config.clone();
        let config_guard = config.read().await;
        (*config_guard).clone()
    }
    /// Access the console logger (no await needed)
    pub fn logger(&self) -> Arc<logger::ConsoleLogger> {
        // Use submodule path
        self.logger.clone()
    }

    /// Access the plugin manager (no await needed)
    pub fn plugin_manager(&self) -> Arc<crate::plugin::PluginManager> {
        self.plugin_manager.clone()
    }

    /// Check if the database was successfully initialized
    pub fn is_database_initialized(&self) -> bool {
        self.database_initialized
    }

    /// Access the memory context adapter
    pub fn memory_adapter(&self) -> Arc<MemoryContextAdapter> {
        self.memory_adapter.clone()
    }
}

impl SamplingContext {
    /// Create a new sampling context linked to the application context
    pub fn new(app_context: ApplicationContext) -> Self {
        Self {
            app_context: Arc::new(app_context),
            sampling_config: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the application context
    pub fn application_context(&self) -> Arc<ApplicationContext> {
        self.app_context.clone()
    }

    /// Set sampling configuration
    pub async fn set_sampling_config(&self, config: HashMap<String, Value>) {
        let sampling_config = self.sampling_config.clone();
        let mut config_lock = sampling_config.write().await;
        *config_lock = config;
    }

    /// Get the sampling configuration
    pub async fn sampling_config(&self) -> HashMap<String, Value> {
        let sampling_config = self.sampling_config.clone();
        let config_lock = sampling_config.read().await;
        config_lock.clone()
    }

    /// Register an active sampling session
    pub async fn register_session(&self, session_id: String, metadata: Value) {
        let active_sessions = self.active_sessions.clone();
        let mut sessions_lock = active_sessions.write().await;
        sessions_lock.insert(session_id, metadata);
    }

    /// Get active sampling sessions
    pub async fn active_sessions(&self) -> HashMap<String, Value> {
        let active_sessions = self.active_sessions.clone();
        let sessions_lock = active_sessions.read().await;
        sessions_lock.clone()
    }
}

/// Global application context initialized at runtime
pub static APPLICATION_CONTEXT: OnceCell<ApplicationContext> = OnceCell::new();

/// Global sampling context initialized at runtime
pub static SAMPLING_CONTEXT: OnceCell<SamplingContext> = OnceCell::new();

/// Initialize the global application context
/// Must be called once during application startup
pub async fn initialize_global_context(
    config_path: &std::path::Path,
    log_level: &str,
    plugin_configs: &[crate::config::PluginConfig],
) -> Result<(), anyhow::Error> {
    let config_path = config_path.to_path_buf();
    let log_level = log_level.to_string();
    let plugin_configs = plugin_configs.to_vec();

    // Initialize the application context
    let app_context = ApplicationContext::initialize(&config_path, &log_level, &plugin_configs)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to initialize application context"))?;

    // Set the global instance
    APPLICATION_CONTEXT
        .set(app_context)
        .map_err(|_| anyhow::anyhow!("Global application context already initialized"))?;

    // Get the app context after it's been initialized
    let app_context = APPLICATION_CONTEXT
        .get()
        .ok_or_else(|| anyhow::anyhow!("Application context not initialized"))?;

    // Initialize the sampling context
    let sampling_context = SamplingContext::new(app_context.clone());
    SAMPLING_CONTEXT
        .set(sampling_context)
        .map_err(|_| anyhow::anyhow!("Global sampling context already initialized"))?;

    Ok(())
}

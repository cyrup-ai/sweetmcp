// src/lib.rs
//! Cognitive Memory System - A revolutionary memory architecture with quantum-inspired routing,
//! emergent agent evolution, and self-modifying capabilities.

pub mod api;
#[cfg(feature = "cognitive")]
pub mod cognitive;
pub mod graph;
pub mod llm;
pub mod memory;
pub mod migration;
pub mod schema;
pub mod utils;
pub mod vector;

// Re-export main types for convenience
pub use memory::MemoryManager;
pub use memory::MemoryMetadata;
pub use memory::MemoryNode;
pub use memory::SurrealDBMemoryManager;
pub use memory::SurrealDBMemoryManager as SurrealMemoryManager;

// Re-export cognitive system (conditional)
#[cfg(feature = "cognitive")]
pub use cognitive::{
    CognitiveMemoryManager, CognitiveMemoryNode, CognitiveSettings, CognitiveState,
    EvolutionMetadata, QuantumSignature,
};

pub use schema::MemoryType;

pub use utils::config::MemoryConfig;
pub use utils::error::Error;

// Conditional re-exports if API feature is enabled
#[cfg(feature = "api")]
pub use api::APIServer;
#[cfg(feature = "api")]
pub use utils::config::APIConfig;

/// Initialize the cognitive memory system (requires "cognitive" feature)
#[cfg(feature = "cognitive")]
pub async fn initialize_cognitive(
    url: &str,
    namespace: &str,
    database: &str,
    settings: CognitiveSettings,
) -> Result<CognitiveMemoryManager, Error> {
    CognitiveMemoryManager::new(url, namespace, database, settings)
        .await
        .map_err(|e| Error::Config(format!("Failed to initialize cognitive system: {}", e)))
}

/// Initialize the traditional memory system with SurrealDB using a configuration object.
/// This is a more robust approach than just a DB URL.
pub async fn initialize(config: &MemoryConfig) -> Result<SurrealMemoryManager, Error> {
    use surrealdb::engine::any::connect;
    // use surrealdb::opt::auth::Root; // Root auth might not always be needed or desired, depends on config

    // Connect to the database using details from config
    let db = connect(&config.database.connection_string)
        .await
        .map_err(|e| Error::Config(format!("Failed to connect to database: {e}")))?;

    // Use namespace and database from config
    db.use_ns(&config.database.namespace)
        .use_db(&config.database.database)
        .await
        .map_err(|e| {
            Error::Config(format!(
                "Failed to use namespace '{}' and database '{}': {}",
                config.database.namespace, config.database.database, e
            ))
        })?;

    // Handle authentication if username and password are provided
    if let (Some(user), Some(pass)) = (&config.database.username, &config.database.password) {
        db.signin(surrealdb::opt::auth::Root {
            username: user.as_str(),
            password: pass.as_str(),
        })
        .await
        .map_err(|e| Error::Config(format!("Database sign-in failed: {e}")))?;
    }

    // Create the memory manager
    let manager = SurrealMemoryManager::new(db.clone());

    // Initialize the manager (e.g., create tables/schemas if they don_t exist)
    manager.initialize().await?;

    Ok(manager)
}

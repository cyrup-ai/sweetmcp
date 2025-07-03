//! Memory storage abstraction layer

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

use crate::memory::{MemoryNode, MemoryRelationship};
use crate::utils::Result;

/// A pending store operation
pub struct PendingStore {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingStore {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingStore {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Store task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending retrieve operation
pub struct PendingRetrieve {
    rx: oneshot::Receiver<Result<MemoryNode>>,
}

impl PendingRetrieve {
    pub fn new(rx: oneshot::Receiver<Result<MemoryNode>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRetrieve {
    type Output = Result<MemoryNode>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Retrieve task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending update operation
pub struct PendingUpdate {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingUpdate {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingUpdate {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Update task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending delete operation
pub struct PendingDelete {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingDelete {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingDelete {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Delete task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending relationships retrieval
pub struct PendingRelationships {
    rx: oneshot::Receiver<Result<Vec<MemoryRelationship>>>,
}

impl PendingRelationships {
    pub fn new(rx: oneshot::Receiver<Result<Vec<MemoryRelationship>>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRelationships {
    type Output = Result<Vec<MemoryRelationship>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Get relationships task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending stats retrieval
pub struct PendingStats {
    rx: oneshot::Receiver<Result<StorageStats>>,
}

impl PendingStats {
    pub fn new(rx: oneshot::Receiver<Result<StorageStats>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingStats {
    type Output = Result<StorageStats>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::Internal(
                "Stats task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Memory storage trait for different backend implementations
pub trait MemoryStorage: Send + Sync {
    /// Store a memory
    fn store(&self, memory: MemoryNode) -> PendingStore;

    /// Retrieve a memory by ID
    fn retrieve(&self, id: String) -> PendingRetrieve;

    /// Update an existing memory
    fn update(&self, memory: MemoryNode) -> PendingUpdate;

    /// Delete a memory
    fn delete(&self, id: String) -> PendingDelete;

    /// Store a relationship
    fn store_relationship(&self, relationship: MemoryRelationship) -> PendingStore;

    /// Get relationships for a memory
    fn get_relationships(&self, memory_id: String) -> PendingRelationships;

    /// Delete a relationship
    fn delete_relationship(&self, relationship_id: String) -> PendingDelete;

    /// Batch operations for efficiency
    fn batch_store(&self, memories: Vec<MemoryNode>) -> PendingStore;

    /// Get storage statistics
    fn stats(&self) -> PendingStats;
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of memories
    pub total_memories: u64,

    /// Total number of relationships
    pub total_relationships: u64,

    /// Storage size in bytes
    pub storage_size_bytes: u64,

    /// Backend-specific stats
    pub backend_stats: HashMap<String, serde_json::Value>,
}

/// In-memory storage implementation (for testing and caching)
pub struct InMemoryStorage {
    memories: std::sync::Arc<tokio::sync::RwLock<HashMap<String, MemoryNode>>>,
    relationships: std::sync::Arc<tokio::sync::RwLock<HashMap<String, Vec<MemoryRelationship>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            memories: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            relationships: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage for InMemoryStorage {
    fn store(&self, memory: MemoryNode) -> PendingStore {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();

        tokio::spawn(async move {
            let mut memories = memories.write().await;
            memories.insert(memory.id.clone(), memory);
            let _ = tx.send(Ok(()));
        });

        PendingStore::new(rx)
    }

    fn retrieve(&self, id: String) -> PendingRetrieve {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();

        tokio::spawn(async move {
            let memories = memories.read().await;
            let result = memories.get(&id).cloned().ok_or_else(|| {
                crate::utils::error::Error::NotFound(format!("Memory {} not found", id))
            });
            let _ = tx.send(result);
        });

        PendingRetrieve::new(rx)
    }

    fn update(&self, memory: MemoryNode) -> PendingUpdate {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();

        tokio::spawn(async move {
            let mut memories = memories.write().await;
            let result = if memories.contains_key(&memory.id) {
                memories.insert(memory.id.clone(), memory.clone());
                Ok(())
            } else {
                Err(crate::utils::error::Error::NotFound(format!(
                    "Memory {} not found",
                    memory.id
                )))
            };
            let _ = tx.send(result);
        });

        PendingUpdate::new(rx)
    }

    fn delete(&self, id: String) -> PendingDelete {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();
        let relationships = self.relationships.clone();

        tokio::spawn(async move {
            let mut memories = memories.write().await;
            let result = if memories.remove(&id).is_some() {
                // Also remove associated relationships
                let mut relationships = relationships.write().await;
                relationships.remove(&id);
                Ok(())
            } else {
                Err(crate::utils::error::Error::NotFound(format!(
                    "Memory {} not found",
                    id
                )))
            };
            let _ = tx.send(result);
        });

        PendingDelete::new(rx)
    }

    fn store_relationship(&self, relationship: MemoryRelationship) -> PendingStore {
        let (tx, rx) = oneshot::channel();
        let relationships = self.relationships.clone();

        tokio::spawn(async move {
            let mut relationships = relationships.write().await;

            // Add to source memory's relationships
            relationships
                .entry(relationship.source_id.clone())
                .or_insert_with(Vec::new)
                .push(relationship.clone());

            // Add to target memory's relationships (for bidirectional access)
            relationships
                .entry(relationship.target_id.clone())
                .or_insert_with(Vec::new)
                .push(relationship);

            let _ = tx.send(Ok(()));
        });

        PendingStore::new(rx)
    }

    fn get_relationships(&self, memory_id: String) -> PendingRelationships {
        let (tx, rx) = oneshot::channel();
        let relationships = self.relationships.clone();

        tokio::spawn(async move {
            let relationships = relationships.read().await;
            let result = Ok(relationships.get(&memory_id).cloned().unwrap_or_default());
            let _ = tx.send(result);
        });

        PendingRelationships::new(rx)
    }

    fn delete_relationship(&self, relationship_id: String) -> PendingDelete {
        let (tx, rx) = oneshot::channel();
        let relationships = self.relationships.clone();

        tokio::spawn(async move {
            let mut relationships = relationships.write().await;

            // Remove from all memory relationship lists
            for relations in relationships.values_mut() {
                relations.retain(|r| r.id != relationship_id);
            }

            let _ = tx.send(Ok(()));
        });

        PendingDelete::new(rx)
    }

    fn batch_store(&self, memories_to_store: Vec<MemoryNode>) -> PendingStore {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();

        tokio::spawn(async move {
            let mut memories = memories.write().await;
            for memory in memories_to_store {
                memories.insert(memory.id.clone(), memory);
            }
            let _ = tx.send(Ok(()));
        });

        PendingStore::new(rx)
    }

    fn stats(&self) -> PendingStats {
        let (tx, rx) = oneshot::channel();
        let memories = self.memories.clone();
        let relationships = self.relationships.clone();

        tokio::spawn(async move {
            let memories = memories.read().await;
            let relationships = relationships.read().await;

            let total_relationships: usize = relationships.values().map(|v| v.len()).sum();

            let result = Ok(StorageStats {
                total_memories: memories.len() as u64,
                total_relationships: total_relationships as u64,
                storage_size_bytes: 0, // Not meaningful for in-memory storage
                backend_stats: HashMap::new(),
            });
            let _ = tx.send(result);
        });

        PendingStats::new(rx)
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,

    /// Connection string
    pub connection_string: Option<String>,

    /// Additional options
    pub options: HashMap<String, serde_json::Value>,
}

/// Supported storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackend {
    /// In-memory storage
    Memory,
    /// SurrealDB
    SurrealDB,
    /// PostgreSQL
    PostgreSQL,
    /// SQLite
    SQLite,
    /// MongoDB
    MongoDB,
    /// Redis
    Redis,
}

/// Storage factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage instance from configuration
    pub async fn create(config: &StorageConfig) -> Result<Box<dyn MemoryStorage>> {
        match config.backend {
            StorageBackend::Memory => Ok(Box::new(InMemoryStorage::new())),
            _ => Err(crate::utils::error::Error::NotImplemented(format!(
                "Storage backend {:?} not implemented",
                config.backend
            ))),
        }
    }
}

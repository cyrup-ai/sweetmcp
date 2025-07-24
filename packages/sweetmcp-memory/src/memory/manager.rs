//! Lock-free implementation using DashMap for blazing-fast concurrent access.

use std::sync::Arc;

use crate::memory::{
    MemoryMetadata, MemoryNode, MemoryRelationship, MemoryType, filter::MemoryFilter,
    storage::MemoryStorage,
    caching::MemoryCache,
    storage_coordinator::StorageCoordinator,
    lifecycle::MemoryLifecycle,
};
use crate::utils::Result;
use crate::vector::VectorStore;

/// High-level memory manager that coordinates between different memory components.
/// Lock-free implementation using DashMap for concurrent access without blocking.
pub struct MemoryCoordinator<S, V>
where
    S: MemoryStorage,
    V: VectorStore,
{
    lifecycle: MemoryLifecycle<S, V>,
}

impl<S, V> MemoryCoordinator<S, V>
where
    S: MemoryStorage + Send + Sync,
    V: VectorStore + Send + Sync,
{
    /// Create a new memory coordinator with lock-free operations
    pub fn new(storage: Arc<S>, vector_store: V) -> Self {
        let coordinator = StorageCoordinator::new(storage, vector_store);
        let cache = MemoryCache::new();
        let lifecycle = MemoryLifecycle::new(coordinator, cache);

        Self { lifecycle }
    }

    /// Add a new memory (lock-free operation)
    pub async fn add_memory(
        &self,
        content: String,
        memory_type: MemoryType,
        metadata: MemoryMetadata,
    ) -> Result<MemoryNode> {
        self.lifecycle.add_memory(content, memory_type, metadata).await
    }

    /// Update an existing memory (lock-free operation)
    pub async fn update_memory(
        &self,
        id: &str,
        content: Option<String>,
        metadata: Option<MemoryMetadata>,
    ) -> Result<MemoryNode> {
        self.lifecycle.update_memory(id, content, metadata).await
    }

    /// Delete a memory (lock-free operation)
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        self.lifecycle.delete_memory(id).await
    }

    /// Search for memories (lock-free operation)
    pub async fn search_memories(
        &self,
        query: &str,
        filter: Option<MemoryFilter>,
        top_k: usize,
    ) -> Result<Vec<MemoryNode>> {
        self.lifecycle.search_memories(query, filter, top_k).await
    }

    /// Get memories by filter (lock-free operation using cache)
    pub async fn get_memories(&self, filter: MemoryFilter) -> Result<Vec<MemoryNode>> {
        // Delegate to lifecycle which uses cache filtering
        Ok(self.lifecycle.cache.get_memories_by_filter(filter))
    }

    /// Get memory count (lock-free atomic read)
    pub fn memory_count(&self) -> usize {
        self.lifecycle.cache.memory_count()
    }

    /// Get memory by ID (lock-free cache lookup)
    pub async fn get_memory(&self, id: &str) -> Result<Option<MemoryNode>> {
        // Try cache first (lock-free read)
        if let Some(cached) = self.lifecycle.cache.get(id) {
            return Ok(Some(cached));
        }

        // Cache miss - try storage
        match self.lifecycle.coordinator.retrieve_memory(id).await {
            Ok(memory) => {
                // Add to cache for future access
                self.lifecycle.cache.insert(memory.clone());
                Ok(Some(memory))
            }
            Err(crate::utils::Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Add a relationship between memories
    pub async fn add_relationship(
        &self,
        source_id: &str,
        target_id: &str,
        relationship_type: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<MemoryRelationship> {
        self.lifecycle
            .add_relationship(source_id, target_id, relationship_type, metadata)
            .await
    }

    /// Get relationships for a memory
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<MemoryRelationship>> {
        self.lifecycle.get_relationships(memory_id).await
    }

    /// Generate embedding for text content with SIMD-optimized performance
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.lifecycle.generate_embedding(text).await
    }
}

// Note: Types already imported at top of file
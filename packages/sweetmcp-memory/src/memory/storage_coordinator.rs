//! Storage coordination logic extracted from memory manager

use std::sync::Arc;

use crate::memory::{
    MemoryNode, MemoryRelationship, MemoryType, filter::MemoryFilter,
    storage::MemoryStorage,
};
use crate::utils::Result;
use crate::vector::VectorStore;

/// Storage coordinator for managing storage and vector store operations
pub struct StorageCoordinator<S, V>
where
    S: MemoryStorage,
    V: VectorStore,
{
    storage: Arc<S>,
    /// Lock-free concurrent access to vector store using Arc
    vector_store: Arc<V>,
}

impl<S, V> StorageCoordinator<S, V>
where
    S: MemoryStorage + Send + Sync,
    V: VectorStore + Send + Sync,
{
    /// Create a new storage coordinator
    pub fn new(storage: Arc<S>, vector_store: V) -> Self {
        Self {
            storage,
            vector_store: Arc::new(vector_store),
        }
    }

    /// Store memory in persistent storage
    pub async fn store_memory(&self, memory: &MemoryNode) -> Result<()> {
        self.storage.store(memory.clone()).await
    }

    /// Update memory in persistent storage
    pub async fn update_memory(&self, memory: &MemoryNode) -> Result<()> {
        self.storage.update(memory.clone()).await
    }

    /// Delete memory from persistent storage
    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        self.storage.delete(id.to_string()).await
    }

    /// Retrieve memory from storage
    pub async fn retrieve_memory(&self, id: &str) -> Result<MemoryNode> {
        self.storage.retrieve(id.to_string()).await
    }

    /// Add vector to vector store (lock-free direct access)
    pub async fn add_vector(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.vector_store.add(id, embedding, metadata).await
    }

    /// Update vector in vector store (lock-free direct access)
    pub async fn update_vector(
        &self,
        id: String,
        embedding: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.vector_store.update(id, embedding, metadata).await
    }

    /// Delete vector from vector store (lock-free direct access)
    pub async fn delete_vector(&self, id: &str) -> Result<()> {
        self.vector_store.delete(id.to_string()).await
    }

    /// Search in vector store (lock-free direct access)
    pub async fn search_vectors(
        &self,
        query_embedding: Vec<f32>,
        top_k: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<crate::vector::SearchResult>> {
        self.vector_store
            .search(query_embedding, top_k, filter.map(|f| f.clone()))
            .await
    }

    /// Generate query embedding (lock-free direct access)
    pub async fn embed_query(&self, query: String) -> Result<Vec<f32>> {
        self.vector_store.embed(query).await
    }

    /// Store relationship in storage
    pub async fn store_relationship(&self, relationship: &MemoryRelationship) -> Result<()> {
        self.storage.store_relationship(relationship.clone()).await
    }

    /// Get relationships from storage
    pub async fn get_relationships(&self, memory_id: &str) -> Result<Vec<MemoryRelationship>> {
        self.storage.get_relationships(memory_id.to_string()).await
    }

    /// Get storage reference
    pub fn storage(&self) -> &Arc<S> {
        &self.storage
    }

    /// Get vector store reference  
    pub fn vector_store(&self) -> &Arc<V> {
        &self.vector_store
    }
}
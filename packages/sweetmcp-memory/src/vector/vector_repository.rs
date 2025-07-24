//! Vector repository for managing vector collections
//! Lock-free implementation using DashMap for blazing-fast concurrent access.

use dashmap::DashMap;
use std::collections::HashMap;

use crate::utils::Result;
use crate::vector::{DistanceMetric, VectorIndexConfig};
use crate::vector::batch_operations::BatchOperations;
use crate::vector::collection_metadata::{VectorCollection, VectorCollectionHandle};
use crate::vector::collection_operations::CollectionOperations;
use crate::vector::vector_operations::VectorOperations;

/// Vector repository for managing multiple vector collections.
/// Lock-free implementation using DashMap for concurrent access without blocking.
pub struct VectorRepository {
    /// Collections storage - lock-free concurrent HashMap using DashMap
    collections: DashMap<String, VectorCollectionHandle>,

    /// Default index configuration
    default_config: VectorIndexConfig,
}

impl VectorRepository {
    /// Create a new vector repository
    pub fn new(default_dimensions: usize) -> Self {
        let default_config = VectorIndexConfig {
            metric: DistanceMetric::Cosine,
            dimensions: default_dimensions,
            index_type: crate::vector::vector_index::IndexType::Flat,
            parameters: HashMap::new(),
        };

        Self {
            collections: DashMap::new(),
            default_config,
        }
    }

    /// Create a new collection (lock-free operation)
    pub fn create_collection(
        &self,
        name: String,
        dimensions: usize,
        metric: DistanceMetric,
    ) -> Result<VectorCollection> {
        CollectionOperations::create_collection(
            &self.collections,
            &self.default_config,
            name,
            dimensions,
            metric,
        )
    }

    /// Delete a collection (lock-free operation)
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        CollectionOperations::delete_collection(&self.collections, name)
    }

    /// Get collection metadata (lock-free operation)
    pub fn get_collection(&self, name: &str) -> Result<VectorCollection> {
        CollectionOperations::get_collection(&self.collections, name)
    }

    /// List all collections (lock-free operation)
    pub fn list_collections(&self) -> Result<Vec<VectorCollection>> {
        CollectionOperations::list_collections(&self.collections)
    }

    /// Add a vector to a collection (lock-free operation)
    pub fn add_vector(&self, collection_name: &str, id: String, vector: Vec<f32>) -> Result<()> {
        VectorOperations::add_vector(&self.collections, collection_name, id, vector)
    }

    /// Remove a vector from a collection (lock-free operation)
    pub fn remove_vector(&self, collection_name: &str, id: &str) -> Result<()> {
        VectorOperations::remove_vector(&self.collections, collection_name, id)
    }

    /// Search for similar vectors in a collection (lock-free operation)
    pub fn search(
        &self,
        collection_name: &str,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        VectorOperations::search(&self.collections, collection_name, query, k)
    }

    /// Build/rebuild index for a collection (lock-free operation)
    pub fn build_index(&self, collection_name: &str) -> Result<()> {
        VectorOperations::build_index(&self.collections, collection_name)
    }
}

/// Batch operations for vector repository
impl VectorRepository {
    /// Add multiple vectors to a collection (lock-free operation)
    pub fn add_vectors_batch(
        &self,
        collection_name: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        BatchOperations::add_vectors_batch(&self.collections, collection_name, vectors)
    }

    /// Remove multiple vectors from a collection (lock-free operation)
    pub fn remove_vectors_batch(&self, collection_name: &str, ids: Vec<String>) -> Result<()> {
        BatchOperations::remove_vectors_batch(&self.collections, collection_name, ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vector_repository() {
        let repo = VectorRepository::new(128);

        // Create collection
        let collection = repo
            .create_collection("test_collection".to_string(), 3, DistanceMetric::Cosine)
            .unwrap();

        assert_eq!(collection.name, "test_collection");
        assert_eq!(collection.dimensions, 3);

        // Add vector
        let id = uuid::Uuid::new_v4().to_string();
        repo.add_vector("test_collection", id.clone(), vec![1.0, 0.0, 0.0])
            .unwrap();

        // Search
        let results = repo.search("test_collection", &[1.0, 0.0, 0.0], 1).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);

        // Delete collection
        repo.delete_collection("test_collection").unwrap();

        // Verify deletion
        assert!(repo.get_collection("test_collection").is_err());
    }
}

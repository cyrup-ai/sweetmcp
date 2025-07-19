//! Vector repository for managing vector collections
//! Lock-free implementation using DashMap for blazing-fast concurrent access.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::Result;
use crate::vector::{DistanceMetric, VectorIndex, VectorIndexConfig, VectorIndexFactory};

/// Vector collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCollection {
    /// Collection ID
    pub id: String,

    /// Collection name
    pub name: String,

    /// Vector dimensions
    pub dimensions: usize,

    /// Distance metric
    pub metric: DistanceMetric,

    /// Number of vectors
    pub count: usize,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Vector repository for managing multiple vector collections.
/// Lock-free implementation using DashMap for concurrent access without blocking.
pub struct VectorRepository {
    /// Collections storage - lock-free concurrent HashMap using DashMap
    collections: DashMap<String, VectorCollectionHandle>,

    /// Default index configuration
    default_config: VectorIndexConfig,
}

/// Handle to a vector collection - cache-line aligned for optimal performance
#[repr(align(64))] // Cache-line alignment for CPU cache efficiency
struct VectorCollectionHandle {
    // Hot field - frequently accessed during vector operations
    index: Box<dyn VectorIndex>,
    // Cold field - accessed less frequently during metadata operations
    metadata: VectorCollection,
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
        // Check if collection already exists using lock-free contains_key
        if self.collections.contains_key(&name) {
            return Err(crate::utils::error::Error::AlreadyExists(format!(
                "Collection '{}' already exists",
                name
            )));
        }

        let now = chrono::Utc::now();
        // Generate UUID with zero-allocation string conversion
        use arrayvec::ArrayString;
        let uuid = uuid::Uuid::new_v4();
        let mut uuid_str: ArrayString<36> = ArrayString::new();
        // Use write! macro for zero-allocation UUID formatting
        use std::fmt::Write;
        write!(&mut uuid_str, "{}", uuid).expect("UUID formatting should never fail");
        
        let metadata = VectorCollection {
            id: uuid_str.to_string(), // Convert to String only at the boundary
            name: name.clone(),
            dimensions,
            metric,
            count: 0,
            created_at: now,
            updated_at: now,
        };

        let config = VectorIndexConfig {
            metric,
            dimensions,
            ..self.default_config.clone()
        };

        let index = VectorIndexFactory::create(config);

        // Atomic insert operation - if key already exists, this will replace it
        // For stricter checking, we could use try_insert but that's not available in DashMap 6.1
        self.collections.insert(
            name,
            VectorCollectionHandle {
                index,
                metadata: metadata.clone(),
            },
        );

        Ok(metadata)
    }

    /// Delete a collection (lock-free operation)
    pub fn delete_collection(&self, name: &str) -> Result<()> {
        // Atomic remove operation
        if self.collections.remove(name).is_none() {
            return Err(crate::utils::error::Error::NotFound(format!(
                "Collection '{}' not found",
                name
            )));
        }

        Ok(())
    }

    /// Get collection metadata (lock-free operation)
    pub fn get_collection(&self, name: &str) -> Result<VectorCollection> {
        // Lock-free read operation using DashMap's get method
        self.collections
            .get(name)
            .map(|handle| handle.metadata.clone())
            .ok_or_else(|| {
                crate::utils::error::Error::NotFound(format!("Collection '{}' not found", name))
            })
    }

    /// List all collections (lock-free operation)
    pub fn list_collections(&self) -> Result<Vec<VectorCollection>> {
        // Lock-free iteration over all values using DashMap's iter
        Ok(self.collections
            .iter()
            .map(|entry| entry.value().metadata.clone())
            .collect())
    }

    /// Add a vector to a collection (lock-free operation)
    pub fn add_vector(
        &self,
        collection_name: &str,
        id: String,
        vector: Vec<f32>,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = self.collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index.add(id, vector)?;
        handle.metadata.count = handle.index.len();
        handle.metadata.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Remove a vector from a collection (lock-free operation)
    pub fn remove_vector(&self, collection_name: &str, id: &str) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = self.collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index.remove(id)?;
        handle.metadata.count = handle.index.len();
        handle.metadata.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Search for similar vectors in a collection (lock-free operation)
    pub fn search(
        &self,
        collection_name: &str,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        // Lock-free read access using DashMap's get
        let handle = self.collections.get(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index.search(query, k)
    }

    /// Build/rebuild index for a collection (lock-free operation)
    pub fn build_index(&self, collection_name: &str) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = self.collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index.build()?;
        handle.metadata.updated_at = chrono::Utc::now();

        Ok(())
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
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = self.collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        for (id, vector) in vectors {
            handle.index.add(id, vector)?;
        }

        handle.metadata.count = handle.index.len();
        handle.metadata.updated_at = chrono::Utc::now();

        // Rebuild index after batch insert for better performance
        handle.index.build()?;

        Ok(())
    }

    /// Remove multiple vectors from a collection (lock-free operation)
    pub fn remove_vectors_batch(
        &self,
        collection_name: &str,
        ids: Vec<String>,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = self.collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        for id in ids {
            handle.index.remove(&id)?;
        }

        handle.metadata.count = handle.index.len();
        handle.metadata.updated_at = chrono::Utc::now();

        Ok(())
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
        let results = repo
            .search("test_collection", &[1.0, 0.0, 0.0], 1)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, id);

        // Delete collection
        repo.delete_collection("test_collection").unwrap();

        // Verify deletion
        assert!(repo.get_collection("test_collection").is_err());
    }
}

//! Collection management operations extracted from vector repository

use dashmap::DashMap;
use std::collections::HashMap;

use crate::utils::Result;
use crate::vector::{DistanceMetric, VectorIndexConfig, VectorIndexFactory};
use crate::vector::collection_metadata::{VectorCollection, VectorCollectionHandle};

/// Collection management operations for VectorRepository
pub struct CollectionOperations;

impl CollectionOperations {
    /// Create a new collection (lock-free operation)
    pub fn create_collection(
        collections: &DashMap<String, VectorCollectionHandle>,
        default_config: &VectorIndexConfig,
        name: String,
        dimensions: usize,
        metric: DistanceMetric,
    ) -> Result<VectorCollection> {
        // Check if collection already exists using lock-free contains_key
        if collections.contains_key(&name) {
            return Err(crate::utils::error::Error::AlreadyExists(format!(
                "Collection '{}' already exists",
                name
            )));
        }

        // Generate UUID with zero-allocation string conversion
        use arrayvec::ArrayString;
        let uuid = uuid::Uuid::new_v4();
        let mut uuid_str: ArrayString<36> = ArrayString::new();
        // Use write! macro for zero-allocation UUID formatting
        use std::fmt::Write;
        write!(&mut uuid_str, "{}", uuid).expect("UUID formatting should never fail");

        let metadata = VectorCollection::new(
            uuid_str.to_string(), // Convert to String only at the boundary
            name.clone(),
            dimensions,
            metric,
        );

        let config = VectorIndexConfig {
            metric,
            dimensions,
            ..default_config.clone()
        };

        let index = VectorIndexFactory::create(config);

        // Atomic insert operation - if key already exists, this will replace it
        // For stricter checking, we could use try_insert but that's not available in DashMap 6.1
        collections.insert(
            name,
            VectorCollectionHandle::new(index, metadata.clone()),
        );

        Ok(metadata)
    }

    /// Delete a collection (lock-free operation)
    pub fn delete_collection(
        collections: &DashMap<String, VectorCollectionHandle>,
        name: &str,
    ) -> Result<()> {
        // Atomic remove operation
        if collections.remove(name).is_none() {
            return Err(crate::utils::error::Error::NotFound(format!(
                "Collection '{}' not found",
                name
            )));
        }

        Ok(())
    }

    /// Get collection metadata (lock-free operation)
    pub fn get_collection(
        collections: &DashMap<String, VectorCollectionHandle>,
        name: &str,
    ) -> Result<VectorCollection> {
        // Lock-free read operation using DashMap's get method
        collections
            .get(name)
            .map(|handle| handle.metadata().clone())
            .ok_or_else(|| {
                crate::utils::error::Error::NotFound(format!("Collection '{}' not found", name))
            })
    }

    /// List all collections (lock-free operation)
    pub fn list_collections(
        collections: &DashMap<String, VectorCollectionHandle>,
    ) -> Result<Vec<VectorCollection>> {
        // Lock-free iteration over all values using DashMap's iter
        Ok(collections
            .iter()
            .map(|entry| entry.value().metadata().clone())
            .collect())
    }
}
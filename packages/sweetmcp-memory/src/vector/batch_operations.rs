//! Batch operations for vector repository extracted from vector repository

use dashmap::DashMap;

use crate::utils::Result;
use crate::vector::collection_metadata::VectorCollectionHandle;

/// Batch operations for VectorRepository
pub struct BatchOperations;

impl BatchOperations {
    /// Add multiple vectors to a collection (lock-free operation)
    pub fn add_vectors_batch(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
        vectors: Vec<(String, Vec<f32>)>,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        for (id, vector) in vectors {
            handle.index_mut().add(id, vector)?;
        }

        let index_len = handle.index().len();
        handle.metadata_mut().update_count(index_len);

        // Rebuild index after batch insert for better performance
        handle.index_mut().build()?;

        Ok(())
    }

    /// Remove multiple vectors from a collection (lock-free operation)
    pub fn remove_vectors_batch(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
        ids: Vec<String>,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        for id in ids {
            handle.index_mut().remove(&id)?;
        }

        let index_len = handle.index().len();
        handle.metadata_mut().update_count(index_len);

        Ok(())
    }
}
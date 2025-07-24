//! Vector manipulation operations extracted from vector repository

use dashmap::DashMap;

use crate::utils::Result;
use crate::vector::collection_metadata::VectorCollectionHandle;

/// Vector manipulation operations for VectorRepository
pub struct VectorOperations;

impl VectorOperations {
    /// Add a vector to a collection (lock-free operation)
    pub fn add_vector(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
        id: String,
        vector: Vec<f32>,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index_mut().add(id, vector)?;
        let index_len = handle.index().len();
        handle.metadata_mut().update_count(index_len);

        Ok(())
    }

    /// Remove a vector from a collection (lock-free operation)
    pub fn remove_vector(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
        id: &str,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index_mut().remove(id)?;
        let index_len = handle.index().len();
        handle.metadata_mut().update_count(index_len);

        Ok(())
    }

    /// Search for similar vectors in a collection (lock-free operation)
    pub fn search(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(String, f32)>> {
        // Lock-free read access using DashMap's get
        let handle = collections.get(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index().search(query, k)
    }

    /// Build/rebuild index for a collection (lock-free operation)
    pub fn build_index(
        collections: &DashMap<String, VectorCollectionHandle>,
        collection_name: &str,
    ) -> Result<()> {
        // Lock-free mutable access using DashMap's get_mut
        let mut handle = collections.get_mut(collection_name).ok_or_else(|| {
            crate::utils::error::Error::NotFound(format!(
                "Collection '{collection_name}' not found"
            ))
        })?;

        handle.index_mut().build()?;
        handle.metadata_mut().touch();

        Ok(())
    }
}
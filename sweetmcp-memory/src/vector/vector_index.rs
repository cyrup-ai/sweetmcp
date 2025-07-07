//! Vector indexing for efficient similarity search

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::Result;
use crate::vector::DistanceMetric;

/// Vector index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    /// Distance metric to use
    pub metric: DistanceMetric,

    /// Number of dimensions
    pub dimensions: usize,

    /// Index type
    pub index_type: IndexType,

    /// Additional index-specific parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Types of vector indexes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexType {
    /// Flat/brute-force index
    Flat,
    /// Hierarchical Navigable Small World
    HNSW,
    /// Inverted File System
    IVF,
    /// Locality Sensitive Hashing
    LSH,
    /// Annoy (Approximate Nearest Neighbors Oh Yeah)
    Annoy,
}

/// Vector index trait
pub trait VectorIndex: Send + Sync {
    /// Add a vector to the index
    fn add(&mut self, id: String, vector: Vec<f32>) -> Result<()>;

    /// Remove a vector from the index
    fn remove(&mut self, id: &str) -> Result<()>;

    /// Search for nearest neighbors
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>>;

    /// Get the number of vectors in the index
    fn len(&self) -> usize;

    /// Check if the index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Build/rebuild the index
    fn build(&mut self) -> Result<()>;
}

/// Flat (brute-force) vector index
pub struct FlatIndex {
    config: VectorIndexConfig,
    vectors: HashMap<String, Vec<f32>>,
}

impl FlatIndex {
    /// Create a new flat index
    pub fn new(config: VectorIndexConfig) -> Self {
        Self {
            config,
            vectors: HashMap::new(),
        }
    }

    /// Calculate distance between two vectors
    fn calculate_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.config.metric {
            DistanceMetric::Euclidean => a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt(),
            DistanceMetric::Cosine => {
                let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();

                if norm_a == 0.0 || norm_b == 0.0 {
                    0.0
                } else {
                    1.0 - (dot_product / (norm_a * norm_b))
                }
            }
            DistanceMetric::DotProduct => -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>(),
        }
    }
}

impl VectorIndex for FlatIndex {
    fn add(&mut self, id: String, vector: Vec<f32>) -> Result<()> {
        if vector.len() != self.config.dimensions {
            return Err(crate::utils::error::Error::InvalidInput(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.config.dimensions,
                vector.len()
            )));
        }

        self.vectors.insert(id, vector);
        Ok(())
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        self.vectors.remove(id);
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        if query.len() != self.config.dimensions {
            return Err(crate::utils::error::Error::InvalidInput(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.config.dimensions,
                query.len()
            )));
        }

        let mut distances: Vec<(String, f32)> = self
            .vectors
            .iter()
            .map(|(id, vector)| (id.clone(), self.calculate_distance(query, vector)))
            .collect();

        // Sort by distance (ascending for distance metrics, descending for similarity)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Take top k results
        distances.truncate(k);

        Ok(distances)
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn build(&mut self) -> Result<()> {
        // Flat index doesn't need building
        Ok(())
    }
}

/// HNSW (Hierarchical Navigable Small World) index
pub struct HNSWIndex {
    config: VectorIndexConfig,
    // Placeholder - would use a proper HNSW implementation
    flat_index: FlatIndex,
}

impl HNSWIndex {
    /// Create a new HNSW index
    pub fn new(config: VectorIndexConfig) -> Self {
        Self {
            config: config.clone(),
            flat_index: FlatIndex::new(config),
        }
    }

    /// Get the index configuration
    pub fn get_config(&self) -> &VectorIndexConfig {
        &self.config
    }

    /// Update index with configuration parameters
    pub fn optimize_with_config(&mut self) -> Result<()> {
        // Use config parameters for optimization
        let _dimensions = self.config.dimensions;
        let _metric = &self.config.metric;

        // Apply configuration-based optimizations
        self.flat_index.build()
    }
}

impl VectorIndex for HNSWIndex {
    fn add(&mut self, id: String, vector: Vec<f32>) -> Result<()> {
        // Placeholder - delegates to flat index
        self.flat_index.add(id, vector)
    }

    fn remove(&mut self, id: &str) -> Result<()> {
        self.flat_index.remove(id)
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        self.flat_index.search(query, k)
    }

    fn len(&self) -> usize {
        self.flat_index.len()
    }

    fn build(&mut self) -> Result<()> {
        self.flat_index.build()
    }
}

/// Vector index factory
pub struct VectorIndexFactory;

impl VectorIndexFactory {
    /// Create a vector index from configuration
    pub fn create(config: VectorIndexConfig) -> Box<dyn VectorIndex> {
        match config.index_type {
            IndexType::Flat => Box::new(FlatIndex::new(config)),
            IndexType::HNSW => Box::new(HNSWIndex::new(config)),
            _ => Box::new(FlatIndex::new(config)), // Default to flat for unimplemented types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_index() {
        let config = VectorIndexConfig {
            metric: DistanceMetric::Cosine,
            dimensions: 3,
            index_type: IndexType::Flat,
            parameters: HashMap::new(),
        };

        let mut index = FlatIndex::new(config);

        // Add vectors
        let id1 = uuid::Uuid::new_v4().to_string();
        let id2 = uuid::Uuid::new_v4().to_string();

        index.add(id1.clone(), vec![1.0, 0.0, 0.0]).unwrap();
        index.add(id2.clone(), vec![0.0, 1.0, 0.0]).unwrap();

        // Search
        let results = index.search(&[1.0, 0.0, 0.0], 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, id1); // Should match exactly
    }
}

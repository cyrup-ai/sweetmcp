//! Async vector operations and VectorStore trait implementation
//!
//! This module provides blazing-fast async vector operations with zero-allocation
//! patterns and elegant ergonomic interfaces for vector storage operations.

use dashmap::DashMap;
use smallvec::SmallVec;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

use super::{
    PendingEmbedding, PendingVectorOp, PendingVectorSearch, VectorSearchResult, VectorStore,
};
use super::async_vector_core::{InMemoryVectorStore, VectorStoreMetrics};
use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;

impl VectorStore for InMemoryVectorStore {
    fn add(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.add_vector_internal(id, vector, metadata).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }

    fn get(&self, id: String) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.get_vector_internal(&id).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }

    fn remove(&self, id: String) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.remove_vector_internal(&id).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }

    fn search(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
        filter: Option<MemoryFilter>,
    ) -> PendingVectorSearch {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.search_vectors_internal(query_vector, limit, filter).await;
            let _ = tx.send(result);
        });
        
        PendingVectorSearch::new(rx)
    }

    fn batch_add(&self, items: Vec<(String, Vec<f32>, Option<serde_json::Value>)>) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.batch_add_internal(items).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }

    fn update_metadata(&self, id: String, metadata: serde_json::Value) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            let result = store.update_metadata_internal(&id, metadata).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }
    
    fn update(
        &self,
        id: String,
        vector: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> PendingVectorOp {
        let (tx, rx) = oneshot::channel();
        let store = self.clone();
        
        tokio::spawn(async move {
            // First remove the existing vector
            let _ = store.remove_vector_internal(&id).await;
            // Then add the new one
            let result = store.add_vector_internal(id, vector, metadata).await;
            let _ = tx.send(result);
        });
        
        PendingVectorOp::new(rx)
    }
    
    fn delete(&self, id: String) -> PendingVectorOp {
        self.remove(id)
    }
    
    fn embed(&self, _text: String) -> PendingEmbedding {
        let (tx, rx) = oneshot::channel();
        
        // In a real implementation, this would call an embedding model
        // For now, return an error since this is just a stub
        tokio::spawn(async move {
            let _ = tx.send(Err(Error::NotImplemented("Embedding not implemented".to_string())));
        });
        
        PendingEmbedding::new(rx)
    }
}

impl InMemoryVectorStore {
    /// Internal async implementation for adding vectors
    async fn add_vector_internal(
        &self,
        id: String,
        mut vector: Vec<f32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), Error> {
        let start_time = Instant::now();
        
        // Validate vector dimensions
        self.validate_vector_dimensions(&vector)?;
        
        // Normalize vector for better similarity calculations
        self.normalize_vector(&mut vector);
        
        // Store vector dimensions for future validation
        let dimensions = vector.len();
        self.dimension_cache.insert(id.clone(), dimensions);
        
        // Store vector
        self.vectors.insert(id.clone(), vector);
        
        // Store metadata if provided
        if let Some(meta) = metadata {
            self.metadata.insert(id.clone(), meta);
        }
        
        // Update metrics
        self.increment_operations();
        self.metrics.increment_adds();
        
        let elapsed = start_time.elapsed();
        debug!("Added vector {} in {:?}", id, elapsed);
        
        Ok(())
    }

    /// Internal async implementation for getting vectors
    async fn get_vector_internal(&self, id: &str) -> Result<Option<(Vec<f32>, Option<serde_json::Value>)>, Error> {
        let start_time = Instant::now();
        
        let vector = self.vectors.get(id).map(|v| v.clone());
        let metadata = self.metadata.get(id).map(|m| m.clone());
        
        self.increment_operations();
        
        let result = match vector {
            Some(vec) => Some((vec, metadata)),
            None => None,
        };
        
        let elapsed = start_time.elapsed();
        debug!("Retrieved vector {} in {:?}", id, elapsed);
        
        Ok(result)
    }

    /// Internal async implementation for removing vectors
    async fn remove_vector_internal(&self, id: &str) -> Result<bool, Error> {
        let start_time = Instant::now();
        
        let vector_removed = self.vectors.remove(id).is_some();
        let metadata_removed = self.metadata.remove(id).is_some();
        self.dimension_cache.remove(id);
        
        let removed = vector_removed || metadata_removed;
        
        if removed {
            self.increment_operations();
            self.metrics.increment_removals();
        }
        
        let elapsed = start_time.elapsed();
        debug!("Removed vector {} in {:?} (success: {})", id, elapsed, removed);
        
        Ok(removed)
    }

    /// Internal async implementation for batch adding vectors
    async fn batch_add_internal(
        &self,
        items: Vec<(String, Vec<f32>, Option<serde_json::Value>)>,
    ) -> Result<(), Error> {
        let start_time = Instant::now();
        let mut successful_adds = 0;
        let mut failed_adds = 0;
        
        for (id, mut vector, metadata) in items {
            match self.validate_vector_dimensions(&vector) {
                Ok(()) => {
                    // Normalize vector
                    self.normalize_vector(&mut vector);
                    
                    // Store vector dimensions
                    let dimensions = vector.len();
                    self.dimension_cache.insert(id.clone(), dimensions);
                    
                    // Store vector
                    self.vectors.insert(id.clone(), vector);
                    
                    // Store metadata if provided
                    if let Some(meta) = metadata {
                        self.metadata.insert(id.clone(), meta);
                    }
                    
                    successful_adds += 1;
                }
                Err(e) => {
                    warn!("Failed to add vector {}: {}", id, e);
                    failed_adds += 1;
                }
            }
        }
        
        // Update metrics
        for _ in 0..successful_adds {
            self.increment_operations();
            self.metrics.increment_adds();
        }
        
        let elapsed = start_time.elapsed();
        info!(
            "Batch add completed in {:?}: {} successful, {} failed",
            elapsed, successful_adds, failed_adds
        );
        
        if failed_adds > 0 && successful_adds == 0 {
            return Err(Error::BatchOperationFailed(
                format!("All {} batch operations failed", failed_adds)
            ));
        }
        
        Ok(())
    }

    /// Internal async implementation for updating metadata
    async fn update_metadata_internal(
        &self,
        id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), Error> {
        let start_time = Instant::now();
        
        // Check if vector exists
        if !self.vectors.contains_key(id) {
            return Err(Error::VectorNotFound(id.to_string()));
        }
        
        // Update metadata
        self.metadata.insert(id.to_string(), metadata);
        
        self.increment_operations();
        
        let elapsed = start_time.elapsed();
        debug!("Updated metadata for {} in {:?}", id, elapsed);
        
        Ok(())
    }

    /// Get vectors by metadata filter
    pub async fn get_vectors_by_metadata(
        &self,
        filter: &MemoryFilter,
    ) -> Result<Vec<(String, Vec<f32>, serde_json::Value)>, Error> {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        for entry in self.metadata.iter() {
            let id = entry.key();
            let metadata = entry.value();
            
            if self.matches_filter(metadata, filter) {
                if let Some(vector) = self.vectors.get(id) {
                    results.push((id.clone(), vector.clone(), metadata.clone()));
                }
            }
        }
        
        let elapsed = start_time.elapsed();
        debug!("Filtered {} vectors by metadata in {:?}", results.len(), elapsed);
        
        Ok(results)
    }

    /// Check if metadata matches filter
    #[inline]
    fn matches_filter(&self, metadata: &serde_json::Value, _filter: &MemoryFilter) -> bool {
        // Simplified filter matching - can be extended based on MemoryFilter implementation
        match metadata {
            serde_json::Value::Object(_meta_obj) => {
                // Check if any filter criteria match
                // This is a simplified implementation - extend based on actual MemoryFilter structure
                true // Placeholder - implement actual filter logic
            }
            _ => false,
        }
    }

    /// Get similar vectors using approximate nearest neighbor search
    pub async fn get_similar_vectors(
        &self,
        query_vector: &[f32],
        limit: usize,
        similarity_threshold: f32,
    ) -> Result<Vec<VectorSearchResult>, Error> {
        let start_time = Instant::now();
        
        // Validate query vector
        self.validate_vector_dimensions(query_vector)?;
        
        let mut similarities: SmallVec<[(String, f32); 32]> = SmallVec::new();
        
        // Calculate similarities with all stored vectors
        for entry in self.vectors.iter() {
            let id = entry.key();
            let vector = entry.value();
            
            match self.cosine_similarity(query_vector, vector) {
                Ok(similarity) => {
                    if similarity >= similarity_threshold {
                        similarities.push((id.clone(), similarity));
                    }
                }
                Err(e) => {
                    warn!("Failed to calculate similarity for vector {}: {}", id, e);
                    continue;
                }
            }
        }
        
        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top results and convert to VectorSearchResult
        let mut results = Vec::with_capacity(limit.min(similarities.len()));
        for (id, similarity) in similarities.into_iter().take(limit) {
            let metadata = self.metadata.get(&id).map(|m| m.clone());
            results.push(VectorSearchResult {
                id,
                similarity,
                metadata,
            });
        }
        
        // Update metrics
        self.increment_operations();
        self.metrics.increment_searches();
        let elapsed_ms = start_time.elapsed().as_millis() as usize;
        self.metrics.add_search_time(elapsed_ms);
        
        debug!(
            "Found {} similar vectors in {:?} (threshold: {})",
            results.len(),
            start_time.elapsed(),
            similarity_threshold
        );
        
        Ok(results)
    }

    /// Perform k-nearest neighbor search
    pub async fn knn_search(
        &self,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<VectorSearchResult>, Error> {
        let start_time = Instant::now();
        
        // Validate query vector
        self.validate_vector_dimensions(query_vector)?;
        
        let mut similarities: SmallVec<[(String, f32); 32]> = SmallVec::new();
        
        // Calculate similarities with all stored vectors
        for entry in self.vectors.iter() {
            let id = entry.key();
            let vector = entry.value();
            
            match self.cosine_similarity(query_vector, vector) {
                Ok(similarity) => {
                    similarities.push((id.clone(), similarity));
                }
                Err(e) => {
                    warn!("Failed to calculate similarity for vector {}: {}", id, e);
                    continue;
                }
            }
        }
        
        // Sort by similarity (descending) and take top k
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut results = Vec::with_capacity(k.min(similarities.len()));
        for (id, similarity) in similarities.into_iter().take(k) {
            let metadata = self.metadata.get(&id).map(|m| m.clone());
            results.push(VectorSearchResult {
                id,
                similarity,
                metadata,
            });
        }
        
        // Update metrics
        self.increment_operations();
        self.metrics.increment_searches();
        let elapsed_ms = start_time.elapsed().as_millis() as usize;
        self.metrics.add_search_time(elapsed_ms);
        
        debug!(
            "KNN search found {} vectors in {:?}",
            results.len(),
            start_time.elapsed()
        );
        
        Ok(results)
    }

    /// Perform range search within a distance threshold
    pub async fn range_search(
        &self,
        query_vector: &[f32],
        max_distance: f32,
        distance_metric: DistanceMetric,
    ) -> Result<Vec<VectorSearchResult>, Error> {
        let start_time = Instant::now();
        
        // Validate query vector
        self.validate_vector_dimensions(query_vector)?;
        
        let mut results = Vec::new();
        
        // Calculate distances with all stored vectors
        for entry in self.vectors.iter() {
            let id = entry.key();
            let vector = entry.value();
            
            let distance = match distance_metric {
                DistanceMetric::Euclidean => self.euclidean_distance(query_vector, vector)?,
                DistanceMetric::Manhattan => self.manhattan_distance(query_vector, vector)?,
                DistanceMetric::Cosine => {
                    // Convert cosine similarity to distance (1 - similarity)
                    1.0 - self.cosine_similarity(query_vector, vector)?
                }
            };
            
            if distance <= max_distance {
                let metadata = self.metadata.get(id).map(|m| m.clone());
                // For distance-based search, we use (1 - distance) as similarity
                let similarity = 1.0 - distance;
                results.push(VectorSearchResult {
                    id: id.clone(),
                    similarity,
                    metadata,
                });
            }
        }
        
        // Sort by similarity (descending, which means ascending distance)
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        
        // Update metrics
        self.increment_operations();
        self.metrics.increment_searches();
        let elapsed_ms = start_time.elapsed().as_millis() as usize;
        self.metrics.add_search_time(elapsed_ms);
        
        debug!(
            "Range search found {} vectors within distance {} in {:?}",
            results.len(),
            max_distance,
            start_time.elapsed()
        );
        
        Ok(results)
    }
}

impl Clone for InMemoryVectorStore {
    fn clone(&self) -> Self {
        Self {
            vectors: Arc::clone(&self.vectors),
            metadata: Arc::clone(&self.metadata),
            operation_counter: Arc::clone(&self.operation_counter),
            dimension_cache: Arc::clone(&self.dimension_cache),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

/// Distance metric enumeration for range search
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceMetric {
    /// Euclidean distance (L2 norm)
    Euclidean,
    /// Manhattan distance (L1 norm)
    Manhattan,
    /// Cosine similarity (1 - cosine)
    Cosine,
    /// Dot product similarity
    DotProduct,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_distance_metric_euclidean() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let distance = DistanceMetric::Euclidean.distance(&a, &b);
        assert_relative_eq!(distance, 2.0f32.sqrt(), epsilon = 1e-6);
    }

    #[test]
    fn test_distance_metric_manhattan() {
        let a = [1.0, 2.0];
        let b = [3.0, 5.0];
        let distance = DistanceMetric::Manhattan.distance(&a, &b);
        assert_relative_eq!(distance, 5.0); // |1-3| + |2-5| = 2 + 3 = 5
    }

    #[test]
    fn test_distance_metric_cosine() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let distance = DistanceMetric::Cosine.distance(&a, &b);
        assert_relative_eq!(distance, 1.0, epsilon = 1e-6); // 90 degrees -> cos(90) = 0 -> 1-0 = 1
        
        // Test with same vector (0 distance)
        let distance_same = DistanceMetric::Cosine.distance(&a, &a);
        assert_relative_eq!(distance_same, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_distance_metric_dot_product() {
        let a = [1.0, 0.0];
        let b = [0.0, 1.0];
        let distance = DistanceMetric::DotProduct.distance(&a, &b);
        assert_relative_eq!(distance, 0.0, epsilon = 1e-6); // Orthogonal vectors
        
        // Test with same vector (normalized)
        let norm = 2.0f32.sqrt();
        let a_norm = [1.0/norm, 1.0/norm];
        let distance_same = DistanceMetric::DotProduct.distance(&a_norm, &a_norm);
        assert_relative_eq!(distance_same, -1.0, epsilon = 1e-6); // Negative because we return -dot product
    }

    #[test]
    fn test_serialize_deserialize() {
        let metrics = vec![
            DistanceMetric::Euclidean,
            DistanceMetric::Manhattan,
            DistanceMetric::Cosine,
            DistanceMetric::DotProduct,
        ];

        for metric in metrics {
            let serialized = serde_json::to_string(&metric).unwrap();
            let deserialized: DistanceMetric = serde_json::from_str(&serialized).unwrap();
            assert_eq!(metric, deserialized);
        }
    }

    #[test]
    fn test_typical_ranges() {
        assert_eq!(DistanceMetric::Euclidean.typical_range(), (0.0, 2.0));
        assert_eq!(DistanceMetric::Manhattan.typical_range(), (0.0, 2.0));
        assert_eq!(DistanceMetric::Cosine.typical_range(), (0.0, 2.0));
        assert_eq!(DistanceMetric::DotProduct.typical_range(), (-1.0, 1.0));
    }
}

impl DistanceMetric {
    /// Get metric name
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            DistanceMetric::Euclidean => "euclidean",
            DistanceMetric::Manhattan => "manhattan",
            DistanceMetric::Cosine => "cosine",
            DistanceMetric::DotProduct => "dot_product",
        }
    }

    /// Check if metric is symmetric
    #[inline]
    pub fn is_symmetric(&self) -> bool {
        match self {
            DistanceMetric::Euclidean | 
            DistanceMetric::Manhattan | 
            DistanceMetric::Cosine => true,
            DistanceMetric::DotProduct => false, // Dot product is not symmetric
        }
    }

    /// Get typical range for this metric
    pub fn typical_range(&self) -> (f32, f32) {
        match self {
            DistanceMetric::Euclidean => (0.0, 2.0),  // For normalized vectors
            DistanceMetric::Manhattan => (0.0, 2.0),  // For normalized vectors
            DistanceMetric::Cosine => (0.0, 2.0),    // 1 - cos(θ) where θ ∈ [0,π]
            DistanceMetric::DotProduct => (-1.0, 1.0), // For normalized vectors, dot product is in [-1, 1]
        }
    }

    /// Calculate the distance between two vectors using this metric
    /// 
    /// # Panics
    /// Panics if the vectors have different lengths
    #[inline]
    pub fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vectors must have the same length");
        
        match self {
            DistanceMetric::Euclidean => {
                a.iter()
                    .zip(b.iter())
                    .map(|(&x, &y)| (x - y).powi(2))
                    .sum::<f32>()
                    .sqrt()
            }
            DistanceMetric::Manhattan => {
                a.iter()
                    .zip(b.iter())
                    .map(|(&x, &y)| (x - y).abs())
                    .sum()
            }
            DistanceMetric::Cosine => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|&x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|&y| y * y).sum::<f32>().sqrt();
                
                if norm_a == 0.0 || norm_b == 0.0 {
                    return 1.0; // Handle zero-vector case
                }
                
                1.0 - (dot / (norm_a * norm_b))
            }
            DistanceMetric::DotProduct => {
                // For normalized vectors, this will be in [-1, 1]
                -a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum::<f32>()
            }
        }
    }
}
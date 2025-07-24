//! Core async vector store structures and management
//!
//! This module provides the core async vector store functionality with zero-allocation
//! patterns and blazing-fast performance for vector storage and retrieval.

use dashmap::DashMap;
use smallvec::SmallVec;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

use super::{
    PendingEmbedding, PendingVectorOp, PendingVectorSearch, VectorSearchResult, VectorStore,
};
use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;

/// In-memory vector store implementation with lock-free concurrent access
pub struct InMemoryVectorStore {
    /// Vector storage with lock-free concurrent access
    vectors: Arc<DashMap<String, Vec<f32>>>,
    /// Metadata storage with lock-free concurrent access
    metadata: Arc<DashMap<String, serde_json::Value>>,
    /// Operation counter for metrics and performance tracking
    operation_counter: Arc<AtomicUsize>,
    /// Vector dimension cache for validation
    dimension_cache: Arc<DashMap<String, usize>>,
    /// Performance metrics
    metrics: Arc<VectorStoreMetrics>,
}

impl InMemoryVectorStore {
    /// Create a new lock-free in-memory vector store
    #[inline]
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(DashMap::new()),
            metadata: Arc::new(DashMap::new()),
            operation_counter: Arc::new(AtomicUsize::new(0)),
            dimension_cache: Arc::new(DashMap::new()),
            metrics: Arc::new(VectorStoreMetrics::new()),
        }
    }

    /// Create with initial capacity for better performance
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vectors: Arc::new(DashMap::with_capacity(capacity)),
            metadata: Arc::new(DashMap::with_capacity(capacity)),
            operation_counter: Arc::new(AtomicUsize::new(0)),
            dimension_cache: Arc::new(DashMap::with_capacity(16)), // Typical number of different dimensions
            metrics: Arc::new(VectorStoreMetrics::new()),
        }
    }

    /// Get the total number of operations performed (for metrics)
    #[inline]
    pub fn operation_count(&self) -> usize {
        self.operation_counter.load(Ordering::Relaxed)
    }

    /// Get the number of stored vectors
    #[inline]
    pub fn vector_count(&self) -> usize {
        self.vectors.len()
    }

    /// Get the number of metadata entries
    #[inline]
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Get vector dimensions for a specific ID
    #[inline]
    pub fn get_vector_dimensions(&self, id: &str) -> Option<usize> {
        self.vectors.get(id).map(|v| v.len())
    }

    /// Check if vector exists
    #[inline]
    pub fn contains_vector(&self, id: &str) -> bool {
        self.vectors.contains_key(id)
    }

    /// Check if metadata exists
    #[inline]
    pub fn contains_metadata(&self, id: &str) -> bool {
        self.metadata.contains_key(id)
    }

    /// Get current metrics
    #[inline]
    pub fn metrics(&self) -> &VectorStoreMetrics {
        &self.metrics
    }

    /// Increment operation counter
    #[inline]
    fn increment_operations(&self) {
        self.operation_counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Validate vector dimensions
    #[inline]
    fn validate_vector_dimensions(&self, vector: &[f32]) -> Result<(), Error> {
        if vector.is_empty() {
            return Err(Error::InvalidVector("Vector cannot be empty".to_string()));
        }

        if vector.len() > 4096 {
            return Err(Error::InvalidVector("Vector dimensions exceed maximum limit".to_string()));
        }

        // Check for invalid values
        for (i, &value) in vector.iter().enumerate() {
            if !value.is_finite() {
                return Err(Error::InvalidVector(
                    format!("Invalid value at dimension {}: {}", i, value)
                ));
            }
        }

        Ok(())
    }

    /// Normalize vector for better similarity calculations
    #[inline]
    fn normalize_vector(&self, vector: &mut [f32]) {
        let magnitude = vector.iter().map(|&x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in vector.iter_mut() {
                *value /= magnitude;
            }
        }
    }

    /// Calculate vector magnitude
    #[inline]
    fn vector_magnitude(&self, vector: &[f32]) -> f32 {
        vector.iter().map(|&x| x * x).sum::<f32>().sqrt()
    }

    /// Calculate cosine similarity between two vectors
    #[inline]
    pub fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> Result<f32, Error> {
        if a.len() != b.len() {
            return Err(Error::DimensionMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
        let magnitude_a = self.vector_magnitude(a);
        let magnitude_b = self.vector_magnitude(b);

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (magnitude_a * magnitude_b))
    }

    /// Calculate euclidean distance between two vectors
    #[inline]
    pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, Error> {
        if a.len() != b.len() {
            return Err(Error::DimensionMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let distance_squared: f32 = a.iter()
            .zip(b.iter())
            .map(|(&x, &y)| {
                let diff = x - y;
                diff * diff
            })
            .sum();

        Ok(distance_squared.sqrt())
    }

    /// Calculate manhattan distance between two vectors
    #[inline]
    pub fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, Error> {
        if a.len() != b.len() {
            return Err(Error::DimensionMismatch {
                expected: a.len(),
                actual: b.len(),
            });
        }

        let distance: f32 = a.iter()
            .zip(b.iter())
            .map(|(&x, &y)| (x - y).abs())
            .sum();

        Ok(distance)
    }

    /// Get vector by ID
    #[inline]
    pub fn get_vector(&self, id: &str) -> Option<Vec<f32>> {
        self.vectors.get(id).map(|v| v.clone())
    }

    /// Get metadata by ID
    #[inline]
    pub fn get_metadata(&self, id: &str) -> Option<serde_json::Value> {
        self.metadata.get(id).map(|v| v.clone())
    }

    /// Remove vector and associated metadata
    #[inline]
    pub fn remove(&self, id: &str) -> bool {
        let vector_removed = self.vectors.remove(id).is_some();
        let metadata_removed = self.metadata.remove(id).is_some();
        
        if vector_removed || metadata_removed {
            self.increment_operations();
            self.metrics.increment_removals();
        }

        vector_removed || metadata_removed
    }

    /// Clear all vectors and metadata
    #[inline]
    pub fn clear(&self) {
        let vector_count = self.vectors.len();
        let metadata_count = self.metadata.len();
        
        self.vectors.clear();
        self.metadata.clear();
        self.dimension_cache.clear();
        
        if vector_count > 0 || metadata_count > 0 {
            self.increment_operations();
            self.metrics.increment_clears();
            info!("Cleared {} vectors and {} metadata entries", vector_count, metadata_count);
        }
    }

    /// Get all vector IDs
    #[inline]
    pub fn get_all_ids(&self) -> Vec<String> {
        self.vectors.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get storage statistics
    #[inline]
    pub fn get_storage_stats(&self) -> VectorStorageStats {
        let vector_count = self.vectors.len();
        let metadata_count = self.metadata.len();
        let total_operations = self.operation_count();
        
        // Calculate memory usage estimation
        let vector_memory = self.vectors.iter()
            .map(|entry| entry.value().len() * std::mem::size_of::<f32>())
            .sum::<usize>();
        
        let metadata_memory = self.metadata.iter()
            .map(|entry| {
                serde_json::to_string(entry.value())
                    .map_or(0, |s| s.len())
            })
            .sum::<usize>();

        VectorStorageStats {
            vector_count,
            metadata_count,
            total_operations,
            estimated_memory_usage: vector_memory + metadata_memory,
            average_vector_size: if vector_count > 0 {
                vector_memory / vector_count
            } else {
                0
            },
        }
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector store performance metrics
#[derive(Debug)]
pub struct VectorStoreMetrics {
    /// Number of add operations
    pub adds: AtomicUsize,
    /// Number of search operations
    pub searches: AtomicUsize,
    /// Number of remove operations
    pub removals: AtomicUsize,
    /// Number of clear operations
    pub clears: AtomicUsize,
    /// Total search time in milliseconds
    pub total_search_time_ms: AtomicUsize,
    /// Number of cache hits
    pub cache_hits: AtomicUsize,
    /// Number of cache misses
    pub cache_misses: AtomicUsize,
}

impl VectorStoreMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            adds: AtomicUsize::new(0),
            searches: AtomicUsize::new(0),
            removals: AtomicUsize::new(0),
            clears: AtomicUsize::new(0),
            total_search_time_ms: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        }
    }

    /// Increment add operations
    #[inline]
    pub fn increment_adds(&self) {
        self.adds.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment search operations
    #[inline]
    pub fn increment_searches(&self) {
        self.searches.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment remove operations
    #[inline]
    pub fn increment_removals(&self) {
        self.removals.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment clear operations
    #[inline]
    pub fn increment_clears(&self) {
        self.clears.fetch_add(1, Ordering::Relaxed);
    }

    /// Add search time
    #[inline]
    pub fn add_search_time(&self, time_ms: usize) {
        self.total_search_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }

    /// Increment cache hits
    #[inline]
    pub fn increment_cache_hits(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment cache misses
    #[inline]
    pub fn increment_cache_misses(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cache hit ratio
    #[inline]
    pub fn cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Get average search time
    #[inline]
    pub fn average_search_time_ms(&self) -> f64 {
        let total_time = self.total_search_time_ms.load(Ordering::Relaxed);
        let search_count = self.searches.load(Ordering::Relaxed);
        
        if search_count == 0 {
            0.0
        } else {
            total_time as f64 / search_count as f64
        }
    }

    /// Get total operations
    #[inline]
    pub fn total_operations(&self) -> usize {
        self.adds.load(Ordering::Relaxed)
            + self.searches.load(Ordering::Relaxed)
            + self.removals.load(Ordering::Relaxed)
            + self.clears.load(Ordering::Relaxed)
    }
}

impl Default for VectorStoreMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Vector storage statistics
#[derive(Debug, Clone)]
pub struct VectorStorageStats {
    /// Number of vectors stored
    pub vector_count: usize,
    /// Number of metadata entries
    pub metadata_count: usize,
    /// Total operations performed
    pub total_operations: usize,
    /// Estimated memory usage in bytes
    pub estimated_memory_usage: usize,
    /// Average vector size in bytes
    pub average_vector_size: usize,
}

impl VectorStorageStats {
    /// Get memory efficiency (vectors per MB)
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.estimated_memory_usage == 0 {
            return 0.0;
        }
        
        let memory_mb = self.estimated_memory_usage as f64 / (1024.0 * 1024.0);
        self.vector_count as f64 / memory_mb
    }

    /// Check if storage is healthy
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.vector_count > 0 && self.memory_efficiency() > 10.0
    }
}
//! Async vector store module integration and re-exports
//!
//! This module provides ergonomic re-exports and integration for the decomposed
//! async vector store components with zero-allocation patterns.

pub use async_vector_core::{
    InMemoryVectorStore, VectorStoreMetrics, VectorStorageStats,
};

pub use async_vector_operations::{
    DistanceMetric,
};

pub use async_vector_optimization::{
    SearchStrategy,
};

// Import and re-export existing modules from the vector directory
use super::async_vector_core;
use super::async_vector_operations;
use super::async_vector_optimization;

/// Create a new in-memory vector store with default configuration
#[inline]
pub fn new_vector_store() -> InMemoryVectorStore {
    InMemoryVectorStore::new()
}

/// Create a new in-memory vector store with specified capacity
#[inline]
pub fn new_vector_store_with_capacity(capacity: usize) -> InMemoryVectorStore {
    InMemoryVectorStore::with_capacity(capacity)
}

/// Vector store builder for ergonomic configuration
#[derive(Debug, Default)]
pub struct VectorStoreBuilder {
    capacity: Option<usize>,
}

impl VectorStoreBuilder {
    /// Create a new vector store builder
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial capacity
    #[inline]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Build the vector store
    #[inline]
    pub fn build(self) -> InMemoryVectorStore {
        match self.capacity {
            Some(capacity) => InMemoryVectorStore::with_capacity(capacity),
            None => InMemoryVectorStore::new(),
        }
    }
}

/// Convenience function to create a vector store builder
#[inline]
pub fn vector_store() -> VectorStoreBuilder {
    VectorStoreBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_store_creation() {
        let store = new_vector_store();
        assert_eq!(store.vector_count(), 0);
        assert_eq!(store.metadata_count(), 0);
    }

    #[test]
    fn test_vector_store_builder() {
        let store = vector_store()
            .with_capacity(1000)
            .build();
        
        assert_eq!(store.vector_count(), 0);
        assert_eq!(store.metadata_count(), 0);
    }

    #[test]
    fn test_distance_metrics() {
        assert_eq!(DistanceMetric::Euclidean.name(), "euclidean");
        assert_eq!(DistanceMetric::Manhattan.name(), "manhattan");
        assert_eq!(DistanceMetric::Cosine.name(), "cosine");
        
        assert!(DistanceMetric::Euclidean.is_symmetric());
        assert!(DistanceMetric::Manhattan.is_symmetric());
        assert!(DistanceMetric::Cosine.is_symmetric());
    }

    #[test]
    fn test_distance_functions() {
        // Test that distance metric functions work
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        
        let _euclidean = DistanceMetric::Euclidean.calculate(&vec1, &vec2);
        let _manhattan = DistanceMetric::Manhattan.calculate(&vec1, &vec2);
        let _cosine = DistanceMetric::Cosine.calculate(&vec1, &vec2);
    }

    #[test]
    fn test_search_strategies() {
        // Test that all search strategies are available
        let _brute = SearchStrategy::BruteForce;
        let _filtered = SearchStrategy::FilteredSearch;
        let _approx = SearchStrategy::ApproximateSearch;
        let _hybrid = SearchStrategy::HybridSearch;
    }
}
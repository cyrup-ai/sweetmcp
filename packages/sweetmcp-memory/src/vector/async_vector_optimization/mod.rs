//! Async Vector Optimization Coordination
//!
//! This module provides blazing-fast async vector optimization with zero allocation
//! optimizations and comprehensive coordination capabilities.

// Core modules
pub mod coordinator_core;
pub mod coordinator_types;
pub mod coordinator_metrics;
pub mod coordinator_analysis;
pub mod coordinator_operations;
pub mod coordinator_config;
pub mod search_strategies;
pub mod optimization_algorithms;

// Re-export key types for ergonomic usage
pub use coordinator_core::AsyncVectorOptimizationCoordinator;
pub use coordinator_types::{
    OptimizationSpec, OptimizationPipelineResult, SearchMetrics, OptimizationMetrics,
    CoordinationMetrics,
};
pub use coordinator_analysis::{
    RecentPerformance, PerformanceTrend, MetricsSummary, VectorCharacteristics,
    OptimizationRecommendation, OptimizationParameters,
};
pub use coordinator_config::{CoordinatorConfig, PerformanceMetrics};

// Re-export macros
pub use crate::{optimize_vectors, search_vectors};

// Re-export related modules for convenience
pub use super::async_vector_operations::{VectorSearchResult, VectorStore, DistanceMetric};
pub use super::search_strategies::SearchStrategy;
pub use super::optimization_algorithms::OptimizationAlgorithm;

/// Create a new async vector optimization coordinator with default configuration
#[inline]
pub fn new_coordinator() -> AsyncVectorOptimizationCoordinator {
    AsyncVectorOptimizationCoordinator::new()
}

/// Create a new async vector optimization coordinator with custom configuration
#[inline]
pub fn new_coordinator_with_config(config: CoordinatorConfig) -> AsyncVectorOptimizationCoordinator {
    AsyncVectorOptimizationCoordinator::with_config(config)
}

/// Create an optimization specification builder
#[inline]
pub fn optimization_spec() -> OptimizationSpec {
    OptimizationSpec::new()
}

/// Create coordinator configuration builder
#[inline]
pub fn coordinator_config() -> CoordinatorConfig {
    CoordinatorConfig::new()
}

/// Utility functions for common operations
pub mod utils {
    use super::*;
    use crate::utils::error::Error;

    /// Quick vector search with default settings
    #[inline]
    pub async fn quick_search(
        query_vector: &[f32],
        vectors: &[(String, Vec<f32>)],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, Error> {
        let mut coordinator = new_coordinator();
        coordinator.execute_optimized_search(
            query_vector,
            vectors,
            limit,
            None,
            DistanceMetric::Cosine,
        ).await
    }

    /// Quick optimization with adaptive algorithms
    #[inline]
    pub async fn quick_optimize(
        vectors: &mut [(String, Vec<f32>)],
    ) -> Result<OptimizationPipelineResult, Error> {
        let coordinator = new_coordinator();
        coordinator.execute_adaptive_optimization(vectors).await
    }

    /// Get recommended configuration for dataset
    #[inline]
    pub fn recommended_config(vector_count: usize, dimensions: usize) -> CoordinatorConfig {
        if vector_count < 1000 {
            CoordinatorConfig::new()
                .with_max_concurrent_operations(2)
                .with_default_search_strategy(SearchStrategy::BruteForce)
        } else if vector_count < 10000 {
            CoordinatorConfig::new()
                .with_max_concurrent_operations(4)
                .with_default_search_strategy(SearchStrategy::FilteredSearch)
        } else {
            CoordinatorConfig::new()
                .with_max_concurrent_operations(8)
                .with_default_search_strategy(SearchStrategy::ApproximateNearestNeighbor)
                .with_adaptive_optimization(true)
        }
    }

    /// Validate vectors for optimization
    #[inline]
    pub fn validate_vectors(vectors: &[(String, Vec<f32>)]) -> Result<(), Error> {
        if vectors.is_empty() {
            return Err(Error::InvalidInput("Empty vector dataset".to_string()));
        }

        let expected_dims = vectors[0].1.len();
        for (id, vector) in vectors {
            if vector.len() != expected_dims {
                return Err(Error::InvalidInput(
                    format!("Vector {} has {} dimensions, expected {}", id, vector.len(), expected_dims)
                ));
            }

            if vector.iter().any(|&x| !x.is_finite()) {
                return Err(Error::InvalidInput(
                    format!("Vector {} contains non-finite values", id)
                ));
            }
        }

        Ok(())
    }

    /// Estimate memory usage for vector dataset
    #[inline]
    pub fn estimate_memory_usage(vector_count: usize, dimensions: usize) -> usize {
        // Base memory for vectors (f32 = 4 bytes)
        let vector_memory = vector_count * dimensions * 4;
        
        // Additional memory for IDs (estimate 32 bytes per ID)
        let id_memory = vector_count * 32;
        
        // Overhead for data structures (estimate 20% overhead)
        let overhead = (vector_memory + id_memory) / 5;
        
        vector_memory + id_memory + overhead
    }

    /// Get performance recommendations based on metrics
    #[inline]
    pub fn performance_recommendations(metrics: &PerformanceMetrics) -> Vec<&'static str> {
        let mut recommendations = Vec::new();

        if !metrics.is_healthy() {
            recommendations.push("System performance is below optimal levels");
        }

        if metrics.overall_score() < 0.5 {
            recommendations.push("Consider reducing concurrent operations or dataset size");
        }

        if metrics.overall_score() > 0.9 {
            recommendations.push("Performance is excellent, consider increasing workload");
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = new_coordinator();
        assert!(coordinator.is_healthy().await);
    }

    #[tokio::test]
    async fn test_coordinator_with_config() {
        let config = CoordinatorConfig::new()
            .with_max_concurrent_operations(2)
            .with_adaptive_optimization(false);
        
        let coordinator = new_coordinator_with_config(config);
        assert!(coordinator.is_healthy().await);
    }

    #[test]
    fn test_optimization_spec_builder() {
        let spec = optimization_spec()
            .with_algorithm(OptimizationAlgorithm::DimensionReduction)
            .with_algorithm(OptimizationAlgorithm::VectorQuantization);
        
        assert_eq!(spec.algorithms.len(), 2);
    }

    #[test]
    fn test_memory_estimation() {
        let memory = utils::estimate_memory_usage(1000, 512);
        assert!(memory > 0);
        assert!(memory > 1000 * 512 * 4); // Should be more than just vector data
    }

    #[test]
    fn test_vector_validation() {
        let vectors = vec![
            ("vec1".to_string(), vec![1.0, 2.0, 3.0]),
            ("vec2".to_string(), vec![4.0, 5.0, 6.0]),
        ];
        
        assert!(utils::validate_vectors(&vectors).is_ok());
        
        let invalid_vectors = vec![
            ("vec1".to_string(), vec![1.0, 2.0, 3.0]),
            ("vec2".to_string(), vec![4.0, 5.0]), // Wrong dimensions
        ];
        
        assert!(utils::validate_vectors(&invalid_vectors).is_err());
    }

    #[test]
    fn test_recommended_config() {
        let small_config = utils::recommended_config(500, 128);
        assert_eq!(small_config.max_concurrent_operations, 2);
        
        let large_config = utils::recommended_config(50000, 512);
        assert_eq!(large_config.max_concurrent_operations, 8);
        assert!(large_config.enable_adaptive_optimization);
    }
}
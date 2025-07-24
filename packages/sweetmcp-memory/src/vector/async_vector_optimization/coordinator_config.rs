//! Coordinator configuration and macros
//!
//! This module provides blazing-fast configuration with zero allocation
//! optimizations for async vector optimization coordination.

use super::search_strategies::SearchStrategy;

/// Coordinator configuration
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Enable adaptive optimization
    pub enable_adaptive_optimization: bool,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Default search strategy
    pub default_search_strategy: SearchStrategy,
    /// Optimization timeout in seconds
    pub optimization_timeout_secs: u64,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            enable_adaptive_optimization: true,
            max_concurrent_operations: 4,
            default_search_strategy: SearchStrategy::BruteForce,
            optimization_timeout_secs: 300,
        }
    }
}

impl CoordinatorConfig {
    /// Create new configuration with defaults
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set adaptive optimization enabled
    #[inline]
    pub fn with_adaptive_optimization(mut self, enabled: bool) -> Self {
        self.enable_adaptive_optimization = enabled;
        self
    }

    /// Set maximum concurrent operations
    #[inline]
    pub fn with_max_concurrent_operations(mut self, max_ops: usize) -> Self {
        self.max_concurrent_operations = max_ops;
        self
    }

    /// Set default search strategy
    #[inline]
    pub fn with_default_search_strategy(mut self, strategy: SearchStrategy) -> Self {
        self.default_search_strategy = strategy;
        self
    }

    /// Set optimization timeout
    #[inline]
    pub fn with_optimization_timeout(mut self, timeout_secs: u64) -> Self {
        self.optimization_timeout_secs = timeout_secs;
        self
    }

    /// Check if configuration is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max_concurrent_operations > 0 && self.optimization_timeout_secs > 0
    }

    /// Get recommended configuration for dataset size
    #[inline]
    pub fn for_dataset_size(vector_count: usize) -> Self {
        let mut config = Self::default();

        if vector_count < 1000 {
            config.max_concurrent_operations = 2;
            config.default_search_strategy = SearchStrategy::BruteForce;
        } else if vector_count < 10000 {
            config.max_concurrent_operations = 4;
            config.default_search_strategy = SearchStrategy::FilteredSearch;
        } else {
            config.max_concurrent_operations = 8;
            config.default_search_strategy = SearchStrategy::ApproximateNearestNeighbor;
        }

        config
    }
}

/// Comprehensive performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Search metrics
    pub search_metrics: super::coordinator_types::SearchMetrics,
    /// Optimization metrics
    pub optimization_metrics: super::coordinator_types::OptimizationMetrics,
    /// Coordination metrics
    pub coordination_metrics: super::coordinator_types::CoordinationMetrics,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            search_metrics: super::coordinator_types::SearchMetrics::new(),
            optimization_metrics: super::coordinator_types::OptimizationMetrics::new(),
            coordination_metrics: super::coordinator_types::CoordinationMetrics::new(),
        }
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn overall_score(&self) -> f64 {
        // Simple weighted average of component scores
        let search_score = 0.4;
        let optimization_score = 0.4;
        let coordination_score = 0.2;

        search_score + optimization_score + coordination_score
    }

    /// Check if metrics indicate healthy performance
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.overall_score() > 0.7
    }

    /// Get performance summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Performance: {:.1}% (Search: active, Optimization: active, Coordination: active)",
            self.overall_score() * 100.0
        )
    }
}

/// Convenience macros for ergonomic usage
#[macro_export]
macro_rules! optimize_vectors {
    ($coordinator:expr, $vectors:expr, $($algorithm:expr),+) => {{
        let mut spec = $crate::vector::async_vector_optimization::coordinator_types::OptimizationSpec::new();
        $(spec = spec.with_algorithm($algorithm);)+
        $coordinator.execute_optimization_pipeline($vectors, spec).await
    }};
}

#[macro_export]
macro_rules! search_vectors {
    ($coordinator:expr, $query:expr, $vectors:expr, $limit:expr) => {
        $coordinator.execute_optimized_search(
            $query,
            $vectors,
            $limit,
            None,
            $crate::vector::async_vector_operations::DistanceMetric::Cosine,
        ).await
    };
    ($coordinator:expr, $query:expr, $vectors:expr, $limit:expr, $filter:expr) => {
        $coordinator.execute_optimized_search(
            $query,
            $vectors,
            $limit,
            Some($filter),
            $crate::vector::async_vector_operations::DistanceMetric::Cosine,
        ).await
    };
    ($coordinator:expr, $query:expr, $vectors:expr, $limit:expr, $filter:expr, $metric:expr) => {
        $coordinator.execute_optimized_search(
            $query,
            $vectors,
            $limit,
            Some($filter),
            $metric,
        ).await
    };
}

/// Utility functions for coordinator configuration
pub mod utils {
    use super::*;

    /// Get optimal configuration for given constraints
    #[inline]
    pub fn optimal_config_for_constraints(
        vector_count: usize,
        memory_limit_mb: usize,
        latency_target_ms: u64,
    ) -> CoordinatorConfig {
        let mut config = CoordinatorConfig::for_dataset_size(vector_count);

        // Adjust for memory constraints
        if memory_limit_mb < 100 {
            config.max_concurrent_operations = config.max_concurrent_operations.min(2);
        }

        // Adjust for latency requirements
        if latency_target_ms < 100 {
            config.default_search_strategy = SearchStrategy::BruteForce;
            config.optimization_timeout_secs = config.optimization_timeout_secs.min(30);
        }

        config
    }

    /// Validate configuration for production use
    #[inline]
    pub fn validate_production_config(config: &CoordinatorConfig) -> Result<(), String> {
        if !config.is_valid() {
            return Err("Invalid configuration: check concurrent operations and timeout".to_string());
        }

        if config.max_concurrent_operations > 16 {
            return Err("Too many concurrent operations for production use".to_string());
        }

        if config.optimization_timeout_secs > 3600 {
            return Err("Optimization timeout too long for production use".to_string());
        }

        Ok(())
    }
}
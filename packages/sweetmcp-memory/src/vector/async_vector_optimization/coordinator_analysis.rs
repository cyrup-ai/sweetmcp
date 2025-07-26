//! Coordinator analysis and performance tracking
//!
//! This module provides blazing-fast analysis with zero allocation
//! optimizations for async vector optimization coordination.

use smallvec::SmallVec;
use std::time::Duration;
use tracing::debug;

use super::optimization_algorithms::OptimizationAlgorithm;

/// Recent performance information
#[derive(Debug, Clone)]
pub struct RecentPerformance {
    pub operation_count: usize,
    pub average_time: Duration,
    pub total_time: Duration,
}

impl Default for RecentPerformance {
    fn default() -> Self {
        Self {
            operation_count: 0,
            average_time: Duration::from_secs(0),
            total_time: Duration::from_secs(0),
        }
    }
}

impl RecentPerformance {
    /// Check if performance is good
    #[inline]
    pub fn is_good(&self) -> bool {
        self.average_time < Duration::from_millis(100) && self.operation_count > 0
    }

    /// Get performance efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.operation_count == 0 || self.total_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.operation_count as f64 / self.total_time.as_secs_f64()
    }
}

/// Performance trend enumeration
// Re-export from canonical location
pub use crate::cognitive::quantum_mcts::entanglement::metrics::performance_trends::PerformanceTrend;

// impl block removed - using canonical implementation from performance_trends.rs

/// Detailed metrics summary
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_search_operations: usize,
    pub total_optimization_operations: usize,
    pub success_rate: f64,
    pub average_search_time: Duration,
    pub average_optimization_time: Duration,
    pub throughput: f64,
    pub is_healthy: bool,
    pub memory_usage_bytes: usize,
}

impl MetricsSummary {
    /// Check if summary indicates excellent performance
    #[inline]
    pub fn is_excellent(&self) -> bool {
        self.success_rate > 0.95 &&
        self.average_search_time < Duration::from_millis(50) &&
        self.average_optimization_time < Duration::from_secs(2) &&
        self.is_healthy
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let success_score = self.success_rate;
        let speed_score = if self.average_search_time < Duration::from_millis(100) { 1.0 } else { 0.7 };
        let health_score = if self.is_healthy { 1.0 } else { 0.5 };
        let throughput_score = (self.throughput / 10.0).min(1.0);

        (success_score + speed_score + health_score + throughput_score) / 4.0
    }


}

/// Vector dataset characteristics
#[derive(Debug, Clone)]
pub struct VectorCharacteristics {
    /// Number of vectors
    pub vector_count: usize,
    /// Vector dimensions
    pub dimensions: usize,
    /// Average magnitude
    pub average_magnitude: f64,
    /// Magnitude variance
    pub magnitude_variance: f64,
    /// Sparsity ratio
    pub sparsity: f64,
    /// Estimated memory usage in MB
    pub estimated_memory_mb: usize,
}

impl Default for VectorCharacteristics {
    fn default() -> Self {
        Self {
            vector_count: 0,
            dimensions: 0,
            average_magnitude: 0.0,
            magnitude_variance: 0.0,
            sparsity: 0.0,
            estimated_memory_mb: 0,
        }
    }
}

impl VectorCharacteristics {
    /// Check if dataset is large
    #[inline]
    pub fn is_large_dataset(&self) -> bool {
        self.vector_count > 10000 || self.estimated_memory_mb > 100
    }

    /// Check if vectors are high-dimensional
    #[inline]
    pub fn is_high_dimensional(&self) -> bool {
        self.dimensions > 512
    }

    /// Check if dataset is sparse
    #[inline]
    pub fn is_sparse(&self) -> bool {
        self.sparsity > 0.5
    }

    /// Get complexity score (0.0-1.0)
    #[inline]
    pub fn complexity_score(&self) -> f64 {
        let size_score = (self.vector_count as f64 / 100000.0).min(1.0);
        let dimension_score = (self.dimensions as f64 / 1000.0).min(1.0);
        let variance_score = (self.magnitude_variance / 10.0).min(1.0);
        let sparsity_score = self.sparsity;

        (size_score + dimension_score + variance_score + sparsity_score) / 4.0
    }
}

/// Optimization parameters
#[derive(Debug, Clone)]
pub struct OptimizationParameters {
    /// Dimension reduction target
    pub dimension_reduction_target: Option<usize>,
    /// Quantization levels
    pub quantization_levels: Option<usize>,
    /// Cache size
    pub cache_size: Option<usize>,
    /// Batch size
    pub batch_size: Option<usize>,
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        Self {
            dimension_reduction_target: None,
            quantization_levels: None,
            cache_size: None,
            batch_size: None,
        }
    }
}

impl OptimizationParameters {
    /// Create parameters for dimension reduction
    #[inline]
    pub fn dimension_reduction(target: usize) -> Self {
        Self {
            dimension_reduction_target: Some(target),
            ..Default::default()
        }
    }

    /// Create parameters for quantization
    #[inline]
    pub fn quantization(levels: usize) -> Self {
        Self {
            quantization_levels: Some(levels),
            ..Default::default()
        }
    }

    /// Create parameters for cache optimization
    #[inline]
    pub fn cache_optimization(size: usize) -> Self {
        Self {
            cache_size: Some(size),
            ..Default::default()
        }
    }

    /// Create parameters for batch optimization
    #[inline]
    pub fn batch_optimization(size: usize) -> Self {
        Self {
            batch_size: Some(size),
            ..Default::default()
        }
    }
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Recommended algorithm
    pub algorithm: OptimizationAlgorithm,
    /// Priority score (0.0-1.0)
    pub priority: f64,
    /// Expected improvement
    pub expected_improvement: f64,
    /// Parameters for the algorithm
    pub parameters: OptimizationParameters,
    /// Reasoning for recommendation
    pub reasoning: String,
}

impl OptimizationRecommendation {
    /// Create new recommendation
    #[inline]
    pub fn new(
        algorithm: OptimizationAlgorithm,
        priority: f64,
        expected_improvement: f64,
        parameters: OptimizationParameters,
        reasoning: String,
    ) -> Self {
        Self {
            algorithm,
            priority,
            expected_improvement,
            parameters,
            reasoning,
        }
    }

    /// Check if recommendation is high priority
    #[inline]
    pub fn is_high_priority(&self) -> bool {
        self.priority > 0.8
    }

    /// Check if recommendation has significant expected improvement
    #[inline]
    pub fn has_significant_improvement(&self) -> bool {
        self.expected_improvement > 10.0
    }
}
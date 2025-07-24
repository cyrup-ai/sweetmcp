//! Coordinator types and data structures
//!
//! This module provides core types with zero allocation patterns
//! and blazing-fast performance for async vector optimization coordination.

use smallvec::SmallVec;
use std::time::{Duration, Instant};
use tracing::debug;

use crate::vector::async_vector_optimization::search_strategies::SearchStrategy;
use crate::vector::async_vector_optimization::optimization_algorithms::{
    OptimizationAlgorithm, DimensionReductionResult, QuantizationResult, 
    IndexOptimizationResult, CacheOptimizationResult, BatchOptimizationResult, 
    MemoryLayoutResult
};

/// Coordination metrics for performance tracking
#[derive(Debug, Clone)]
pub struct CoordinationMetrics {
    /// Total search operations performed
    pub total_search_operations: usize,
    /// Total optimization operations performed
    pub total_optimization_operations: usize,
    /// Search operation times
    search_times: SmallVec<[Duration; 32]>,
    /// Optimization operation times
    optimization_times: SmallVec<[Duration; 32]>,
    /// Success count
    successful_operations: usize,
    /// Failure count
    failed_operations: usize,
    /// Last operation timestamp
    last_operation: Option<Instant>,
}

impl CoordinationMetrics {
    /// Create new coordination metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_search_operations: 0,
            total_optimization_operations: 0,
            search_times: SmallVec::new(),
            optimization_times: SmallVec::new(),
            successful_operations: 0,
            failed_operations: 0,
            last_operation: None,
        }
    }

    /// Record search operation
    #[inline]
    pub fn record_search_operation(&mut self, duration: Duration, result_count: usize) {
        self.total_search_operations += 1;
        
        // Maintain rolling window of recent times
        if self.search_times.len() >= 32 {
            self.search_times.remove(0);
        }
        self.search_times.push(duration);
        
        if result_count > 0 {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        
        self.last_operation = Some(Instant::now());
        debug!("Search operation recorded: {:?}, {} results", duration, result_count);
    }

    /// Record optimization pipeline operation
    #[inline]
    pub fn record_optimization_pipeline(&mut self, duration: Duration, algorithm_count: usize) {
        self.total_optimization_operations += 1;
        
        // Maintain rolling window of recent times
        if self.optimization_times.len() >= 32 {
            self.optimization_times.remove(0);
        }
        self.optimization_times.push(duration);
        
        if algorithm_count > 0 {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }
        
        self.last_operation = Some(Instant::now());
        debug!("Optimization pipeline recorded: {:?}, {} algorithms", duration, algorithm_count);
    }

    /// Get average search time
    #[inline]
    pub fn average_search_time(&self) -> Duration {
        if self.search_times.is_empty() {
            return Duration::from_secs(0);
        }
        
        let total: Duration = self.search_times.iter().sum();
        total / self.search_times.len() as u32
    }

    /// Get average optimization time
    #[inline]
    pub fn average_optimization_time(&self) -> Duration {
        if self.optimization_times.is_empty() {
            return Duration::from_secs(0);
        }
        
        let total: Duration = self.optimization_times.iter().sum();
        total / self.optimization_times.len() as u32
    }

    /// Get success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_operations + self.failed_operations;
        if total == 0 {
            return 0.0;
        }
        self.successful_operations as f64 / total as f64
    }

    /// Check if metrics indicate healthy performance
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.success_rate() > 0.9 &&
        self.average_search_time() < Duration::from_millis(100) &&
        self.average_optimization_time() < Duration::from_secs(5)
    }

    /// Reset all metrics
    #[inline]
    pub fn reset(&mut self) {
        self.total_search_operations = 0;
        self.total_optimization_operations = 0;
        self.search_times.clear();
        self.optimization_times.clear();
        self.successful_operations = 0;
        self.failed_operations = 0;
        self.last_operation = None;
        debug!("Coordination metrics reset");
    }

    /// Get memory usage of metrics
    #[inline]
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.search_times.capacity() * std::mem::size_of::<Duration>() +
        self.optimization_times.capacity() * std::mem::size_of::<Duration>()
    }

    /// Get throughput (operations per second)
    #[inline]
    pub fn throughput(&self) -> f64 {
        if let Some(last_op) = self.last_operation {
            let elapsed = last_op.elapsed();
            if elapsed.as_secs_f64() > 0.0 {
                let total_ops = self.total_search_operations + self.total_optimization_operations;
                return total_ops as f64 / elapsed.as_secs_f64();
            }
        }
        0.0
    }
}

impl Default for CoordinationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinator configuration
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Default search strategy
    pub default_search_strategy: SearchStrategy,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Operation timeout
    pub operation_timeout: Duration,
    /// Enable adaptive optimization
    pub enable_adaptive_optimization: bool,
    /// Cache optimization results
    pub cache_optimization_results: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl CoordinatorConfig {
    /// Create new coordinator configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            default_search_strategy: SearchStrategy::BruteForce,
            max_concurrent_operations: 4,
            operation_timeout: Duration::from_secs(30),
            enable_adaptive_optimization: true,
            cache_optimization_results: true,
            max_cache_size: 1000,
        }
    }

    /// Create high-performance configuration
    #[inline]
    pub fn high_performance() -> Self {
        Self {
            default_search_strategy: SearchStrategy::ApproximateNearestNeighbor,
            max_concurrent_operations: 8,
            operation_timeout: Duration::from_secs(60),
            enable_adaptive_optimization: true,
            cache_optimization_results: true,
            max_cache_size: 2000,
        }
    }

    /// Create memory-optimized configuration
    #[inline]
    pub fn memory_optimized() -> Self {
        Self {
            default_search_strategy: SearchStrategy::BruteForce,
            max_concurrent_operations: 2,
            operation_timeout: Duration::from_secs(15),
            enable_adaptive_optimization: false,
            cache_optimization_results: false,
            max_cache_size: 100,
        }
    }

    /// Validate configuration
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max_concurrent_operations > 0 &&
        self.operation_timeout > Duration::from_secs(0) &&
        self.max_cache_size > 0
    }
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization specification for pipeline execution
#[derive(Debug, Clone)]
pub struct OptimizationSpec {
    /// Algorithms to execute
    pub algorithms: SmallVec<[OptimizationAlgorithm; 8]>,
    /// Dimension reduction target
    pub dimension_reduction_target: Option<usize>,
    /// Quantization levels
    pub quantization_levels: Option<usize>,
    /// Cache size
    pub cache_size: Option<usize>,
    /// Batch size
    pub batch_size: Option<usize>,
}

impl OptimizationSpec {
    /// Create new optimization specification
    #[inline]
    pub fn new() -> Self {
        Self {
            algorithms: SmallVec::new(),
            dimension_reduction_target: None,
            quantization_levels: None,
            cache_size: None,
            batch_size: None,
        }
    }

    /// Create specification from recommendations
    #[inline]
    pub fn from_recommendations(recommendations: &[OptimizationRecommendation]) -> Self {
        let mut spec = Self::new();
        
        for recommendation in recommendations {
            spec.algorithms.push(recommendation.algorithm);
            
            match recommendation.algorithm {
                OptimizationAlgorithm::DimensionReduction => {
                    spec.dimension_reduction_target = recommendation.parameters.dimension_reduction_target;
                }
                OptimizationAlgorithm::VectorQuantization => {
                    spec.quantization_levels = recommendation.parameters.quantization_levels;
                }
                OptimizationAlgorithm::CacheOptimization => {
                    spec.cache_size = recommendation.parameters.cache_size;
                }
                OptimizationAlgorithm::BatchOptimization => {
                    spec.batch_size = recommendation.parameters.batch_size;
                }
                _ => {}
            }
        }
        
        spec
    }

    /// Add algorithm to specification
    #[inline]
    pub fn with_algorithm(mut self, algorithm: OptimizationAlgorithm) -> Self {
        self.algorithms.push(algorithm);
        self
    }

    /// Set dimension reduction target
    #[inline]
    pub fn with_dimension_reduction(mut self, target: usize) -> Self {
        self.dimension_reduction_target = Some(target);
        self
    }

    /// Set quantization levels
    #[inline]
    pub fn with_quantization(mut self, levels: usize) -> Self {
        self.quantization_levels = Some(levels);
        self
    }
}

impl Default for OptimizationSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization pipeline execution result
#[derive(Debug, Clone)]
pub struct OptimizationPipelineResult {
    /// Dimension reduction result
    pub dimension_reduction: Option<DimensionReductionResult>,
    /// Quantization result
    pub quantization: Option<QuantizationResult>,
    /// Index optimization result
    pub index_optimization: Option<IndexOptimizationResult>,
    /// Cache optimization result
    pub cache_optimization: Option<CacheOptimizationResult>,
    /// Batch optimization result
    pub batch_optimization: Option<BatchOptimizationResult>,
    /// Memory layout result
    pub memory_layout: Option<MemoryLayoutResult>,
    /// Total execution time
    pub total_execution_time: Duration,
}

impl OptimizationPipelineResult {
    /// Create new pipeline result
    #[inline]
    pub fn new() -> Self {
        Self {
            dimension_reduction: None,
            quantization: None,
            index_optimization: None,
            cache_optimization: None,
            batch_optimization: None,
            memory_layout: None,
            total_execution_time: Duration::from_secs(0),
        }
    }

    /// Check if any optimizations were successful
    #[inline]
    pub fn has_successful_optimizations(&self) -> bool {
        self.dimension_reduction.is_some() ||
        self.quantization.is_some() ||
        self.index_optimization.is_some() ||
        self.cache_optimization.is_some() ||
        self.batch_optimization.is_some() ||
        self.memory_layout.is_some()
    }

    /// Get total improvement percentage
    #[inline]
    pub fn total_improvement(&self) -> f64 {
        let mut total = 0.0;
        let mut count = 0;

        if let Some(ref result) = self.dimension_reduction {
            total += result.improvement_percentage;
            count += 1;
        }
        if let Some(ref result) = self.quantization {
            total += result.improvement_percentage;
            count += 1;
        }
        if let Some(ref result) = self.index_optimization {
            total += result.improvement_percentage;
            count += 1;
        }
        if let Some(ref result) = self.cache_optimization {
            total += result.improvement_percentage;
            count += 1;
        }
        if let Some(ref result) = self.batch_optimization {
            total += result.improvement_percentage;
            count += 1;
        }
        if let Some(ref result) = self.memory_layout {
            total += result.improvement_percentage;
            count += 1;
        }

        if count > 0 {
            total / count as f64
        } else {
            0.0
        }
    }
}

impl Default for OptimizationPipelineResult {
    fn default() -> Self {
        Self::new()
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
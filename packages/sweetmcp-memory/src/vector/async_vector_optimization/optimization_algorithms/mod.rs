//! Vector optimization algorithms module
//!
//! This module provides blazing-fast optimization algorithms with zero allocation
//! optimizations and elegant ergonomic interfaces for vector performance enhancement.

pub mod algorithm_types;
pub mod optimization_executor;
pub mod algorithm_implementations;
pub mod optimization_results;
pub mod optimization_metrics;

// Re-export main types for ergonomic usage
pub use algorithm_types::{
    OptimizationAlgorithm, AlgorithmComplexity, ExecutionStrategy, AlgorithmSelectionCriteria
};
pub use optimization_executor::OptimizationExecutor;
pub use optimization_results::{
    DimensionReductionResult, QuantizationResult, IndexOptimizationResult,
    CacheOptimizationResult, BatchOptimizationResult, MemoryLayoutResult,
    CombinedOptimizationResult, IndexStructure, AccessPatterns, CacheLayout,
    MemoryLayoutAnalysis, OptimizedLayout, IndexType
};
pub use optimization_metrics::{
    OptimizationMetrics, OptimizationConfig, PerformanceCache, MetricsSummary,
    CacheStatistics, PerformanceTrend, TrendDirection
};

/// Builder for creating optimization executors with specific configurations
pub struct OptimizationExecutorBuilder {
    algorithms: Vec<OptimizationAlgorithm>,
    config: OptimizationConfig,
}

impl Default for OptimizationExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationExecutorBuilder {
    /// Create new optimization executor builder
    #[inline]
    pub fn new() -> Self {
        Self {
            algorithms: Vec::new(),
            config: OptimizationConfig::default(),
        }
    }

    /// Add optimization algorithm
    #[inline]
    pub fn with_algorithm(mut self, algorithm: OptimizationAlgorithm) -> Self {
        if !self.algorithms.contains(&algorithm) {
            self.algorithms.push(algorithm);
        }
        self
    }

    /// Add multiple algorithms
    #[inline]
    pub fn with_algorithms(mut self, algorithms: &[OptimizationAlgorithm]) -> Self {
        for &algorithm in algorithms {
            if !self.algorithms.contains(&algorithm) {
                self.algorithms.push(algorithm);
            }
        }
        self
    }

    /// Set configuration
    #[inline]
    pub fn with_config(mut self, config: OptimizationConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable aggressive mode
    #[inline]
    pub fn aggressive(mut self) -> Self {
        self.config = self.config.with_aggressive_mode(true);
        self
    }

    /// Set conservative mode
    #[inline]
    pub fn conservative(mut self) -> Self {
        self.config = OptimizationConfig::conservative();
        self
    }

    /// Set high performance mode
    #[inline]
    pub fn high_performance(mut self) -> Self {
        self.config = OptimizationConfig::high_performance();
        self
    }

    /// Auto-select algorithms based on data characteristics
    #[inline]
    pub fn auto_select_algorithms(mut self, vector_count: usize, dimensions: usize) -> Self {
        self.algorithms = OptimizationAlgorithm::suitable_algorithms(vector_count, dimensions);
        self
    }

    /// Build the optimization executor
    #[inline]
    pub fn build(self) -> OptimizationExecutor {
        let mut executor = OptimizationExecutor::with_config(self.config);
        for algorithm in self.algorithms {
            executor.add_algorithm(algorithm);
        }
        executor
    }
}

/// Convenience functions for creating common optimization scenarios
impl OptimizationExecutor {
    /// Create executor optimized for large datasets
    #[inline]
    pub fn for_large_dataset() -> Self {
        OptimizationExecutorBuilder::new()
            .with_algorithms(&[
                OptimizationAlgorithm::IndexOptimization,
                OptimizationAlgorithm::BatchOptimization,
                OptimizationAlgorithm::CacheOptimization,
            ])
            .high_performance()
            .build()
    }

    /// Create executor optimized for memory efficiency
    #[inline]
    pub fn for_memory_efficiency() -> Self {
        OptimizationExecutorBuilder::new()
            .with_algorithms(&[
                OptimizationAlgorithm::DimensionReduction,
                OptimizationAlgorithm::VectorQuantization,
                OptimizationAlgorithm::MemoryLayoutOptimization,
            ])
            .conservative()
            .build()
    }

    /// Create executor optimized for speed
    #[inline]
    pub fn for_speed() -> Self {
        OptimizationExecutorBuilder::new()
            .with_algorithms(&[
                OptimizationAlgorithm::CacheOptimization,
                OptimizationAlgorithm::BatchOptimization,
                OptimizationAlgorithm::IndexOptimization,
            ])
            .aggressive()
            .build()
    }

    /// Create executor with all algorithms
    #[inline]
    pub fn with_all_algorithms() -> Self {
        OptimizationExecutorBuilder::new()
            .with_algorithms(&OptimizationAlgorithm::all_by_priority())
            .high_performance()
            .build()
    }

    /// Execute all configured optimizations
    #[inline]
    pub async fn execute_all_optimizations(
        &mut self,
        vectors: &mut [(String, Vec<f32>)],
    ) -> Result<CombinedOptimizationResult, crate::utils::error::Error> {
        let mut result = CombinedOptimizationResult::new();
        let start_time = std::time::Instant::now();

        // Execute each algorithm if it's active
        for &algorithm in self.get_active_algorithms() {
            match algorithm {
                OptimizationAlgorithm::DimensionReduction => {
                    if let Ok(dim_result) = self.execute_dimension_reduction(vectors, vectors.first().map(|(_, v)| v.len() / 2).unwrap_or(50)).await {
                        result.dimension_reduction = Some(dim_result);
                    }
                }
                OptimizationAlgorithm::VectorQuantization => {
                    if let Ok(quant_result) = self.execute_vector_quantization(vectors, 256).await {
                        result.quantization = Some(quant_result);
                    }
                }
                OptimizationAlgorithm::IndexOptimization => {
                    if let Ok(index_result) = self.execute_index_optimization(vectors).await {
                        result.index_optimization = Some(index_result);
                    }
                }
                OptimizationAlgorithm::CacheOptimization => {
                    if let Ok(cache_result) = self.execute_cache_optimization(vectors, 1000).await {
                        result.cache_optimization = Some(cache_result);
                    }
                }
                OptimizationAlgorithm::BatchOptimization => {
                    if let Ok(batch_result) = self.execute_batch_optimization(vectors, 100).await {
                        result.batch_optimization = Some(batch_result);
                    }
                }
                OptimizationAlgorithm::MemoryLayoutOptimization => {
                    if let Ok(layout_result) = self.execute_memory_layout_optimization(vectors).await {
                        result.memory_layout = Some(layout_result);
                    }
                }
            }
        }

        result.total_execution_time = start_time.elapsed();
        Ok(result)
    }
}

/// Optimization algorithm selection utilities
pub struct AlgorithmSelector;

impl AlgorithmSelector {
    /// Select optimal algorithms for given parameters
    #[inline]
    pub fn select_optimal(
        vector_count: usize,
        dimensions: usize,
        criteria: AlgorithmSelectionCriteria,
    ) -> Vec<OptimizationAlgorithm> {
        criteria.select_algorithms(vector_count, dimensions)
    }

    /// Select algorithms for memory-constrained environments
    #[inline]
    pub fn select_for_memory_constraint(
        vector_count: usize,
        dimensions: usize,
        max_memory_mb: usize,
    ) -> Vec<OptimizationAlgorithm> {
        let criteria = AlgorithmSelectionCriteria::new()
            .with_complexity(AlgorithmComplexity::Low)
            .with_max_execution_time(5000); // 5 seconds

        let mut algorithms = criteria.select_algorithms(vector_count, dimensions);

        // Filter out memory-intensive algorithms for small memory limits
        if max_memory_mb < 512 {
            algorithms.retain(|&alg| !matches!(alg, 
                OptimizationAlgorithm::DimensionReduction | 
                OptimizationAlgorithm::IndexOptimization
            ));
        }

        algorithms
    }

    /// Select algorithms for time-constrained environments
    #[inline]
    pub fn select_for_time_constraint(
        vector_count: usize,
        dimensions: usize,
        max_time_ms: u64,
    ) -> Vec<OptimizationAlgorithm> {
        let criteria = AlgorithmSelectionCriteria::new()
            .with_max_execution_time(max_time_ms)
            .with_strategy(ExecutionStrategy::Parallel);

        criteria.select_algorithms(vector_count, dimensions)
    }

    /// Select algorithms for quality optimization
    #[inline]
    pub fn select_for_quality(
        vector_count: usize,
        dimensions: usize,
    ) -> Vec<OptimizationAlgorithm> {
        let criteria = AlgorithmSelectionCriteria::new()
            .with_min_improvement(0.2) // 20% minimum improvement
            .with_complexity(AlgorithmComplexity::High)
            .aggressive();

        criteria.select_algorithms(vector_count, dimensions)
    }
}

/// Performance analysis utilities
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Analyze optimization results
    #[inline]
    pub fn analyze_results(result: &CombinedOptimizationResult) -> PerformanceAnalysis {
        PerformanceAnalysis {
            overall_score: result.overall_performance_score(),
            successful_optimizations: result.optimization_count(),
            total_execution_time: result.total_execution_time,
            has_improvements: result.has_successful_optimizations(),
        }
    }

    /// Compare two optimization results
    #[inline]
    pub fn compare_results(
        result1: &CombinedOptimizationResult,
        result2: &CombinedOptimizationResult,
    ) -> ResultComparison {
        let score1 = result1.overall_performance_score();
        let score2 = result2.overall_performance_score();

        ResultComparison {
            score_difference: score2 - score1,
            time_difference: result2.total_execution_time.saturating_sub(result1.total_execution_time),
            better_result: if score2 > score1 { 2 } else if score1 > score2 { 1 } else { 0 },
        }
    }

    /// Get optimization recommendations
    #[inline]
    pub fn get_recommendations(
        vector_count: usize,
        dimensions: usize,
        current_performance: f64,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        if current_performance < 0.5 {
            recommendations.push(OptimizationRecommendation {
                algorithm: OptimizationAlgorithm::IndexOptimization,
                priority: 10,
                reason: "Low performance detected, index optimization recommended".to_string(),
            });
        }

        if dimensions > 1000 {
            recommendations.push(OptimizationRecommendation {
                algorithm: OptimizationAlgorithm::DimensionReduction,
                priority: 8,
                reason: "High dimensionality detected, dimension reduction recommended".to_string(),
            });
        }

        if vector_count > 10000 {
            recommendations.push(OptimizationRecommendation {
                algorithm: OptimizationAlgorithm::BatchOptimization,
                priority: 7,
                reason: "Large dataset detected, batch optimization recommended".to_string(),
            });
        }

        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }
}

/// Performance analysis result
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub overall_score: f64,
    pub successful_optimizations: usize,
    pub total_execution_time: std::time::Duration,
    pub has_improvements: bool,
}

/// Result comparison
#[derive(Debug, Clone)]
pub struct ResultComparison {
    pub score_difference: f64,
    pub time_difference: std::time::Duration,
    pub better_result: u8, // 0 = equal, 1 = first better, 2 = second better
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub algorithm: OptimizationAlgorithm,
    pub priority: u8,
    pub reason: String,
}

/// Macro for creating optimization executors with fluent syntax
#[macro_export]
macro_rules! optimization_executor {
    (algorithms: [$($alg:expr),*], config: $config:expr) => {
        {
            let mut builder = OptimizationExecutorBuilder::new();
            $(builder = builder.with_algorithm($alg);)*
            builder.with_config($config).build()
        }
    };
    (algorithms: [$($alg:expr),*]) => {
        {
            let mut builder = OptimizationExecutorBuilder::new();
            $(builder = builder.with_algorithm($alg);)*
            builder.build()
        }
    };
    (auto: $vector_count:expr, $dimensions:expr) => {
        OptimizationExecutorBuilder::new()
            .auto_select_algorithms($vector_count, $dimensions)
            .build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_executor_builder() {
        let executor = OptimizationExecutorBuilder::new()
            .with_algorithm(OptimizationAlgorithm::IndexOptimization)
            .with_algorithm(OptimizationAlgorithm::CacheOptimization)
            .aggressive()
            .build();

        assert_eq!(executor.algorithm_count(), 2);
        assert!(executor.is_algorithm_active(OptimizationAlgorithm::IndexOptimization));
        assert!(executor.is_algorithm_active(OptimizationAlgorithm::CacheOptimization));
    }

    #[test]
    fn test_algorithm_selection() {
        let algorithms = AlgorithmSelector::select_optimal(
            1000,
            100,
            AlgorithmSelectionCriteria::default(),
        );

        assert!(!algorithms.is_empty());
        assert!(algorithms.contains(&OptimizationAlgorithm::IndexOptimization));
    }

    #[test]
    fn test_performance_analysis() {
        let result = CombinedOptimizationResult::new();
        let analysis = PerformanceAnalyzer::analyze_results(&result);

        assert_eq!(analysis.overall_score, 0.0);
        assert_eq!(analysis.successful_optimizations, 0);
    }
}

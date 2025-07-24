//! Vector optimization algorithm types and classifications
//!
//! This module provides blazing-fast algorithm type definitions with zero allocation
//! optimizations and elegant ergonomic interfaces for optimization classification.

/// Vector optimization algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationAlgorithm {
    /// Dimension reduction using PCA
    DimensionReduction,
    /// Vector quantization for compression
    VectorQuantization,
    /// Index optimization for faster searches
    IndexOptimization,
    /// Cache optimization for frequent queries
    CacheOptimization,
    /// Batch processing optimization
    BatchOptimization,
    /// Memory layout optimization
    MemoryLayoutOptimization,
}

impl OptimizationAlgorithm {
    /// Get algorithm description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationAlgorithm::DimensionReduction => "Reduce vector dimensions while preserving similarity",
            OptimizationAlgorithm::VectorQuantization => "Compress vectors using quantization techniques",
            OptimizationAlgorithm::IndexOptimization => "Optimize search indices for faster retrieval",
            OptimizationAlgorithm::CacheOptimization => "Cache frequently accessed vectors",
            OptimizationAlgorithm::BatchOptimization => "Process vectors in optimized batches",
            OptimizationAlgorithm::MemoryLayoutOptimization => "Optimize memory layout for cache efficiency",
        }
    }

    /// Get expected performance improvement
    #[inline]
    pub fn expected_improvement(&self) -> f64 {
        match self {
            OptimizationAlgorithm::DimensionReduction => 0.4, // 40% improvement
            OptimizationAlgorithm::VectorQuantization => 0.6, // 60% improvement
            OptimizationAlgorithm::IndexOptimization => 0.8, // 80% improvement
            OptimizationAlgorithm::CacheOptimization => 0.3, // 30% improvement
            OptimizationAlgorithm::BatchOptimization => 0.5, // 50% improvement
            OptimizationAlgorithm::MemoryLayoutOptimization => 0.2, // 20% improvement
        }
    }

    /// Get algorithm complexity level
    #[inline]
    pub fn complexity_level(&self) -> AlgorithmComplexity {
        match self {
            OptimizationAlgorithm::DimensionReduction => AlgorithmComplexity::High,
            OptimizationAlgorithm::VectorQuantization => AlgorithmComplexity::Medium,
            OptimizationAlgorithm::IndexOptimization => AlgorithmComplexity::High,
            OptimizationAlgorithm::CacheOptimization => AlgorithmComplexity::Low,
            OptimizationAlgorithm::BatchOptimization => AlgorithmComplexity::Low,
            OptimizationAlgorithm::MemoryLayoutOptimization => AlgorithmComplexity::Medium,
        }
    }

    /// Get algorithm execution time estimate
    #[inline]
    pub fn estimated_execution_time_ms(&self, vector_count: usize) -> u64 {
        let base_time = match self {
            OptimizationAlgorithm::DimensionReduction => 100,
            OptimizationAlgorithm::VectorQuantization => 50,
            OptimizationAlgorithm::IndexOptimization => 200,
            OptimizationAlgorithm::CacheOptimization => 20,
            OptimizationAlgorithm::BatchOptimization => 10,
            OptimizationAlgorithm::MemoryLayoutOptimization => 30,
        };

        // Scale with vector count (logarithmically for better algorithms)
        let scaling_factor = match self {
            OptimizationAlgorithm::IndexOptimization | OptimizationAlgorithm::DimensionReduction => {
                (vector_count as f64).ln().max(1.0)
            }
            _ => (vector_count as f64).sqrt().max(1.0),
        };

        (base_time as f64 * scaling_factor) as u64
    }

    /// Check if algorithm is suitable for vector count
    #[inline]
    pub fn is_suitable_for_count(&self, vector_count: usize) -> bool {
        match self {
            OptimizationAlgorithm::DimensionReduction => vector_count >= 100,
            OptimizationAlgorithm::VectorQuantization => vector_count >= 50,
            OptimizationAlgorithm::IndexOptimization => vector_count >= 1000,
            OptimizationAlgorithm::CacheOptimization => vector_count >= 10,
            OptimizationAlgorithm::BatchOptimization => vector_count >= 100,
            OptimizationAlgorithm::MemoryLayoutOptimization => vector_count >= 10,
        }
    }

    /// Check if algorithm is suitable for vector dimensions
    #[inline]
    pub fn is_suitable_for_dimensions(&self, dimensions: usize) -> bool {
        match self {
            OptimizationAlgorithm::DimensionReduction => dimensions >= 50,
            OptimizationAlgorithm::VectorQuantization => dimensions >= 10,
            OptimizationAlgorithm::IndexOptimization => dimensions >= 10,
            OptimizationAlgorithm::CacheOptimization => true,
            OptimizationAlgorithm::BatchOptimization => true,
            OptimizationAlgorithm::MemoryLayoutOptimization => dimensions >= 10,
        }
    }

    /// Get algorithm priority (higher is more important)
    #[inline]
    pub fn priority(&self) -> u8 {
        match self {
            OptimizationAlgorithm::IndexOptimization => 10,
            OptimizationAlgorithm::VectorQuantization => 8,
            OptimizationAlgorithm::DimensionReduction => 7,
            OptimizationAlgorithm::BatchOptimization => 6,
            OptimizationAlgorithm::CacheOptimization => 5,
            OptimizationAlgorithm::MemoryLayoutOptimization => 4,
        }
    }

    /// Get all algorithms sorted by priority
    #[inline]
    pub fn all_by_priority() -> Vec<OptimizationAlgorithm> {
        let mut algorithms = vec![
            OptimizationAlgorithm::DimensionReduction,
            OptimizationAlgorithm::VectorQuantization,
            OptimizationAlgorithm::IndexOptimization,
            OptimizationAlgorithm::CacheOptimization,
            OptimizationAlgorithm::BatchOptimization,
            OptimizationAlgorithm::MemoryLayoutOptimization,
        ];
        
        algorithms.sort_by(|a, b| b.priority().cmp(&a.priority()));
        algorithms
    }

    /// Get algorithms suitable for given parameters
    #[inline]
    pub fn suitable_algorithms(vector_count: usize, dimensions: usize) -> Vec<OptimizationAlgorithm> {
        Self::all_by_priority()
            .into_iter()
            .filter(|alg| alg.is_suitable_for_count(vector_count) && alg.is_suitable_for_dimensions(dimensions))
            .collect()
    }
}

/// Algorithm complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmComplexity {
    Low,
    Medium,
    High,
}

impl AlgorithmComplexity {
    /// Get complexity description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            AlgorithmComplexity::Low => "Low complexity, fast execution",
            AlgorithmComplexity::Medium => "Medium complexity, moderate execution time",
            AlgorithmComplexity::High => "High complexity, longer execution time but better results",
        }
    }

    /// Get complexity score (0.0-1.0)
    #[inline]
    pub fn score(&self) -> f64 {
        match self {
            AlgorithmComplexity::Low => 0.2,
            AlgorithmComplexity::Medium => 0.5,
            AlgorithmComplexity::High => 0.8,
        }
    }
}

/// Algorithm execution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStrategy {
    /// Execute algorithms sequentially
    Sequential,
    /// Execute algorithms in parallel where possible
    Parallel,
    /// Execute algorithms adaptively based on system load
    Adaptive,
}

impl ExecutionStrategy {
    /// Get strategy description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionStrategy::Sequential => "Execute algorithms one after another",
            ExecutionStrategy::Parallel => "Execute compatible algorithms simultaneously",
            ExecutionStrategy::Adaptive => "Adapt execution based on system resources",
        }
    }

    /// Check if strategy supports parallel execution
    #[inline]
    pub fn supports_parallel(&self) -> bool {
        matches!(self, ExecutionStrategy::Parallel | ExecutionStrategy::Adaptive)
    }
}

/// Algorithm selection criteria
#[derive(Debug, Clone)]
pub struct AlgorithmSelectionCriteria {
    /// Maximum execution time allowed (milliseconds)
    pub max_execution_time_ms: u64,
    /// Minimum improvement threshold
    pub min_improvement_threshold: f64,
    /// Preferred complexity level
    pub preferred_complexity: Option<AlgorithmComplexity>,
    /// Execution strategy
    pub execution_strategy: ExecutionStrategy,
    /// Enable aggressive optimizations
    pub aggressive_mode: bool,
}

impl Default for AlgorithmSelectionCriteria {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 10000, // 10 seconds
            min_improvement_threshold: 0.1, // 10% minimum improvement
            preferred_complexity: None,
            execution_strategy: ExecutionStrategy::Sequential,
            aggressive_mode: false,
        }
    }
}

impl AlgorithmSelectionCriteria {
    /// Create new selection criteria
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum execution time
    #[inline]
    pub fn with_max_execution_time(mut self, max_time_ms: u64) -> Self {
        self.max_execution_time_ms = max_time_ms;
        self
    }

    /// Set minimum improvement threshold
    #[inline]
    pub fn with_min_improvement(mut self, threshold: f64) -> Self {
        self.min_improvement_threshold = threshold;
        self
    }

    /// Set preferred complexity level
    #[inline]
    pub fn with_complexity(mut self, complexity: AlgorithmComplexity) -> Self {
        self.preferred_complexity = Some(complexity);
        self
    }

    /// Set execution strategy
    #[inline]
    pub fn with_strategy(mut self, strategy: ExecutionStrategy) -> Self {
        self.execution_strategy = strategy;
        self
    }

    /// Enable aggressive mode
    #[inline]
    pub fn aggressive(mut self) -> Self {
        self.aggressive_mode = true;
        self
    }

    /// Select algorithms based on criteria
    #[inline]
    pub fn select_algorithms(&self, vector_count: usize, dimensions: usize) -> Vec<OptimizationAlgorithm> {
        let mut suitable = OptimizationAlgorithm::suitable_algorithms(vector_count, dimensions);

        // Filter by execution time
        suitable.retain(|alg| {
            alg.estimated_execution_time_ms(vector_count) <= self.max_execution_time_ms
        });

        // Filter by improvement threshold
        suitable.retain(|alg| {
            alg.expected_improvement() >= self.min_improvement_threshold
        });

        // Filter by complexity preference
        if let Some(preferred_complexity) = self.preferred_complexity {
            suitable.retain(|alg| alg.complexity_level() == preferred_complexity);
        }

        // In aggressive mode, include all suitable algorithms
        if !self.aggressive_mode {
            // Limit to top 3 algorithms for conservative mode
            suitable.truncate(3);
        }

        suitable
    }
}

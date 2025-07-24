//! Optimization strategy configuration and management
//!
//! This module provides blazing-fast strategy configuration with zero allocation
//! optimizations and elegant ergonomic interfaces for optimization planning.

use std::time::Duration;

use super::super::optimization_recommendations::RecommendationType;

/// Optimization strategy configuration
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationStrategy {
    /// Maximum execution time per optimization
    pub max_execution_time: Duration,
    /// Maximum number of operations to execute
    pub max_operations: usize,
    /// Minimum improvement threshold to continue
    pub min_improvement_threshold: f64,
    /// Enable aggressive optimizations
    pub enable_aggressive_mode: bool,
    /// Enable parallel execution
    pub enable_parallel_execution: bool,
    /// Priority ordering for optimization types
    pub priority_order: Vec<RecommendationType>,
    /// Stop early if improvement target reached
    pub early_stop_threshold: f64,
}

impl OptimizationStrategy {
    /// Create conservative optimization strategy
    #[inline]
    pub fn conservative() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_operations: 10,
            min_improvement_threshold: 1.0,
            enable_aggressive_mode: false,
            enable_parallel_execution: false,
            priority_order: vec![
                RecommendationType::CacheOptimization,
                RecommendationType::IndexOptimization,
                RecommendationType::Compression,
            ],
            early_stop_threshold: 10.0,
        }
    }

    /// Create balanced optimization strategy
    #[inline]
    pub fn balanced() -> Self {
        Self {
            max_execution_time: Duration::from_secs(60),
            max_operations: 25,
            min_improvement_threshold: 0.5,
            enable_aggressive_mode: false,
            enable_parallel_execution: true,
            priority_order: vec![
                RecommendationType::Defragmentation,
                RecommendationType::CacheOptimization,
                RecommendationType::IndexOptimization,
                RecommendationType::Compression,
                RecommendationType::RelationshipPruning,
            ],
            early_stop_threshold: 20.0,
        }
    }

    /// Create aggressive optimization strategy
    #[inline]
    pub fn aggressive() -> Self {
        Self {
            max_execution_time: Duration::from_secs(120),
            max_operations: 50,
            min_improvement_threshold: 0.1,
            enable_aggressive_mode: true,
            enable_parallel_execution: true,
            priority_order: vec![
                RecommendationType::Defragmentation,
                RecommendationType::MemoryReallocation,
                RecommendationType::DataStructureOptimization,
                RecommendationType::CacheOptimization,
                RecommendationType::IndexOptimization,
                RecommendationType::Compression,
                RecommendationType::RelationshipPruning,
                RecommendationType::AccessPatternOptimization,
                RecommendationType::GarbageCollectionOptimization,
                RecommendationType::MemoryPoolOptimization,
            ],
            early_stop_threshold: 50.0,
        }
    }

    /// Create high-performance strategy
    #[inline]
    pub fn high_performance() -> Self {
        Self {
            max_execution_time: Duration::from_secs(300), // 5 minutes
            max_operations: 100,
            min_improvement_threshold: 0.05,
            enable_aggressive_mode: true,
            enable_parallel_execution: true,
            priority_order: vec![
                RecommendationType::IndexOptimization,
                RecommendationType::CacheOptimization,
                RecommendationType::Defragmentation,
                RecommendationType::MemoryReallocation,
                RecommendationType::DataStructureOptimization,
                RecommendationType::Compression,
                RecommendationType::RelationshipPruning,
                RecommendationType::AccessPatternOptimization,
                RecommendationType::GarbageCollectionOptimization,
                RecommendationType::MemoryPoolOptimization,
            ],
            early_stop_threshold: 75.0,
        }
    }

    /// Create memory-focused strategy
    #[inline]
    pub fn memory_focused() -> Self {
        Self {
            max_execution_time: Duration::from_secs(90),
            max_operations: 30,
            min_improvement_threshold: 0.2,
            enable_aggressive_mode: true,
            enable_parallel_execution: false, // Sequential for memory safety
            priority_order: vec![
                RecommendationType::Defragmentation,
                RecommendationType::MemoryReallocation,
                RecommendationType::Compression,
                RecommendationType::GarbageCollectionOptimization,
                RecommendationType::MemoryPoolOptimization,
                RecommendationType::DataStructureOptimization,
            ],
            early_stop_threshold: 30.0,
        }
    }

    /// Create speed-focused strategy
    #[inline]
    pub fn speed_focused() -> Self {
        Self {
            max_execution_time: Duration::from_secs(15), // Fast execution
            max_operations: 5,
            min_improvement_threshold: 2.0, // Higher threshold for quick wins
            enable_aggressive_mode: false,
            enable_parallel_execution: true,
            priority_order: vec![
                RecommendationType::CacheOptimization,
                RecommendationType::IndexOptimization,
                RecommendationType::AccessPatternOptimization,
            ],
            early_stop_threshold: 5.0,
        }
    }

    /// Check if strategy is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max_execution_time > Duration::from_secs(0) &&
        self.max_operations > 0 &&
        self.min_improvement_threshold >= 0.0 &&
        !self.priority_order.is_empty() &&
        self.early_stop_threshold >= 0.0
    }

    /// Get timeout for single operation
    #[inline]
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_millis(
            self.max_execution_time.as_millis() as u64 / self.max_operations.max(1) as u64
        )
    }

    /// Get strategy type name
    #[inline]
    pub fn strategy_type(&self) -> String {
        if self == &Self::conservative() {
            "Conservative".to_string()
        } else if self == &Self::balanced() {
            "Balanced".to_string()
        } else if self == &Self::aggressive() {
            "Aggressive".to_string()
        } else if self == &Self::high_performance() {
            "High Performance".to_string()
        } else if self == &Self::memory_focused() {
            "Memory Focused".to_string()
        } else if self == &Self::speed_focused() {
            "Speed Focused".to_string()
        } else {
            "Custom".to_string()
        }
    }

    /// Check if strategy is aggressive
    #[inline]
    pub fn is_aggressive(&self) -> bool {
        self.enable_aggressive_mode
    }

    /// Check if strategy supports parallel execution
    #[inline]
    pub fn supports_parallel(&self) -> bool {
        self.enable_parallel_execution
    }

    /// Get priority for recommendation type
    #[inline]
    pub fn get_priority(&self, recommendation_type: &RecommendationType) -> Option<usize> {
        self.priority_order.iter().position(|r| r == recommendation_type)
    }

    /// Check if recommendation type is included in strategy
    #[inline]
    pub fn includes_recommendation(&self, recommendation_type: &RecommendationType) -> bool {
        self.priority_order.contains(recommendation_type)
    }

    /// Get estimated execution time
    #[inline]
    pub fn estimated_execution_time(&self) -> Duration {
        // Conservative estimate based on operation count and complexity
        let base_time_per_op = if self.enable_aggressive_mode {
            Duration::from_millis(500)
        } else {
            Duration::from_millis(200)
        };

        let total_estimated = base_time_per_op * self.max_operations as u32;
        total_estimated.min(self.max_execution_time)
    }

    /// Create strategy with custom parameters
    #[inline]
    pub fn custom(
        max_execution_time: Duration,
        max_operations: usize,
        min_improvement_threshold: f64,
        enable_aggressive_mode: bool,
        enable_parallel_execution: bool,
        priority_order: Vec<RecommendationType>,
        early_stop_threshold: f64,
    ) -> Self {
        Self {
            max_execution_time,
            max_operations,
            min_improvement_threshold,
            enable_aggressive_mode,
            enable_parallel_execution,
            priority_order,
            early_stop_threshold,
        }
    }

    /// Builder pattern for creating custom strategies
    #[inline]
    pub fn builder() -> OptimizationStrategyBuilder {
        OptimizationStrategyBuilder::new()
    }

    /// Validate strategy parameters
    #[inline]
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_execution_time.as_secs() == 0 {
            return Err("Maximum execution time must be greater than 0");
        }
        if self.max_operations == 0 {
            return Err("Maximum operations must be greater than 0");
        }
        if self.min_improvement_threshold < 0.0 {
            return Err("Minimum improvement threshold cannot be negative");
        }
        if self.priority_order.is_empty() {
            return Err("Priority order cannot be empty");
        }
        if self.early_stop_threshold < 0.0 {
            return Err("Early stop threshold cannot be negative");
        }
        Ok(())
    }

    /// Get strategy complexity score (0.0-1.0)
    #[inline]
    pub fn complexity_score(&self) -> f64 {
        let operation_score = (self.max_operations as f64 / 100.0).min(1.0);
        let time_score = (self.max_execution_time.as_secs() as f64 / 300.0).min(1.0);
        let priority_score = (self.priority_order.len() as f64 / 10.0).min(1.0);
        let aggressive_score = if self.enable_aggressive_mode { 1.0 } else { 0.5 };

        (operation_score + time_score + priority_score + aggressive_score) / 4.0
    }

    /// Check if strategy is suitable for item count
    #[inline]
    pub fn is_suitable_for_item_count(&self, item_count: usize) -> bool {
        let items_per_operation = item_count / self.max_operations.max(1);
        
        // Conservative strategies should handle fewer items per operation
        if !self.enable_aggressive_mode && items_per_operation > 1000 {
            return false;
        }
        
        // Aggressive strategies can handle more items
        if self.enable_aggressive_mode && items_per_operation > 10000 {
            return false;
        }
        
        true
    }
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Builder for creating custom optimization strategies
pub struct OptimizationStrategyBuilder {
    max_execution_time: Duration,
    max_operations: usize,
    min_improvement_threshold: f64,
    enable_aggressive_mode: bool,
    enable_parallel_execution: bool,
    priority_order: Vec<RecommendationType>,
    early_stop_threshold: f64,
}

impl OptimizationStrategyBuilder {
    /// Create new strategy builder
    #[inline]
    pub fn new() -> Self {
        Self {
            max_execution_time: Duration::from_secs(60),
            max_operations: 25,
            min_improvement_threshold: 0.5,
            enable_aggressive_mode: false,
            enable_parallel_execution: true,
            priority_order: vec![
                RecommendationType::CacheOptimization,
                RecommendationType::IndexOptimization,
                RecommendationType::Compression,
            ],
            early_stop_threshold: 20.0,
        }
    }

    /// Set maximum execution time
    #[inline]
    pub fn max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = duration;
        self
    }

    /// Set maximum operations
    #[inline]
    pub fn max_operations(mut self, count: usize) -> Self {
        self.max_operations = count;
        self
    }

    /// Set minimum improvement threshold
    #[inline]
    pub fn min_improvement_threshold(mut self, threshold: f64) -> Self {
        self.min_improvement_threshold = threshold;
        self
    }

    /// Enable aggressive mode
    #[inline]
    pub fn aggressive_mode(mut self, enable: bool) -> Self {
        self.enable_aggressive_mode = enable;
        self
    }

    /// Enable parallel execution
    #[inline]
    pub fn parallel_execution(mut self, enable: bool) -> Self {
        self.enable_parallel_execution = enable;
        self
    }

    /// Set priority order
    #[inline]
    pub fn priority_order(mut self, order: Vec<RecommendationType>) -> Self {
        self.priority_order = order;
        self
    }

    /// Add recommendation to priority order
    #[inline]
    pub fn add_priority(mut self, recommendation: RecommendationType) -> Self {
        if !self.priority_order.contains(&recommendation) {
            self.priority_order.push(recommendation);
        }
        self
    }

    /// Set early stop threshold
    #[inline]
    pub fn early_stop_threshold(mut self, threshold: f64) -> Self {
        self.early_stop_threshold = threshold;
        self
    }

    /// Build the optimization strategy
    #[inline]
    pub fn build(self) -> OptimizationStrategy {
        OptimizationStrategy {
            max_execution_time: self.max_execution_time,
            max_operations: self.max_operations,
            min_improvement_threshold: self.min_improvement_threshold,
            enable_aggressive_mode: self.enable_aggressive_mode,
            enable_parallel_execution: self.enable_parallel_execution,
            priority_order: self.priority_order,
            early_stop_threshold: self.early_stop_threshold,
        }
    }
}

impl Default for OptimizationStrategyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

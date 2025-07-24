//! Core optimization operations with zero allocation patterns
//!
//! This module provides the foundational optimization operations with blazing-fast
//! performance, zero allocation optimizations, and elegant ergonomic interfaces.

pub mod optimization_executor;
pub mod optimization_strategy;
pub mod execution_metrics;
pub mod operation_cache;
pub mod safety_constraints;
pub mod optimization_results;

// Re-export main types for ergonomic usage
pub use optimization_executor::{
    OptimizationExecutor,
    ExecutorHealthStatus,
    ExecutorConfiguration,
    PerformanceSummary,
};

pub use optimization_strategy::{
    OptimizationStrategy,
    OptimizationStrategyBuilder,
};

pub use execution_metrics::{
    ExecutionMetrics,
    ExecutionRecord,
    PerformanceTrend,
    PerformanceSummary as MetricsPerformanceSummary,
    RecentPerformanceStats,
};

pub use operation_cache::{
    OperationCache,
    CachedResult,
    CacheStatistics,
    CacheHealthStatus,
    CachePerformanceLevel,
    CachePerformanceMetrics,
};

pub use safety_constraints::{
    SafetyConstraints,
    SafetyConstraintsBuilder,
    ConstraintSummary,
};

pub use optimization_results::{
    SingleOptimizationResult,
    OptimizationResult,
    DetailedResultInfo,
    PerformanceMetrics,
};

/// Core optimization operations utilities
pub mod utils {
    use super::*;
    use std::time::Duration;

    /// Create a conservative optimization setup
    #[inline]
    pub fn create_conservative_setup() -> (OptimizationStrategy, SafetyConstraints) {
        (
            OptimizationStrategy::conservative(),
            SafetyConstraints::strict(),
        )
    }

    /// Create a balanced optimization setup
    #[inline]
    pub fn create_balanced_setup() -> (OptimizationStrategy, SafetyConstraints) {
        (
            OptimizationStrategy::balanced(),
            SafetyConstraints::balanced(),
        )
    }

    /// Create an aggressive optimization setup
    #[inline]
    pub fn create_aggressive_setup() -> (OptimizationStrategy, SafetyConstraints) {
        (
            OptimizationStrategy::aggressive(),
            SafetyConstraints::relaxed(),
        )
    }

    /// Create a production-ready optimization setup
    #[inline]
    pub fn create_production_setup() -> (OptimizationStrategy, SafetyConstraints) {
        (
            OptimizationStrategy::balanced(),
            SafetyConstraints::production(),
        )
    }

    /// Create a development optimization setup
    #[inline]
    pub fn create_development_setup() -> (OptimizationStrategy, SafetyConstraints) {
        (
            OptimizationStrategy::aggressive(),
            SafetyConstraints::development(),
        )
    }

    /// Validate optimization configuration
    #[inline]
    pub fn validate_configuration(
        strategy: &OptimizationStrategy,
        constraints: &SafetyConstraints,
    ) -> Result<(), String> {
        if let Err(e) = strategy.validate() {
            return Err(format!("Invalid strategy: {}", e));
        }

        if let Err(e) = constraints.validate_configuration() {
            return Err(format!("Invalid constraints: {}", e));
        }

        // Check compatibility
        if strategy.max_execution_time > Duration::from_secs(600) && constraints.is_conservative() {
            return Err("Long execution time incompatible with conservative constraints".to_string());
        }

        if strategy.is_aggressive() && constraints.strictness_score() > 0.8 {
            return Err("Aggressive strategy incompatible with strict constraints".to_string());
        }

        Ok(())
    }

    /// Get recommended cache size for strategy
    #[inline]
    pub fn recommended_cache_size(strategy: &OptimizationStrategy) -> usize {
        if strategy.is_aggressive() {
            100
        } else if strategy == &OptimizationStrategy::balanced() {
            50
        } else {
            25
        }
    }

    /// Create optimized executor for item count
    #[inline]
    pub fn create_executor_for_item_count(item_count: usize) -> OptimizationExecutor {
        let (strategy, constraints) = if item_count < 1000 {
            create_conservative_setup()
        } else if item_count < 10000 {
            create_balanced_setup()
        } else {
            create_aggressive_setup()
        };

        let cache_size = recommended_cache_size(&strategy);
        OptimizationExecutor::with_all_params(strategy, constraints, cache_size)
    }

    /// Calculate optimal batch size
    #[inline]
    pub fn calculate_optimal_batch_size(
        total_items: usize,
        constraints: &SafetyConstraints,
    ) -> usize {
        let max_per_operation = constraints.max_items_per_operation;
        
        if total_items <= max_per_operation {
            total_items
        } else {
            // Calculate batch size that minimizes operations while respecting constraints
            let ideal_batches = (total_items as f64 / max_per_operation as f64).ceil() as usize;
            (total_items / ideal_batches).max(1)
        }
    }

    /// Estimate optimization time
    #[inline]
    pub fn estimate_optimization_time(
        item_count: usize,
        strategy: &OptimizationStrategy,
        constraints: &SafetyConstraints,
    ) -> Duration {
        let batch_size = calculate_optimal_batch_size(item_count, constraints);
        let batches = (item_count / batch_size.max(1)) + 1;
        
        let time_per_batch = strategy.operation_timeout();
        time_per_batch * batches as u32
    }

    /// Check if optimization is recommended
    #[inline]
    pub fn is_optimization_recommended(
        item_count: usize,
        available_memory: usize,
        cpu_usage: f64,
    ) -> bool {
        // Don't optimize if system is under stress
        if cpu_usage > 90.0 || available_memory < 100 * 1024 * 1024 {
            return false;
        }

        // Don't optimize very small datasets
        if item_count < 100 {
            return false;
        }

        true
    }

    /// Get optimization priority score (0.0-1.0)
    #[inline]
    pub fn optimization_priority_score(
        item_count: usize,
        memory_usage: usize,
        last_optimization_age: Duration,
    ) -> f64 {
        let size_score = (item_count as f64 / 10000.0).min(1.0);
        let memory_score = (memory_usage as f64 / (1024.0 * 1024.0 * 100.0)).min(1.0);
        let age_score = (last_optimization_age.as_secs() as f64 / 86400.0).min(1.0); // Days

        (size_score + memory_score + age_score) / 3.0
    }
}

/// Presets for common optimization scenarios
pub mod presets {
    use super::*;

    /// Real-time optimization preset (fast, minimal impact)
    #[inline]
    pub fn real_time() -> OptimizationExecutor {
        let strategy = OptimizationStrategy::builder()
            .max_execution_time(std::time::Duration::from_secs(5))
            .max_operations(3)
            .min_improvement_threshold(2.0)
            .aggressive_mode(false)
            .parallel_execution(false)
            .build();

        let constraints = SafetyConstraints::builder()
            .max_memory_mb(50)
            .max_cpu_usage(30.0)
            .require_backup(false)
            .enable_rollback(true)
            .max_items_per_operation(500)
            .operation_timeout_secs(2)
            .build()
            .expect("Valid constraints");

        OptimizationExecutor::with_all_params(strategy, constraints, 10)
    }

    /// Background optimization preset (thorough, low priority)
    #[inline]
    pub fn background() -> OptimizationExecutor {
        let strategy = OptimizationStrategy::builder()
            .max_execution_time(std::time::Duration::from_secs(300))
            .max_operations(50)
            .min_improvement_threshold(0.1)
            .aggressive_mode(true)
            .parallel_execution(true)
            .build();

        let constraints = SafetyConstraints::builder()
            .max_memory_mb(200)
            .max_cpu_usage(50.0)
            .require_backup(true)
            .enable_rollback(true)
            .max_items_per_operation(5000)
            .operation_timeout_secs(30)
            .build()
            .expect("Valid constraints");

        OptimizationExecutor::with_all_params(strategy, constraints, 100)
    }

    /// Maintenance optimization preset (comprehensive, scheduled)
    #[inline]
    pub fn maintenance() -> OptimizationExecutor {
        let strategy = OptimizationStrategy::builder()
            .max_execution_time(std::time::Duration::from_secs(600))
            .max_operations(100)
            .min_improvement_threshold(0.05)
            .aggressive_mode(true)
            .parallel_execution(true)
            .build();

        let constraints = SafetyConstraints::builder()
            .max_memory_mb(500)
            .max_cpu_usage(80.0)
            .require_backup(true)
            .enable_rollback(true)
            .max_items_per_operation(10000)
            .operation_timeout_secs(60)
            .build()
            .expect("Valid constraints");

        OptimizationExecutor::with_all_params(strategy, constraints, 200)
    }

    /// Emergency optimization preset (quick fixes only)
    #[inline]
    pub fn emergency() -> OptimizationExecutor {
        let strategy = OptimizationStrategy::builder()
            .max_execution_time(std::time::Duration::from_secs(10))
            .max_operations(5)
            .min_improvement_threshold(5.0)
            .aggressive_mode(false)
            .parallel_execution(false)
            .build();

        let constraints = SafetyConstraints::builder()
            .max_memory_mb(100)
            .max_cpu_usage(40.0)
            .require_backup(false)
            .enable_rollback(true)
            .max_items_per_operation(1000)
            .operation_timeout_secs(3)
            .build()
            .expect("Valid constraints");

        OptimizationExecutor::with_all_params(strategy, constraints, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservative_setup() {
        let (strategy, constraints) = utils::create_conservative_setup();
        assert!(!strategy.is_aggressive());
        assert!(constraints.is_conservative());
        assert!(utils::validate_configuration(&strategy, &constraints).is_ok());
    }

    #[test]
    fn test_aggressive_setup() {
        let (strategy, constraints) = utils::create_aggressive_setup();
        assert!(strategy.is_aggressive());
        assert!(constraints.allows_aggressive_optimization());
        assert!(utils::validate_configuration(&strategy, &constraints).is_ok());
    }

    #[test]
    fn test_executor_creation() {
        let executor = utils::create_executor_for_item_count(5000);
        assert!(executor.is_ready());
        assert!(executor.validate_configuration().is_ok());
    }

    #[test]
    fn test_presets() {
        let real_time = presets::real_time();
        let background = presets::background();
        let maintenance = presets::maintenance();
        let emergency = presets::emergency();

        assert!(real_time.is_ready());
        assert!(background.is_ready());
        assert!(maintenance.is_ready());
        assert!(emergency.is_ready());
    }

    #[test]
    fn test_batch_size_calculation() {
        let constraints = SafetyConstraints::balanced();
        
        let small_batch = utils::calculate_optimal_batch_size(100, &constraints);
        assert_eq!(small_batch, 100);
        
        let large_batch = utils::calculate_optimal_batch_size(50000, &constraints);
        assert!(large_batch <= constraints.max_items_per_operation);
        assert!(large_batch > 0);
    }

    #[test]
    fn test_optimization_recommendation() {
        assert!(utils::is_optimization_recommended(1000, 200 * 1024 * 1024, 50.0));
        assert!(!utils::is_optimization_recommended(50, 200 * 1024 * 1024, 50.0));
        assert!(!utils::is_optimization_recommended(1000, 50 * 1024 * 1024, 95.0));
    }
}

//! Core optimization executor with zero allocation patterns
//!
//! This module provides the main optimization executor with zero allocation
//! patterns and blazing-fast performance for memory optimization operations.

use std::time::Duration;
use tracing::{debug, info, warn};

use crate::utils::{Result, error::Error};
use super::{
    optimization_strategy::OptimizationStrategy,
    execution_metrics::ExecutionMetrics,
    operation_cache::{OperationCache, CacheStatistics},
    safety_constraints::SafetyConstraints,
};
use super::super::{
    optimization_recommendations::{OptimizationRecommendation, RecommendationType, AnalysisResults},
    health_check::{HealthCheckReport, HealthIssue, IssueSeverity, PerformanceMetrics, ResourceUtilization},
};
use super::super::super::{
    semantic_item::SemanticItem,
    semantic_relationship::SemanticRelationship,
    memory_manager_core::MemoryStatistics,
};

/// Memory optimization executor with zero allocation patterns
pub struct OptimizationExecutor {
    /// Current optimization strategy
    strategy: OptimizationStrategy,
    /// Execution metrics
    metrics: ExecutionMetrics,
    /// Operation cache for performance
    operation_cache: OperationCache,
    /// Safety constraints
    safety_constraints: SafetyConstraints,
}

impl OptimizationExecutor {
    /// Create new optimization executor with zero allocation optimizations
    #[inline]
    pub fn new(strategy: OptimizationStrategy) -> Self {
        Self {
            strategy,
            metrics: ExecutionMetrics::new(),
            operation_cache: OperationCache::new(),
            safety_constraints: SafetyConstraints::default(),
        }
    }

    /// Create with custom safety constraints
    #[inline]
    pub fn with_constraints(strategy: OptimizationStrategy, constraints: SafetyConstraints) -> Self {
        Self {
            strategy,
            metrics: ExecutionMetrics::new(),
            operation_cache: OperationCache::new(),
            safety_constraints: constraints,
        }
    }

    /// Create with custom cache capacity
    #[inline]
    pub fn with_cache_capacity(strategy: OptimizationStrategy, cache_capacity: usize) -> Self {
        Self {
            strategy,
            metrics: ExecutionMetrics::new(),
            operation_cache: OperationCache::with_capacity(cache_capacity),
            safety_constraints: SafetyConstraints::default(),
        }
    }

    /// Create with all custom parameters
    #[inline]
    pub fn with_all_params(
        strategy: OptimizationStrategy,
        constraints: SafetyConstraints,
        cache_capacity: usize,
    ) -> Self {
        Self {
            strategy,
            metrics: ExecutionMetrics::new(),
            operation_cache: OperationCache::with_capacity(cache_capacity),
            safety_constraints: constraints,
        }
    }

    /// Get current strategy
    #[inline]
    pub fn strategy(&self) -> &OptimizationStrategy {
        &self.strategy
    }

    /// Update optimization strategy
    #[inline]
    pub fn set_strategy(&mut self, strategy: OptimizationStrategy) {
        debug!("Updating optimization strategy");
        self.strategy = strategy;
    }

    /// Get execution metrics
    #[inline]
    pub fn metrics(&self) -> &ExecutionMetrics {
        &self.metrics
    }

    /// Get mutable execution metrics
    #[inline]
    pub fn metrics_mut(&mut self) -> &mut ExecutionMetrics {
        &mut self.metrics
    }

    /// Get safety constraints
    #[inline]
    pub fn safety_constraints(&self) -> &SafetyConstraints {
        &self.safety_constraints
    }

    /// Update safety constraints
    #[inline]
    pub fn set_safety_constraints(&mut self, constraints: SafetyConstraints) {
        debug!("Updating safety constraints");
        self.safety_constraints = constraints;
    }

    /// Get operation cache
    #[inline]
    pub fn operation_cache(&self) -> &OperationCache {
        &self.operation_cache
    }

    /// Get mutable operation cache
    #[inline]
    pub fn operation_cache_mut(&mut self) -> &mut OperationCache {
        &mut self.operation_cache
    }

    /// Clear operation cache
    #[inline]
    pub fn clear_cache(&mut self) {
        debug!("Clearing operation cache");
        self.operation_cache.clear();
    }

    /// Get cache statistics
    #[inline]
    pub fn cache_stats(&self) -> CacheStatistics {
        self.operation_cache.statistics()
    }

    /// Check if executor is ready for optimization
    #[inline]
    pub fn is_ready(&self) -> bool {
        let strategy_valid = self.strategy.is_valid();
        let constraints_satisfied = self.safety_constraints.are_satisfied();
        
        if !strategy_valid {
            warn!("Optimization strategy is not valid");
        }
        if !constraints_satisfied {
            warn!("Safety constraints are not satisfied");
        }
        
        strategy_valid && constraints_satisfied
    }

    /// Reset execution metrics
    #[inline]
    pub fn reset_metrics(&mut self) {
        debug!("Resetting execution metrics");
        self.metrics.reset();
    }

    /// Reset all state
    #[inline]
    pub fn reset_all(&mut self) {
        debug!("Resetting all executor state");
        self.metrics.reset();
        self.operation_cache.clear();
    }

    /// Check if cache should be cleared based on performance
    #[inline]
    pub fn should_clear_cache(&self) -> bool {
        let stats = self.cache_stats();
        stats.hit_rate < 0.3 || stats.hits + stats.misses > 1000
    }

    /// Optimize cache if needed
    #[inline]
    pub fn optimize_cache_if_needed(&mut self) {
        if self.should_clear_cache() {
            debug!("Cache performance is poor, clearing cache");
            self.clear_cache();
        }
    }

    /// Get executor health status
    #[inline]
    pub fn health_status(&self) -> ExecutorHealthStatus {
        let cache_stats = self.cache_stats();
        let metrics_trend = self.metrics.performance_trend();
        
        let cache_healthy = cache_stats.hit_rate > 0.5;
        let metrics_healthy = matches!(metrics_trend, super::execution_metrics::PerformanceTrend::Improving | super::execution_metrics::PerformanceTrend::Stable);
        let constraints_satisfied = self.safety_constraints.are_satisfied();
        
        if cache_healthy && metrics_healthy && constraints_satisfied {
            ExecutorHealthStatus::Healthy
        } else if !constraints_satisfied {
            ExecutorHealthStatus::Critical
        } else if !cache_healthy || !metrics_healthy {
            ExecutorHealthStatus::Degraded
        } else {
            ExecutorHealthStatus::Unknown
        }
    }

    /// Get executor configuration summary
    #[inline]
    pub fn configuration_summary(&self) -> ExecutorConfiguration {
        ExecutorConfiguration {
            strategy_type: self.strategy.strategy_type(),
            cache_capacity: self.operation_cache.capacity(),
            safety_level: self.safety_constraints.safety_level(),
            max_execution_time: self.strategy.max_execution_time,
            max_operations: self.strategy.max_operations,
            parallel_execution_enabled: self.strategy.enable_parallel_execution,
        }
    }

    /// Validate executor configuration
    #[inline]
    pub fn validate_configuration(&self) -> Result<()> {
        if !self.strategy.is_valid() {
            return Err(Error::InvalidConfiguration("Invalid optimization strategy".to_string()));
        }
        
        if !self.safety_constraints.are_satisfied() {
            return Err(Error::InvalidConfiguration("Safety constraints not satisfied".to_string()));
        }
        
        if self.operation_cache.capacity() == 0 {
            return Err(Error::InvalidConfiguration("Cache capacity cannot be zero".to_string()));
        }
        
        Ok(())
    }

    /// Get performance summary
    #[inline]
    pub fn performance_summary(&self) -> PerformanceSummary {
        let cache_stats = self.cache_stats();
        
        PerformanceSummary {
            total_executions: self.metrics.total_executions,
            average_improvement: self.metrics.average_improvement,
            success_rate: self.metrics.success_rate,
            cache_hit_rate: cache_stats.hit_rate,
            performance_trend: self.metrics.performance_trend(),
            health_status: self.health_status(),
        }
    }
}

impl Default for OptimizationExecutor {
    fn default() -> Self {
        Self::new(OptimizationStrategy::default())
    }
}

/// Executor health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorHealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

impl ExecutorHealthStatus {
    /// Get status description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            ExecutorHealthStatus::Healthy => "Executor is operating normally",
            ExecutorHealthStatus::Degraded => "Executor performance is degraded",
            ExecutorHealthStatus::Critical => "Executor has critical issues",
            ExecutorHealthStatus::Unknown => "Executor status is unknown",
        }
    }

    /// Check if status requires attention
    #[inline]
    pub fn requires_attention(&self) -> bool {
        matches!(self, ExecutorHealthStatus::Degraded | ExecutorHealthStatus::Critical)
    }
}

/// Executor configuration summary
#[derive(Debug, Clone)]
pub struct ExecutorConfiguration {
    pub strategy_type: String,
    pub cache_capacity: usize,
    pub safety_level: String,
    pub max_execution_time: Duration,
    pub max_operations: usize,
    pub parallel_execution_enabled: bool,
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_executions: usize,
    pub average_improvement: f64,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub performance_trend: super::execution_metrics::PerformanceTrend,
    pub health_status: ExecutorHealthStatus,
}

impl PerformanceSummary {
    /// Check if performance is good
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.success_rate > 0.8 &&
        self.cache_hit_rate > 0.6 &&
        self.average_improvement > 1.0 &&
        matches!(self.performance_trend, super::execution_metrics::PerformanceTrend::Improving | super::execution_metrics::PerformanceTrend::Stable)
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let success_score = self.success_rate;
        let cache_score = self.cache_hit_rate;
        let improvement_score = (self.average_improvement / 10.0).min(1.0);
        let trend_score = match self.performance_trend {
            super::execution_metrics::PerformanceTrend::Improving => 1.0,
            super::execution_metrics::PerformanceTrend::Stable => 0.8,
            super::execution_metrics::PerformanceTrend::Declining => 0.4,
        };
        
        (success_score + cache_score + improvement_score + trend_score) / 4.0
    }
}

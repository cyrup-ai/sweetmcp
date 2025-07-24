//! Async vector optimization coordinator core
//!
//! This module provides the core coordinator with zero allocation patterns
//! and blazing-fast performance for async vector optimization operations.

use smallvec::SmallVec;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::memory::filter::MemoryFilter;
use crate::utils::error::Error;
use super::{VectorSearchResult, VectorStore};
use crate::vector::async_vector_operations::DistanceMetric;
use crate::vector::async_vector_optimization::search_strategies::{SearchStrategy, SearchStrategyExecutor, SearchMetrics};
use crate::vector::async_vector_optimization::optimization_algorithms::{OptimizationExecutor, OptimizationAlgorithm};

use super::coordinator_types::{
    CoordinationMetrics, CoordinatorConfig, OptimizationSpec, OptimizationPipelineResult,
    VectorCharacteristics, OptimizationRecommendation
};

/// High-level coordinator for async vector optimization operations
pub struct AsyncVectorOptimizationCoordinator {
    /// Search strategy executor
    search_executor: Arc<RwLock<SearchStrategyExecutor>>,
    /// Optimization algorithm executor
    optimization_executor: Arc<RwLock<OptimizationExecutor>>,
    /// Coordination metrics
    metrics: CoordinationMetrics,
    /// Configuration
    config: CoordinatorConfig,
}

impl AsyncVectorOptimizationCoordinator {
    /// Create new async vector optimization coordinator
    #[inline]
    pub fn new() -> Self {
        Self {
            search_executor: Arc::new(RwLock::new(SearchStrategyExecutor::new(SearchStrategy::BruteForce))),
            optimization_executor: Arc::new(RwLock::new(OptimizationExecutor::new())),
            metrics: CoordinationMetrics::new(),
            config: CoordinatorConfig::default(),
        }
    }

    /// Create coordinator with custom configuration
    #[inline]
    pub fn with_config(config: CoordinatorConfig) -> Self {
        Self {
            search_executor: Arc::new(RwLock::new(SearchStrategyExecutor::new(config.default_search_strategy))),
            optimization_executor: Arc::new(RwLock::new(OptimizationExecutor::new())),
            metrics: CoordinationMetrics::new(),
            config,
        }
    }

    /// Create coordinator with custom search strategy
    #[inline]
    pub fn with_search_strategy(strategy: SearchStrategy) -> Self {
        Self {
            search_executor: Arc::new(RwLock::new(SearchStrategyExecutor::new(strategy))),
            optimization_executor: Arc::new(RwLock::new(OptimizationExecutor::new())),
            metrics: CoordinationMetrics::new(),
            config: CoordinatorConfig::default(),
        }
    }

    /// Get current coordination metrics
    #[inline]
    pub fn metrics(&self) -> &CoordinationMetrics {
        &self.metrics
    }

    /// Get current configuration
    #[inline]
    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }

    /// Update coordinator configuration
    #[inline]
    pub async fn update_config(&mut self, config: CoordinatorConfig) -> Result<(), Error> {
        // Update search strategy if changed
        if config.default_search_strategy != self.config.default_search_strategy {
            let mut executor = self.search_executor.write().await;
            *executor = SearchStrategyExecutor::new(config.default_search_strategy);
        }

        self.config = config;
        debug!("Coordinator configuration updated");
        Ok(())
    }

    /// Reset coordination metrics
    #[inline]
    pub fn reset_metrics(&mut self) {
        self.metrics.reset();
        debug!("Coordination metrics reset");
    }

    /// Get search executor statistics
    #[inline]
    pub async fn search_statistics(&self) -> Result<SearchMetrics, Error> {
        let executor = self.search_executor.read().await;
        Ok(executor.metrics().clone())
    }

    /// Check coordinator health status
    #[inline]
    pub async fn health_status(&self) -> CoordinatorHealthStatus {
        let search_healthy = match self.search_executor.try_read() {
            Ok(_) => true,
            Err(_) => false,
        };

        let optimization_healthy = match self.optimization_executor.try_read() {
            Ok(_) => true,
            Err(_) => false,
        };

        let metrics_healthy = self.metrics.is_healthy();

        if search_healthy && optimization_healthy && metrics_healthy {
            CoordinatorHealthStatus::Healthy
        } else if !search_healthy || !optimization_healthy {
            CoordinatorHealthStatus::Critical
        } else {
            CoordinatorHealthStatus::Degraded
        }
    }

    /// Validate coordinator state
    #[inline]
    pub async fn validate_state(&self) -> Result<(), Error> {
        // Check if executors are accessible
        let _search_executor = self.search_executor.read().await;
        let _optimization_executor = self.optimization_executor.read().await;

        // Validate configuration
        if !self.config.is_valid() {
            return Err(Error::InvalidConfiguration("Invalid coordinator configuration".to_string()));
        }

        Ok(())
    }

    /// Get coordinator performance summary
    #[inline]
    pub fn performance_summary(&self) -> CoordinatorPerformanceSummary {
        CoordinatorPerformanceSummary {
            total_search_operations: self.metrics.total_search_operations,
            total_optimization_operations: self.metrics.total_optimization_operations,
            average_search_time: self.metrics.average_search_time(),
            average_optimization_time: self.metrics.average_optimization_time(),
            success_rate: self.metrics.success_rate(),
            health_status: futures::executor::block_on(self.health_status()),
        }
    }

    /// Shutdown coordinator gracefully
    #[inline]
    pub async fn shutdown(&self) -> Result<(), Error> {
        debug!("Shutting down async vector optimization coordinator");
        
        // Ensure all operations complete
        let _search_executor = self.search_executor.write().await;
        let _optimization_executor = self.optimization_executor.write().await;
        
        info!("Async vector optimization coordinator shutdown complete");
        Ok(())
    }

    /// Clone coordinator with shared state
    #[inline]
    pub fn clone_shared(&self) -> Self {
        Self {
            search_executor: Arc::clone(&self.search_executor),
            optimization_executor: Arc::clone(&self.optimization_executor),
            metrics: self.metrics.clone(),
            config: self.config.clone(),
        }
    }

    /// Get executor references for advanced operations
    #[inline]
    pub fn executors(&self) -> (Arc<RwLock<SearchStrategyExecutor>>, Arc<RwLock<OptimizationExecutor>>) {
        (Arc::clone(&self.search_executor), Arc::clone(&self.optimization_executor))
    }

    /// Check if coordinator is ready for operations
    #[inline]
    pub async fn is_ready(&self) -> bool {
        self.validate_state().await.is_ok()
    }

    /// Get coordinator resource usage
    #[inline]
    pub fn resource_usage(&self) -> CoordinatorResourceUsage {
        CoordinatorResourceUsage {
            search_executor_memory: std::mem::size_of::<SearchStrategyExecutor>(),
            optimization_executor_memory: std::mem::size_of::<OptimizationExecutor>(),
            metrics_memory: self.metrics.memory_usage(),
            config_memory: std::mem::size_of::<CoordinatorConfig>(),
            total_memory: self.estimated_memory_usage(),
        }
    }

    /// Estimate total memory usage
    #[inline]
    fn estimated_memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.metrics.memory_usage() +
        std::mem::size_of::<SearchStrategyExecutor>() +
        std::mem::size_of::<OptimizationExecutor>()
    }
}

impl Default for AsyncVectorOptimizationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AsyncVectorOptimizationCoordinator {
    fn clone(&self) -> Self {
        self.clone_shared()
    }
}

/// Coordinator health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinatorHealthStatus {
    Healthy,
    Degraded,
    Critical,
}

impl CoordinatorHealthStatus {
    /// Get status description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            CoordinatorHealthStatus::Healthy => "Coordinator is operating normally",
            CoordinatorHealthStatus::Degraded => "Coordinator has performance issues",
            CoordinatorHealthStatus::Critical => "Coordinator has critical failures",
        }
    }

    /// Check if status requires attention
    #[inline]
    pub fn requires_attention(&self) -> bool {
        matches!(self, CoordinatorHealthStatus::Degraded | CoordinatorHealthStatus::Critical)
    }
}

/// Coordinator performance summary
#[derive(Debug, Clone)]
pub struct CoordinatorPerformanceSummary {
    pub total_search_operations: usize,
    pub total_optimization_operations: usize,
    pub average_search_time: std::time::Duration,
    pub average_optimization_time: std::time::Duration,
    pub success_rate: f64,
    pub health_status: CoordinatorHealthStatus,
}

impl CoordinatorPerformanceSummary {
    /// Check if performance is good
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.success_rate > 0.9 &&
        matches!(self.health_status, CoordinatorHealthStatus::Healthy) &&
        self.average_search_time < std::time::Duration::from_millis(100) &&
        self.average_optimization_time < std::time::Duration::from_secs(5)
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let success_score = self.success_rate;
        let health_score = match self.health_status {
            CoordinatorHealthStatus::Healthy => 1.0,
            CoordinatorHealthStatus::Degraded => 0.6,
            CoordinatorHealthStatus::Critical => 0.2,
        };
        let speed_score = if self.average_search_time < std::time::Duration::from_millis(50) { 1.0 } else { 0.7 };
        
        (success_score + health_score + speed_score) / 3.0
    }
}

/// Coordinator resource usage information
#[derive(Debug, Clone)]
pub struct CoordinatorResourceUsage {
    pub search_executor_memory: usize,
    pub optimization_executor_memory: usize,
    pub metrics_memory: usize,
    pub config_memory: usize,
    pub total_memory: usize,
}

impl CoordinatorResourceUsage {
    /// Get memory usage in MB
    #[inline]
    pub fn memory_mb(&self) -> f64 {
        self.total_memory as f64 / (1024.0 * 1024.0)
    }

    /// Check if memory usage is within acceptable limits
    #[inline]
    pub fn is_memory_usage_acceptable(&self) -> bool {
        self.memory_mb() < 100.0 // Less than 100MB
    }
}
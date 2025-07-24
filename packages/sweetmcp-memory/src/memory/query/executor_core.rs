//! Core query executor structures and configuration
//!
//! This module provides the core query executor functionality with zero-allocation
//! patterns and blazing-fast performance for memory query execution.

use futures::StreamExt;
use smallvec::SmallVec;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use crate::utils::{Result, error::Error};
use super::core::{MemoryQuery, SortOrder};
use super::super::{
    memory_manager::MemoryManager,
    memory_node::MemoryNode,
};

/// Query executor for complex memory queries
#[derive(Debug)]
pub struct MemoryQueryExecutor {
    /// Query configuration
    config: QueryConfig,
}

impl MemoryQueryExecutor {
    /// Create a new query executor
    #[inline]
    pub fn new(config: QueryConfig) -> Self {
        Self { config }
    }

    /// Create a query executor with default configuration
    #[inline]
    pub fn with_defaults() -> Self {
        Self::new(QueryConfig::default())
    }

    /// Create a query executor with optimized configuration
    #[inline]
    pub fn optimized() -> Self {
        Self::new(QueryConfig::optimized())
    }

    /// Create a query executor with high-performance configuration
    #[inline]
    pub fn high_performance() -> Self {
        Self::new(QueryConfig::high_performance())
    }

    /// Get current configuration
    #[inline]
    pub fn config(&self) -> &QueryConfig {
        &self.config
    }

    /// Update configuration
    #[inline]
    pub fn set_config(&mut self, config: QueryConfig) {
        self.config = config;
    }

    /// Enable query optimization
    #[inline]
    pub fn enable_optimization(&mut self) {
        self.config.optimize = true;
    }

    /// Disable query optimization
    #[inline]
    pub fn disable_optimization(&mut self) {
        self.config.optimize = false;
    }

    /// Set query timeout
    #[inline]
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.config.timeout_ms = timeout_ms;
    }

    /// Set maximum parallel operations
    #[inline]
    pub fn set_max_parallel(&mut self, max_parallel: usize) {
        self.config.max_parallel = max_parallel;
    }

    /// Enable profiling
    #[inline]
    pub fn enable_profiling(&mut self) {
        self.config.enable_profiling = true;
    }

    /// Disable profiling
    #[inline]
    pub fn disable_profiling(&mut self) {
        self.config.enable_profiling = false;
    }
}

/// Configuration for query execution
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Enable query optimization
    pub optimize: bool,
    /// Enable caching
    pub cache: bool,
    /// Query timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of parallel operations
    pub max_parallel: usize,
    /// Enable result streaming
    pub enable_streaming: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Enable query profiling
    pub enable_profiling: bool,
    /// Batch size for processing
    pub batch_size: usize,
    /// Enable early termination optimization
    pub enable_early_termination: bool,
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            optimize: true,
            cache: true,
            timeout_ms: 5000,
            max_parallel: 10,
            enable_streaming: true,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            enable_profiling: false,
            batch_size: 1000,
            enable_early_termination: true,
            memory_pressure_threshold: 0.8,
        }
    }
}

impl QueryConfig {
    /// Create new query configuration
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create optimized configuration for performance
    #[inline]
    pub fn optimized() -> Self {
        Self {
            optimize: true,
            cache: true,
            timeout_ms: 10000,
            max_parallel: 20,
            enable_streaming: true,
            max_memory_bytes: 200 * 1024 * 1024, // 200MB
            enable_profiling: false,
            batch_size: 2000,
            enable_early_termination: true,
            memory_pressure_threshold: 0.9,
        }
    }

    /// Create high-performance configuration
    #[inline]
    pub fn high_performance() -> Self {
        Self {
            optimize: true,
            cache: true,
            timeout_ms: 30000,
            max_parallel: 50,
            enable_streaming: true,
            max_memory_bytes: 500 * 1024 * 1024, // 500MB
            enable_profiling: true,
            batch_size: 5000,
            enable_early_termination: true,
            memory_pressure_threshold: 0.95,
        }
    }

    /// Create memory-constrained configuration
    #[inline]
    pub fn memory_constrained() -> Self {
        Self {
            optimize: true,
            cache: false,
            timeout_ms: 3000,
            max_parallel: 5,
            enable_streaming: true,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB
            enable_profiling: false,
            batch_size: 500,
            enable_early_termination: true,
            memory_pressure_threshold: 0.6,
        }
    }

    /// Create configuration for debugging
    #[inline]
    pub fn debug() -> Self {
        Self {
            optimize: false,
            cache: false,
            timeout_ms: 60000,
            max_parallel: 1,
            enable_streaming: false,
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            enable_profiling: true,
            batch_size: 100,
            enable_early_termination: false,
            memory_pressure_threshold: 1.0,
        }
    }

    /// Check if configuration is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.timeout_ms > 0
            && self.max_parallel > 0
            && self.max_memory_bytes > 0
            && self.batch_size > 0
            && self.memory_pressure_threshold >= 0.0
            && self.memory_pressure_threshold <= 1.0
    }

    /// Get memory limit in bytes
    #[inline]
    pub fn memory_limit(&self) -> usize {
        self.max_memory_bytes
    }

    /// Get timeout duration
    #[inline]
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }

    /// Check if optimization is enabled
    #[inline]
    pub fn is_optimization_enabled(&self) -> bool {
        self.optimize
    }

    /// Check if caching is enabled
    #[inline]
    pub fn is_caching_enabled(&self) -> bool {
        self.cache
    }

    /// Check if streaming is enabled
    #[inline]
    pub fn is_streaming_enabled(&self) -> bool {
        self.enable_streaming
    }

    /// Check if profiling is enabled
    #[inline]
    pub fn is_profiling_enabled(&self) -> bool {
        self.enable_profiling
    }

    /// Check if early termination is enabled
    #[inline]
    pub fn is_early_termination_enabled(&self) -> bool {
        self.enable_early_termination
    }

    /// Get effective batch size based on memory constraints
    #[inline]
    pub fn effective_batch_size(&self, estimated_item_size: usize) -> usize {
        if estimated_item_size == 0 {
            return self.batch_size;
        }
        
        let memory_based_batch = self.max_memory_bytes / (estimated_item_size * 2); // Safety factor
        memory_based_batch.min(self.batch_size).max(1)
    }

    /// Check if memory pressure threshold is exceeded
    #[inline]
    pub fn is_memory_pressure_exceeded(&self, current_usage: usize) -> bool {
        let threshold_bytes = (self.max_memory_bytes as f64 * self.memory_pressure_threshold) as usize;
        current_usage > threshold_bytes
    }
}

/// Query execution statistics
#[derive(Debug, Clone)]
pub struct QueryExecutionStats {
    /// Total execution time
    pub execution_time: Duration,
    /// Number of items processed
    pub items_processed: usize,
    /// Number of items returned
    pub items_returned: usize,
    /// Peak memory usage in bytes
    pub peak_memory_usage: usize,
    /// Number of parallel operations
    pub parallel_operations: usize,
    /// Whether query was optimized
    pub was_optimized: bool,
    /// Whether early termination occurred
    pub early_termination: bool,
    /// Cache hit ratio (0.0-1.0)
    pub cache_hit_ratio: f64,
}

impl Default for QueryExecutionStats {
    fn default() -> Self {
        Self {
            execution_time: Duration::ZERO,
            items_processed: 0,
            items_returned: 0,
            peak_memory_usage: 0,
            parallel_operations: 0,
            was_optimized: false,
            early_termination: false,
            cache_hit_ratio: 0.0,
        }
    }
}

impl QueryExecutionStats {
    /// Create new execution statistics
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get throughput (items per second)
    #[inline]
    pub fn throughput(&self) -> f64 {
        if self.execution_time.is_zero() {
            return 0.0;
        }
        
        self.items_processed as f64 / self.execution_time.as_secs_f64()
    }

    /// Get selectivity ratio (returned/processed)
    #[inline]
    pub fn selectivity_ratio(&self) -> f64 {
        if self.items_processed == 0 {
            return 0.0;
        }
        
        self.items_returned as f64 / self.items_processed as f64
    }

    /// Get memory efficiency (items per MB)
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.peak_memory_usage == 0 {
            return 0.0;
        }
        
        let memory_mb = self.peak_memory_usage as f64 / (1024.0 * 1024.0);
        self.items_returned as f64 / memory_mb
    }

    /// Check if execution was efficient
    #[inline]
    pub fn is_efficient(&self) -> bool {
        self.throughput() > 1000.0 && self.memory_efficiency() > 100.0
    }
}
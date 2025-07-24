//! Optimization metrics, configuration, and performance tracking
//!
//! This module provides blazing-fast metrics collection with zero allocation
//! optimizations and elegant ergonomic interfaces for performance monitoring.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::algorithm_types::OptimizationAlgorithm;

/// Optimization execution metrics
#[derive(Debug)]
pub struct OptimizationMetrics {
    /// Total number of optimizations executed
    pub total_optimizations: AtomicUsize,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: AtomicUsize,
    /// Average improvement percentage
    pub average_improvement: AtomicUsize,
}

impl OptimizationMetrics {
    /// Create new optimization metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_optimizations: AtomicUsize::new(0),
            total_execution_time_ms: AtomicUsize::new(0),
            average_improvement: AtomicUsize::new(0),
        }
    }

    /// Record optimization execution
    #[inline]
    pub fn record_optimization(
        &self,
        algorithm: OptimizationAlgorithm,
        execution_time: Duration,
        improvement: f64,
    ) {
        self.total_optimizations.fetch_add(1, Ordering::Relaxed);
        self.total_execution_time_ms.fetch_add(execution_time.as_millis() as usize, Ordering::Relaxed);
        self.average_improvement.store((improvement * 100.0) as usize, Ordering::Relaxed);
    }

    /// Get total optimizations count
    #[inline]
    pub fn get_total_optimizations(&self) -> usize {
        self.total_optimizations.load(Ordering::Relaxed)
    }

    /// Get total execution time
    #[inline]
    pub fn get_total_execution_time(&self) -> Duration {
        Duration::from_millis(self.total_execution_time_ms.load(Ordering::Relaxed) as u64)
    }

    /// Get average improvement
    #[inline]
    pub fn get_average_improvement(&self) -> f64 {
        self.average_improvement.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get average execution time per optimization
    #[inline]
    pub fn get_average_execution_time(&self) -> Duration {
        let total_optimizations = self.get_total_optimizations();
        if total_optimizations == 0 {
            return Duration::from_millis(0);
        }

        let total_time_ms = self.total_execution_time_ms.load(Ordering::Relaxed);
        Duration::from_millis((total_time_ms / total_optimizations) as u64)
    }

    /// Get throughput (optimizations per second)
    #[inline]
    pub fn get_throughput(&self) -> f64 {
        let total_time = self.get_total_execution_time();
        let total_optimizations = self.get_total_optimizations();

        if total_time.as_secs_f64() == 0.0 || total_optimizations == 0 {
            return 0.0;
        }

        total_optimizations as f64 / total_time.as_secs_f64()
    }

    /// Reset all metrics
    #[inline]
    pub fn reset(&self) {
        self.total_optimizations.store(0, Ordering::Relaxed);
        self.total_execution_time_ms.store(0, Ordering::Relaxed);
        self.average_improvement.store(0, Ordering::Relaxed);
    }

    /// Get metrics summary
    #[inline]
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_optimizations: self.get_total_optimizations(),
            total_execution_time: self.get_total_execution_time(),
            average_improvement: self.get_average_improvement(),
            average_execution_time: self.get_average_execution_time(),
            throughput: self.get_throughput(),
        }
    }
}

impl Default for OptimizationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics summary structure
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_optimizations: usize,
    pub total_execution_time: Duration,
    pub average_improvement: f64,
    pub average_execution_time: Duration,
    pub throughput: f64,
}

impl MetricsSummary {
    /// Check if performance is good
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.average_improvement > 0.2 && // 20% improvement
        self.average_execution_time.as_millis() < 1000 && // Under 1 second
        self.throughput > 1.0 // At least 1 optimization per second
    }

    /// Get performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let improvement_score = (self.average_improvement / 1.0).min(1.0); // Normalize to 100%
        let time_score = 1.0 / (self.average_execution_time.as_secs_f64() + 0.1);
        let throughput_score = (self.throughput / 10.0).min(1.0); // Normalize to 10 ops/sec

        (improvement_score + time_score.min(1.0) + throughput_score) / 3.0
    }
}

/// Optimization configuration
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable aggressive optimizations
    pub aggressive_mode: bool,
    /// Maximum memory usage for optimizations (MB)
    pub max_memory_mb: usize,
    /// Parallel processing threads
    pub max_threads: usize,
    /// Cache size for optimization results
    pub cache_size: usize,
    /// Timeout for individual optimizations (milliseconds)
    pub optimization_timeout_ms: u64,
    /// Minimum improvement threshold to accept optimization
    pub min_improvement_threshold: f64,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            aggressive_mode: false,
            max_memory_mb: 1024,
            max_threads: 4,
            cache_size: 1000,
            optimization_timeout_ms: 30000, // 30 seconds
            min_improvement_threshold: 0.05, // 5% minimum improvement
            enable_monitoring: true,
            enable_detailed_logging: false,
        }
    }
}

impl OptimizationConfig {
    /// Create new optimization configuration
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create conservative configuration
    #[inline]
    pub fn conservative() -> Self {
        Self {
            aggressive_mode: false,
            max_memory_mb: 512,
            max_threads: 2,
            cache_size: 500,
            optimization_timeout_ms: 10000, // 10 seconds
            min_improvement_threshold: 0.1, // 10% minimum improvement
            enable_monitoring: true,
            enable_detailed_logging: false,
        }
    }

    /// Create aggressive configuration
    #[inline]
    pub fn aggressive() -> Self {
        Self {
            aggressive_mode: true,
            max_memory_mb: 4096,
            max_threads: 8,
            cache_size: 5000,
            optimization_timeout_ms: 120000, // 2 minutes
            min_improvement_threshold: 0.01, // 1% minimum improvement
            enable_monitoring: true,
            enable_detailed_logging: true,
        }
    }

    /// Create high-performance configuration
    #[inline]
    pub fn high_performance() -> Self {
        Self {
            aggressive_mode: true,
            max_memory_mb: 8192,
            max_threads: 16,
            cache_size: 10000,
            optimization_timeout_ms: 300000, // 5 minutes
            min_improvement_threshold: 0.005, // 0.5% minimum improvement
            enable_monitoring: true,
            enable_detailed_logging: true,
        }
    }

    /// Set aggressive mode
    #[inline]
    pub fn with_aggressive_mode(mut self, aggressive: bool) -> Self {
        self.aggressive_mode = aggressive;
        self
    }

    /// Set maximum memory usage
    #[inline]
    pub fn with_max_memory(mut self, max_memory_mb: usize) -> Self {
        self.max_memory_mb = max_memory_mb;
        self
    }

    /// Set maximum threads
    #[inline]
    pub fn with_max_threads(mut self, max_threads: usize) -> Self {
        self.max_threads = max_threads;
        self
    }

    /// Set cache size
    #[inline]
    pub fn with_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    /// Set optimization timeout
    #[inline]
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.optimization_timeout_ms = timeout_ms;
        self
    }

    /// Set minimum improvement threshold
    #[inline]
    pub fn with_min_improvement(mut self, threshold: f64) -> Self {
        self.min_improvement_threshold = threshold;
        self
    }

    /// Enable monitoring
    #[inline]
    pub fn with_monitoring(mut self, enable: bool) -> Self {
        self.enable_monitoring = enable;
        self
    }

    /// Enable detailed logging
    #[inline]
    pub fn with_detailed_logging(mut self, enable: bool) -> Self {
        self.enable_detailed_logging = enable;
        self
    }

    /// Validate configuration
    #[inline]
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.max_memory_mb == 0 {
            return Err("Maximum memory must be greater than 0");
        }
        if self.max_threads == 0 {
            return Err("Maximum threads must be greater than 0");
        }
        if self.cache_size == 0 {
            return Err("Cache size must be greater than 0");
        }
        if self.optimization_timeout_ms == 0 {
            return Err("Optimization timeout must be greater than 0");
        }
        if self.min_improvement_threshold < 0.0 || self.min_improvement_threshold > 1.0 {
            return Err("Minimum improvement threshold must be between 0.0 and 1.0");
        }
        Ok(())
    }

    /// Get memory limit in bytes
    #[inline]
    pub fn memory_limit_bytes(&self) -> usize {
        self.max_memory_mb * 1024 * 1024
    }

    /// Get timeout as Duration
    #[inline]
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_millis(self.optimization_timeout_ms)
    }

    /// Check if configuration is suitable for vector count
    #[inline]
    pub fn is_suitable_for_vector_count(&self, vector_count: usize) -> bool {
        let estimated_memory_per_vector = 1024; // 1KB per vector estimate
        let total_estimated_memory = vector_count * estimated_memory_per_vector;
        
        total_estimated_memory <= self.memory_limit_bytes()
    }
}

/// Performance cache for optimization results
#[derive(Debug)]
pub struct PerformanceCache {
    /// Cache size limit
    max_size: usize,
    /// Current cache entries
    entries: AtomicUsize,
    /// Cache hit count
    hits: AtomicUsize,
    /// Cache miss count
    misses: AtomicUsize,
}

impl PerformanceCache {
    /// Create new performance cache
    #[inline]
    pub fn new() -> Self {
        Self {
            max_size: 1000,
            entries: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Create cache with specific size
    #[inline]
    pub fn with_size(max_size: usize) -> Self {
        Self {
            max_size,
            entries: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    /// Check if cache has capacity
    #[inline]
    pub fn has_capacity(&self) -> bool {
        self.entries.load(Ordering::Relaxed) < self.max_size
    }

    /// Add cache entry
    #[inline]
    pub fn add_entry(&self) {
        if self.has_capacity() {
            self.entries.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record cache hit
    #[inline]
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    #[inline]
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cache hit rate
    #[inline]
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            return 0.0;
        }

        hits as f64 / total as f64
    }

    /// Get cache utilization
    #[inline]
    pub fn utilization(&self) -> f64 {
        let entries = self.entries.load(Ordering::Relaxed);
        entries as f64 / self.max_size as f64
    }

    /// Get cache statistics
    #[inline]
    pub fn get_statistics(&self) -> CacheStatistics {
        CacheStatistics {
            max_size: self.max_size,
            current_entries: self.entries.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            hit_rate: self.hit_rate(),
            utilization: self.utilization(),
        }
    }

    /// Clear cache statistics
    #[inline]
    pub fn clear_statistics(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
    }

    /// Reset cache
    #[inline]
    pub fn reset(&self) {
        self.entries.store(0, Ordering::Relaxed);
        self.clear_statistics();
    }

    /// Check if cache performance is good
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.hit_rate() > 0.8 && self.utilization() < 0.9
    }
}

impl Default for PerformanceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub max_size: usize,
    pub current_entries: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
    pub utilization: f64,
}

impl CacheStatistics {
    /// Get total requests
    #[inline]
    pub fn total_requests(&self) -> usize {
        self.hits + self.misses
    }

    /// Check if statistics are healthy
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.hit_rate > 0.7 && self.utilization < 0.95
    }

    /// Get performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let hit_rate_score = self.hit_rate;
        let utilization_score = 1.0 - self.utilization; // Lower utilization is better
        
        (hit_rate_score + utilization_score) / 2.0
    }
}

/// Performance trend tracking
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    /// Recent performance scores
    recent_scores: Vec<f64>,
    /// Maximum history to keep
    max_history: usize,
}

impl PerformanceTrend {
    /// Create new performance trend tracker
    #[inline]
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_scores: Vec::with_capacity(max_history),
            max_history,
        }
    }

    /// Add performance score
    #[inline]
    pub fn add_score(&mut self, score: f64) {
        self.recent_scores.push(score);
        
        if self.recent_scores.len() > self.max_history {
            self.recent_scores.remove(0);
        }
    }

    /// Get trend direction
    #[inline]
    pub fn get_trend(&self) -> TrendDirection {
        if self.recent_scores.len() < 3 {
            return TrendDirection::Unknown;
        }

        let mid_point = self.recent_scores.len() / 2;
        let first_half_avg: f64 = self.recent_scores[..mid_point].iter().sum::<f64>() / mid_point as f64;
        let second_half_avg: f64 = self.recent_scores[mid_point..].iter().sum::<f64>() / (self.recent_scores.len() - mid_point) as f64;

        let difference = second_half_avg - first_half_avg;

        if difference > 0.05 {
            TrendDirection::Improving
        } else if difference < -0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Get average score
    #[inline]
    pub fn average_score(&self) -> f64 {
        if self.recent_scores.is_empty() {
            return 0.0;
        }

        self.recent_scores.iter().sum::<f64>() / self.recent_scores.len() as f64
    }

    /// Get latest score
    #[inline]
    pub fn latest_score(&self) -> Option<f64> {
        self.recent_scores.last().copied()
    }
}

impl Default for PerformanceTrend {
    fn default() -> Self {
        Self::new(10)
    }
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}

impl TrendDirection {
    /// Get trend description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "Performance is improving",
            TrendDirection::Stable => "Performance is stable",
            TrendDirection::Declining => "Performance is declining",
            TrendDirection::Unknown => "Trend is unknown",
        }
    }

    /// Check if trend is positive
    #[inline]
    pub fn is_positive(&self) -> bool {
        matches!(self, TrendDirection::Improving | TrendDirection::Stable)
    }
}

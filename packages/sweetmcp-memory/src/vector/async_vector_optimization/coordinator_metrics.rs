//! Coordination metrics and performance tracking
//!
//! This module provides blazing-fast metrics collection with zero allocation
//! optimizations for async vector optimization coordination.

use smallvec::SmallVec;
use std::time::{Duration, Instant};
use tracing::debug;

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

    /// Get detailed metrics summary
    #[inline]
    pub fn detailed_summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_search_operations: self.total_search_operations,
            total_optimization_operations: self.total_optimization_operations,
            success_rate: self.success_rate(),
            average_search_time: self.average_search_time(),
            average_optimization_time: self.average_optimization_time(),
            throughput: self.throughput(),
            is_healthy: self.is_healthy(),
            memory_usage_bytes: self.memory_usage(),
        }
    }
}

impl Default for CoordinationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Performance trend enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
}

impl PerformanceTrend {
    /// Get trend description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Performance is improving",
            PerformanceTrend::Stable => "Performance is stable",
            PerformanceTrend::Declining => "Performance is declining",
        }
    }

    /// Check if trend is positive
    #[inline]
    pub fn is_positive(&self) -> bool {
        matches!(self, PerformanceTrend::Improving | PerformanceTrend::Stable)
    }
}

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
    pub performance_trend: PerformanceTrend,
    pub memory_usage_bytes: usize,
}

impl MetricsSummary {
    /// Check if summary indicates excellent performance
    #[inline]
    pub fn is_excellent(&self) -> bool {
        self.success_rate > 0.95 &&
        self.average_search_time < Duration::from_millis(50) &&
        self.average_optimization_time < Duration::from_secs(2) &&
        matches!(self.performance_trend, PerformanceTrend::Improving)
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn performance_score(&self) -> f64 {
        let success_score = self.success_rate;
        let speed_score = if self.average_search_time < Duration::from_millis(100) { 1.0 } else { 0.7 };
        let trend_score = match self.performance_trend {
            PerformanceTrend::Improving => 1.0,
            PerformanceTrend::Stable => 0.8,
            PerformanceTrend::Declining => 0.4,
        };
        let health_score = if self.is_healthy { 1.0 } else { 0.5 };

        (success_score + speed_score + trend_score + health_score) / 4.0
    }
}
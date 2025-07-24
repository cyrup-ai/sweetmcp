//! Execution metrics and performance tracking
//!
//! This module provides blazing-fast metrics collection with zero allocation
//! optimizations and elegant ergonomic interfaces for performance monitoring.

use std::time::{Duration, SystemTime};

/// Execution metrics for optimization operations
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// Total executions performed
    pub total_executions: usize,
    /// Total improvement achieved
    pub total_improvement: f64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average improvement per execution
    pub average_improvement: f64,
    /// Average execution time per operation
    pub average_execution_time: Duration,
    /// Success rate
    pub success_rate: f64,
    /// Last execution timestamp
    pub last_execution: Option<SystemTime>,
    /// Execution history (limited to recent entries)
    execution_history: Vec<ExecutionRecord>,
}

impl ExecutionMetrics {
    /// Create new execution metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_executions: 0,
            total_improvement: 0.0,
            total_execution_time: Duration::from_secs(0),
            average_improvement: 0.0,
            average_execution_time: Duration::from_secs(0),
            success_rate: 0.0,
            last_execution: None,
            execution_history: Vec::with_capacity(100), // Keep last 100 executions
        }
    }

    /// Record execution results
    #[inline]
    pub fn record_execution(&mut self, operations_count: usize, improvement: f64, execution_time: Duration) {
        self.total_executions += 1;
        self.total_improvement += improvement;
        self.total_execution_time += execution_time;
        self.last_execution = Some(SystemTime::now());

        // Update averages
        self.average_improvement = self.total_improvement / self.total_executions as f64;
        self.average_execution_time = self.total_execution_time / self.total_executions as u32;

        // Record execution for history
        let record = ExecutionRecord {
            timestamp: SystemTime::now(),
            operations_count,
            improvement,
            execution_time,
            success: true,
        };

        // Maintain history size limit
        if self.execution_history.len() >= 100 {
            self.execution_history.remove(0);
        }
        self.execution_history.push(record);

        // Update success rate
        self.update_success_rate();
    }

    /// Record failed execution
    #[inline]
    pub fn record_failure(&mut self, execution_time: Duration) {
        self.total_executions += 1;
        self.total_execution_time += execution_time;
        self.last_execution = Some(SystemTime::now());

        // Update averages
        self.average_execution_time = self.total_execution_time / self.total_executions as u32;

        // Record failure for history
        let record = ExecutionRecord {
            timestamp: SystemTime::now(),
            operations_count: 0,
            improvement: 0.0,
            execution_time,
            success: false,
        };

        // Maintain history size limit
        if self.execution_history.len() >= 100 {
            self.execution_history.remove(0);
        }
        self.execution_history.push(record);

        // Update success rate
        self.update_success_rate();
    }

    /// Update success rate calculation
    #[inline]
    fn update_success_rate(&mut self) {
        if self.execution_history.is_empty() {
            self.success_rate = 0.0;
            return;
        }

        let successful_executions = self.execution_history.iter()
            .filter(|r| r.success)
            .count();
        self.success_rate = successful_executions as f64 / self.execution_history.len() as f64;
    }

    /// Reset all metrics
    #[inline]
    pub fn reset(&mut self) {
        self.total_executions = 0;
        self.total_improvement = 0.0;
        self.total_execution_time = Duration::from_secs(0);
        self.average_improvement = 0.0;
        self.average_execution_time = Duration::from_secs(0);
        self.success_rate = 0.0;
        self.last_execution = None;
        self.execution_history.clear();
    }

    /// Get recent execution history
    #[inline]
    pub fn recent_history(&self, count: usize) -> &[ExecutionRecord] {
        let start = self.execution_history.len().saturating_sub(count);
        &self.execution_history[start..]
    }

    /// Get all execution history
    #[inline]
    pub fn execution_history(&self) -> &[ExecutionRecord] {
        &self.execution_history
    }

    /// Get performance trend
    #[inline]
    pub fn performance_trend(&self) -> PerformanceTrend {
        if self.execution_history.len() < 2 {
            return PerformanceTrend::Stable;
        }

        let recent_count = (self.execution_history.len() / 2).max(1);
        let recent_avg = self.execution_history.iter()
            .rev()
            .take(recent_count)
            .map(|r| r.improvement)
            .sum::<f64>() / recent_count as f64;

        let older_avg = self.execution_history.iter()
            .take(self.execution_history.len() - recent_count)
            .map(|r| r.improvement)
            .sum::<f64>() / (self.execution_history.len() - recent_count) as f64;

        let improvement_ratio = recent_avg / older_avg.max(0.001);

        if improvement_ratio > 1.1 {
            PerformanceTrend::Improving
        } else if improvement_ratio < 0.9 {
            PerformanceTrend::Declining
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Get throughput (executions per second)
    #[inline]
    pub fn throughput(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 || self.total_executions == 0 {
            return 0.0;
        }
        
        self.total_executions as f64 / self.total_execution_time.as_secs_f64()
    }

    /// Get efficiency score (improvement per second)
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.total_improvement / self.total_execution_time.as_secs_f64()
    }

    /// Check if metrics indicate good performance
    #[inline]
    pub fn is_performance_good(&self) -> bool {
        self.success_rate > 0.8 &&
        self.average_improvement > 1.0 &&
        matches!(self.performance_trend(), PerformanceTrend::Improving | PerformanceTrend::Stable)
    }

    /// Get performance summary
    #[inline]
    pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_executions: self.total_executions,
            success_rate: self.success_rate,
            average_improvement: self.average_improvement,
            throughput: self.throughput(),
            efficiency_score: self.efficiency_score(),
            trend: self.performance_trend(),
            last_execution: self.last_execution,
        }
    }

    /// Get recent performance statistics
    #[inline]
    pub fn recent_performance_stats(&self, count: usize) -> RecentPerformanceStats {
        let recent = self.recent_history(count);
        
        if recent.is_empty() {
            return RecentPerformanceStats::default();
        }

        let successful_count = recent.iter().filter(|r| r.success).count();
        let total_improvement: f64 = recent.iter().map(|r| r.improvement).sum();
        let total_time: Duration = recent.iter().map(|r| r.execution_time).sum();
        let average_improvement = total_improvement / recent.len() as f64;
        let average_time = total_time / recent.len() as u32;

        RecentPerformanceStats {
            executions: recent.len(),
            success_rate: successful_count as f64 / recent.len() as f64,
            average_improvement,
            average_execution_time: average_time,
            total_improvement,
            total_execution_time: total_time,
        }
    }

    /// Check if recent performance is declining
    #[inline]
    pub fn is_recent_performance_declining(&self, window_size: usize) -> bool {
        if self.execution_history.len() < window_size * 2 {
            return false;
        }

        let recent_stats = self.recent_performance_stats(window_size);
        let older_start = self.execution_history.len() - window_size * 2;
        let older_end = self.execution_history.len() - window_size;
        let older_records = &self.execution_history[older_start..older_end];

        if older_records.is_empty() {
            return false;
        }

        let older_avg_improvement: f64 = older_records.iter().map(|r| r.improvement).sum::<f64>() / older_records.len() as f64;

        recent_stats.average_improvement < older_avg_improvement * 0.9
    }
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual execution record
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub timestamp: SystemTime,
    pub operations_count: usize,
    pub improvement: f64,
    pub execution_time: Duration,
    pub success: bool,
}

impl ExecutionRecord {
    /// Create new execution record
    #[inline]
    pub fn new(operations_count: usize, improvement: f64, execution_time: Duration, success: bool) -> Self {
        Self {
            timestamp: SystemTime::now(),
            operations_count,
            improvement,
            execution_time,
            success,
        }
    }

    /// Get age of this record
    #[inline]
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed().unwrap_or(Duration::from_secs(0))
    }

    /// Check if record is recent (within last hour)
    #[inline]
    pub fn is_recent(&self) -> bool {
        self.age() < Duration::from_secs(3600)
    }

    /// Get efficiency score for this execution
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.improvement / self.execution_time.as_secs_f64()
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
            PerformanceTrend::Improving => "Performance is improving over time",
            PerformanceTrend::Stable => "Performance is stable",
            PerformanceTrend::Declining => "Performance is declining over time",
        }
    }

    /// Check if trend is positive
    #[inline]
    pub fn is_positive(&self) -> bool {
        matches!(self, PerformanceTrend::Improving | PerformanceTrend::Stable)
    }

    /// Get trend score (0.0-1.0)
    #[inline]
    pub fn score(&self) -> f64 {
        match self {
            PerformanceTrend::Improving => 1.0,
            PerformanceTrend::Stable => 0.8,
            PerformanceTrend::Declining => 0.4,
        }
    }
}

/// Performance summary structure
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_executions: usize,
    pub success_rate: f64,
    pub average_improvement: f64,
    pub throughput: f64,
    pub efficiency_score: f64,
    pub trend: PerformanceTrend,
    pub last_execution: Option<SystemTime>,
}

impl PerformanceSummary {
    /// Check if performance is excellent
    #[inline]
    pub fn is_excellent(&self) -> bool {
        self.success_rate > 0.95 &&
        self.average_improvement > 5.0 &&
        self.efficiency_score > 2.0 &&
        matches!(self.trend, PerformanceTrend::Improving)
    }

    /// Check if performance is good
    #[inline]
    pub fn is_good(&self) -> bool {
        self.success_rate > 0.8 &&
        self.average_improvement > 1.0 &&
        self.efficiency_score > 0.5 &&
        self.trend.is_positive()
    }

    /// Check if performance is poor
    #[inline]
    pub fn is_poor(&self) -> bool {
        self.success_rate < 0.5 ||
        self.average_improvement < 0.1 ||
        matches!(self.trend, PerformanceTrend::Declining)
    }

    /// Get overall performance score (0.0-1.0)
    #[inline]
    pub fn overall_score(&self) -> f64 {
        let success_score = self.success_rate;
        let improvement_score = (self.average_improvement / 10.0).min(1.0);
        let efficiency_score = (self.efficiency_score / 5.0).min(1.0);
        let trend_score = self.trend.score();
        
        (success_score + improvement_score + efficiency_score + trend_score) / 4.0
    }
}

/// Recent performance statistics
#[derive(Debug, Clone)]
pub struct RecentPerformanceStats {
    pub executions: usize,
    pub success_rate: f64,
    pub average_improvement: f64,
    pub average_execution_time: Duration,
    pub total_improvement: f64,
    pub total_execution_time: Duration,
}

impl Default for RecentPerformanceStats {
    fn default() -> Self {
        Self {
            executions: 0,
            success_rate: 0.0,
            average_improvement: 0.0,
            average_execution_time: Duration::from_secs(0),
            total_improvement: 0.0,
            total_execution_time: Duration::from_secs(0),
        }
    }
}

impl RecentPerformanceStats {
    /// Check if recent performance is good
    #[inline]
    pub fn is_good(&self) -> bool {
        self.success_rate > 0.8 && self.average_improvement > 1.0
    }

    /// Get throughput for recent executions
    #[inline]
    pub fn throughput(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 || self.executions == 0 {
            return 0.0;
        }
        
        self.executions as f64 / self.total_execution_time.as_secs_f64()
    }

    /// Get efficiency score for recent executions
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.total_execution_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        
        self.total_improvement / self.total_execution_time.as_secs_f64()
    }
}

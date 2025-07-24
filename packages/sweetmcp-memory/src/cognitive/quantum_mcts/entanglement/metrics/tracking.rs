//! Performance tracking and timing utilities for quantum entanglement metrics
//!
//! This module provides blazing-fast timing utilities with zero-allocation
//! patterns and comprehensive performance tracking capabilities.

use std::time::{Duration, Instant};
use super::counters::EntanglementCounters;

/// High-precision performance tracker for timing operations
#[derive(Debug)]
pub struct PerformanceTracker {
    /// Start time of the operation
    start_time: Instant,
    /// Optional operation name for debugging
    operation_name: Option<String>,
}

impl PerformanceTracker {
    /// Start timing an operation
    #[inline]
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: None,
        }
    }
    
    /// Start timing a named operation
    #[inline]
    pub fn start_named(operation_name: String) -> Self {
        Self {
            start_time: Instant::now(),
            operation_name: Some(operation_name),
        }
    }
    
    /// Get elapsed time since start
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get elapsed time in microseconds
    #[inline]
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed().as_micros() as u64
    }
    
    /// Get elapsed time in nanoseconds
    #[inline]
    pub fn elapsed_ns(&self) -> u128 {
        self.elapsed().as_nanos()
    }
    
    /// Record operation and stop tracking
    pub fn record_and_stop(self, metrics: &EntanglementCounters) -> Duration {
        let elapsed = self.elapsed();
        metrics.record_entanglement_operation(elapsed);
        elapsed
    }
    
    /// Record operation with custom duration and stop tracking
    pub fn record_custom_and_stop(self, metrics: &EntanglementCounters, duration: Duration) -> Duration {
        metrics.record_entanglement_operation(duration);
        duration
    }
    
    /// Get operation name if set
    pub fn operation_name(&self) -> Option<&str> {
        self.operation_name.as_deref()
    }
    
    /// Check if operation is taking too long (over threshold)
    pub fn is_slow(&self, threshold_us: u64) -> bool {
        self.elapsed_us() > threshold_us
    }
    
    /// Get performance category based on elapsed time
    pub fn performance_category(&self) -> PerformanceCategory {
        let elapsed_us = self.elapsed_us();
        
        match elapsed_us {
            t if t <= 100 => PerformanceCategory::Excellent,
            t if t <= 500 => PerformanceCategory::Good,
            t if t <= 1000 => PerformanceCategory::Acceptable,
            t if t <= 2000 => PerformanceCategory::Slow,
            _ => PerformanceCategory::VerySlow,
        }
    }
}

/// Performance categories based on operation timing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceCategory {
    /// Excellent performance (≤100μs)
    Excellent,
    /// Good performance (≤500μs)
    Good,
    /// Acceptable performance (≤1000μs)
    Acceptable,
    /// Slow performance (≤2000μs)
    Slow,
    /// Very slow performance (>2000μs)
    VerySlow,
}

impl PerformanceCategory {
    /// Get performance score (0.0 to 1.0)
    pub fn score(&self) -> f64 {
        match self {
            PerformanceCategory::Excellent => 1.0,
            PerformanceCategory::Good => 0.8,
            PerformanceCategory::Acceptable => 0.6,
            PerformanceCategory::Slow => 0.4,
            PerformanceCategory::VerySlow => 0.2,
        }
    }
    
    /// Get performance grade (A-F)
    pub fn grade(&self) -> char {
        match self {
            PerformanceCategory::Excellent => 'A',
            PerformanceCategory::Good => 'B',
            PerformanceCategory::Acceptable => 'C',
            PerformanceCategory::Slow => 'D',
            PerformanceCategory::VerySlow => 'F',
        }
    }
    
    /// Get description string
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceCategory::Excellent => "Excellent",
            PerformanceCategory::Good => "Good",
            PerformanceCategory::Acceptable => "Acceptable",
            PerformanceCategory::Slow => "Slow",
            PerformanceCategory::VerySlow => "Very Slow",
        }
    }
}

/// Batch performance tracker for multiple operations
#[derive(Debug)]
pub struct BatchPerformanceTracker {
    /// Start time of the batch
    start_time: Instant,
    /// Number of operations in the batch
    operation_count: u64,
    /// Total time spent in individual operations
    total_operation_time: Duration,
    /// Minimum operation time observed
    min_operation_time: Option<Duration>,
    /// Maximum operation time observed
    max_operation_time: Option<Duration>,
    /// Batch name for identification
    batch_name: Option<String>,
}

impl BatchPerformanceTracker {
    /// Create new batch tracker
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: 0,
            total_operation_time: Duration::from_nanos(0),
            min_operation_time: None,
            max_operation_time: None,
            batch_name: None,
        }
    }
    
    /// Create new named batch tracker
    pub fn new_named(batch_name: String) -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: 0,
            total_operation_time: Duration::from_nanos(0),
            min_operation_time: None,
            max_operation_time: None,
            batch_name: Some(batch_name),
        }
    }
    
    /// Record an operation duration
    pub fn record_operation(&mut self, duration: Duration) {
        self.operation_count += 1;
        self.total_operation_time += duration;
        
        // Update min/max
        match self.min_operation_time {
            None => self.min_operation_time = Some(duration),
            Some(min) if duration < min => self.min_operation_time = Some(duration),
            _ => {}
        }
        
        match self.max_operation_time {
            None => self.max_operation_time = Some(duration),
            Some(max) if duration > max => self.max_operation_time = Some(duration),
            _ => {}
        }
    }
    
    /// Record multiple operations with total duration
    pub fn record_operations(&mut self, count: u64, total_duration: Duration) {
        self.operation_count += count;
        self.total_operation_time += total_duration;
        
        if count > 0 {
            let avg_duration = Duration::from_nanos(total_duration.as_nanos() / count as u128);
            
            // Update min/max with average (approximation for batch operations)
            match self.min_operation_time {
                None => self.min_operation_time = Some(avg_duration),
                Some(min) if avg_duration < min => self.min_operation_time = Some(avg_duration),
                _ => {}
            }
            
            match self.max_operation_time {
                None => self.max_operation_time = Some(avg_duration),
                Some(max) if avg_duration > max => self.max_operation_time = Some(avg_duration),
                _ => {}
            }
        }
    }
    
    /// Get batch statistics
    pub fn statistics(&self) -> BatchStatistics {
        let batch_duration = self.start_time.elapsed();
        let average_operation_time = if self.operation_count > 0 {
            Duration::from_nanos(self.total_operation_time.as_nanos() / self.operation_count as u128)
        } else {
            Duration::from_nanos(0)
        };
        
        let operations_per_second = if batch_duration.as_secs_f64() > 0.0 {
            self.operation_count as f64 / batch_duration.as_secs_f64()
        } else {
            0.0
        };
        
        BatchStatistics {
            batch_name: self.batch_name.clone(),
            operation_count: self.operation_count,
            batch_duration,
            total_operation_time: self.total_operation_time,
            average_operation_time,
            min_operation_time: self.min_operation_time.unwrap_or(Duration::from_nanos(0)),
            max_operation_time: self.max_operation_time.unwrap_or(Duration::from_nanos(0)),
            operations_per_second,
        }
    }
    
    /// Finalize batch and record to metrics
    pub fn finalize(self, metrics: &EntanglementCounters) -> BatchStatistics {
        let stats = self.statistics();
        
        // Record batch operations to metrics
        if self.operation_count > 0 {
            metrics.record_entanglement_operations(self.operation_count, self.total_operation_time);
        }
        
        stats
    }
    
    /// Get current operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }
    
    /// Get elapsed batch time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Check if batch is taking too long
    pub fn is_slow(&self, threshold_ms: u64) -> bool {
        self.elapsed().as_millis() > threshold_ms as u128
    }
}

impl Default for BatchPerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a batch of operations
#[derive(Debug, Clone)]
pub struct BatchStatistics {
    /// Optional batch name
    pub batch_name: Option<String>,
    /// Number of operations in the batch
    pub operation_count: u64,
    /// Total time for the entire batch
    pub batch_duration: Duration,
    /// Total time spent in individual operations
    pub total_operation_time: Duration,
    /// Average time per operation
    pub average_operation_time: Duration,
    /// Minimum operation time
    pub min_operation_time: Duration,
    /// Maximum operation time
    pub max_operation_time: Duration,
    /// Operations per second throughput
    pub operations_per_second: f64,
}

impl BatchStatistics {
    /// Get efficiency ratio (operation time / batch time)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.batch_duration.as_nanos() > 0 {
            self.total_operation_time.as_nanos() as f64 / self.batch_duration.as_nanos() as f64
        } else {
            0.0
        }
    }
    
    /// Get overhead ratio (1.0 - efficiency_ratio)
    pub fn overhead_ratio(&self) -> f64 {
        1.0 - self.efficiency_ratio()
    }
    
    /// Get performance category for the batch
    pub fn performance_category(&self) -> PerformanceCategory {
        let avg_us = self.average_operation_time.as_micros() as u64;
        
        match avg_us {
            t if t <= 100 => PerformanceCategory::Excellent,
            t if t <= 500 => PerformanceCategory::Good,
            t if t <= 1000 => PerformanceCategory::Acceptable,
            t if t <= 2000 => PerformanceCategory::Slow,
            _ => PerformanceCategory::VerySlow,
        }
    }
    
    /// Check if batch performance is acceptable
    pub fn is_acceptable_performance(&self) -> bool {
        matches!(
            self.performance_category(),
            PerformanceCategory::Excellent | PerformanceCategory::Good | PerformanceCategory::Acceptable
        )
    }
    
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        let name = self.batch_name.as_deref().unwrap_or("Batch");
        format!(
            "{}: {} ops in {:.1}ms (avg: {:.1}μs, {:.1} ops/sec, {})",
            name,
            self.operation_count,
            self.batch_duration.as_secs_f64() * 1000.0,
            self.average_operation_time.as_micros(),
            self.operations_per_second,
            self.performance_category().description()
        )
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        let name = self.batch_name.as_deref().unwrap_or("Batch");
        format!(
            "=== {} Performance Report ===\n\
            Operations: {}\n\
            Batch Duration: {:.3}ms\n\
            Total Operation Time: {:.3}ms\n\
            Average Operation Time: {:.1}μs\n\
            Min Operation Time: {:.1}μs\n\
            Max Operation Time: {:.1}μs\n\
            Throughput: {:.1} ops/sec\n\
            Efficiency: {:.1}%\n\
            Overhead: {:.1}%\n\
            Performance: {} (Grade: {})",
            name,
            self.operation_count,
            self.batch_duration.as_secs_f64() * 1000.0,
            self.total_operation_time.as_secs_f64() * 1000.0,
            self.average_operation_time.as_micros(),
            self.min_operation_time.as_micros(),
            self.max_operation_time.as_micros(),
            self.operations_per_second,
            self.efficiency_ratio() * 100.0,
            self.overhead_ratio() * 100.0,
            self.performance_category().description(),
            self.performance_category().grade()
        )
    }
}

/// Timing utilities for common operations
pub struct TimingUtils;

impl TimingUtils {
    /// Time a closure and return both result and duration
    pub fn time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }
    
    /// Time a closure and record to metrics
    pub fn time_and_record<F, R>(f: F, metrics: &EntanglementCounters) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = Self::time(f);
        metrics.record_entanglement_operation(duration);
        (result, duration)
    }
    
    /// Time an async closure and return both result and duration
    pub async fn time_async<F, Fut, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }
    
    /// Time an async closure and record to metrics
    pub async fn time_async_and_record<F, Fut, R>(f: F, metrics: &EntanglementCounters) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let (result, duration) = Self::time_async(f).await;
        metrics.record_entanglement_operation(duration);
        (result, duration)
    }
    
    /// Convert duration to human-readable string
    pub fn format_duration(duration: Duration) -> String {
        let nanos = duration.as_nanos();
        
        if nanos < 1_000 {
            format!("{}ns", nanos)
        } else if nanos < 1_000_000 {
            format!("{:.1}μs", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.1}ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.1}s", nanos as f64 / 1_000_000_000.0)
        }
    }
    
    /// Get performance category from duration
    pub fn categorize_duration(duration: Duration) -> PerformanceCategory {
        let us = duration.as_micros() as u64;
        
        match us {
            t if t <= 100 => PerformanceCategory::Excellent,
            t if t <= 500 => PerformanceCategory::Good,
            t if t <= 1000 => PerformanceCategory::Acceptable,
            t if t <= 2000 => PerformanceCategory::Slow,
            _ => PerformanceCategory::VerySlow,
        }
    }
    
    /// Calculate percentile from a sorted list of durations
    pub fn calculate_percentile(sorted_durations: &[Duration], percentile: f64) -> Duration {
        if sorted_durations.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let index = ((sorted_durations.len() as f64 - 1.0) * percentile / 100.0) as usize;
        sorted_durations[index.min(sorted_durations.len() - 1)]
    }
    
    /// Calculate median from a sorted list of durations
    pub fn calculate_median(sorted_durations: &[Duration]) -> Duration {
        Self::calculate_percentile(sorted_durations, 50.0)
    }
    
    /// Calculate 95th percentile from a sorted list of durations
    pub fn calculate_p95(sorted_durations: &[Duration]) -> Duration {
        Self::calculate_percentile(sorted_durations, 95.0)
    }
    
    /// Calculate 99th percentile from a sorted list of durations
    pub fn calculate_p99(sorted_durations: &[Duration]) -> Duration {
        Self::calculate_percentile(sorted_durations, 99.0)
    }
}

/// Specialized tracker for influence calculations
#[derive(Debug)]
pub struct InfluenceTracker {
    /// Start time of the influence calculation
    start_time: Instant,
    /// Number of calculations performed
    calculation_count: u64,
}

impl InfluenceTracker {
    /// Start tracking influence calculations
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            calculation_count: 0,
        }
    }
    
    /// Record a single influence calculation
    pub fn record_calculation(&mut self) {
        self.calculation_count += 1;
    }
    
    /// Record multiple influence calculations
    pub fn record_calculations(&mut self, count: u64) {
        self.calculation_count += count;
    }
    
    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get calculation count
    pub fn calculation_count(&self) -> u64 {
        self.calculation_count
    }
    
    /// Finalize tracking and record to metrics
    pub fn finalize(self, metrics: &EntanglementCounters) -> InfluenceStatistics {
        let elapsed = self.elapsed();
        
        if self.calculation_count > 0 {
            metrics.record_influence_calculations(self.calculation_count, elapsed);
        }
        
        InfluenceStatistics {
            calculation_count: self.calculation_count,
            total_time: elapsed,
            average_time: if self.calculation_count > 0 {
                Duration::from_nanos(elapsed.as_nanos() / self.calculation_count as u128)
            } else {
                Duration::from_nanos(0)
            },
            calculations_per_second: if elapsed.as_secs_f64() > 0.0 {
                self.calculation_count as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            },
        }
    }
}

/// Statistics for influence calculations
#[derive(Debug, Clone)]
pub struct InfluenceStatistics {
    /// Number of calculations performed
    pub calculation_count: u64,
    /// Total time spent
    pub total_time: Duration,
    /// Average time per calculation
    pub average_time: Duration,
    /// Calculations per second
    pub calculations_per_second: f64,
}

impl InfluenceStatistics {
    /// Get performance category
    pub fn performance_category(&self) -> PerformanceCategory {
        TimingUtils::categorize_duration(self.average_time)
    }
    
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Influence: {} calcs in {} (avg: {}, {:.1}/sec, {})",
            self.calculation_count,
            TimingUtils::format_duration(self.total_time),
            TimingUtils::format_duration(self.average_time),
            self.calculations_per_second,
            self.performance_category().description()
        )
    }
}
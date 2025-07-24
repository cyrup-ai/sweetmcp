//! Core benchmarking utilities for quantum entanglement
//!
//! This module provides the fundamental benchmarking structures and operations
//! with zero-allocation patterns and blazing-fast performance.

use std::time::{Duration, Instant};
use super::tracking::{PerformanceCategory, TimingUtils};

/// Comprehensive benchmarking utility for entanglement operations
#[derive(Debug)]
pub struct EntanglementBenchmark {
    /// Name of the benchmark
    benchmark_name: String,
    /// Start time of the benchmark
    start_time: Instant,
    /// Individual operation samples
    samples: Vec<Duration>,
    /// Maximum number of samples to retain
    max_samples: usize,
    /// Total operations performed
    total_operations: u64,
    /// Total time across all operations
    total_time: Duration,
    /// Minimum duration observed
    min_duration: Option<Duration>,
    /// Maximum duration observed
    max_duration: Option<Duration>,
}

impl EntanglementBenchmark {
    /// Create new benchmark with default sample limit
    pub fn new(benchmark_name: String) -> Self {
        Self::with_sample_limit(benchmark_name, 10000)
    }
    
    /// Create new benchmark with custom sample limit
    pub fn with_sample_limit(benchmark_name: String, max_samples: usize) -> Self {
        Self {
            benchmark_name,
            start_time: Instant::now(),
            samples: Vec::with_capacity(max_samples.min(1000)),
            max_samples,
            total_operations: 0,
            total_time: Duration::from_nanos(0),
            min_duration: None,
            max_duration: None,
        }
    }
    
    /// Record a single operation duration
    pub fn record_operation(&mut self, duration: Duration) {
        self.total_operations += 1;
        self.total_time += duration;
        
        // Update min/max
        self.min_duration = Some(
            self.min_duration.map_or(duration, |min| min.min(duration))
        );
        self.max_duration = Some(
            self.max_duration.map_or(duration, |max| max.max(duration))
        );
        
        // Store sample if we have capacity
        if self.samples.len() < self.max_samples {
            self.samples.push(duration);
        } else {
            // Replace oldest sample (simple circular buffer)
            let index = (self.total_operations as usize - 1) % self.max_samples;
            self.samples[index] = duration;
        }
    }
    
    /// Record operation with timing closure
    pub fn time_operation<F, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        self.record_operation(duration);
        result
    }
    
    /// Record async operation with timing
    pub async fn time_async_operation<F, Fut, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        self.record_operation(duration);
        result
    }
    
    /// Get current benchmark statistics
    pub fn current_stats(&self) -> BenchmarkStats {
        if self.total_operations == 0 {
            return BenchmarkStats::default();
        }
        
        let average_duration = Duration::from_nanos(
            self.total_time.as_nanos() as u64 / self.total_operations
        );
        
        let (median, percentile_95, percentile_99) = self.calculate_percentiles();
        
        BenchmarkStats {
            total_operations: self.total_operations,
            total_time: self.total_time,
            average_duration,
            min_duration: self.min_duration.unwrap_or_default(),
            max_duration: self.max_duration.unwrap_or_default(),
            median_duration: median,
            percentile_95: percentile_95,
            percentile_99: percentile_99,
            operations_per_second: self.calculate_ops_per_second(),
            sample_count: self.samples.len(),
        }
    }
    
    /// Calculate percentiles from samples
    fn calculate_percentiles(&self) -> (Duration, Duration, Duration) {
        if self.samples.is_empty() {
            return (Duration::default(), Duration::default(), Duration::default());
        }
        
        let mut sorted_samples = self.samples.clone();
        sorted_samples.sort();
        
        let len = sorted_samples.len();
        let median_idx = len / 2;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;
        
        (
            sorted_samples[median_idx],
            sorted_samples[p95_idx.min(len - 1)],
            sorted_samples[p99_idx.min(len - 1)],
        )
    }
    
    /// Calculate operations per second
    fn calculate_ops_per_second(&self) -> f64 {
        if self.total_time.is_zero() {
            return 0.0;
        }
        
        self.total_operations as f64 / self.total_time.as_secs_f64()
    }
    
    /// Reset benchmark statistics
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.samples.clear();
        self.total_operations = 0;
        self.total_time = Duration::from_nanos(0);
        self.min_duration = None;
        self.max_duration = None;
    }
    
    /// Get benchmark name
    #[inline]
    pub fn name(&self) -> &str {
        &self.benchmark_name
    }
    
    /// Get total elapsed time since benchmark creation
    pub fn total_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Check if benchmark has sufficient samples for analysis
    pub fn has_sufficient_samples(&self, min_samples: usize) -> bool {
        self.samples.len() >= min_samples && self.total_operations >= min_samples as u64
    }
    
    /// Get sample utilization ratio
    pub fn sample_utilization(&self) -> f64 {
        if self.max_samples == 0 {
            0.0
        } else {
            self.samples.len() as f64 / self.max_samples as f64
        }
    }
}

/// Benchmark statistics snapshot
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub total_operations: u64,
    pub total_time: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub median_duration: Duration,
    pub percentile_95: Duration,
    pub percentile_99: Duration,
    pub operations_per_second: f64,
    pub sample_count: usize,
}

impl Default for BenchmarkStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            total_time: Duration::default(),
            average_duration: Duration::default(),
            min_duration: Duration::default(),
            max_duration: Duration::default(),
            median_duration: Duration::default(),
            percentile_95: Duration::default(),
            percentile_99: Duration::default(),
            operations_per_second: 0.0,
            sample_count: 0,
        }
    }
}

impl BenchmarkStats {
    /// Get performance grade based on operations per second
    pub fn performance_grade(&self) -> char {
        if self.operations_per_second >= 10000.0 {
            'A'
        } else if self.operations_per_second >= 5000.0 {
            'B'
        } else if self.operations_per_second >= 1000.0 {
            'C'
        } else if self.operations_per_second >= 100.0 {
            'D'
        } else {
            'F'
        }
    }
    
    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self, min_ops_per_second: f64) -> bool {
        self.operations_per_second >= min_ops_per_second
    }
    
    /// Get coefficient of variation (std_dev / mean)
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.average_duration.is_zero() {
            return 0.0;
        }
        
        // Approximate using percentile spread
        let spread = self.percentile_95.saturating_sub(self.median_duration);
        spread.as_nanos() as f64 / self.average_duration.as_nanos() as f64
    }
    
    /// Check if performance is consistent
    pub fn is_consistent(&self, max_cv: f64) -> bool {
        self.coefficient_of_variation() <= max_cv
    }
    
    /// Get performance summary string
    pub fn summary(&self) -> String {
        format!(
            "Ops: {}, Avg: {:.2}ms, OPS: {:.1}, Grade: {}",
            self.total_operations,
            self.average_duration.as_secs_f64() * 1000.0,
            self.operations_per_second,
            self.performance_grade()
        )
    }
}
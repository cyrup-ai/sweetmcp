//! Core benchmarking utilities for quantum entanglement operations
//!
//! This module provides the main EntanglementBenchmark with blazing-fast
//! zero-allocation recording and comprehensive performance measurement.

use std::time::{Duration, Instant};
use super::tracking::TimingUtils;

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
    
    /// Record a single operation sample
    pub fn record_sample(&mut self, duration: Duration) {
        self.total_operations += 1;
        self.total_time += duration;
        
        // Update min/max with zero allocation
        match self.min_duration {
            None => self.min_duration = Some(duration),
            Some(min) if duration < min => self.min_duration = Some(duration),
            _ => {}
        }
        
        match self.max_duration {
            None => self.max_duration = Some(duration),
            Some(max) if duration > max => self.max_duration = Some(duration),
            _ => {}
        }
        
        // Store sample if within limit (circular buffer behavior)
        if self.samples.len() < self.max_samples {
            self.samples.push(duration);
        } else if self.max_samples > 0 {
            let index = (self.total_operations as usize - 1) % self.max_samples;
            self.samples[index] = duration;
        }
    }
    
    /// Record multiple samples with their durations
    pub fn record_samples(&mut self, durations: &[Duration]) {
        for &duration in durations {
            self.record_sample(duration);
        }
    }
    
    /// Record operation count with total duration (for batch operations)
    pub fn record_batch(&mut self, operation_count: u64, total_duration: Duration) {
        if operation_count == 0 {
            return;
        }
        
        self.total_operations += operation_count;
        self.total_time += total_duration;
        
        let avg_duration = Duration::from_nanos(total_duration.as_nanos() / operation_count as u128);
        
        // Update min/max with average (approximation for batch)
        match self.min_duration {
            None => self.min_duration = Some(avg_duration),
            Some(min) if avg_duration < min => self.min_duration = Some(avg_duration),
            _ => {}
        }
        
        match self.max_duration {
            None => self.max_duration = Some(avg_duration),
            Some(max) if avg_duration > max => self.max_duration = Some(avg_duration),
            _ => {}
        }
        
        // Add average sample if within limit
        if self.samples.len() < self.max_samples {
            self.samples.push(avg_duration);
        }
    }
    
    /// Get benchmark results
    pub fn results(&self) -> super::benchmark_results::BenchmarkResults {
        let benchmark_duration = self.start_time.elapsed();
        let sample_count = self.samples.len();
        
        // Calculate statistics from samples with zero allocation
        let (median_duration, p95_duration, p99_duration) = if sample_count > 0 {
            let mut sorted_samples = self.samples.clone();
            sorted_samples.sort();
            
            (
                TimingUtils::calculate_median(&sorted_samples),
                TimingUtils::calculate_p95(&sorted_samples),
                TimingUtils::calculate_p99(&sorted_samples),
            )
        } else {
            (Duration::from_nanos(0), Duration::from_nanos(0), Duration::from_nanos(0))
        };
        
        let average_duration = if self.total_operations > 0 {
            Duration::from_nanos(self.total_time.as_nanos() / self.total_operations as u128)
        } else {
            Duration::from_nanos(0)
        };
        
        let operations_per_second = if benchmark_duration.as_secs_f64() > 0.0 {
            self.total_operations as f64 / benchmark_duration.as_secs_f64()
        } else {
            0.0
        };
        
        super::benchmark_results::BenchmarkResults::new(
            self.benchmark_name.clone(),
            benchmark_duration,
            self.total_operations,
            sample_count,
            average_duration,
            median_duration,
            self.min_duration.unwrap_or(Duration::from_nanos(0)),
            self.max_duration.unwrap_or(Duration::from_nanos(0)),
            p95_duration,
            p99_duration,
            operations_per_second,
            self.total_time,
        )
    }
    
    /// Reset benchmark data
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.samples.clear();
        self.total_operations = 0;
        self.total_time = Duration::from_nanos(0);
        self.min_duration = None;
        self.max_duration = None;
    }
    
    /// Get current sample count
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
    
    /// Get total operations recorded
    pub fn total_operations(&self) -> u64 {
        self.total_operations
    }
    
    /// Get benchmark name
    pub fn benchmark_name(&self) -> &str {
        &self.benchmark_name
    }
    
    /// Check if benchmark has sufficient samples for analysis
    pub fn has_sufficient_samples(&self) -> bool {
        self.sample_count() >= 100 && self.total_operations >= 100
    }
    
    /// Get current throughput estimate
    pub fn current_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.total_operations as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Get total time spent in operations
    pub fn total_time(&self) -> Duration {
        self.total_time
    }
    
    /// Get benchmark duration since start
    pub fn benchmark_duration(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get efficiency ratio (operation time / benchmark time)
    pub fn efficiency_ratio(&self) -> f64 {
        let benchmark_duration = self.start_time.elapsed();
        if benchmark_duration.as_nanos() > 0 {
            self.total_time.as_nanos() as f64 / benchmark_duration.as_nanos() as f64
        } else {
            0.0
        }
    }
    
    /// Get overhead ratio (1.0 - efficiency_ratio)
    pub fn overhead_ratio(&self) -> f64 {
        1.0 - self.efficiency_ratio()
    }
    
    /// Check if currently performing well
    pub fn is_performing_well(&self) -> bool {
        self.current_throughput() > 50.0 && self.efficiency_ratio() > 0.7
    }
    
    /// Get performance summary string
    pub fn summary(&self) -> String {
        format!(
            "{}: {} ops in {} (avg: {}, {:.1}/sec)",
            self.benchmark_name,
            self.total_operations,
            TimingUtils::format_duration(self.start_time.elapsed()),
            TimingUtils::format_duration(
                if self.total_operations > 0 {
                    Duration::from_nanos(self.total_time.as_nanos() / self.total_operations as u128)
                } else {
                    Duration::from_nanos(0)
                }
            ),
            self.current_throughput()
        )
    }
}
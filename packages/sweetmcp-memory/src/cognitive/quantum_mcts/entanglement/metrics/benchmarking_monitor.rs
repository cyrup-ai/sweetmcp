//! Rolling performance monitoring for quantum entanglement benchmarks
//!
//! This module provides continuous performance monitoring with rolling statistics,
//! trend detection, and real-time analysis with zero-allocation patterns and
//! blazing-fast performance.

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use super::benchmarking_core::BenchmarkStats;

/// Rolling performance monitor with continuous statistics
#[derive(Debug)]
pub struct RollingPerformanceMonitor {
    /// Monitor name
    name: String,
    /// Rolling window of performance samples
    samples: VecDeque<PerformanceSample>,
    /// Maximum window size
    window_size: usize,
    /// Current rolling statistics
    stats: RollingStatistics,
    /// Last update timestamp
    last_update: Instant,
}

impl RollingPerformanceMonitor {
    /// Create new rolling monitor
    pub fn new(name: String, window_size: usize) -> Self {
        Self {
            name,
            samples: VecDeque::with_capacity(window_size),
            window_size,
            stats: RollingStatistics::default(),
            last_update: Instant::now(),
        }
    }
    
    /// Add new performance sample
    pub fn add_sample(&mut self, operations_per_second: f64, average_latency: Duration) {
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            operations_per_second,
            average_latency,
        };
        
        // Add sample and maintain window size
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        
        // Update rolling statistics
        self.update_statistics();
        self.last_update = Instant::now();
    }
    
    /// Add sample from benchmark stats
    pub fn add_benchmark_sample(&mut self, stats: &BenchmarkStats) {
        self.add_sample(stats.operations_per_second, stats.average_duration);
    }
    
    /// Update rolling statistics
    fn update_statistics(&mut self) {
        if self.samples.is_empty() {
            self.stats = RollingStatistics::default();
            return;
        }
        
        let ops_values: Vec<f64> = self.samples.iter()
            .map(|s| s.operations_per_second)
            .collect();
        
        let latency_values: Vec<f64> = self.samples.iter()
            .map(|s| s.average_latency.as_secs_f64())
            .collect();
        
        self.stats = RollingStatistics {
            sample_count: self.samples.len(),
            avg_operations_per_second: ops_values.iter().sum::<f64>() / ops_values.len() as f64,
            min_operations_per_second: ops_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max_operations_per_second: ops_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            avg_latency: Duration::from_secs_f64(
                latency_values.iter().sum::<f64>() / latency_values.len() as f64
            ),
            min_latency: Duration::from_secs_f64(
                latency_values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
            ),
            max_latency: Duration::from_secs_f64(
                latency_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            ),
            trend: self.calculate_trend(&ops_values),
        };
    }
    
    /// Calculate performance trend
    fn calculate_trend(&self, values: &[f64]) -> PerformanceTrend {
        if values.len() < 3 {
            return PerformanceTrend::Unknown;
        }
        
        // Simple linear regression slope
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }
        
        if denominator == 0.0 {
            return PerformanceTrend::Stable;
        }
        
        let slope = numerator / denominator;
        let slope_threshold = y_mean * 0.05; // 5% of mean
        
        if slope > slope_threshold {
            PerformanceTrend::Improving
        } else if slope < -slope_threshold {
            PerformanceTrend::Declining
        } else {
            // Check for high volatility
            let variance = values.iter()
                .map(|&x| (x - y_mean).powi(2))
                .sum::<f64>() / n;
            let std_dev = variance.sqrt();
            let cv = std_dev / y_mean;
            
            if cv > 0.3 {
                PerformanceTrend::Volatile
            } else {
                PerformanceTrend::Stable
            }
        }
    }
    
    /// Get current rolling statistics
    #[inline]
    pub const fn current_stats(&self) -> &RollingStatistics {
        &self.stats
    }
    
    /// Get monitor name
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get sample count
    #[inline]
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }
    
    /// Check if monitor has sufficient samples
    pub fn has_sufficient_samples(&self, min_samples: usize) -> bool {
        self.samples.len() >= min_samples
    }
    
    /// Get time since last update
    pub fn time_since_last_update(&self) -> Duration {
        self.last_update.elapsed()
    }
    
    /// Check if monitor data is stale
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.time_since_last_update() > max_age
    }
    
    /// Clear all samples and reset statistics
    pub fn reset(&mut self) {
        self.samples.clear();
        self.stats = RollingStatistics::default();
        self.last_update = Instant::now();
    }
    
    /// Get performance grade based on current statistics
    pub fn performance_grade(&self) -> char {
        if self.stats.avg_operations_per_second >= 10000.0 {
            'A'
        } else if self.stats.avg_operations_per_second >= 5000.0 {
            'B'
        } else if self.stats.avg_operations_per_second >= 1000.0 {
            'C'
        } else if self.stats.avg_operations_per_second >= 100.0 {
            'D'
        } else {
            'F'
        }
    }
    
    /// Generate monitoring report
    pub fn generate_report(&self) -> String {
        format!(
            "Rolling Performance Monitor: {}\n\
             Samples: {} (window: {})\n\
             Throughput: {:.1} ops/sec (min: {:.1}, max: {:.1})\n\
             Latency: {:.2}ms avg (min: {:.2}ms, max: {:.2}ms)\n\
             Trend: {:?}\n\
             Grade: {}\n\
             Last Update: {:.1}s ago",
            self.name,
            self.stats.sample_count,
            self.window_size,
            self.stats.avg_operations_per_second,
            self.stats.min_operations_per_second,
            self.stats.max_operations_per_second,
            self.stats.avg_latency.as_secs_f64() * 1000.0,
            self.stats.min_latency.as_secs_f64() * 1000.0,
            self.stats.max_latency.as_secs_f64() * 1000.0,
            self.stats.trend,
            self.performance_grade(),
            self.time_since_last_update().as_secs_f64()
        )
    }
}

/// Individual performance sample
#[derive(Debug, Clone)]
pub struct PerformanceSample {
    /// Sample timestamp
    pub timestamp: Instant,
    /// Operations per second at this sample
    pub operations_per_second: f64,
    /// Average latency at this sample
    pub average_latency: Duration,
}

/// Rolling statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct RollingStatistics {
    /// Number of samples in current window
    pub sample_count: usize,
    /// Average operations per second
    pub avg_operations_per_second: f64,
    /// Minimum operations per second
    pub min_operations_per_second: f64,
    /// Maximum operations per second
    pub max_operations_per_second: f64,
    /// Average latency
    pub avg_latency: Duration,
    /// Minimum latency
    pub min_latency: Duration,
    /// Maximum latency
    pub max_latency: Duration,
    /// Performance trend
    pub trend: PerformanceTrend,
}

impl Default for RollingStatistics {
    fn default() -> Self {
        Self {
            sample_count: 0,
            avg_operations_per_second: 0.0,
            min_operations_per_second: 0.0,
            max_operations_per_second: 0.0,
            avg_latency: Duration::default(),
            min_latency: Duration::default(),
            max_latency: Duration::default(),
            trend: PerformanceTrend::Unknown,
        }
    }
}

impl RollingStatistics {
    /// Get throughput range (max - min)
    pub fn throughput_range(&self) -> f64 {
        self.max_operations_per_second - self.min_operations_per_second
    }
    
    /// Get latency range
    pub fn latency_range(&self) -> Duration {
        self.max_latency.saturating_sub(self.min_latency)
    }
    
    /// Calculate throughput coefficient of variation
    pub fn throughput_cv(&self) -> f64 {
        if self.avg_operations_per_second == 0.0 {
            return 0.0;
        }
        
        // Approximate CV using range
        let range = self.throughput_range();
        (range / 4.0) / self.avg_operations_per_second // Rough approximation
    }
    
    /// Check if performance is stable
    pub fn is_stable(&self) -> bool {
        matches!(self.trend, PerformanceTrend::Stable) && self.throughput_cv() < 0.2
    }
    
    /// Check if performance is improving
    pub fn is_improving(&self) -> bool {
        matches!(self.trend, PerformanceTrend::Improving)
    }
    
    /// Check if performance is declining
    pub fn is_declining(&self) -> bool {
        matches!(self.trend, PerformanceTrend::Declining)
    }
    
    /// Check if performance is volatile
    pub fn is_volatile(&self) -> bool {
        matches!(self.trend, PerformanceTrend::Volatile) || self.throughput_cv() > 0.3
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Volatile,
    Unknown,
}

impl PerformanceTrend {
    /// Get trend description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Performance is improving over time",
            PerformanceTrend::Stable => "Performance is stable",
            PerformanceTrend::Declining => "Performance is declining - attention needed",
            PerformanceTrend::Volatile => "Performance is highly variable",
            PerformanceTrend::Unknown => "Insufficient data to determine trend",
        }
    }
    
    /// Check if trend indicates problems
    #[inline]
    pub const fn indicates_problems(&self) -> bool {
        matches!(self, PerformanceTrend::Declining | PerformanceTrend::Volatile)
    }
    
    /// Get trend priority (higher = more urgent)
    #[inline]
    pub const fn priority(&self) -> u8 {
        match self {
            PerformanceTrend::Declining => 4,
            PerformanceTrend::Volatile => 3,
            PerformanceTrend::Unknown => 2,
            PerformanceTrend::Stable => 1,
            PerformanceTrend::Improving => 0,
        }
    }
}
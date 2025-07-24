//! Benchmark results and comparison utilities
//!
//! This module provides comprehensive benchmark result analysis with blazing-fast
//! zero-allocation performance assessment and comparison capabilities.

use std::time::Duration;
use super::tracking::{PerformanceCategory, TimingUtils};

/// Comprehensive benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Name of the benchmark
    pub benchmark_name: String,
    /// Total duration of the benchmark
    pub benchmark_duration: Duration,
    /// Total number of operations performed
    pub total_operations: u64,
    /// Number of individual samples collected
    pub sample_count: usize,
    /// Average operation duration
    pub average_duration: Duration,
    /// Median operation duration
    pub median_duration: Duration,
    /// Minimum operation duration
    pub min_duration: Duration,
    /// Maximum operation duration
    pub max_duration: Duration,
    /// 95th percentile duration
    pub p95_duration: Duration,
    /// 99th percentile duration
    pub p99_duration: Duration,
    /// Operations per second throughput
    pub operations_per_second: f64,
    /// Total time spent in operations
    pub total_time: Duration,
}

impl BenchmarkResults {
    /// Create new benchmark results
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        benchmark_name: String,
        benchmark_duration: Duration,
        total_operations: u64,
        sample_count: usize,
        average_duration: Duration,
        median_duration: Duration,
        min_duration: Duration,
        max_duration: Duration,
        p95_duration: Duration,
        p99_duration: Duration,
        operations_per_second: f64,
        total_time: Duration,
    ) -> Self {
        Self {
            benchmark_name,
            benchmark_duration,
            total_operations,
            sample_count,
            average_duration,
            median_duration,
            min_duration,
            max_duration,
            p95_duration,
            p99_duration,
            operations_per_second,
            total_time,
        }
    }

    /// Get performance category based on average duration
    pub fn performance_category(&self) -> PerformanceCategory {
        TimingUtils::categorize_duration(self.average_duration)
    }
    
    /// Get efficiency ratio (operation time / benchmark time)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.benchmark_duration.as_nanos() > 0 {
            self.total_time.as_nanos() as f64 / self.benchmark_duration.as_nanos() as f64
        } else {
            0.0
        }
    }
    
    /// Get overhead ratio (1.0 - efficiency_ratio)
    pub fn overhead_ratio(&self) -> f64 {
        1.0 - self.efficiency_ratio()
    }
    
    /// Check if results indicate good performance
    pub fn has_good_performance(&self) -> bool {
        matches!(
            self.performance_category(),
            PerformanceCategory::Excellent | PerformanceCategory::Good
        ) && self.operations_per_second > 50.0
    }
    
    /// Check if results indicate performance issues
    pub fn has_performance_issues(&self) -> bool {
        matches!(
            self.performance_category(),
            PerformanceCategory::Slow | PerformanceCategory::VerySlow
        ) || self.operations_per_second < 10.0
    }
    
    /// Get performance grade
    pub fn performance_grade(&self) -> char {
        self.performance_category().grade()
    }
    
    /// Get performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        let category_score = self.performance_category().score();
        let throughput_score = (self.operations_per_second / 100.0).min(1.0);
        let efficiency_score = self.efficiency_ratio();
        
        // Weighted combination
        (category_score * 0.4) + (throughput_score * 0.3) + (efficiency_score * 0.3)
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "{}: {} ops in {} (avg: {}, {:.1}/sec, {})",
            self.benchmark_name,
            self.total_operations,
            TimingUtils::format_duration(self.benchmark_duration),
            TimingUtils::format_duration(self.average_duration),
            self.operations_per_second,
            self.performance_category().description()
        )
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        format!(
            "=== {} Benchmark Results ===\n\
            Total Operations: {}\n\
            Sample Count: {}\n\
            Benchmark Duration: {}\n\
            Total Operation Time: {}\n\
            \n\
            --- Timing Statistics ---\n\
            Average: {}\n\
            Median: {}\n\
            Minimum: {}\n\
            Maximum: {}\n\
            95th Percentile: {}\n\
            99th Percentile: {}\n\
            \n\
            --- Performance Metrics ---\n\
            Throughput: {:.1} ops/sec\n\
            Efficiency: {:.1}%\n\
            Overhead: {:.1}%\n\
            Performance: {} (Grade: {})\n\
            Score: {:.2}/1.0",
            self.benchmark_name,
            self.total_operations,
            self.sample_count,
            TimingUtils::format_duration(self.benchmark_duration),
            TimingUtils::format_duration(self.total_time),
            TimingUtils::format_duration(self.average_duration),
            TimingUtils::format_duration(self.median_duration),
            TimingUtils::format_duration(self.min_duration),
            TimingUtils::format_duration(self.max_duration),
            TimingUtils::format_duration(self.p95_duration),
            TimingUtils::format_duration(self.p99_duration),
            self.operations_per_second,
            self.efficiency_ratio() * 100.0,
            self.overhead_ratio() * 100.0,
            self.performance_category().description(),
            self.performance_grade(),
            self.performance_score()
        )
    }
    
    /// Compare with another benchmark result
    pub fn compare_with(&self, other: &BenchmarkResults) -> BenchmarkComparison {
        let throughput_ratio = if other.operations_per_second > 0.0 {
            self.operations_per_second / other.operations_per_second
        } else {
            1.0
        };
        
        let latency_ratio = if other.average_duration.as_nanos() > 0 {
            self.average_duration.as_nanos() as f64 / other.average_duration.as_nanos() as f64
        } else {
            1.0
        };
        
        let efficiency_ratio = if other.efficiency_ratio() > 0.0 {
            self.efficiency_ratio() / other.efficiency_ratio()
        } else {
            1.0
        };
        
        BenchmarkComparison {
            baseline_name: other.benchmark_name.clone(),
            current_name: self.benchmark_name.clone(),
            throughput_ratio,
            latency_ratio,
            efficiency_ratio,
            throughput_improvement: (throughput_ratio - 1.0) * 100.0,
            latency_improvement: (1.0 - latency_ratio) * 100.0,
            efficiency_improvement: (efficiency_ratio - 1.0) * 100.0,
        }
    }
}

/// Comparison between two benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkComparison {
    /// Name of the baseline benchmark
    pub baseline_name: String,
    /// Name of the current benchmark
    pub current_name: String,
    /// Throughput ratio (current/baseline)
    pub throughput_ratio: f64,
    /// Latency ratio (current/baseline, lower is better)
    pub latency_ratio: f64,
    /// Efficiency ratio (current/baseline)
    pub efficiency_ratio: f64,
    /// Throughput improvement percentage
    pub throughput_improvement: f64,
    /// Latency improvement percentage (positive = better)
    pub latency_improvement: f64,
    /// Efficiency improvement percentage
    pub efficiency_improvement: f64,
}

impl BenchmarkComparison {
    /// Check if current benchmark is better overall
    pub fn is_improvement(&self) -> bool {
        self.throughput_improvement > 0.0 && self.latency_improvement > 0.0
    }
    
    /// Check if current benchmark is significantly better
    pub fn is_significant_improvement(&self) -> bool {
        self.throughput_improvement > 10.0 && self.latency_improvement > 10.0
    }
    
    /// Check if current benchmark is worse
    pub fn is_regression(&self) -> bool {
        self.throughput_improvement < -5.0 || self.latency_improvement < -5.0
    }
    
    /// Get comparison summary
    pub fn summary(&self) -> String {
        let status = if self.is_significant_improvement() {
            "SIGNIFICANT IMPROVEMENT"
        } else if self.is_improvement() {
            "IMPROVEMENT"
        } else if self.is_regression() {
            "REGRESSION"
        } else {
            "SIMILAR"
        };
        
        format!(
            "{} vs {}: {} (Throughput: {:+.1}%, Latency: {:+.1}%, Efficiency: {:+.1}%)",
            self.current_name,
            self.baseline_name,
            status,
            self.throughput_improvement,
            self.latency_improvement,
            self.efficiency_improvement
        )
    }
}
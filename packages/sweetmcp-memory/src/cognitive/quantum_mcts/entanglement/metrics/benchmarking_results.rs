//! Benchmark results analysis and comparison utilities
//!
//! This module provides comprehensive benchmark result analysis, comparison,
//! and reporting capabilities with zero-allocation patterns and blazing-fast performance.

use std::time::Duration;
use super::benchmarking_core::{BenchmarkStats, EntanglementBenchmark};

/// Comprehensive benchmark results with detailed analysis
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Benchmark name
    pub benchmark_name: String,
    /// Core statistics
    pub stats: BenchmarkStats,
    /// Performance category
    pub category: PerformanceCategory,
    /// Trend analysis
    pub trend: PerformanceTrend,
    /// Quality assessment
    pub quality: ResultQuality,
    /// Timestamp of results
    pub timestamp: std::time::SystemTime,
}

impl BenchmarkResults {
    /// Create benchmark results from stats
    pub fn from_stats(benchmark_name: String, stats: BenchmarkStats) -> Self {
        let category = Self::categorize_performance(&stats);
        let trend = PerformanceTrend::Stable; // Default, would need historical data
        let quality = Self::assess_quality(&stats);
        
        Self {
            benchmark_name,
            stats,
            category,
            trend,
            quality,
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    /// Create from benchmark instance
    pub fn from_benchmark(benchmark: &EntanglementBenchmark) -> Self {
        let stats = benchmark.current_stats();
        Self::from_stats(benchmark.name().to_string(), stats)
    }
    
    /// Categorize performance level
    fn categorize_performance(stats: &BenchmarkStats) -> PerformanceCategory {
        match stats.performance_grade() {
            'A' => PerformanceCategory::Excellent,
            'B' => PerformanceCategory::Good,
            'C' => PerformanceCategory::Average,
            'D' => PerformanceCategory::Poor,
            _ => PerformanceCategory::Critical,
        }
    }
    
    /// Assess result quality
    fn assess_quality(stats: &BenchmarkStats) -> ResultQuality {
        if stats.sample_count < 10 {
            ResultQuality::Insufficient
        } else if stats.sample_count < 100 {
            ResultQuality::Limited
        } else if stats.is_consistent(0.5) {
            ResultQuality::High
        } else {
            ResultQuality::Moderate
        }
    }
    
    /// Get detailed performance report
    pub fn detailed_report(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Performance: {:?} (Grade: {})\n\
             Operations: {} in {:.2}s\n\
             Throughput: {:.1} ops/sec\n\
             Latency: avg={:.2}ms, p95={:.2}ms, p99={:.2}ms\n\
             Trend: {:?}\n\
             Quality: {:?}\n\
             Timestamp: {:?}",
            self.benchmark_name,
            self.category,
            self.stats.performance_grade(),
            self.stats.total_operations,
            self.stats.total_time.as_secs_f64(),
            self.stats.operations_per_second,
            self.stats.average_duration.as_secs_f64() * 1000.0,
            self.stats.percentile_95.as_secs_f64() * 1000.0,
            self.stats.percentile_99.as_secs_f64() * 1000.0,
            self.trend,
            self.quality,
            self.timestamp
        )
    }
    
    /// Generate performance recommendations
    pub fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match self.category {
            PerformanceCategory::Critical | PerformanceCategory::Poor => {
                recommendations.push("Performance is below acceptable levels - immediate optimization required".to_string());
                recommendations.push("Consider profiling to identify bottlenecks".to_string());
            }
            PerformanceCategory::Average => {
                recommendations.push("Performance is acceptable but has room for improvement".to_string());
            }
            PerformanceCategory::Good | PerformanceCategory::Excellent => {
                recommendations.push("Performance is good - maintain current optimization level".to_string());
            }
        }
        
        if !self.stats.is_consistent(0.3) {
            recommendations.push("High performance variability detected - investigate consistency issues".to_string());
        }
        
        match self.quality {
            ResultQuality::Insufficient => {
                recommendations.push("Insufficient samples for reliable analysis - collect more data".to_string());
            }
            ResultQuality::Limited => {
                recommendations.push("Limited sample size - consider longer benchmark duration".to_string());
            }
            _ => {}
        }
        
        recommendations
    }
    
    /// Check if results indicate performance regression
    pub fn indicates_regression(&self, baseline: &BenchmarkResults) -> bool {
        let current_ops = self.stats.operations_per_second;
        let baseline_ops = baseline.stats.operations_per_second;
        
        if baseline_ops == 0.0 {
            return false;
        }
        
        let regression_threshold = 0.9; // 10% degradation
        current_ops / baseline_ops < regression_threshold
    }
    
    /// Calculate performance improvement over baseline
    pub fn improvement_over(&self, baseline: &BenchmarkResults) -> f64 {
        let current_ops = self.stats.operations_per_second;
        let baseline_ops = baseline.stats.operations_per_second;
        
        if baseline_ops == 0.0 {
            return 0.0;
        }
        
        (current_ops - baseline_ops) / baseline_ops
    }
    
    /// Get result age in seconds
    pub fn age_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(self.timestamp)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
    
    /// Check if results are stale
    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        self.age_seconds() > max_age_seconds
    }
}

/// Performance category classification
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceCategory {
    Excellent,  // A grade
    Good,       // B grade
    Average,    // C grade
    Poor,       // D grade
    Critical,   // F grade
}

impl PerformanceCategory {
    /// Get category description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            PerformanceCategory::Excellent => "Excellent performance - exceeds expectations",
            PerformanceCategory::Good => "Good performance - meets requirements",
            PerformanceCategory::Average => "Average performance - acceptable",
            PerformanceCategory::Poor => "Poor performance - below expectations",
            PerformanceCategory::Critical => "Critical performance - immediate attention required",
        }
    }
    
    /// Get priority level for optimization
    #[inline]
    pub const fn optimization_priority(&self) -> u8 {
        match self {
            PerformanceCategory::Excellent => 1,
            PerformanceCategory::Good => 2,
            PerformanceCategory::Average => 3,
            PerformanceCategory::Poor => 4,
            PerformanceCategory::Critical => 5,
        }
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
}

/// Result quality assessment
#[derive(Debug, Clone, PartialEq)]
pub enum ResultQuality {
    High,         // Large sample, consistent results
    Moderate,     // Adequate sample, some variability
    Limited,      // Small sample size
    Insufficient, // Too few samples for reliable analysis
}

impl ResultQuality {
    /// Get quality description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            ResultQuality::High => "High quality results with good statistical confidence",
            ResultQuality::Moderate => "Moderate quality results with adequate confidence",
            ResultQuality::Limited => "Limited quality results due to small sample size",
            ResultQuality::Insufficient => "Insufficient data for reliable analysis",
        }
    }
    
    /// Check if quality is sufficient for decision making
    #[inline]
    pub const fn is_sufficient(&self) -> bool {
        matches!(self, ResultQuality::High | ResultQuality::Moderate)
    }
    
    /// Get confidence level (0.0 to 1.0)
    #[inline]
    pub const fn confidence_level(&self) -> f64 {
        match self {
            ResultQuality::High => 0.95,
            ResultQuality::Moderate => 0.80,
            ResultQuality::Limited => 0.60,
            ResultQuality::Insufficient => 0.30,
        }
    }
}
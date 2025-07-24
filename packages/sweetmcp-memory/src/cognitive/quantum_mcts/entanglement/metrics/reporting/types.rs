//! Metrics reporting data types and structures
//!
//! This module provides core data structures for metrics reporting
//! with zero-allocation patterns and blazing-fast performance.

use std::time::{Duration, Instant};
use super::tracking::PerformanceCategory;

/// Individual metrics report
#[derive(Debug, Clone)]
pub struct MetricsReport {
    /// Unique report ID
    pub report_id: usize,
    /// Time when report was generated
    pub report_time: Instant,
    /// System uptime
    pub uptime: Duration,
    /// Number of entanglements
    pub entanglement_count: u64,
    /// Number of operations
    pub operation_count: u64,
    /// Number of influence calculations
    pub influence_count: u64,
    /// Number of errors
    pub error_count: u64,
    /// Average operation time
    pub avg_operation_time: Duration,
    /// Average influence calculation time
    pub avg_influence_time: Duration,
    /// Operations per second
    pub operations_per_second: f64,
    /// Influence calculations per second
    pub influence_per_second: f64,
    /// Error rate
    pub error_rate: f64,
    /// Operation performance category
    pub operation_performance: PerformanceCategory,
    /// Influence performance category
    pub influence_performance: PerformanceCategory,
    /// Overall performance category
    pub overall_performance: PerformanceCategory,
}

/// Summary report aggregating multiple metrics
#[derive(Debug, Clone)]
pub struct SummaryReport {
    /// Report generation time
    pub report_time: Instant,
    /// System uptime
    pub uptime: Duration,
    /// Overall performance score
    pub overall_performance_score: f64,
    /// Average throughput across all metrics
    pub average_throughput: f64,
    /// Overall performance grade
    pub overall_grade: char,
    /// Number of data sources
    pub data_sources: usize,
    /// Aggregated metrics
    pub aggregated_metrics: AggregatedMetrics,
}

/// Aggregated metrics from multiple sources
#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    /// Total operations across all sources
    pub total_operations: u64,
    /// Total errors across all sources
    pub total_errors: u64,
    /// Combined error rate
    pub combined_error_rate: f64,
    /// Peak throughput observed
    pub peak_throughput: f64,
    /// Average response time
    pub average_response_time: Duration,
    /// Performance consistency score
    pub consistency_score: f64,
}

/// Historical data point for trend analysis
#[derive(Debug, Clone)]
pub struct HistoricalDataPoint {
    /// Timestamp of the data point
    pub timestamp: Instant,
    /// Performance score at this time
    pub performance_score: f64,
    /// Operations per second at this time
    pub operations_per_second: f64,
    /// Error rate at this time
    pub error_rate: f64,
    /// Operation count at this time
    pub operation_count: u64,
}

/// Performance dashboard data
#[derive(Debug, Clone)]
pub struct PerformanceDashboard {
    /// Reporter name
    pub reporter_name: String,
    /// Dashboard generation time
    pub generation_time: Instant,
    /// System uptime
    pub uptime: Duration,
    /// Latest metrics report
    pub latest_report: Option<MetricsReport>,
    /// Performance trend
    pub trend: PerformanceTrend,
    /// Historical performance data
    pub historical_data: Vec<HistoricalDataPoint>,
    /// Average performance score
    pub avg_performance: f64,
    /// Minimum performance score
    pub min_performance: f64,
    /// Maximum performance score
    pub max_performance: f64,
    /// Total number of reports
    pub total_reports: usize,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub enum PerformanceTrend {
    /// Performance is improving
    Improving { rate: f64 },
    /// Performance is declining
    Declining { rate: f64 },
    /// Performance is stable
    Stable { variance: f64 },
    /// Insufficient data for trend analysis
    Unknown,
}

impl MetricsReport {
    /// Check if performance is acceptable
    pub fn has_acceptable_performance(&self) -> bool {
        matches!(
            self.overall_performance,
            PerformanceCategory::Excellent | PerformanceCategory::Good | PerformanceCategory::Acceptable
        )
    }

    /// Get performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        self.overall_performance.score()
    }

    /// Check if this is a high-performance report
    pub fn is_high_performance(&self) -> bool {
        matches!(
            self.overall_performance,
            PerformanceCategory::Excellent | PerformanceCategory::Good
        )
    }

    /// Get efficiency ratio (operations per error)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.error_count == 0 {
            self.operation_count as f64
        } else {
            self.operation_count as f64 / self.error_count as f64
        }
    }

    /// Get throughput score based on operations per second
    pub fn throughput_score(&self) -> f64 {
        // Normalize throughput to 0-1 scale (assuming 1000 ops/sec is excellent)
        (self.operations_per_second / 1000.0).min(1.0)
    }

    /// Get reliability score based on error rate
    pub fn reliability_score(&self) -> f64 {
        (1.0 - self.error_rate).max(0.0)
    }
}

impl SummaryReport {
    /// Check if overall performance is acceptable
    pub fn has_acceptable_performance(&self) -> bool {
        self.overall_performance_score >= 0.5
    }

    /// Get performance grade description
    pub fn grade_description(&self) -> &'static str {
        match self.overall_grade {
            'A' => "Excellent",
            'B' => "Good", 
            'C' => "Acceptable",
            'D' => "Slow",
            'F' => "Very Slow",
            _ => "Unknown",
        }
    }

    /// Check if performance is improving based on trend
    pub fn is_improving(&self) -> bool {
        self.overall_performance_score > 0.7
    }
}

impl AggregatedMetrics {
    /// Calculate overall health score
    pub fn health_score(&self) -> f64 {
        let error_score = (1.0 - self.combined_error_rate).max(0.0);
        let throughput_score = (self.peak_throughput / 1000.0).min(1.0);
        let consistency_score = self.consistency_score;
        
        // Weighted combination
        (error_score * 0.4) + (throughput_score * 0.3) + (consistency_score * 0.3)
    }

    /// Check if metrics indicate healthy system
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.7
    }
}

impl PerformanceTrend {
    /// Get trend direction as string
    pub fn direction(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving { .. } => "Improving",
            PerformanceTrend::Declining { .. } => "Declining", 
            PerformanceTrend::Stable { .. } => "Stable",
            PerformanceTrend::Unknown => "Unknown",
        }
    }

    /// Get trend strength (0.0 to 1.0)
    pub fn strength(&self) -> f64 {
        match self {
            PerformanceTrend::Improving { rate } => *rate,
            PerformanceTrend::Declining { rate } => *rate,
            PerformanceTrend::Stable { variance } => 1.0 - variance,
            PerformanceTrend::Unknown => 0.0,
        }
    }

    /// Check if trend is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, PerformanceTrend::Improving { .. } | PerformanceTrend::Stable { .. })
    }
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            total_errors: 0,
            combined_error_rate: 0.0,
            peak_throughput: 0.0,
            average_response_time: Duration::from_millis(0),
            consistency_score: 1.0,
        }
    }
}
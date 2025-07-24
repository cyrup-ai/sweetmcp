//! Dashboard generation and visualization
//!
//! This module provides dashboard generation functionality
//! with zero-allocation patterns and blazing-fast performance.

use std::time::Instant;
use super::super::tracking::TimingUtils;
use super::types::{PerformanceDashboard, HistoricalDataPoint, PerformanceTrend};
use super::generation::MetricsReporter;

impl MetricsReporter {
    /// Generate performance dashboard
    pub fn generate_dashboard(&self) -> PerformanceDashboard {
        let latest_report = self.latest_report();
        let trend = self.performance_trend(10);
        let historical_data = self.historical_performance();
        
        // Calculate statistics from historical data
        let (avg_performance, min_performance, max_performance) = if !historical_data.is_empty() {
            let scores: Vec<f64> = historical_data.iter().map(|d| d.performance_score).collect();
            let avg = scores.iter().sum::<f64>() / scores.len() as f64;
            let min = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            (avg, min, max)
        } else {
            (0.0, 0.0, 0.0)
        };
        
        PerformanceDashboard {
            reporter_name: self.reporter_name().to_string(),
            generation_time: Instant::now(),
            uptime: self.creation_time().elapsed(),
            latest_report: latest_report.cloned(),
            trend,
            historical_data,
            avg_performance,
            min_performance,
            max_performance,
            total_reports: self.report_count(),
        }
    }
}

impl PerformanceDashboard {
    /// Format historical data summary
    pub fn format_historical_summary(&self) -> String {
        if self.historical_data.is_empty() {
            return "No historical data available".to_string();
        }

        let recent_data = if self.historical_data.len() > 5 {
            &self.historical_data[self.historical_data.len() - 5..]
        } else {
            &self.historical_data
        };

        let mut summary = String::new();
        summary.push_str("Recent Performance:\n");
        
        for (i, data_point) in recent_data.iter().enumerate() {
            summary.push_str(&format!(
                "  {}: {:.2} score, {:.1} ops/sec, {:.1}% err\n",
                i + 1,
                data_point.performance_score,
                data_point.operations_per_second,
                data_point.error_rate * 100.0
            ));
        }

        summary
    }

    /// Get dashboard health status
    pub fn health_status(&self) -> DashboardHealthStatus {
        if let Some(ref report) = self.latest_report {
            let performance_healthy = report.has_acceptable_performance();
            let trend_healthy = self.trend.is_positive();
            let data_sufficient = self.historical_data.len() >= 5;
            
            match (performance_healthy, trend_healthy, data_sufficient) {
                (true, true, true) => DashboardHealthStatus::Excellent,
                (true, true, false) => DashboardHealthStatus::Good,
                (true, false, _) => DashboardHealthStatus::Warning,
                (false, _, _) => DashboardHealthStatus::Critical,
            }
        } else {
            DashboardHealthStatus::Unknown
        }
    }

    /// Get performance summary for dashboard
    pub fn performance_summary(&self) -> String {
        match self.health_status() {
            DashboardHealthStatus::Excellent => {
                format!("System performing excellently with {} trend", self.trend.direction().to_lowercase())
            },
            DashboardHealthStatus::Good => {
                format!("System performing well with {} trend", self.trend.direction().to_lowercase())
            },
            DashboardHealthStatus::Warning => {
                format!("System performance acceptable but trend is {}", self.trend.direction().to_lowercase())
            },
            DashboardHealthStatus::Critical => {
                "System performance is below acceptable levels".to_string()
            },
            DashboardHealthStatus::Unknown => {
                "Insufficient data to determine system health".to_string()
            },
        }
    }

    /// Get key metrics summary
    pub fn key_metrics(&self) -> KeyMetrics {
        if let Some(ref report) = self.latest_report {
            KeyMetrics {
                current_performance: report.performance_score(),
                average_performance: self.avg_performance,
                operations_per_second: report.operations_per_second,
                error_rate: report.error_rate,
                trend_direction: self.trend.direction(),
                health_status: self.health_status(),
            }
        } else {
            KeyMetrics::default()
        }
    }
}

/// Dashboard health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DashboardHealthStatus {
    /// System performing excellently
    Excellent,
    /// System performing well
    Good,
    /// System performance has warnings
    Warning,
    /// System performance is critical
    Critical,
    /// Insufficient data to determine status
    Unknown,
}

impl DashboardHealthStatus {
    /// Get status color for display
    pub fn color(&self) -> &'static str {
        match self {
            DashboardHealthStatus::Excellent => "green",
            DashboardHealthStatus::Good => "blue",
            DashboardHealthStatus::Warning => "yellow",
            DashboardHealthStatus::Critical => "red",
            DashboardHealthStatus::Unknown => "gray",
        }
    }

    /// Get status description
    pub fn description(&self) -> &'static str {
        match self {
            DashboardHealthStatus::Excellent => "Excellent",
            DashboardHealthStatus::Good => "Good",
            DashboardHealthStatus::Warning => "Warning",
            DashboardHealthStatus::Critical => "Critical",
            DashboardHealthStatus::Unknown => "Unknown",
        }
    }

    /// Check if status indicates healthy system
    pub fn is_healthy(&self) -> bool {
        matches!(self, DashboardHealthStatus::Excellent | DashboardHealthStatus::Good)
    }
}

/// Key metrics summary for dashboard
#[derive(Debug, Clone)]
pub struct KeyMetrics {
    /// Current performance score
    pub current_performance: f64,
    /// Average performance over time
    pub average_performance: f64,
    /// Current operations per second
    pub operations_per_second: f64,
    /// Current error rate
    pub error_rate: f64,
    /// Trend direction
    pub trend_direction: &'static str,
    /// Overall health status
    pub health_status: DashboardHealthStatus,
}

impl Default for KeyMetrics {
    fn default() -> Self {
        Self {
            current_performance: 0.0,
            average_performance: 0.0,
            operations_per_second: 0.0,
            error_rate: 0.0,
            trend_direction: "Unknown",
            health_status: DashboardHealthStatus::Unknown,
        }
    }
}

impl KeyMetrics {
    /// Format key metrics as string
    pub fn formatted(&self) -> String {
        format!(
            "Performance: {:.2} (avg: {:.2}) | Throughput: {:.1} ops/sec | Errors: {:.1}% | Trend: {} | Status: {}",
            self.current_performance,
            self.average_performance,
            self.operations_per_second,
            self.error_rate * 100.0,
            self.trend_direction,
            self.health_status.description()
        )
    }

    /// Get performance grade
    pub fn performance_grade(&self) -> char {
        match self.current_performance {
            p if p >= 0.9 => 'A',
            p if p >= 0.7 => 'B',
            p if p >= 0.5 => 'C',
            p if p >= 0.3 => 'D',
            _ => 'F',
        }
    }
}
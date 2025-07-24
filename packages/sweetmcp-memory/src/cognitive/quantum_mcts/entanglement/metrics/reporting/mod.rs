//! Metrics reporting module integration
//!
//! This module provides comprehensive metrics reporting functionality
//! with zero-allocation patterns and blazing-fast performance.

pub mod types;
pub mod generation;
pub mod summary;
pub mod formatting;
pub mod dashboard;
pub mod visualization;

// Re-export core types for ergonomic access
pub use types::{
    MetricsReport, SummaryReport, PerformanceDashboard, HistoricalDataPoint,
    PerformanceTrend, AggregatedMetrics, PerformanceGrade
};

// Re-export generation functionality
pub use generation::MetricsReporter;

// Re-export formatting utilities
pub use formatting::ReportFormatter;

// Re-export dashboard functionality
pub use dashboard::{DashboardHealthStatus, KeyMetrics};

// Re-export visualization utilities
pub use visualization::DashboardVisualizer;

/// Main reporting interface for metrics
pub struct MetricsReporting {
    reporter: MetricsReporter,
}

impl MetricsReporting {
    /// Create new metrics reporting instance
    pub fn new(reporter_name: &str) -> Self {
        Self {
            reporter: MetricsReporter::new(reporter_name),
        }
    }

    /// Generate comprehensive metrics report
    pub fn generate_report(&mut self) -> MetricsReport {
        self.reporter.generate_report()
    }

    /// Generate summary report from multiple sources
    pub fn generate_summary(&self, reports: &[MetricsReport]) -> SummaryReport {
        self.reporter.generate_summary_report(None, None, reports)
    }

    /// Generate performance dashboard
    pub fn generate_dashboard(&self) -> PerformanceDashboard {
        self.reporter.generate_dashboard()
    }

    /// Get formatted report table
    pub fn format_reports_table(&self, reports: &[MetricsReport]) -> String {
        ReportFormatter::format_reports_table(reports)
    }

    /// Get performance comparison
    pub fn format_performance_comparison(&self, reports: &[MetricsReport]) -> String {
        ReportFormatter::format_performance_comparison(reports)
    }

    /// Get trend analysis
    pub fn format_trend_analysis(&self, historical_data: &[HistoricalDataPoint]) -> String {
        ReportFormatter::format_trend_analysis(historical_data)
    }

    /// Create ASCII dashboard
    pub fn create_ascii_dashboard(&self) -> String {
        let dashboard = self.generate_dashboard();
        DashboardVisualizer::create_ascii_dashboard(&dashboard)
    }

    /// Create mini dashboard
    pub fn create_mini_dashboard(&self) -> String {
        let dashboard = self.generate_dashboard();
        DashboardVisualizer::create_mini_dashboard(&dashboard)
    }

    /// Create alert dashboard
    pub fn create_alert_dashboard(&self) -> String {
        let dashboard = self.generate_dashboard();
        DashboardVisualizer::create_alert_dashboard(&dashboard)
    }

    /// Get reporter name
    pub fn reporter_name(&self) -> &str {
        self.reporter.reporter_name()
    }

    /// Get report count
    pub fn report_count(&self) -> usize {
        self.reporter.report_count()
    }

    /// Get latest report
    pub fn latest_report(&self) -> Option<&MetricsReport> {
        self.reporter.latest_report()
    }

    /// Get historical performance data
    pub fn historical_performance(&self) -> Vec<HistoricalDataPoint> {
        self.reporter.historical_performance()
    }

    /// Get performance trend
    pub fn performance_trend(&self, window_size: usize) -> PerformanceTrend {
        self.reporter.performance_trend(window_size)
    }

    /// Check if system has acceptable performance
    pub fn has_acceptable_performance(&self) -> bool {
        if let Some(report) = self.latest_report() {
            report.has_acceptable_performance()
        } else {
            false
        }
    }

    /// Get current health status
    pub fn health_status(&self) -> DashboardHealthStatus {
        let dashboard = self.generate_dashboard();
        dashboard.health_status()
    }

    /// Get key metrics summary
    pub fn key_metrics(&self) -> KeyMetrics {
        let dashboard = self.generate_dashboard();
        dashboard.key_metrics()
    }
}

/// Convenience functions for quick reporting
impl MetricsReporting {
    /// Quick performance summary
    pub fn quick_summary(&self) -> String {
        if let Some(report) = self.latest_report() {
            report.performance_summary()
        } else {
            "No reports available".to_string()
        }
    }

    /// Quick status line
    pub fn quick_status(&self) -> String {
        let dashboard = self.generate_dashboard();
        DashboardVisualizer::create_status_line(&dashboard)
    }

    /// Quick health check
    pub fn quick_health_check(&self) -> String {
        match self.health_status() {
            DashboardHealthStatus::Excellent => "✅ System performing excellently".to_string(),
            DashboardHealthStatus::Good => "✅ System performing well".to_string(),
            DashboardHealthStatus::Warning => "⚠️ System performance has warnings".to_string(),
            DashboardHealthStatus::Critical => "🚨 System performance is critical".to_string(),
            DashboardHealthStatus::Unknown => "❓ System status unknown".to_string(),
        }
    }

    /// Quick performance grade
    pub fn quick_grade(&self) -> char {
        let key_metrics = self.key_metrics();
        key_metrics.performance_grade()
    }

    /// Quick error status
    pub fn quick_error_status(&self) -> String {
        let key_metrics = self.key_metrics();
        DashboardVisualizer::create_error_indicator(key_metrics.error_rate)
    }

    /// Quick throughput status
    pub fn quick_throughput_status(&self) -> String {
        let key_metrics = self.key_metrics();
        DashboardVisualizer::create_throughput_indicator(key_metrics.operations_per_second)
    }

    /// Quick trend status
    pub fn quick_trend_status(&self) -> String {
        let dashboard = self.generate_dashboard();
        DashboardVisualizer::create_trend_indicator(dashboard.trend.strength(), dashboard.trend.is_positive())
    }
}

/// Builder for metrics reporting configuration
pub struct MetricsReportingBuilder {
    reporter_name: String,
}

impl MetricsReportingBuilder {
    /// Create new builder
    pub fn new(reporter_name: &str) -> Self {
        Self {
            reporter_name: reporter_name.to_string(),
        }
    }

    /// Build metrics reporting instance
    pub fn build(self) -> MetricsReporting {
        MetricsReporting::new(&self.reporter_name)
    }
}

/// Create new metrics reporting instance
pub fn create_metrics_reporting(reporter_name: &str) -> MetricsReporting {
    MetricsReporting::new(reporter_name)
}

/// Create metrics reporting builder
pub fn metrics_reporting_builder(reporter_name: &str) -> MetricsReportingBuilder {
    MetricsReportingBuilder::new(reporter_name)
}
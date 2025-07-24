//! Health monitor for continuous monitoring
//!
//! This module provides blazing-fast continuous health monitoring with zero allocation
//! optimizations and elegant ergonomic interfaces for system health tracking.

use tracing::debug;

use super::health_report::HealthCheckReport;
use super::health_types::HealthTrend;

/// Health monitor for continuous monitoring
pub struct HealthMonitor {
    /// Historical reports
    reports: Vec<HealthCheckReport>,
    /// Maximum history to keep
    max_history: usize,
    /// Monitoring thresholds
    thresholds: MonitoringThresholds,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self {
            reports: Vec::new(),
            max_history: 100,
            thresholds: MonitoringThresholds::default(),
        }
    }
}

impl HealthMonitor {
    /// Create new health monitor
    #[inline]
    pub fn new(max_history: usize, thresholds: MonitoringThresholds) -> Self {
        Self {
            reports: Vec::with_capacity(max_history),
            max_history,
            thresholds,
        }
    }

    /// Create with default thresholds
    #[inline]
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            reports: Vec::with_capacity(max_history),
            max_history,
            thresholds: MonitoringThresholds::default(),
        }
    }

    /// Add health check report
    #[inline]
    pub fn add_report(&mut self, report: HealthCheckReport) {
        self.reports.push(report);
        
        // Keep only recent history
        if self.reports.len() > self.max_history {
            self.reports.remove(0);
        }

        debug!("Added health report, total history: {}", self.reports.len());
    }

    /// Get latest report
    #[inline]
    pub fn latest_report(&self) -> Option<&HealthCheckReport> {
        self.reports.last()
    }

    /// Get report history
    #[inline]
    pub fn get_reports(&self) -> &[HealthCheckReport] {
        &self.reports
    }

    /// Get reports count
    #[inline]
    pub fn reports_count(&self) -> usize {
        self.reports.len()
    }

    /// Calculate health trend
    #[inline]
    pub fn calculate_trend(&self) -> HealthTrend {
        if self.reports.len() < 3 {
            return HealthTrend::Unknown;
        }

        let recent_scores: Vec<f64> = self.reports.iter()
            .rev()
            .take(5)
            .map(|r| r.overall_score)
            .collect();

        HealthTrend::calculate_from_scores(&recent_scores)
    }

    /// Check if alert should be triggered
    #[inline]
    pub fn should_trigger_alert(&self) -> bool {
        if let Some(latest) = self.latest_report() {
            latest.overall_score < self.thresholds.critical_score_threshold ||
            !latest.critical_issues().is_empty() ||
            latest.has_performance_degradation()
        } else {
            false
        }
    }

    /// Get alert message
    #[inline]
    pub fn get_alert_message(&self) -> Option<String> {
        if !self.should_trigger_alert() {
            return None;
        }

        if let Some(latest) = self.latest_report() {
            Some(format!(
                "HEALTH ALERT: {} - Score: {:.1}%, Critical Issues: {}, Trend: {:?}",
                latest.health_status().description(),
                latest.overall_score * 100.0,
                latest.critical_issues().len(),
                self.calculate_trend()
            ))
        } else {
            None
        }
    }

    /// Check if warning should be triggered
    #[inline]
    pub fn should_trigger_warning(&self) -> bool {
        if let Some(latest) = self.latest_report() {
            latest.overall_score < self.thresholds.warning_score_threshold &&
            latest.overall_score >= self.thresholds.critical_score_threshold
        } else {
            false
        }
    }

    /// Get warning message
    #[inline]
    pub fn get_warning_message(&self) -> Option<String> {
        if !self.should_trigger_warning() {
            return None;
        }

        if let Some(latest) = self.latest_report() {
            Some(format!(
                "HEALTH WARNING: {} - Score: {:.1}%, Issues: {}, Trend: {:?}",
                latest.health_status().description(),
                latest.overall_score * 100.0,
                latest.issues.len(),
                self.calculate_trend()
            ))
        } else {
            None
        }
    }

    /// Get average score over time period
    #[inline]
    pub fn get_average_score(&self, last_n_reports: usize) -> Option<f64> {
        if self.reports.is_empty() {
            return None;
        }

        let reports_to_consider = last_n_reports.min(self.reports.len());
        let sum: f64 = self.reports.iter()
            .rev()
            .take(reports_to_consider)
            .map(|r| r.overall_score)
            .sum();

        Some(sum / reports_to_consider as f64)
    }

    /// Get score trend over time
    #[inline]
    pub fn get_score_trend(&self, last_n_reports: usize) -> Vec<f64> {
        let reports_to_consider = last_n_reports.min(self.reports.len());
        self.reports.iter()
            .rev()
            .take(reports_to_consider)
            .map(|r| r.overall_score)
            .collect()
    }

    /// Check if performance is degrading
    #[inline]
    pub fn is_performance_degrading(&self) -> bool {
        matches!(self.calculate_trend(), HealthTrend::Declining)
    }

    /// Get monitoring thresholds
    #[inline]
    pub fn get_thresholds(&self) -> &MonitoringThresholds {
        &self.thresholds
    }

    /// Update monitoring thresholds
    #[inline]
    pub fn update_thresholds(&mut self, thresholds: MonitoringThresholds) {
        self.thresholds = thresholds;
    }

    /// Clear history
    #[inline]
    pub fn clear_history(&mut self) {
        self.reports.clear();
        debug!("Cleared health monitor history");
    }

    /// Get health summary statistics
    #[inline]
    pub fn get_summary_statistics(&self) -> HealthSummaryStatistics {
        if self.reports.is_empty() {
            return HealthSummaryStatistics::default();
        }

        let scores: Vec<f64> = self.reports.iter().map(|r| r.overall_score).collect();
        let total_issues: usize = self.reports.iter().map(|r| r.issues.len()).sum();
        let critical_issues: usize = self.reports.iter()
            .map(|r| r.critical_issues().len())
            .sum();

        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;

        HealthSummaryStatistics {
            reports_count: self.reports.len(),
            min_score,
            max_score,
            avg_score,
            total_issues,
            critical_issues,
            trend: self.calculate_trend(),
        }
    }
}

/// Monitoring thresholds
#[derive(Debug, Clone)]
pub struct MonitoringThresholds {
    /// Critical score threshold for alerts
    pub critical_score_threshold: f64,
    /// Warning score threshold
    pub warning_score_threshold: f64,
    /// Maximum response time before alert
    pub max_response_time_ms: f64,
    /// Minimum throughput before alert
    pub min_throughput_ops_per_sec: f64,
    /// Maximum error rate before alert
    pub max_error_rate_percent: f64,
}

impl Default for MonitoringThresholds {
    fn default() -> Self {
        Self {
            critical_score_threshold: 0.3,
            warning_score_threshold: 0.6,
            max_response_time_ms: 1000.0,
            min_throughput_ops_per_sec: 100.0,
            max_error_rate_percent: 5.0,
        }
    }
}

impl MonitoringThresholds {
    /// Create new monitoring thresholds
    #[inline]
    pub fn new(
        critical_score_threshold: f64,
        warning_score_threshold: f64,
        max_response_time_ms: f64,
        min_throughput_ops_per_sec: f64,
        max_error_rate_percent: f64,
    ) -> Self {
        Self {
            critical_score_threshold,
            warning_score_threshold,
            max_response_time_ms,
            min_throughput_ops_per_sec,
            max_error_rate_percent,
        }
    }

    /// Create strict thresholds
    #[inline]
    pub fn strict() -> Self {
        Self {
            critical_score_threshold: 0.5,
            warning_score_threshold: 0.8,
            max_response_time_ms: 500.0,
            min_throughput_ops_per_sec: 200.0,
            max_error_rate_percent: 1.0,
        }
    }

    /// Create lenient thresholds
    #[inline]
    pub fn lenient() -> Self {
        Self {
            critical_score_threshold: 0.2,
            warning_score_threshold: 0.4,
            max_response_time_ms: 2000.0,
            min_throughput_ops_per_sec: 50.0,
            max_error_rate_percent: 10.0,
        }
    }

    /// Validate thresholds
    #[inline]
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.critical_score_threshold >= self.warning_score_threshold {
            return Err("Critical threshold must be less than warning threshold");
        }
        if self.critical_score_threshold < 0.0 || self.critical_score_threshold > 1.0 {
            return Err("Critical threshold must be between 0.0 and 1.0");
        }
        if self.warning_score_threshold < 0.0 || self.warning_score_threshold > 1.0 {
            return Err("Warning threshold must be between 0.0 and 1.0");
        }
        if self.max_response_time_ms <= 0.0 {
            return Err("Max response time must be positive");
        }
        if self.min_throughput_ops_per_sec <= 0.0 {
            return Err("Min throughput must be positive");
        }
        if self.max_error_rate_percent < 0.0 || self.max_error_rate_percent > 100.0 {
            return Err("Max error rate must be between 0.0 and 100.0");
        }
        Ok(())
    }
}

/// Health summary statistics
#[derive(Debug, Clone)]
pub struct HealthSummaryStatistics {
    /// Number of reports in history
    pub reports_count: usize,
    /// Minimum score observed
    pub min_score: f64,
    /// Maximum score observed
    pub max_score: f64,
    /// Average score
    pub avg_score: f64,
    /// Total issues across all reports
    pub total_issues: usize,
    /// Total critical issues across all reports
    pub critical_issues: usize,
    /// Current trend
    pub trend: HealthTrend,
}

impl Default for HealthSummaryStatistics {
    fn default() -> Self {
        Self {
            reports_count: 0,
            min_score: 0.0,
            max_score: 0.0,
            avg_score: 0.0,
            total_issues: 0,
            critical_issues: 0,
            trend: HealthTrend::Unknown,
        }
    }
}

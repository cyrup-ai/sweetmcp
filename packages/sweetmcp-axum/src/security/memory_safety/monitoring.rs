//! Memory safety monitoring and alerting system
//!
//! This module provides comprehensive monitoring, alerting, and reporting
//! capabilities for memory safety validation with zero allocation patterns
//! and blazing-fast performance for production environments.

use crate::security::memory_safety::core::*;
use crate::security::memory_safety::validation::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Alert thresholds for memory safety monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Maximum memory leaks before alert
    pub max_memory_leaks: u64,
    /// Maximum buffer overflows before alert
    pub max_buffer_overflows: u64,
    /// Maximum use-after-free violations before alert
    pub max_use_after_free: u64,
    /// Maximum concurrent violations before alert
    pub max_concurrent_violations: u64,
    /// Minimum success rate before alert
    pub min_success_rate: f64,
    /// Maximum validation time before alert (microseconds)
    pub max_validation_time_us: u64,
    /// Maximum cache miss rate before alert
    pub max_cache_miss_rate: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_memory_leaks: 10,
            max_buffer_overflows: 1,
            max_use_after_free: 1,
            max_concurrent_violations: 5,
            min_success_rate: 0.95,
            max_validation_time_us: 10000, // 10ms
            max_cache_miss_rate: 0.2,      // 20%
        }
    }
}

/// Memory safety monitoring system
pub struct MemorySafetyMonitor {
    /// Memory safety validator
    validator: Arc<MemorySafetyValidator>,
    /// Alert thresholds
    alert_thresholds: AlertThresholds,
    /// Violation history
    violation_history: Arc<Mutex<VecDeque<MemorySafetyViolation>>>,
    /// Maximum history size
    max_history_size: usize,
    /// Monitoring enabled flag
    monitoring_enabled: bool,
}

impl MemorySafetyMonitor {
    /// Create new memory safety monitor
    pub fn new(validator: Arc<MemorySafetyValidator>) -> Self {
        Self {
            validator,
            alert_thresholds: AlertThresholds::default(),
            violation_history: Arc::new(Mutex::new(VecDeque::new())),
            max_history_size: 1000,
            monitoring_enabled: true,
        }
    }

    /// Create monitor with custom thresholds
    pub fn with_thresholds(
        validator: Arc<MemorySafetyValidator>,
        thresholds: AlertThresholds,
    ) -> Self {
        Self {
            validator,
            alert_thresholds: thresholds,
            violation_history: Arc::new(Mutex::new(VecDeque::new())),
            max_history_size: 1000,
            monitoring_enabled: true,
        }
    }

    /// Enable or disable monitoring
    pub fn set_monitoring_enabled(&mut self, enabled: bool) {
        self.monitoring_enabled = enabled;
    }

    /// Check if monitoring is enabled
    pub fn is_monitoring_enabled(&self) -> bool {
        self.monitoring_enabled
    }

    /// Update alert thresholds
    pub fn update_thresholds(&mut self, thresholds: AlertThresholds) {
        self.alert_thresholds = thresholds;
    }

    /// Get current alert thresholds
    pub fn get_thresholds(&self) -> &AlertThresholds {
        &self.alert_thresholds
    }

    /// Check if alerts should be triggered
    pub fn check_alerts(&self) -> Vec<AlertType> {
        if !self.monitoring_enabled {
            return Vec::new();
        }

        let mut alerts = Vec::new();
        let metrics = self.validator.get_metrics();

        // Check memory leak threshold
        if metrics.metrics_snapshot.memory_leak_violations > self.alert_thresholds.max_memory_leaks {
            alerts.push(AlertType::MemoryLeak);
        }

        // Check buffer overflow threshold
        if metrics.metrics_snapshot.buffer_overflow_violations > self.alert_thresholds.max_buffer_overflows {
            alerts.push(AlertType::BufferOverflow);
        }

        // Check use-after-free threshold
        if metrics.metrics_snapshot.use_after_free_violations > self.alert_thresholds.max_use_after_free {
            alerts.push(AlertType::UseAfterFree);
        }

        // Check success rate threshold
        if metrics.success_rate() < self.alert_thresholds.min_success_rate {
            alerts.push(AlertType::LowSuccessRate);
        }

        // Check validation time threshold
        if metrics.metrics_snapshot.avg_validation_time_us > self.alert_thresholds.max_validation_time_us {
            alerts.push(AlertType::HighValidationTime);
        }

        // Check cache miss rate threshold
        if metrics.cache_hit_rate() < (1.0 - self.alert_thresholds.max_cache_miss_rate) {
            alerts.push(AlertType::HighCacheMissRate);
        }

        alerts
    }

    /// Add violation to history
    pub fn add_violation(&self, violation: MemorySafetyViolation) {
        if !self.monitoring_enabled {
            return;
        }

        if let Ok(mut history) = self.violation_history.lock() {
            history.push_back(violation);
            if history.len() > self.max_history_size {
                history.pop_front();
            }
        }
    }

    /// Get recent violations
    pub fn get_recent_violations(&self, count: usize) -> Vec<MemorySafetyViolation> {
        if let Ok(history) = self.violation_history.lock() {
            history.iter().rev().take(count).cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get violations by type
    pub fn get_violations_by_type(&self, violation_type: SafetyViolationType) -> Vec<MemorySafetyViolation> {
        if let Ok(history) = self.violation_history.lock() {
            history
                .iter()
                .filter(|v| v.violation_type == violation_type)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get violations by severity
    pub fn get_violations_by_severity(&self, severity: SafetyViolationSeverity) -> Vec<MemorySafetyViolation> {
        if let Ok(history) = self.violation_history.lock() {
            history
                .iter()
                .filter(|v| v.severity == severity)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear violation history
    pub fn clear_history(&self) {
        if let Ok(mut history) = self.violation_history.lock() {
            history.clear();
        }
    }

    /// Get monitoring statistics
    pub fn get_monitoring_stats(&self) -> MonitoringStats {
        let metrics = self.validator.get_metrics();
        let violation_count = if let Ok(history) = self.violation_history.lock() {
            history.len()
        } else {
            0
        };

        MonitoringStats {
            total_violations_tracked: violation_count,
            active_alerts: self.check_alerts(),
            monitoring_enabled: self.monitoring_enabled,
            validator_metrics: metrics,
            alert_thresholds: self.alert_thresholds.clone(),
            history_size: violation_count,
            max_history_size: self.max_history_size,
        }
    }

    /// Generate monitoring report
    pub fn generate_report(&self) -> MonitoringReport {
        let stats = self.get_monitoring_stats();
        let recent_violations = self.get_recent_violations(10);
        let critical_violations = self.get_violations_by_severity(SafetyViolationSeverity::Critical);

        MonitoringReport {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            stats,
            recent_violations,
            critical_violations: critical_violations.len(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// Generate recommendations based on current state
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let alerts = self.check_alerts();
        let metrics = self.validator.get_metrics();

        if alerts.contains(&AlertType::MemoryLeak) {
            recommendations.push("Consider implementing more aggressive garbage collection or reviewing allocation patterns".to_string());
        }

        if alerts.contains(&AlertType::BufferOverflow) {
            recommendations.push("Review buffer bounds checking and consider using safer memory operations".to_string());
        }

        if alerts.contains(&AlertType::UseAfterFree) {
            recommendations.push("Implement stricter lifetime management and consider using smart pointers".to_string());
        }

        if alerts.contains(&AlertType::LowSuccessRate) {
            recommendations.push("Review validation rules and consider adjusting thresholds or improving code quality".to_string());
        }

        if alerts.contains(&AlertType::HighValidationTime) {
            recommendations.push("Consider optimizing validation rules or increasing cache size".to_string());
        }

        if alerts.contains(&AlertType::HighCacheMissRate) {
            recommendations.push("Consider increasing cache size or reviewing cache key generation".to_string());
        }

        if metrics.cache_hit_rate() < 0.5 {
            recommendations.push("Cache hit rate is very low - consider reviewing caching strategy".to_string());
        }

        if metrics.tracked_allocations > 10000 {
            recommendations.push("High number of tracked allocations - consider periodic cleanup".to_string());
        }

        recommendations
    }

    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Trim existing history if necessary
        if let Ok(mut history) = self.violation_history.lock() {
            while history.len() > size {
                history.pop_front();
            }
        }
    }

    /// Get validator reference
    pub fn get_validator(&self) -> &Arc<MemorySafetyValidator> {
        &self.validator
    }
}

/// Alert types for memory safety violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
    /// Memory leak detected
    MemoryLeak,
    /// Buffer overflow detected
    BufferOverflow,
    /// Use-after-free detected
    UseAfterFree,
    /// Low success rate
    LowSuccessRate,
    /// High validation time
    HighValidationTime,
    /// High cache miss rate
    HighCacheMissRate,
}

impl AlertType {
    /// Get alert severity
    pub fn severity(&self) -> SafetyViolationSeverity {
        match self {
            AlertType::BufferOverflow | AlertType::UseAfterFree => SafetyViolationSeverity::Critical,
            AlertType::MemoryLeak => SafetyViolationSeverity::High,
            AlertType::LowSuccessRate | AlertType::HighValidationTime => SafetyViolationSeverity::Medium,
            AlertType::HighCacheMissRate => SafetyViolationSeverity::Low,
        }
    }

    /// Get alert description
    pub fn description(&self) -> &'static str {
        match self {
            AlertType::MemoryLeak => "Memory leak threshold exceeded",
            AlertType::BufferOverflow => "Buffer overflow threshold exceeded",
            AlertType::UseAfterFree => "Use-after-free threshold exceeded",
            AlertType::LowSuccessRate => "Validation success rate below threshold",
            AlertType::HighValidationTime => "Validation time above threshold",
            AlertType::HighCacheMissRate => "Cache miss rate above threshold",
        }
    }

    /// Check if alert requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(
            self.severity(),
            SafetyViolationSeverity::Critical | SafetyViolationSeverity::High
        )
    }
}

/// Monitoring statistics
#[derive(Debug, Clone)]
pub struct MonitoringStats {
    /// Total violations tracked
    pub total_violations_tracked: usize,
    /// Active alerts
    pub active_alerts: Vec<AlertType>,
    /// Monitoring enabled flag
    pub monitoring_enabled: bool,
    /// Validator metrics
    pub validator_metrics: ValidationMetrics,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Current history size
    pub history_size: usize,
    /// Maximum history size
    pub max_history_size: usize,
}

impl MonitoringStats {
    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.active_alerts.is_empty() && self.validator_metrics.is_performance_acceptable()
    }

    /// Get health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        if !self.monitoring_enabled {
            return 0.5; // Neutral score when monitoring is disabled
        }

        let mut score = 1.0;

        // Deduct points for active alerts
        for alert in &self.active_alerts {
            match alert.severity() {
                SafetyViolationSeverity::Critical => score -= 0.3,
                SafetyViolationSeverity::High => score -= 0.2,
                SafetyViolationSeverity::Medium => score -= 0.1,
                SafetyViolationSeverity::Low => score -= 0.05,
            }
        }

        // Adjust based on performance metrics
        let success_rate = self.validator_metrics.success_rate();
        if success_rate < 0.9 {
            score -= (0.9 - success_rate) * 0.5;
        }

        let cache_hit_rate = self.validator_metrics.cache_hit_rate();
        if cache_hit_rate < 0.8 {
            score -= (0.8 - cache_hit_rate) * 0.2;
        }

        score.max(0.0).min(1.0)
    }

    /// Get critical alert count
    pub fn critical_alert_count(&self) -> usize {
        self.active_alerts
            .iter()
            .filter(|alert| alert.severity() == SafetyViolationSeverity::Critical)
            .count()
    }

    /// Get high severity alert count
    pub fn high_alert_count(&self) -> usize {
        self.active_alerts
            .iter()
            .filter(|alert| alert.severity() == SafetyViolationSeverity::High)
            .count()
    }
}

/// Monitoring report
#[derive(Debug, Clone)]
pub struct MonitoringReport {
    /// Report timestamp
    pub timestamp: u64,
    /// Monitoring statistics
    pub stats: MonitoringStats,
    /// Recent violations
    pub recent_violations: Vec<MemorySafetyViolation>,
    /// Critical violations count
    pub critical_violations: usize,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl MonitoringReport {
    /// Check if report indicates critical issues
    pub fn has_critical_issues(&self) -> bool {
        self.critical_violations > 0 || self.stats.critical_alert_count() > 0
    }

    /// Get report summary
    pub fn get_summary(&self) -> String {
        let health_score = self.stats.health_score();
        let health_status = if health_score > 0.8 {
            "Healthy"
        } else if health_score > 0.6 {
            "Warning"
        } else if health_score > 0.4 {
            "Critical"
        } else {
            "Emergency"
        };

        format!(
            "Memory Safety Report - Status: {} (Score: {:.2}), Active Alerts: {}, Recent Violations: {}, Critical Issues: {}",
            health_status,
            health_score,
            self.stats.active_alerts.len(),
            self.recent_violations.len(),
            self.critical_violations
        )
    }

    /// Get detailed analysis
    pub fn get_detailed_analysis(&self) -> String {
        let mut analysis = String::new();
        
        analysis.push_str(&format!("=== Memory Safety Analysis Report ===\n"));
        analysis.push_str(&format!("Timestamp: {}\n", self.timestamp));
        analysis.push_str(&format!("Health Score: {:.2}\n", self.stats.health_score()));
        analysis.push_str(&format!("Monitoring Enabled: {}\n", self.stats.monitoring_enabled));
        analysis.push_str(&format!("\n"));

        analysis.push_str(&format!("=== Alert Summary ===\n"));
        analysis.push_str(&format!("Total Active Alerts: {}\n", self.stats.active_alerts.len()));
        analysis.push_str(&format!("Critical Alerts: {}\n", self.stats.critical_alert_count()));
        analysis.push_str(&format!("High Severity Alerts: {}\n", self.stats.high_alert_count()));
        analysis.push_str(&format!("\n"));

        if !self.stats.active_alerts.is_empty() {
            analysis.push_str(&format!("Active Alerts:\n"));
            for alert in &self.stats.active_alerts {
                analysis.push_str(&format!("  - {} ({}): {}\n", 
                    format!("{:?}", alert), 
                    format!("{:?}", alert.severity()),
                    alert.description()
                ));
            }
            analysis.push_str(&format!("\n"));
        }

        analysis.push_str(&format!("=== Performance Metrics ===\n"));
        let metrics = &self.stats.validator_metrics;
        analysis.push_str(&format!("Total Validations: {}\n", metrics.total_validations));
        analysis.push_str(&format!("Success Rate: {:.2}%\n", metrics.success_rate() * 100.0));
        analysis.push_str(&format!("Cache Hit Rate: {:.2}%\n", metrics.cache_hit_rate() * 100.0));
        analysis.push_str(&format!("Tracked Allocations: {}\n", metrics.tracked_allocations));
        analysis.push_str(&format!("\n"));

        if !self.recent_violations.is_empty() {
            analysis.push_str(&format!("=== Recent Violations ===\n"));
            for (i, violation) in self.recent_violations.iter().take(5).enumerate() {
                analysis.push_str(&format!("{}. {} ({}): {}\n", 
                    i + 1,
                    format!("{:?}", violation.violation_type),
                    format!("{:?}", violation.severity),
                    violation.message.as_str()
                ));
            }
            analysis.push_str(&format!("\n"));
        }

        if !self.recommendations.is_empty() {
            analysis.push_str(&format!("=== Recommendations ===\n"));
            for (i, recommendation) in self.recommendations.iter().enumerate() {
                analysis.push_str(&format!("{}. {}\n", i + 1, recommendation));
            }
        }

        analysis
    }
}
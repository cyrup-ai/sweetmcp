//! Health check report implementation
//!
//! This module provides blazing-fast health report generation with zero allocation
//! optimizations and elegant ergonomic interfaces for system health assessment.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::{debug, warn};

use super::health_types::{HealthIssue, HealthStatus, HealthTrend, IssueSeverity};
use super::health_metrics::{PerformanceMetrics, ResourceUtilization};
use super::super::optimization_recommendations::OptimizationRecommendation;

/// Health check report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckReport {
    /// Overall health score (0.0-1.0)
    pub overall_score: f64,
    /// Individual component scores
    pub component_scores: HashMap<String, f64>,
    /// Identified issues
    pub issues: Vec<HealthIssue>,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    /// Health trend direction
    pub trend: HealthTrend,
    /// Last check timestamp
    pub timestamp: SystemTime,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
}

impl HealthCheckReport {
    /// Create new health check report with zero allocation optimizations
    #[inline]
    pub fn new() -> Self {
        Self {
            overall_score: 0.0,
            component_scores: HashMap::new(),
            issues: Vec::new(),
            recommendations: Vec::new(),
            trend: HealthTrend::Stable,
            timestamp: SystemTime::now(),
            performance_metrics: PerformanceMetrics::default(),
            resource_utilization: ResourceUtilization::default(),
        }
    }

    /// Add component score
    #[inline]
    pub fn add_component_score(&mut self, component: String, score: f64) {
        self.component_scores.insert(component, score.clamp(0.0, 1.0));
    }

    /// Add health issue
    #[inline]
    pub fn add_issue(&mut self, issue: HealthIssue) {
        self.issues.push(issue);
    }

    /// Add recommendation
    #[inline]
    pub fn add_recommendation(&mut self, recommendation: OptimizationRecommendation) {
        self.recommendations.push(recommendation);
    }

    /// Calculate overall score from components with weighted averaging
    #[inline]
    pub fn calculate_overall_score(&mut self) {
        if self.component_scores.is_empty() {
            self.overall_score = 0.0;
            return;
        }

        // Weighted scoring based on component importance
        let weights = self.get_component_weights();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (component, &score) in &self.component_scores {
            let weight = weights.get(component).copied().unwrap_or(1.0);
            weighted_sum += score * weight;
            total_weight += weight;
        }

        self.overall_score = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        // Apply issue penalties
        let issue_penalty = self.calculate_issue_penalty();
        self.overall_score = (self.overall_score - issue_penalty).max(0.0);
    }

    /// Get component weights for scoring
    #[inline]
    fn get_component_weights(&self) -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        weights.insert("memory_usage".to_string(), 2.0);
        weights.insert("cache_performance".to_string(), 1.8);
        weights.insert("index_efficiency".to_string(), 1.6);
        weights.insert("fragmentation".to_string(), 1.4);
        weights.insert("compression".to_string(), 1.2);
        weights.insert("access_patterns".to_string(), 1.0);
        weights.insert("relationship_health".to_string(), 1.3);
        weights.insert("data_integrity".to_string(), 2.0);
        weights
    }

    /// Calculate penalty from issues
    #[inline]
    fn calculate_issue_penalty(&self) -> f64 {
        let mut penalty = 0.0;
        
        for issue in &self.issues {
            penalty += match issue.severity {
                IssueSeverity::Critical => 0.3,
                IssueSeverity::High => 0.2,
                IssueSeverity::Medium => 0.1,
                IssueSeverity::Low => 0.05,
            };
        }

        penalty.min(0.8) // Cap penalty at 80%
    }

    /// Get health status
    #[inline]
    pub fn health_status(&self) -> HealthStatus {
        if self.overall_score >= 0.9 {
            HealthStatus::Excellent
        } else if self.overall_score >= 0.7 {
            HealthStatus::Good
        } else if self.overall_score >= 0.5 {
            HealthStatus::Fair
        } else if self.overall_score >= 0.3 {
            HealthStatus::Poor
        } else {
            HealthStatus::Critical
        }
    }

    /// Get high priority recommendations
    #[inline]
    pub fn high_priority_recommendations(&self) -> Vec<&OptimizationRecommendation> {
        self.recommendations.iter()
            .filter(|r| r.is_high_priority())
            .collect()
    }

    /// Get critical issues
    #[inline]
    pub fn critical_issues(&self) -> Vec<&HealthIssue> {
        self.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect()
    }

    /// Get high severity issues
    #[inline]
    pub fn high_severity_issues(&self) -> Vec<&HealthIssue> {
        self.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
            .collect()
    }

    /// Check if immediate action is required
    #[inline]
    pub fn requires_immediate_action(&self) -> bool {
        self.overall_score < 0.3 || 
        !self.critical_issues().is_empty() ||
        self.has_performance_degradation()
    }

    /// Check if performance degradation is detected
    #[inline]
    pub fn has_performance_degradation(&self) -> bool {
        self.performance_metrics.response_time_ms > 1000.0 ||
        self.performance_metrics.throughput_ops_per_sec < 100.0 ||
        self.resource_utilization.memory_usage_percent > 90.0
    }

    /// Get health summary string
    #[inline]
    pub fn get_health_summary(&self) -> String {
        format!(
            "Health: {} ({:.1}%), Issues: {} critical, {} total, Recommendations: {}",
            self.health_status().description(),
            self.overall_score * 100.0,
            self.critical_issues().len(),
            self.issues.len(),
            self.recommendations.len()
        )
    }

    /// Get detailed health report
    #[inline]
    pub fn get_detailed_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("=== Memory Health Report ===\n"));
        report.push_str(&format!("Overall Score: {:.1}% ({})\n", 
                                self.overall_score * 100.0, 
                                self.health_status().description()));
        report.push_str(&format!("Trend: {:?}\n", self.trend));
        report.push_str(&format!("Timestamp: {:?}\n\n", self.timestamp));

        // Component scores
        report.push_str("Component Scores:\n");
        for (component, score) in &self.component_scores {
            report.push_str(&format!("  {}: {:.1}%\n", component, score * 100.0));
        }

        // Performance metrics
        report.push_str(&format!("\nPerformance Metrics:\n"));
        report.push_str(&format!("  Response Time: {:.1}ms\n", self.performance_metrics.response_time_ms));
        report.push_str(&format!("  Throughput: {:.1} ops/sec\n", self.performance_metrics.throughput_ops_per_sec));
        report.push_str(&format!("  Error Rate: {:.2}%\n", self.performance_metrics.error_rate_percent));

        // Resource utilization
        report.push_str(&format!("\nResource Utilization:\n"));
        report.push_str(&format!("  Memory: {:.1}%\n", self.resource_utilization.memory_usage_percent));
        report.push_str(&format!("  CPU: {:.1}%\n", self.resource_utilization.cpu_usage_percent));
        report.push_str(&format!("  Disk I/O: {:.1}%\n", self.resource_utilization.disk_io_percent));

        // Issues
        if !self.issues.is_empty() {
            report.push_str(&format!("\nIssues ({}):\n", self.issues.len()));
            for issue in &self.issues {
                report.push_str(&format!("  [{}] {}: {}\n", 
                                        issue.severity.short_name(),
                                        issue.component,
                                        issue.description));
            }
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            report.push_str(&format!("\nRecommendations ({}):\n", self.recommendations.len()));
            for rec in &self.recommendations {
                report.push_str(&format!("  [P{}] {}: {:.1}% improvement\n",
                                        rec.priority,
                                        rec.description,
                                        rec.expected_improvement));
            }
        }

        report
    }
}

impl Default for HealthCheckReport {
    fn default() -> Self {
        Self::new()
    }
}

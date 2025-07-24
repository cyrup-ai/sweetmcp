//! Health check module for memory optimization
//!
//! This module provides blazing-fast health monitoring with zero allocation
//! optimizations and elegant ergonomic interfaces for system health assessment.

pub mod health_report;
pub mod health_types;
pub mod health_metrics;
pub mod health_monitor;

// Re-export main types for ergonomic usage
pub use health_report::HealthCheckReport;
pub use health_types::{
    HealthIssue, IssueSeverity, IssueCategory, HealthStatus, HealthTrend, HealthScore
};
pub use health_metrics::{PerformanceMetrics, ResourceUtilization};
pub use health_monitor::{
    HealthMonitor, MonitoringThresholds, HealthSummaryStatistics
};

/// Builder for creating health check reports
pub struct HealthCheckReportBuilder {
    report: HealthCheckReport,
}

impl Default for HealthCheckReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCheckReportBuilder {
    /// Create new health check report builder
    #[inline]
    pub fn new() -> Self {
        Self {
            report: HealthCheckReport::new(),
        }
    }

    /// Add component score
    #[inline]
    pub fn with_component_score(mut self, component: String, score: f64) -> Self {
        self.report.add_component_score(component, score);
        self
    }

    /// Add health issue
    #[inline]
    pub fn with_issue(mut self, issue: HealthIssue) -> Self {
        self.report.add_issue(issue);
        self
    }

    /// Add recommendation
    #[inline]
    pub fn with_recommendation(mut self, recommendation: crate::memory::semantic::memory_optimization::optimization_recommendations::OptimizationRecommendation) -> Self {
        self.report.add_recommendation(recommendation);
        self
    }

    /// Set health trend
    #[inline]
    pub fn with_trend(mut self, trend: HealthTrend) -> Self {
        self.report.trend = trend;
        self
    }

    /// Set performance metrics
    #[inline]
    pub fn with_performance_metrics(mut self, metrics: PerformanceMetrics) -> Self {
        self.report.performance_metrics = metrics;
        self
    }

    /// Set resource utilization
    #[inline]
    pub fn with_resource_utilization(mut self, utilization: ResourceUtilization) -> Self {
        self.report.resource_utilization = utilization;
        self
    }

    /// Build the health check report
    #[inline]
    pub fn build(mut self) -> HealthCheckReport {
        self.report.calculate_overall_score();
        self.report
    }
}

/// Builder for creating health issues
pub struct HealthIssueBuilder {
    description: Option<String>,
    severity: Option<IssueSeverity>,
    component: Option<String>,
    performance_impact: Option<f64>,
    suggested_actions: Vec<String>,
}

impl Default for HealthIssueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthIssueBuilder {
    /// Create new health issue builder
    #[inline]
    pub fn new() -> Self {
        Self {
            description: None,
            severity: None,
            component: None,
            performance_impact: None,
            suggested_actions: Vec::new(),
        }
    }

    /// Set description
    #[inline]
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set severity
    #[inline]
    pub fn severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set component
    #[inline]
    pub fn component(mut self, component: String) -> Self {
        self.component = Some(component);
        self
    }

    /// Set performance impact
    #[inline]
    pub fn performance_impact(mut self, impact: f64) -> Self {
        self.performance_impact = Some(impact);
        self
    }

    /// Add suggested action
    #[inline]
    pub fn suggested_action(mut self, action: String) -> Self {
        self.suggested_actions.push(action);
        self
    }

    /// Build the health issue
    #[inline]
    pub fn build(self) -> Result<HealthIssue, &'static str> {
        let description = self.description.ok_or("Description is required")?;
        let severity = self.severity.ok_or("Severity is required")?;
        let component = self.component.ok_or("Component is required")?;
        let performance_impact = self.performance_impact.unwrap_or(0.0);

        let mut issue = HealthIssue::new(description, severity, component, performance_impact);
        for action in self.suggested_actions {
            issue.add_suggested_action(action);
        }

        Ok(issue)
    }
}

/// Convenience functions for creating common health issues
impl HealthIssue {
    /// Create memory usage issue
    #[inline]
    pub fn memory_usage_critical(usage_percent: f64) -> Self {
        let mut issue = Self::new(
            format!("Critical memory usage: {:.1}%", usage_percent),
            IssueSeverity::Critical,
            "memory_usage".to_string(),
            0.8,
        );
        issue.add_suggested_action("Reduce memory allocation".to_string());
        issue.add_suggested_action("Enable garbage collection".to_string());
        issue.add_suggested_action("Optimize data structures".to_string());
        issue
    }

    /// Create performance degradation issue
    #[inline]
    pub fn performance_degradation(response_time_ms: f64) -> Self {
        let mut issue = Self::new(
            format!("Performance degradation: {:.1}ms response time", response_time_ms),
            IssueSeverity::High,
            "performance".to_string(),
            0.6,
        );
        issue.add_suggested_action("Optimize query performance".to_string());
        issue.add_suggested_action("Review indexing strategy".to_string());
        issue.add_suggested_action("Check for resource contention".to_string());
        issue
    }

    /// Create cache efficiency issue
    #[inline]
    pub fn cache_efficiency_low(hit_rate_percent: f64) -> Self {
        let mut issue = Self::new(
            format!("Low cache hit rate: {:.1}%", hit_rate_percent),
            IssueSeverity::Medium,
            "cache_performance".to_string(),
            0.4,
        );
        issue.add_suggested_action("Increase cache size".to_string());
        issue.add_suggested_action("Optimize cache eviction policy".to_string());
        issue.add_suggested_action("Review access patterns".to_string());
        issue
    }

    /// Create fragmentation issue
    #[inline]
    pub fn fragmentation_high(fragmentation_percent: f64) -> Self {
        let mut issue = Self::new(
            format!("High memory fragmentation: {:.1}%", fragmentation_percent),
            IssueSeverity::Medium,
            "fragmentation".to_string(),
            0.5,
        );
        issue.add_suggested_action("Run defragmentation".to_string());
        issue.add_suggested_action("Optimize allocation patterns".to_string());
        issue.add_suggested_action("Consider memory pool usage".to_string());
        issue
    }
}

/// Convenience functions for creating performance metrics
impl PerformanceMetrics {
    /// Create metrics indicating excellent performance
    #[inline]
    pub fn excellent() -> Self {
        Self::with_values(50.0, 1000.0, 0.1, 10.0, 5.0)
    }

    /// Create metrics indicating good performance
    #[inline]
    pub fn good() -> Self {
        Self::with_values(200.0, 500.0, 0.5, 25.0, 15.0)
    }

    /// Create metrics indicating poor performance
    #[inline]
    pub fn poor() -> Self {
        Self::with_values(800.0, 100.0, 3.0, 75.0, 45.0)
    }

    /// Create metrics indicating critical performance
    #[inline]
    pub fn critical() -> Self {
        Self::with_values(1500.0, 50.0, 8.0, 150.0, 90.0)
    }
}

/// Convenience functions for creating resource utilization
impl ResourceUtilization {
    /// Create utilization indicating healthy resource usage
    #[inline]
    pub fn healthy() -> Self {
        Self::with_values(30.0, 25.0, 20.0, 15.0, 50, 10)
    }

    /// Create utilization indicating moderate resource usage
    #[inline]
    pub fn moderate() -> Self {
        Self::with_values(60.0, 55.0, 50.0, 45.0, 200, 25)
    }

    /// Create utilization indicating high resource usage
    #[inline]
    pub fn high() -> Self {
        Self::with_values(85.0, 80.0, 75.0, 70.0, 800, 75)
    }

    /// Create utilization indicating critical resource usage
    #[inline]
    pub fn critical() -> Self {
        Self::with_values(95.0, 92.0, 90.0, 88.0, 1500, 150)
    }
}

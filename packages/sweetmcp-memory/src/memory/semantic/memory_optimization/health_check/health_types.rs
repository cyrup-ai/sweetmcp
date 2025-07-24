//! Health check types and enumerations
//!
//! This module provides blazing-fast health type definitions with zero allocation
//! optimizations and elegant ergonomic interfaces for health status classification.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Health score for memory optimization assessment
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HealthScore {
    /// Score value (0.0-1.0, where 1.0 is perfect health)
    pub value: f64,
}

impl HealthScore {
    /// Create new health score
    pub fn new(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }
    
    /// Perfect health score
    pub fn perfect() -> Self {
        Self { value: 1.0 }
    }
    
    /// Critical health score
    pub fn critical() -> Self {
        Self { value: 0.0 }
    }
    
    /// Check if score indicates healthy state
    pub fn is_healthy(&self) -> bool {
        self.value >= 0.7
    }
    
    /// Check if score indicates critical state
    pub fn is_critical(&self) -> bool {
        self.value <= 0.3
    }
}

/// Health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: IssueSeverity,
    /// Affected component
    pub component: String,
    /// Impact on performance (0.0-1.0)
    pub performance_impact: f64,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
    /// Detection timestamp
    pub detected_at: SystemTime,
    /// Issue category
    pub category: IssueCategory,
}

impl HealthIssue {
    /// Create new health issue
    #[inline]
    pub fn new(
        description: String,
        severity: IssueSeverity,
        component: String,
        performance_impact: f64,
    ) -> Self {
        Self {
            description,
            severity,
            component: component.clone(),
            performance_impact: performance_impact.clamp(0.0, 1.0),
            suggested_actions: Vec::new(),
            detected_at: SystemTime::now(),
            category: IssueCategory::from_component(&component),
        }
    }

    /// Add suggested action
    #[inline]
    pub fn add_suggested_action(&mut self, action: String) {
        self.suggested_actions.push(action);
    }

    /// Check if issue is critical
    #[inline]
    pub fn is_critical(&self) -> bool {
        self.severity == IssueSeverity::Critical
    }

    /// Check if issue has high performance impact
    #[inline]
    pub fn has_high_performance_impact(&self) -> bool {
        self.performance_impact > 0.5
    }

    /// Get urgency score
    #[inline]
    pub fn urgency_score(&self) -> f64 {
        let severity_weight = match self.severity {
            IssueSeverity::Critical => 1.0,
            IssueSeverity::High => 0.8,
            IssueSeverity::Medium => 0.5,
            IssueSeverity::Low => 0.2,
        };

        severity_weight * self.performance_impact
    }
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl IssueSeverity {
    /// Get severity description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            IssueSeverity::Critical => "Critical - Immediate action required",
            IssueSeverity::High => "High - Action required within 24 hours",
            IssueSeverity::Medium => "Medium - Action required within a week",
            IssueSeverity::Low => "Low - Action can be scheduled",
        }
    }

    /// Get short name for display
    #[inline]
    pub fn short_name(&self) -> &'static str {
        match self {
            IssueSeverity::Critical => "CRIT",
            IssueSeverity::High => "HIGH",
            IssueSeverity::Medium => "MED",
            IssueSeverity::Low => "LOW",
        }
    }
}

/// Issue categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    Memory,
    Performance,
    DataIntegrity,
    Configuration,
    Resource,
    Network,
    Unknown,
}

impl IssueCategory {
    /// Create category from component name
    #[inline]
    pub fn from_component(component: &str) -> Self {
        match component.to_lowercase().as_str() {
            s if s.contains("memory") => IssueCategory::Memory,
            s if s.contains("cache") || s.contains("performance") => IssueCategory::Performance,
            s if s.contains("data") || s.contains("integrity") => IssueCategory::DataIntegrity,
            s if s.contains("config") => IssueCategory::Configuration,
            s if s.contains("resource") || s.contains("cpu") || s.contains("disk") => IssueCategory::Resource,
            s if s.contains("network") => IssueCategory::Network,
            _ => IssueCategory::Unknown,
        }
    }

    /// Get category description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            IssueCategory::Memory => "Memory Management",
            IssueCategory::Performance => "Performance Optimization",
            IssueCategory::DataIntegrity => "Data Integrity",
            IssueCategory::Configuration => "Configuration",
            IssueCategory::Resource => "Resource Management",
            IssueCategory::Network => "Network Operations",
            IssueCategory::Unknown => "Unknown Category",
        }
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl HealthStatus {
    /// Get status description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            HealthStatus::Excellent => "Excellent",
            HealthStatus::Good => "Good",
            HealthStatus::Fair => "Fair",
            HealthStatus::Poor => "Poor",
            HealthStatus::Critical => "Critical",
        }
    }

    /// Get color code for display
    #[inline]
    pub fn color_code(&self) -> &'static str {
        match self {
            HealthStatus::Excellent => "green",
            HealthStatus::Good => "lightgreen",
            HealthStatus::Fair => "yellow",
            HealthStatus::Poor => "orange",
            HealthStatus::Critical => "red",
        }
    }

    /// Get numeric score representation
    #[inline]
    pub fn numeric_score(&self) -> f64 {
        match self {
            HealthStatus::Excellent => 1.0,
            HealthStatus::Good => 0.8,
            HealthStatus::Fair => 0.6,
            HealthStatus::Poor => 0.4,
            HealthStatus::Critical => 0.2,
        }
    }

    /// Create from numeric score
    #[inline]
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            HealthStatus::Excellent
        } else if score >= 0.7 {
            HealthStatus::Good
        } else if score >= 0.5 {
            HealthStatus::Fair
        } else if score >= 0.3 {
            HealthStatus::Poor
        } else {
            HealthStatus::Critical
        }
    }
}

/// Health trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Declining,
    Unknown,
}

impl HealthTrend {
    /// Get trend description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            HealthTrend::Improving => "Improving",
            HealthTrend::Stable => "Stable",
            HealthTrend::Declining => "Declining",
            HealthTrend::Unknown => "Unknown",
        }
    }

    /// Get trend symbol for display
    #[inline]
    pub fn symbol(&self) -> &'static str {
        match self {
            HealthTrend::Improving => "↗",
            HealthTrend::Stable => "→",
            HealthTrend::Declining => "↘",
            HealthTrend::Unknown => "?",
        }
    }

    /// Get trend color code
    #[inline]
    pub fn color_code(&self) -> &'static str {
        match self {
            HealthTrend::Improving => "green",
            HealthTrend::Stable => "blue",
            HealthTrend::Declining => "red",
            HealthTrend::Unknown => "gray",
        }
    }

    /// Calculate trend from score history
    #[inline]
    pub fn calculate_from_scores(scores: &[f64]) -> Self {
        if scores.len() < 3 {
            return HealthTrend::Unknown;
        }

        let mid_point = scores.len() / 2;
        let first_half_avg: f64 = scores[..mid_point].iter().sum::<f64>() / mid_point as f64;
        let second_half_avg: f64 = scores[mid_point..].iter().sum::<f64>() / (scores.len() - mid_point) as f64;

        let difference = second_half_avg - first_half_avg;

        if difference > 0.05 {
            HealthTrend::Improving
        } else if difference < -0.05 {
            HealthTrend::Declining
        } else {
            HealthTrend::Stable
        }
    }
}

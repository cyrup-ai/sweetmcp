//! Quantum entanglement engine issue types and severity management
//!
//! This module provides issue classification and severity management types
//! with zero-allocation patterns and blazing-fast performance.

/// Network issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl IssueSeverity {
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            IssueSeverity::Info => "Informational - no action required",
            IssueSeverity::Warning => "Warning - monitor closely",
            IssueSeverity::Error => "Error - action recommended",
            IssueSeverity::Critical => "Critical - immediate action required",
        }
    }

    /// Get color code
    pub fn color_code(&self) -> &'static str {
        match self {
            IssueSeverity::Info => "blue",
            IssueSeverity::Warning => "yellow",
            IssueSeverity::Error => "orange",
            IssueSeverity::Critical => "red",
        }
    }

    /// Get priority value
    pub fn priority_value(&self) -> u8 {
        match self {
            IssueSeverity::Info => 1,
            IssueSeverity::Warning => 2,
            IssueSeverity::Error => 3,
            IssueSeverity::Critical => 4,
        }
    }

    /// Check if severity requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, IssueSeverity::Critical | IssueSeverity::Error)
    }

    /// Get recommended response time in minutes
    pub fn response_time_minutes(&self) -> u32 {
        match self {
            IssueSeverity::Info => 1440,      // 24 hours
            IssueSeverity::Warning => 240,    // 4 hours
            IssueSeverity::Error => 60,       // 1 hour
            IssueSeverity::Critical => 5,     // 5 minutes
        }
    }

    /// Create severity from health score
    pub fn from_health_score(health_score: f64) -> Self {
        if health_score < 25.0 {
            IssueSeverity::Critical
        } else if health_score < 50.0 {
            IssueSeverity::Error
        } else if health_score < 75.0 {
            IssueSeverity::Warning
        } else {
            IssueSeverity::Info
        }
    }
}

/// Network issue with metadata
#[derive(Debug, Clone)]
pub struct NetworkIssue {
    pub description: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub recommendation: String,
    pub detected_at: std::time::SystemTime,
    pub affected_nodes: Vec<String>,
    pub impact_score: f64,
}

impl NetworkIssue {
    /// Create new network issue
    pub fn new(
        description: String,
        severity: IssueSeverity,
        category: IssueCategory,
        recommendation: String,
    ) -> Self {
        Self {
            description,
            severity,
            category,
            recommendation,
            detected_at: std::time::SystemTime::now(),
            affected_nodes: Vec::new(),
            impact_score: 0.0,
        }
    }

    /// Create new network issue with affected nodes
    pub fn with_affected_nodes(
        description: String,
        severity: IssueSeverity,
        category: IssueCategory,
        recommendation: String,
        affected_nodes: Vec<String>,
        impact_score: f64,
    ) -> Self {
        Self {
            description,
            severity,
            category,
            recommendation,
            detected_at: std::time::SystemTime::now(),
            affected_nodes,
            impact_score,
        }
    }

    /// Check if issue requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        self.severity.requires_immediate_attention() || self.impact_score > 0.8
    }

    /// Get formatted issue summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {}: {} (Impact: {:.1}%)",
            self.severity.description(),
            self.category.name(),
            self.description,
            self.impact_score * 100.0
        )
    }

    /// Get detailed issue report
    pub fn detailed_report(&self) -> String {
        let affected_nodes_str = if self.affected_nodes.is_empty() {
            "None specified".to_string()
        } else {
            self.affected_nodes.join(", ")
        };

        format!(
            "Issue: {}\nSeverity: {} ({})\nCategory: {} - {}\nImpact Score: {:.1}%\nAffected Nodes: {}\nRecommendation: {}\nDetected: {:?}",
            self.description,
            self.severity.description(),
            self.severity.color_code(),
            self.category.name(),
            self.category.description(),
            self.impact_score * 100.0,
            affected_nodes_str,
            self.recommendation,
            self.detected_at
        )
    }

    /// Get priority for resolution
    pub fn resolution_priority(&self) -> super::engine_health_types::OptimizationPriority {
        match self.severity {
            IssueSeverity::Critical => super::engine_health_types::OptimizationPriority::Critical,
            IssueSeverity::Error => super::engine_health_types::OptimizationPriority::High,
            IssueSeverity::Warning => super::engine_health_types::OptimizationPriority::Medium,
            IssueSeverity::Info => super::engine_health_types::OptimizationPriority::Low,
        }
    }

    /// Check if issue affects network connectivity
    pub fn affects_connectivity(&self) -> bool {
        matches!(self.category, IssueCategory::Connectivity) || 
        self.description.to_lowercase().contains("disconnect") ||
        self.description.to_lowercase().contains("unreachable")
    }

    /// Check if issue affects performance
    pub fn affects_performance(&self) -> bool {
        matches!(self.category, IssueCategory::Performance) ||
        self.impact_score > 0.5
    }

    /// Get estimated resolution time in minutes
    pub fn estimated_resolution_time(&self) -> u32 {
        let base_time = match self.severity {
            IssueSeverity::Info => 30,
            IssueSeverity::Warning => 60,
            IssueSeverity::Error => 120,
            IssueSeverity::Critical => 240,
        };

        let complexity_multiplier = match self.category {
            IssueCategory::Configuration => 1.0,
            IssueCategory::Performance => 1.5,
            IssueCategory::Connectivity => 2.0,
            IssueCategory::Scalability => 2.5,
            IssueCategory::Reliability => 3.0,
        };

        let node_factor = if self.affected_nodes.len() > 10 {
            1.5
        } else if self.affected_nodes.len() > 5 {
            1.2
        } else {
            1.0
        };

        (base_time as f64 * complexity_multiplier * node_factor) as u32
    }
}

/// Categories of network issues
#[derive(Debug, Clone, PartialEq)]
pub enum IssueCategory {
    Connectivity,
    Performance,
    Scalability,
    Reliability,
    Configuration,
}

impl IssueCategory {
    /// Get category name
    pub fn name(&self) -> &'static str {
        match self {
            IssueCategory::Connectivity => "Connectivity",
            IssueCategory::Performance => "Performance",
            IssueCategory::Scalability => "Scalability",
            IssueCategory::Reliability => "Reliability",
            IssueCategory::Configuration => "Configuration",
        }
    }

    /// Get category description
    pub fn description(&self) -> &'static str {
        match self {
            IssueCategory::Connectivity => "Issues related to network connectivity and reachability",
            IssueCategory::Performance => "Issues affecting network performance and efficiency",
            IssueCategory::Scalability => "Issues limiting network growth and scalability",
            IssueCategory::Reliability => "Issues affecting network reliability and robustness",
            IssueCategory::Configuration => "Issues with network configuration and settings",
        }
    }

    /// Get category icon
    pub fn icon(&self) -> &'static str {
        match self {
            IssueCategory::Connectivity => "🔗",
            IssueCategory::Performance => "⚡",
            IssueCategory::Scalability => "📈",
            IssueCategory::Reliability => "🛡️",
            IssueCategory::Configuration => "⚙️",
        }
    }

    /// Get typical resolution strategies
    pub fn resolution_strategies(&self) -> Vec<&'static str> {
        match self {
            IssueCategory::Connectivity => vec![
                "Create bridging entanglements",
                "Repair broken connections",
                "Add redundant paths",
                "Optimize routing tables",
            ],
            IssueCategory::Performance => vec![
                "Optimize entanglement strengths",
                "Balance network load",
                "Reduce path lengths",
                "Eliminate bottlenecks",
            ],
            IssueCategory::Scalability => vec![
                "Add network capacity",
                "Implement hierarchical structure",
                "Optimize resource allocation",
                "Plan for growth patterns",
            ],
            IssueCategory::Reliability => vec![
                "Add redundancy",
                "Implement failover mechanisms",
                "Monitor critical paths",
                "Create backup routes",
            ],
            IssueCategory::Configuration => vec![
                "Adjust parameters",
                "Update thresholds",
                "Recalibrate algorithms",
                "Validate settings",
            ],
        }
    }

    /// Get priority weight for this category
    pub fn priority_weight(&self) -> f64 {
        match self {
            IssueCategory::Connectivity => 0.9,    // Highest priority
            IssueCategory::Reliability => 0.8,     // High priority
            IssueCategory::Performance => 0.7,     // Medium-high priority
            IssueCategory::Scalability => 0.6,     // Medium priority
            IssueCategory::Configuration => 0.5,   // Lower priority
        }
    }

    /// Check if category is critical for network operation
    pub fn is_critical(&self) -> bool {
        matches!(self, IssueCategory::Connectivity | IssueCategory::Reliability)
    }
}
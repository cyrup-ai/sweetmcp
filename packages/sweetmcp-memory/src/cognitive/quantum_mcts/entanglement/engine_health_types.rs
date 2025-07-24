//! Quantum entanglement engine health monitoring types
//!
//! This module provides health assessment data types and enumerations
//! with zero-allocation patterns and blazing-fast performance.

/// Critical node information
#[derive(Debug, Clone)]
pub struct CriticalNode {
    pub node_id: String,
    pub criticality_score: f64,
    pub criticality_type: CriticalityType,
    pub reason: String,
}

impl CriticalNode {
    /// Check if node requires immediate attention
    pub fn requires_attention(&self) -> bool {
        self.criticality_score >= 0.8
    }

    /// Get priority level for addressing this critical node
    pub fn priority_level(&self) -> OptimizationPriority {
        if self.criticality_score >= 0.9 {
            OptimizationPriority::Critical
        } else if self.criticality_score >= 0.7 {
            OptimizationPriority::High
        } else if self.criticality_score >= 0.5 {
            OptimizationPriority::Medium
        } else {
            OptimizationPriority::Low
        }
    }
}

/// Types of node criticality
#[derive(Debug, Clone, PartialEq)]
pub enum CriticalityType {
    Hub,        // High-degree node
    Bridge,     // Connects different components
    Bottleneck, // Critical for information flow
    Isolate,    // Poorly connected node
}

impl CriticalityType {
    /// Get description of criticality type
    pub fn description(&self) -> &'static str {
        match self {
            CriticalityType::Hub => "High-degree hub node with many connections",
            CriticalityType::Bridge => "Bridge node connecting different network components",
            CriticalityType::Bottleneck => "Bottleneck node critical for information flow",
            CriticalityType::Isolate => "Isolated node with poor connectivity",
        }
    }

    /// Get recommended action for this criticality type
    pub fn recommended_action(&self) -> &'static str {
        match self {
            CriticalityType::Hub => "Monitor for overload and consider load balancing",
            CriticalityType::Bridge => "Ensure redundancy and backup connections",
            CriticalityType::Bottleneck => "Optimize routing and add parallel paths",
            CriticalityType::Isolate => "Improve connectivity with strategic entanglements",
        }
    }

    /// Get severity level for this criticality type
    pub fn severity_level(&self) -> u8 {
        match self {
            CriticalityType::Bottleneck => 4, // Highest severity
            CriticalityType::Bridge => 3,     // High severity
            CriticalityType::Hub => 2,        // Medium severity
            CriticalityType::Isolate => 1,    // Low severity
        }
    }

    /// Check if this criticality type affects network connectivity
    pub fn affects_connectivity(&self) -> bool {
        matches!(self, CriticalityType::Bridge | CriticalityType::Bottleneck)
    }

    /// Check if this criticality type affects performance
    pub fn affects_performance(&self) -> bool {
        matches!(self, CriticalityType::Hub | CriticalityType::Bottleneck)
    }
}

/// Optimization priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl OptimizationPriority {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationPriority::Low => "Low priority - network is performing well",
            OptimizationPriority::Medium => "Medium priority - minor improvements needed",
            OptimizationPriority::High => "High priority - significant issues detected",
            OptimizationPriority::Critical => "Critical priority - immediate optimization required",
        }
    }

    /// Check if optimization should be performed immediately
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, OptimizationPriority::Critical | OptimizationPriority::High)
    }

    /// Get recommended time frame for addressing this priority
    pub fn time_frame(&self) -> &'static str {
        match self {
            OptimizationPriority::Low => "Within next maintenance window",
            OptimizationPriority::Medium => "Within 24 hours",
            OptimizationPriority::High => "Within 1 hour",
            OptimizationPriority::Critical => "Immediately",
        }
    }

    /// Get numeric priority value for sorting
    pub fn priority_value(&self) -> u8 {
        match self {
            OptimizationPriority::Low => 1,
            OptimizationPriority::Medium => 2,
            OptimizationPriority::High => 3,
            OptimizationPriority::Critical => 4,
        }
    }

    /// Get color code for UI display
    pub fn color_code(&self) -> &'static str {
        match self {
            OptimizationPriority::Low => "green",
            OptimizationPriority::Medium => "yellow",
            OptimizationPriority::High => "orange",
            OptimizationPriority::Critical => "red",
        }
    }

    /// Create priority from health score
    pub fn from_health_score(health_score: f64) -> Self {
        if health_score < 50.0 {
            OptimizationPriority::Critical
        } else if health_score < 70.0 {
            OptimizationPriority::High
        } else if health_score < 85.0 {
            OptimizationPriority::Medium
        } else {
            OptimizationPriority::Low
        }
    }

    /// Create priority from issue count
    pub fn from_issue_count(issue_count: usize) -> Self {
        if issue_count >= 5 {
            OptimizationPriority::Critical
        } else if issue_count >= 3 {
            OptimizationPriority::High
        } else if issue_count >= 1 {
            OptimizationPriority::Medium
        } else {
            OptimizationPriority::Low
        }
    }

    /// Combine multiple priorities and return the highest
    pub fn combine(priorities: &[OptimizationPriority]) -> Self {
        priorities.iter().max().cloned().unwrap_or(OptimizationPriority::Low)
    }
}

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthStatus {
    Critical,
    Poor,
    Fair,
    Good,
    Excellent,
}

impl HealthStatus {
    /// Create health status from score
    pub fn from_score(score: f64) -> Self {
        if score >= 90.0 {
            HealthStatus::Excellent
        } else if score >= 75.0 {
            HealthStatus::Good
        } else if score >= 50.0 {
            HealthStatus::Fair
        } else if score >= 25.0 {
            HealthStatus::Poor
        } else {
            HealthStatus::Critical
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            HealthStatus::Critical => "Critical - immediate attention required",
            HealthStatus::Poor => "Poor - significant issues present",
            HealthStatus::Fair => "Fair - some improvements needed",
            HealthStatus::Good => "Good - minor optimizations possible",
            HealthStatus::Excellent => "Excellent - optimal performance",
        }
    }

    /// Get color code
    pub fn color_code(&self) -> &'static str {
        match self {
            HealthStatus::Critical => "red",
            HealthStatus::Poor => "orange",
            HealthStatus::Fair => "yellow",
            HealthStatus::Good => "lightgreen",
            HealthStatus::Excellent => "green",
        }
    }

    /// Check if status indicates problems
    pub fn has_problems(&self) -> bool {
        matches!(self, HealthStatus::Critical | HealthStatus::Poor | HealthStatus::Fair)
    }

    /// Get recommended action
    pub fn recommended_action(&self) -> &'static str {
        match self {
            HealthStatus::Critical => "Immediate optimization required",
            HealthStatus::Poor => "Schedule optimization soon",
            HealthStatus::Fair => "Consider optimization during maintenance",
            HealthStatus::Good => "Monitor and maintain current state",
            HealthStatus::Excellent => "Continue current practices",
        }
    }
}
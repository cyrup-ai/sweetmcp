//! Bottleneck detection types and utilities
//!
//! This module provides bottleneck detection data structures
//! with zero-allocation patterns and blazing-fast performance.

use super::node_search_types::NodeMatch;

/// Bottleneck node information
#[derive(Debug, Clone)]
pub struct BottleneckNode {
    pub node_id: String,
    pub bottleneck_type: BottleneckType,
    pub severity: BottleneckSeverity,
    pub description: String,
}

impl BottleneckNode {
    /// Create new bottleneck node
    #[inline]
    pub fn new(
        node_id: String,
        bottleneck_type: BottleneckType,
        severity: BottleneckSeverity,
        description: String,
    ) -> Self {
        Self {
            node_id,
            bottleneck_type,
            severity,
            description,
        }
    }

    /// Get formatted summary
    pub fn summary(&self) -> String {
        format!(
            "[{}] {} ({}): {}",
            self.severity,
            self.bottleneck_type.description(),
            self.node_id,
            self.description
        )
    }

    /// Check if this is a high severity bottleneck
    #[inline]
    pub fn is_high_severity(&self) -> bool {
        matches!(self.severity, BottleneckSeverity::High)
    }

    /// Check if this is a critical bottleneck type
    #[inline]
    pub fn is_critical_type(&self) -> bool {
        matches!(
            self.bottleneck_type,
            BottleneckType::LowRewardHighVisits | BottleneckType::UnbalancedChildren
        )
    }

    /// Get priority score for sorting bottlenecks
    #[inline]
    pub fn priority_score(&self) -> u8 {
        let severity_score = self.severity.priority_score();
        let type_score = if self.is_critical_type() { 2 } else { 1 };
        severity_score * type_score
    }

    /// Check if this bottleneck requires immediate attention
    #[inline]
    pub fn requires_immediate_attention(&self) -> bool {
        self.is_high_severity() && self.is_critical_type()
    }
}

/// Types of bottlenecks in MCTS trees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckType {
    LowRewardHighVisits,
    SingleChild,
    UnbalancedChildren,
    DeepLowVisits,
}

impl BottleneckType {
    /// Get description of bottleneck type
    pub fn description(&self) -> &'static str {
        match self {
            BottleneckType::LowRewardHighVisits => "Low reward despite high visits",
            BottleneckType::SingleChild => "Single child limiting exploration",
            BottleneckType::UnbalancedChildren => "Unbalanced child visit distribution",
            BottleneckType::DeepLowVisits => "Deep node with insufficient visits",
        }
    }

    /// Get suggested action for this bottleneck type
    pub fn suggested_action(&self) -> &'static str {
        match self {
            BottleneckType::LowRewardHighVisits => "Consider pruning or reducing exploration",
            BottleneckType::SingleChild => "Investigate why only one child exists",
            BottleneckType::UnbalancedChildren => "Rebalance exploration across children",
            BottleneckType::DeepLowVisits => "Increase exploration at this depth",
        }
    }

    /// Get impact level of this bottleneck type
    pub fn impact_level(&self) -> BottleneckImpact {
        match self {
            BottleneckType::LowRewardHighVisits => BottleneckImpact::High,
            BottleneckType::UnbalancedChildren => BottleneckImpact::High,
            BottleneckType::SingleChild => BottleneckImpact::Medium,
            BottleneckType::DeepLowVisits => BottleneckImpact::Low,
        }
    }

    /// Check if this bottleneck type affects exploration
    #[inline]
    pub fn affects_exploration(&self) -> bool {
        matches!(
            self,
            BottleneckType::SingleChild | BottleneckType::UnbalancedChildren | BottleneckType::DeepLowVisits
        )
    }

    /// Check if this bottleneck type affects exploitation
    #[inline]
    pub fn affects_exploitation(&self) -> bool {
        matches!(self, BottleneckType::LowRewardHighVisits)
    }
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
}

impl BottleneckSeverity {
    /// Get color representation for display
    pub fn color(&self) -> &'static str {
        match self {
            BottleneckSeverity::Low => "green",
            BottleneckSeverity::Medium => "yellow",
            BottleneckSeverity::High => "red",
        }
    }

    /// Get priority score (higher = more urgent)
    #[inline]
    pub fn priority_score(&self) -> u8 {
        match self {
            BottleneckSeverity::Low => 1,
            BottleneckSeverity::Medium => 2,
            BottleneckSeverity::High => 3,
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            BottleneckSeverity::Low => "🟢",
            BottleneckSeverity::Medium => "🟡",
            BottleneckSeverity::High => "🔴",
        }
    }
}

impl std::fmt::Display for BottleneckSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BottleneckSeverity::Low => write!(f, "Low"),
            BottleneckSeverity::Medium => write!(f, "Medium"),
            BottleneckSeverity::High => write!(f, "High"),
        }
    }
}

/// Bottleneck impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BottleneckImpact {
    Low,
    Medium,
    High,
}

impl BottleneckImpact {
    /// Get description of impact level
    pub fn description(&self) -> &'static str {
        match self {
            BottleneckImpact::Low => "Minor impact on performance",
            BottleneckImpact::Medium => "Moderate impact on performance",
            BottleneckImpact::High => "Significant impact on performance",
        }
    }
}

/// Collection of bottleneck analysis results
#[derive(Debug, Clone, Default)]
pub struct BottleneckAnalysis {
    pub bottlenecks: Vec<BottleneckNode>,
    pub total_nodes_analyzed: usize,
    pub high_severity_count: usize,
    pub medium_severity_count: usize,
    pub low_severity_count: usize,
}

impl BottleneckAnalysis {
    /// Create new bottleneck analysis
    pub fn new(bottlenecks: Vec<BottleneckNode>, total_nodes_analyzed: usize) -> Self {
        let high_severity_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::High)).count();
        let medium_severity_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Medium)).count();
        let low_severity_count = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Low)).count();

        Self {
            bottlenecks,
            total_nodes_analyzed,
            high_severity_count,
            medium_severity_count,
            low_severity_count,
        }
    }

    /// Get total bottleneck count
    #[inline]
    pub fn total_count(&self) -> usize {
        self.bottlenecks.len()
    }

    /// Get bottleneck percentage
    #[inline]
    pub fn bottleneck_percentage(&self) -> f64 {
        if self.total_nodes_analyzed == 0 {
            0.0
        } else {
            (self.total_count() as f64 / self.total_nodes_analyzed as f64) * 100.0
        }
    }

    /// Get critical bottlenecks (high severity)
    pub fn critical_bottlenecks(&self) -> Vec<&BottleneckNode> {
        self.bottlenecks.iter().filter(|b| b.is_high_severity()).collect()
    }

    /// Get bottlenecks by type
    pub fn bottlenecks_by_type(&self, bottleneck_type: BottleneckType) -> Vec<&BottleneckNode> {
        self.bottlenecks.iter().filter(|b| b.bottleneck_type == bottleneck_type).collect()
    }

    /// Get summary report
    pub fn summary_report(&self) -> String {
        format!(
            "Bottleneck Analysis Summary:\n\
            Total nodes analyzed: {}\n\
            Bottlenecks found: {} ({:.1}%)\n\
            - High severity: {}\n\
            - Medium severity: {}\n\
            - Low severity: {}\n\
            \n\
            Critical bottlenecks requiring immediate attention: {}",
            self.total_nodes_analyzed,
            self.total_count(),
            self.bottleneck_percentage(),
            self.high_severity_count,
            self.medium_severity_count,
            self.low_severity_count,
            self.critical_bottlenecks().len()
        )
    }

    /// Check if tree has critical performance issues
    #[inline]
    pub fn has_critical_issues(&self) -> bool {
        self.high_severity_count > 0
    }

    /// Get recommended actions
    pub fn recommended_actions(&self) -> Vec<String> {
        let mut actions = Vec::new();
        
        if self.high_severity_count > 0 {
            actions.push("Address high severity bottlenecks immediately".to_string());
        }
        
        if self.bottleneck_percentage() > 20.0 {
            actions.push("High bottleneck percentage indicates systematic issues".to_string());
        }
        
        let exploration_issues = self.bottlenecks.iter().filter(|b| b.bottleneck_type.affects_exploration()).count();
        if exploration_issues > self.total_count() / 2 {
            actions.push("Consider adjusting exploration parameters".to_string());
        }
        
        let exploitation_issues = self.bottlenecks.iter().filter(|b| b.bottleneck_type.affects_exploitation()).count();
        if exploitation_issues > self.total_count() / 2 {
            actions.push("Consider adjusting exploitation parameters".to_string());
        }
        
        actions
    }
}
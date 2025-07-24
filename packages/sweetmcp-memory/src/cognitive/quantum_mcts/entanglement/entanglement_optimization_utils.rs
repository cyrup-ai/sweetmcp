//! Optimization utilities for quantum entanglement system
//!
//! This module provides optimization algorithms and utility functions for
//! analyzing and improving quantum entanglement networks with zero allocation
//! patterns and blazing-fast performance.

use super::entanglement_types::{
    OptimizationStrategy, OptimizationUrgency, ComprehensiveHealthReport, 
    HealthTrend, OptimizationContext, NetworkTopology, EntanglementMetrics
};
use super::engine_issue_collection::IssueCollection;

/// Calculate network health score from multiple metrics
pub fn calculate_composite_health_score(
    topology: &NetworkTopology,
    metrics: &EntanglementMetrics,
    issues: &IssueCollection,
) -> f64 {
    let topology_score = calculate_topology_health_score(topology);
    let metrics_score = calculate_metrics_health_score(metrics);
    let issues_score = issues.health_score();
    
    // Weighted average: topology 40%, metrics 30%, issues 30%
    (topology_score * 0.4) + (metrics_score * 0.3) + (issues_score * 0.3)
}

/// Calculate health score from topology metrics
pub fn calculate_topology_health_score(topology: &NetworkTopology) -> f64 {
    let mut score = 0.0;
    
    // Connectivity (25% weight)
    score += if topology.is_connected { 0.25 } else { 0.0 };
    
    // Density (25% weight) - optimal around 0.3
    let density_score = 1.0 - (topology.network_density - 0.3).abs() / 0.3;
    score += density_score.max(0.0) * 0.25;
    
    // Clustering (25% weight) - optimal around 0.5
    let clustering_score = 1.0 - (topology.clustering_coefficient - 0.5).abs() / 0.5;
    score += clustering_score.max(0.0) * 0.25;
    
    // Path length (25% weight) - shorter is better, but not too short
    let path_score = if topology.average_path_length > 0.0 {
        let optimal_path = (topology.total_nodes as f64).log2().max(2.0);
        1.0 - (topology.average_path_length - optimal_path).abs() / optimal_path
    } else {
        0.0
    };
    score += path_score.max(0.0) * 0.25;
    
    score.min(1.0).max(0.0)
}

/// Calculate health score from entanglement metrics
pub fn calculate_metrics_health_score(metrics: &EntanglementMetrics) -> f64 {
    let mut score = 1.0;
    
    // Penalize if no entanglements exist
    if metrics.total_entanglements == 0 {
        return 0.0;
    }
    
    // Reward balanced strength distribution
    if metrics.average_strength > 0.0 {
        let strength_balance = 1.0 - (metrics.max_strength - metrics.average_strength) / metrics.max_strength;
        score *= strength_balance.max(0.5); // Don't penalize too heavily
    }
    
    // Reward reasonable entanglement density
    let density_factor = if metrics.total_entanglements > 1000 {
        0.9 // Slight penalty for very large networks
    } else if metrics.total_entanglements < 10 {
        0.8 // Penalty for very small networks
    } else {
        1.0
    };
    score *= density_factor;
    
    score.min(1.0).max(0.0)
}

/// Recommend optimization strategy based on network state
pub fn recommend_optimization_strategy(
    topology: &NetworkTopology,
    metrics: &EntanglementMetrics,
    issues: &IssueCollection,
) -> OptimizationStrategy {
    let health_score = calculate_composite_health_score(topology, metrics, issues);
    
    if health_score < 0.3 {
        OptimizationStrategy::Emergency
    } else if health_score < 0.6 {
        OptimizationStrategy::Aggressive
    } else if health_score < 0.8 {
        OptimizationStrategy::Moderate
    } else {
        OptimizationStrategy::Maintenance
    }
}

/// Calculate optimization urgency level
pub fn calculate_optimization_urgency(
    topology: &NetworkTopology,
    issues: &IssueCollection,
) -> OptimizationUrgency {
    let critical_issues = issues.critical_issues();
    let connectivity_issues = issues.connectivity_issues();
    
    if !critical_issues.is_empty() || !topology.is_connected {
        OptimizationUrgency::Immediate
    } else if connectivity_issues.len() > 2 || topology.network_density < 0.05 {
        OptimizationUrgency::High
    } else if issues.issues.len() > 5 || topology.clustering_coefficient < 0.1 {
        OptimizationUrgency::Medium
    } else {
        OptimizationUrgency::Low
    }
}

/// Create optimization context from current network state
pub fn create_optimization_context(
    topology: &NetworkTopology,
    metrics: &EntanglementMetrics,
    issues: &IssueCollection,
    historical_reports: &[ComprehensiveHealthReport],
) -> OptimizationContext {
    let strategy = recommend_optimization_strategy(topology, metrics, issues);
    let urgency = calculate_optimization_urgency(topology, issues);
    let trend = analyze_health_trends(historical_reports);
    
    OptimizationContext::new(strategy, urgency, trend)
}

/// Analyze health trends from historical reports
pub fn analyze_health_trends(
    historical_reports: &[ComprehensiveHealthReport],
) -> HealthTrend {
    if historical_reports.len() < 2 {
        return HealthTrend::Insufficient;
    }
    
    let recent_health = historical_reports.last().unwrap().analysis_report.health.overall_health;
    let previous_health = historical_reports[historical_reports.len() - 2].analysis_report.health.overall_health;
    
    let change = recent_health - previous_health;
    
    if change > 5.0 {
        HealthTrend::Improving
    } else if change < -5.0 {
        HealthTrend::Declining
    } else {
        HealthTrend::Stable
    }
}

/// Calculate optimization priority score
pub fn calculate_optimization_priority(
    strategy: &OptimizationStrategy,
    urgency: &OptimizationUrgency,
    trend: &HealthTrend,
) -> f64 {
    let strategy_weight = strategy.intensity_level();
    let urgency_weight = urgency.priority_score() as f64 / 4.0;
    let trend_weight = if trend.indicates_problems() { 1.0 } else { 0.5 };
    
    (strategy_weight * 0.4) + (urgency_weight * 0.4) + (trend_weight * 0.2)
}

/// Calculate network efficiency score
pub fn calculate_network_efficiency(topology: &NetworkTopology) -> f64 {
    if topology.total_nodes <= 1 {
        return 1.0;
    }
    
    let theoretical_max_edges = topology.total_nodes * (topology.total_nodes - 1) / 2;
    let edge_efficiency = if theoretical_max_edges > 0 {
        topology.total_edges as f64 / theoretical_max_edges as f64
    } else {
        0.0
    };
    
    let path_efficiency = if topology.average_path_length > 0.0 {
        1.0 / topology.average_path_length
    } else {
        0.0
    };
    
    // Combine edge efficiency and path efficiency
    (edge_efficiency * 0.6) + (path_efficiency * 0.4)
}

/// Calculate entanglement strength distribution balance
pub fn calculate_strength_balance(metrics: &EntanglementMetrics) -> f64 {
    if metrics.total_entanglements == 0 || metrics.max_strength == 0.0 {
        return 0.0;
    }
    
    // Calculate coefficient of variation (lower is more balanced)
    let cv = if metrics.average_strength > 0.0 {
        (metrics.max_strength - metrics.average_strength) / metrics.average_strength
    } else {
        f64::INFINITY
    };
    
    // Convert to balance score (higher is better)
    if cv.is_infinite() {
        0.0
    } else {
        1.0 / (1.0 + cv)
    }
}

/// Identify optimization bottlenecks
pub fn identify_optimization_bottlenecks(
    topology: &NetworkTopology,
    metrics: &EntanglementMetrics,
    issues: &IssueCollection,
) -> Vec<OptimizationBottleneck> {
    let mut bottlenecks = Vec::new();
    
    // Check connectivity bottlenecks
    if !topology.is_connected {
        bottlenecks.push(OptimizationBottleneck {
            bottleneck_type: BottleneckType::Connectivity,
            severity: BottleneckSeverity::Critical,
            description: "Network is not fully connected".to_string(),
            impact_score: 1.0,
        });
    }
    
    // Check density bottlenecks
    if topology.network_density < 0.05 {
        bottlenecks.push(OptimizationBottleneck {
            bottleneck_type: BottleneckType::Density,
            severity: BottleneckSeverity::High,
            description: "Network density is too low".to_string(),
            impact_score: 0.8,
        });
    }
    
    // Check clustering bottlenecks
    if topology.clustering_coefficient < 0.1 {
        bottlenecks.push(OptimizationBottleneck {
            bottleneck_type: BottleneckType::Clustering,
            severity: BottleneckSeverity::Medium,
            description: "Poor clustering coefficient".to_string(),
            impact_score: 0.6,
        });
    }
    
    // Check entanglement strength bottlenecks
    let strength_balance = calculate_strength_balance(metrics);
    if strength_balance < 0.3 {
        bottlenecks.push(OptimizationBottleneck {
            bottleneck_type: BottleneckType::StrengthImbalance,
            severity: BottleneckSeverity::Medium,
            description: "Entanglement strength distribution is imbalanced".to_string(),
            impact_score: 0.5,
        });
    }
    
    // Check issue-based bottlenecks
    if !issues.critical_issues().is_empty() {
        bottlenecks.push(OptimizationBottleneck {
            bottleneck_type: BottleneckType::CriticalIssues,
            severity: BottleneckSeverity::Critical,
            description: format!("{} critical issues detected", issues.critical_issues().len()),
            impact_score: 0.9,
        });
    }
    
    // Sort by impact score (highest first)
    bottlenecks.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));
    
    bottlenecks
}

/// Optimization bottleneck identification
#[derive(Debug, Clone)]
pub struct OptimizationBottleneck {
    pub bottleneck_type: BottleneckType,
    pub severity: BottleneckSeverity,
    pub description: String,
    pub impact_score: f64, // 0.0 to 1.0
}

/// Types of optimization bottlenecks
#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckType {
    Connectivity,
    Density,
    Clustering,
    PathLength,
    StrengthImbalance,
    CriticalIssues,
    Performance,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl BottleneckSeverity {
    /// Get severity description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            BottleneckSeverity::Low => "Low impact bottleneck",
            BottleneckSeverity::Medium => "Medium impact bottleneck",
            BottleneckSeverity::High => "High impact bottleneck",
            BottleneckSeverity::Critical => "Critical bottleneck requiring immediate attention",
        }
    }

    /// Get priority score for addressing bottleneck
    #[inline]
    pub const fn priority_score(&self) -> u8 {
        match self {
            BottleneckSeverity::Low => 1,
            BottleneckSeverity::Medium => 2,
            BottleneckSeverity::High => 3,
            BottleneckSeverity::Critical => 4,
        }
    }
}

impl OptimizationBottleneck {
    /// Check if bottleneck requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self.severity, BottleneckSeverity::Critical) || self.impact_score > 0.8
    }

    /// Get recommended action for bottleneck
    pub fn recommended_action(&self) -> &'static str {
        match self.bottleneck_type {
            BottleneckType::Connectivity => "Establish additional connections between isolated components",
            BottleneckType::Density => "Increase network density by adding strategic connections",
            BottleneckType::Clustering => "Improve local clustering by connecting nearby nodes",
            BottleneckType::PathLength => "Optimize path lengths by adding shortcut connections",
            BottleneckType::StrengthImbalance => "Rebalance entanglement strengths across the network",
            BottleneckType::CriticalIssues => "Address critical issues immediately",
            BottleneckType::Performance => "Optimize performance-critical components",
        }
    }
}
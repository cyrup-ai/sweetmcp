//! Balancing analysis and calculation methods
//!
//! This module provides blazing-fast network balance analysis with zero-allocation
//! patterns and comprehensive statistical calculations.

use std::collections::HashMap;
use crate::cognitive::types::CognitiveError;
use super::super::{
    analysis::NetworkTopology,
    metrics::PerformanceTracker,
};
use super::super::super::node_state::QuantumMCTSNode;
use super::core::QuantumEntanglementEngine;
use super::balancing_types::{NetworkBalanceAnalysis, NodeBalance, DistributionStatistics};

impl QuantumEntanglementEngine {
    /// Analyze current network balance
    pub async fn analyze_current_balance(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<NetworkBalanceAnalysis, CognitiveError> {
        let mut node_balances = Vec::new();
        let mut total_imbalance = 0.0;
        let target_degree = topology.average_degree;
        
        // Analyze balance for each node
        for (node_id, node) in tree {
            let current_entanglements = self.manager.get_node_entanglement_count(node_id).await?;
            let optimal_entanglements = self.calculate_optimal_entanglement_count(node, target_degree);
            
            let balance_score = self.calculate_node_balance_score(current_entanglements, optimal_entanglements);
            let rebalancing_priority = self.calculate_rebalancing_priority(node, balance_score);
            
            total_imbalance += balance_score;
            
            node_balances.push(NodeBalance::new(
                node_id.clone(),
                current_entanglements,
                optimal_entanglements,
                balance_score,
                rebalancing_priority,
            ));
        }
        
        // Sort by rebalancing priority
        node_balances.sort_by(|a, b| b.rebalancing_priority.partial_cmp(&a.rebalancing_priority).unwrap_or(std::cmp::Ordering::Equal));
        
        let average_imbalance = if !node_balances.is_empty() {
            total_imbalance / node_balances.len() as f64
        } else {
            0.0
        };
        
        // Calculate distribution statistics
        let distribution_stats = self.calculate_distribution_statistics(&node_balances);
        
        Ok(NetworkBalanceAnalysis::new(
            node_balances,
            average_imbalance,
            total_imbalance,
            distribution_stats,
            average_imbalance > 0.3,
        ))
    }
    
    /// Calculate optimal entanglement count for a node
    pub fn calculate_optimal_entanglement_count(&self, node: &QuantumMCTSNode, target_degree: f64) -> usize {
        // Base optimal count on network average
        let base_optimal = target_degree as usize;
        
        // Adjust based on node importance (visits and value)
        let importance_factor = (node.visits as f64 * 0.001 + node.value.abs() * 0.1).min(2.0).max(0.5);
        let adjusted_optimal = (base_optimal as f64 * importance_factor) as usize;
        
        // Ensure reasonable bounds
        adjusted_optimal.max(1).min(self.config.max_entanglements_per_node)
    }
    
    /// Calculate node balance score
    pub fn calculate_node_balance_score(&self, current: usize, optimal: usize) -> f64 {
        if optimal == 0 {
            return if current == 0 { 0.0 } else { 1.0 };
        }
        
        let ratio = current as f64 / optimal as f64;
        
        // Perfect balance = 0.0, maximum imbalance = 1.0
        if ratio > 1.0 {
            ((ratio - 1.0) / ratio).min(1.0)
        } else {
            (1.0 - ratio).min(1.0)
        }
    }
    
    /// Calculate rebalancing priority for a node
    pub fn calculate_rebalancing_priority(&self, node: &QuantumMCTSNode, balance_score: f64) -> f64 {
        // Base priority on balance score
        let mut priority = balance_score;
        
        // Increase priority for high-value nodes
        if node.value > 0.5 {
            priority *= 1.5;
        }
        
        // Increase priority for frequently visited nodes
        if node.visits > 100 {
            priority *= 1.3;
        }
        
        // Increase priority for nodes with extreme imbalance
        if balance_score > 0.7 {
            priority *= 1.8;
        }
        
        priority.min(10.0)
    }
    
    /// Calculate distribution statistics
    pub fn calculate_distribution_statistics(&self, node_balances: &[NodeBalance]) -> DistributionStatistics {
        if node_balances.is_empty() {
            return DistributionStatistics::default();
        }
        
        let entanglement_counts: Vec<usize> = node_balances.iter().map(|nb| nb.current_entanglements).collect();
        let balance_scores: Vec<f64> = node_balances.iter().map(|nb| nb.balance_score).collect();
        
        let mean_entanglements = entanglement_counts.iter().sum::<usize>() as f64 / entanglement_counts.len() as f64;
        let mean_balance_score = balance_scores.iter().sum::<f64>() / balance_scores.len() as f64;
        
        let variance_entanglements = entanglement_counts.iter()
            .map(|&count| (count as f64 - mean_entanglements).powi(2))
            .sum::<f64>() / entanglement_counts.len() as f64;
        
        let std_dev_entanglements = variance_entanglements.sqrt();
        
        let min_entanglements = entanglement_counts.iter().min().copied().unwrap_or(0);
        let max_entanglements = entanglement_counts.iter().max().copied().unwrap_or(0);
        
        let coefficient_of_variation = if mean_entanglements > 0.0 { 
            std_dev_entanglements / mean_entanglements 
        } else { 
            0.0 
        };
        
        DistributionStatistics::new(
            mean_entanglements,
            std_dev_entanglements,
            min_entanglements,
            max_entanglements,
            mean_balance_score,
            coefficient_of_variation,
        )
    }
    
    /// Check if balancing should be performed
    pub fn should_perform_balancing(&self, analysis: &NetworkBalanceAnalysis, topology: &NetworkTopology) -> bool {
        // Don't balance if network is too small
        if topology.total_nodes < 10 {
            return false;
        }
        
        // Don't balance if already well balanced
        if analysis.average_imbalance < 0.2 {
            return false;
        }
        
        // Don't balance if distribution is already good
        if analysis.distribution_stats.coefficient_of_variation < 0.3 {
            return false;
        }
        
        // Balance if there are high-priority nodes needing attention
        let high_priority_nodes = analysis.node_balances.iter()
            .filter(|nb| nb.rebalancing_priority > 1.0)
            .count();
        
        high_priority_nodes > 0 || analysis.needs_balancing
    }
    
    /// Calculate balance improvement percentage
    pub fn calculate_balance_improvement(&self, initial: &NetworkBalanceAnalysis, final_analysis: &NetworkBalanceAnalysis) -> f64 {
        if initial.average_imbalance == 0.0 {
            return 0.0;
        }
        
        let imbalance_reduction = initial.average_imbalance - final_analysis.average_imbalance;
        let improvement_percentage = (imbalance_reduction / initial.average_imbalance) * 100.0;
        
        improvement_percentage.max(0.0).min(100.0)
    }
    
    /// Calculate efficiency improvement percentage
    pub fn calculate_efficiency_improvement(&self, initial: &NetworkTopology, final_topo: &NetworkTopology) -> f64 {
        let initial_efficiency = if initial.average_degree > 0.0 {
            initial.clustering_coefficient / initial.average_degree
        } else {
            0.0
        };
        
        let final_efficiency = if final_topo.average_degree > 0.0 {
            final_topo.clustering_coefficient / final_topo.average_degree
        } else {
            0.0
        };
        
        if initial_efficiency == 0.0 {
            return 0.0;
        }
        
        let efficiency_improvement = ((final_efficiency - initial_efficiency) / initial_efficiency) * 100.0;
        efficiency_improvement.max(-50.0).min(100.0)
    }
    
    /// Analyze node criticality for balancing
    pub fn analyze_node_criticality(&self, node: &QuantumMCTSNode, balance: &NodeBalance) -> NodeCriticality {
        let mut criticality_score = balance.balance_score;
        
        // Adjust for node importance
        let importance_multiplier = (node.visits as f64 * 0.01 + node.value * 2.0).max(1.0).min(5.0);
        criticality_score *= importance_multiplier;
        
        // Determine criticality level
        if criticality_score > 3.0 {
            NodeCriticality::Critical
        } else if criticality_score > 1.5 {
            NodeCriticality::High
        } else if criticality_score > 0.5 {
            NodeCriticality::Medium
        } else {
            NodeCriticality::Low
        }
    }
    
    /// Calculate network health score
    pub fn calculate_network_health_score(&self, analysis: &NetworkBalanceAnalysis, topology: &NetworkTopology) -> f64 {
        // Balance component (0.0 = poor, 1.0 = excellent)
        let balance_score = (1.0 - analysis.average_imbalance.min(1.0)).max(0.0);
        
        // Distribution component
        let distribution_score = analysis.distribution_stats.quality_score();
        
        // Topology component
        let topology_score = if topology.total_nodes > 0 {
            (topology.clustering_coefficient * topology.average_degree / topology.total_nodes as f64).min(1.0)
        } else {
            0.0
        };
        
        // Weighted combination
        (balance_score * 0.4 + distribution_score * 0.4 + topology_score * 0.2).max(0.0).min(1.0)
    }
}

/// Node criticality levels for balancing priority
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeCriticality {
    /// Critical - immediate attention required
    Critical,
    /// High - attention needed soon
    High,
    /// Medium - moderate priority
    Medium,
    /// Low - minimal priority
    Low,
}

impl NodeCriticality {
    /// Get criticality description
    pub fn description(&self) -> &'static str {
        match self {
            NodeCriticality::Critical => "Critical",
            NodeCriticality::High => "High",
            NodeCriticality::Medium => "Medium",
            NodeCriticality::Low => "Low",
        }
    }
    
    /// Get priority multiplier
    pub fn priority_multiplier(&self) -> f64 {
        match self {
            NodeCriticality::Critical => 4.0,
            NodeCriticality::High => 2.5,
            NodeCriticality::Medium => 1.5,
            NodeCriticality::Low => 1.0,
        }
    }
    
    /// Check if requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, NodeCriticality::Critical)
    }
}
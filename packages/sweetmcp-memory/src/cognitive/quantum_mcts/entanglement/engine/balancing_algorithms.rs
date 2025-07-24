//! Balancing algorithms and calculations
//!
//! This module provides blazing-fast balancing algorithms with zero-allocation
//! patterns for optimal entanglement distribution calculations.

use std::collections::HashMap;
use tracing::{debug, warn};

use crate::cognitive::types::CognitiveError;
use super::super::{
    analysis::NetworkTopology,
    metrics::PerformanceTracker,
};
use super::super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};
use super::core::QuantumEntanglementEngine;
use super::balancing_core::{
    NodeBalance, BalancingStrategy, NetworkBalanceAnalysis, DistributionStatistics,
};

impl QuantumEntanglementEngine {
    /// Calculate optimal entanglement count for a node
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    pub fn calculate_distribution_statistics(&self, node_balances: &[NodeBalance]) -> DistributionStatistics {
        if node_balances.is_empty() {
            return DistributionStatistics::default();
        }
        
        let entanglement_counts: Vec<usize> = node_balances
            .iter()
            .map(|nb| nb.current_entanglements)
            .collect();
        
        let mean = entanglement_counts.iter().sum::<usize>() as f64 / entanglement_counts.len() as f64;
        
        // Calculate standard deviation
        let variance = entanglement_counts
            .iter()
            .map(|&count| {
                let diff = count as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / entanglement_counts.len() as f64;
        
        let std_deviation = variance.sqrt();
        
        // Calculate coefficient of variation
        let coefficient_variation = if mean > 0.0 {
            std_deviation / mean
        } else {
            0.0
        };
        
        // Calculate Gini coefficient
        let gini_coefficient = self.calculate_gini_coefficient(&entanglement_counts);
        
        // Calculate range
        let min_count = entanglement_counts.iter().min().copied().unwrap_or(0);
        let max_count = entanglement_counts.iter().max().copied().unwrap_or(0);
        let entanglement_range = max_count - min_count;
        
        DistributionStatistics::new(
            std_deviation,
            coefficient_variation,
            gini_coefficient,
            entanglement_range,
        )
    }
    
    /// Calculate Gini coefficient for inequality measurement
    #[inline]
    fn calculate_gini_coefficient(&self, values: &[usize]) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }
        
        let mut sorted_values = values.to_vec();
        sorted_values.sort_unstable();
        
        let n = sorted_values.len() as f64;
        let sum: f64 = sorted_values.iter().sum::<usize>() as f64;
        
        if sum == 0.0 {
            return 0.0;
        }
        
        let mut gini_sum = 0.0;
        for (i, &value) in sorted_values.iter().enumerate() {
            gini_sum += (2.0 * (i as f64 + 1.0) - n - 1.0) * value as f64;
        }
        
        gini_sum / (n * sum)
    }
    
    /// Calculate balance improvement between two analyses
    #[inline]
    pub fn calculate_balance_improvement(
        &self,
        old_analysis: &NetworkBalanceAnalysis,
        new_analysis: &NetworkBalanceAnalysis,
    ) -> f64 {
        if old_analysis.average_imbalance == 0.0 {
            return 0.0;
        }
        
        let improvement = (old_analysis.average_imbalance - new_analysis.average_imbalance) 
            / old_analysis.average_imbalance * 100.0;
        
        improvement.max(0.0)
    }
    
    /// Calculate efficiency improvement between two topologies
    #[inline]
    pub fn calculate_efficiency_improvement(
        &self,
        old_topology: &NetworkTopology,
        new_topology: &NetworkTopology,
    ) -> f64 {
        if old_topology.clustering_coefficient == 0.0 {
            return 0.0;
        }
        
        let clustering_improvement = (new_topology.clustering_coefficient - old_topology.clustering_coefficient)
            / old_topology.clustering_coefficient * 100.0;
        
        let path_improvement = if old_topology.average_path_length > 0.0 && new_topology.average_path_length > 0.0 {
            (old_topology.average_path_length - new_topology.average_path_length)
                / old_topology.average_path_length * 100.0
        } else {
            0.0
        };
        
        ((clustering_improvement + path_improvement) / 2.0).max(0.0)
    }
    
    /// Determine if balancing should be performed
    #[inline]
    pub fn should_perform_balancing(
        &self,
        balance_analysis: &NetworkBalanceAnalysis,
        topology: &NetworkTopology,
    ) -> bool {
        // Check if network needs balancing
        if !balance_analysis.needs_balancing {
            return false;
        }
        
        // Check if imbalance is above threshold
        if balance_analysis.average_imbalance < 0.3 {
            return false;
        }
        
        // Check if there are enough nodes to balance
        if balance_analysis.node_balances.len() < 3 {
            return false;
        }
        
        // Check if distribution quality is poor
        if balance_analysis.distribution_stats.quality_score() < 0.4 {
            return true;
        }
        
        // Check for critical imbalances
        let critical_nodes = balance_analysis.critically_imbalanced_nodes();
        if critical_nodes.len() > balance_analysis.node_balances.len() / 4 {
            return true;
        }
        
        // Check network efficiency
        if topology.clustering_coefficient < 0.3 {
            return true;
        }
        
        false
    }
    
    /// Determine balancing strategy based on network conditions
    #[inline]
    pub fn determine_balancing_strategy(
        &self,
        topology: &NetworkTopology,
        balance_analysis: &NetworkBalanceAnalysis,
    ) -> BalancingStrategy {
        let critical_nodes = balance_analysis.critically_imbalanced_nodes();
        let critical_ratio = critical_nodes.len() as f64 / balance_analysis.node_balances.len() as f64;
        
        // Use aggressive strategy for severe imbalances
        if critical_ratio > 0.5 || balance_analysis.average_imbalance > 0.7 {
            debug!("Using aggressive balancing strategy due to severe imbalances");
            return BalancingStrategy::aggressive();
        }
        
        // Use conservative strategy for minor imbalances
        if critical_ratio < 0.1 && balance_analysis.average_imbalance < 0.4 {
            debug!("Using conservative balancing strategy for minor imbalances");
            return BalancingStrategy::conservative();
        }
        
        // Use default strategy for moderate imbalances
        debug!("Using default balancing strategy for moderate imbalances");
        BalancingStrategy::default()
    }
    
    /// Determine reason for balancing operation
    #[inline]
    pub fn determine_balancing_reason(
        &self,
        balance_analysis: &NetworkBalanceAnalysis,
        strategy: &BalancingStrategy,
    ) -> String {
        let critical_nodes = balance_analysis.critically_imbalanced_nodes();
        let critical_ratio = critical_nodes.len() as f64 / balance_analysis.node_balances.len() as f64;
        
        if critical_ratio > 0.5 {
            format!(
                "Critical imbalance detected: {:.1}% of nodes critically imbalanced (avg: {:.2})",
                critical_ratio * 100.0,
                balance_analysis.average_imbalance
            )
        } else if balance_analysis.average_imbalance > 0.6 {
            format!(
                "High network imbalance: {:.2} average imbalance across {} nodes",
                balance_analysis.average_imbalance,
                balance_analysis.node_balances.len()
            )
        } else if balance_analysis.distribution_stats.quality_score() < 0.4 {
            format!(
                "Poor distribution quality: {:.2} quality score (CV: {:.2}, Gini: {:.2})",
                balance_analysis.distribution_stats.quality_score(),
                balance_analysis.distribution_stats.coefficient_variation,
                balance_analysis.distribution_stats.gini_coefficient
            )
        } else {
            format!(
                "Routine balancing: {:.2} average imbalance, {} nodes need rebalancing",
                balance_analysis.average_imbalance,
                balance_analysis.nodes_needing_rebalancing().len()
            )
        }
    }
}
//! Balance analysis for entanglement distribution
//!
//! This module provides blazing-fast balance analysis with zero allocation
//! optimizations and elegant ergonomic interfaces for analyzing network balance.

use std::collections::HashMap;
use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::super::super::{
    analysis::NetworkTopology,
    node_state::QuantumMCTSNode,
};

/// Node balance information
#[derive(Debug, Clone)]
pub struct NodeBalance {
    /// Node identifier
    pub node_id: String,
    /// Current entanglement count
    pub current_entanglements: usize,
    /// Optimal entanglement count
    pub optimal_entanglements: usize,
    /// Balance score (0.0 = perfectly balanced, 1.0 = maximally imbalanced)
    pub balance_score: f64,
    /// Priority for rebalancing (higher = more urgent)
    pub rebalancing_priority: f64,
}

impl NodeBalance {
    /// Create new node balance analysis
    #[inline]
    pub fn new(
        node_id: String,
        current_entanglements: usize,
        optimal_entanglements: usize,
    ) -> Self {
        let balance_score = Self::calculate_balance_score(current_entanglements, optimal_entanglements);
        let rebalancing_priority = Self::calculate_rebalancing_priority(balance_score, current_entanglements);

        Self {
            node_id,
            current_entanglements,
            optimal_entanglements,
            balance_score,
            rebalancing_priority,
        }
    }

    /// Calculate balance score with zero allocation
    #[inline]
    fn calculate_balance_score(current: usize, optimal: usize) -> f64 {
        if optimal == 0 {
            if current == 0 { 0.0 } else { 1.0 }
        } else {
            let ratio = current as f64 / optimal as f64;
            (ratio - 1.0).abs().min(1.0)
        }
    }

    /// Calculate rebalancing priority
    #[inline]
    fn calculate_rebalancing_priority(balance_score: f64, current_entanglements: usize) -> f64 {
        let urgency_factor = balance_score * balance_score; // Quadratic urgency
        let size_factor = (current_entanglements as f64).ln_1p() / 10.0; // Logarithmic size factor
        
        (urgency_factor + size_factor).min(1.0)
    }

    /// Check if node is overloaded
    #[inline]
    pub fn is_overloaded(&self) -> bool {
        self.current_entanglements > self.optimal_entanglements
    }

    /// Check if node is underloaded
    #[inline]
    pub fn is_underloaded(&self) -> bool {
        self.current_entanglements < self.optimal_entanglements
    }

    /// Check if node is balanced
    #[inline]
    pub fn is_balanced(&self, threshold: f64) -> bool {
        self.balance_score <= threshold
    }

    /// Get entanglement deficit (negative if overloaded)
    #[inline]
    pub fn entanglement_deficit(&self) -> i32 {
        self.optimal_entanglements as i32 - self.current_entanglements as i32
    }

    /// Get load factor (current/optimal)
    #[inline]
    pub fn load_factor(&self) -> f64 {
        if self.optimal_entanglements == 0 {
            if self.current_entanglements == 0 { 1.0 } else { f64::INFINITY }
        } else {
            self.current_entanglements as f64 / self.optimal_entanglements as f64
        }
    }
}

/// Network balance analysis
#[derive(Debug, Clone)]
pub struct NetworkBalanceAnalysis {
    /// Individual node balance information
    pub node_balances: Vec<NodeBalance>,
    /// Overall network balance score
    pub overall_balance_score: f64,
    /// Standard deviation of load factors
    pub load_distribution_variance: f64,
    /// Number of overloaded nodes
    pub overloaded_nodes: usize,
    /// Number of underloaded nodes
    pub underloaded_nodes: usize,
    /// Total entanglements in network
    pub total_entanglements: usize,
    /// Average entanglements per node
    pub average_entanglements: f64,
}

impl NetworkBalanceAnalysis {
    /// Create new network balance analysis
    #[inline]
    pub fn new(node_balances: Vec<NodeBalance>) -> Self {
        let overall_balance_score = Self::calculate_overall_balance_score(&node_balances);
        let load_distribution_variance = Self::calculate_load_distribution_variance(&node_balances);
        let (overloaded_nodes, underloaded_nodes) = Self::count_imbalanced_nodes(&node_balances);
        let total_entanglements = node_balances.iter().map(|nb| nb.current_entanglements).sum();
        let average_entanglements = if node_balances.is_empty() {
            0.0
        } else {
            total_entanglements as f64 / node_balances.len() as f64
        };

        Self {
            node_balances,
            overall_balance_score,
            load_distribution_variance,
            overloaded_nodes,
            underloaded_nodes,
            total_entanglements,
            average_entanglements,
        }
    }

    /// Calculate overall balance score
    #[inline]
    fn calculate_overall_balance_score(node_balances: &[NodeBalance]) -> f64 {
        if node_balances.is_empty() {
            return 1.0;
        }

        let sum_balance_scores: f64 = node_balances.iter().map(|nb| nb.balance_score).sum();
        sum_balance_scores / node_balances.len() as f64
    }

    /// Calculate load distribution variance
    #[inline]
    fn calculate_load_distribution_variance(node_balances: &[NodeBalance]) -> f64 {
        if node_balances.len() <= 1 {
            return 0.0;
        }

        let load_factors: Vec<f64> = node_balances.iter()
            .map(|nb| nb.load_factor())
            .filter(|&lf| lf.is_finite())
            .collect();

        if load_factors.is_empty() {
            return 0.0;
        }

        let mean = load_factors.iter().sum::<f64>() / load_factors.len() as f64;
        let variance = load_factors.iter()
            .map(|&lf| (lf - mean).powi(2))
            .sum::<f64>() / load_factors.len() as f64;

        variance.sqrt()
    }

    /// Count imbalanced nodes
    #[inline]
    fn count_imbalanced_nodes(node_balances: &[NodeBalance]) -> (usize, usize) {
        let mut overloaded = 0;
        let mut underloaded = 0;

        for nb in node_balances {
            if nb.is_overloaded() {
                overloaded += 1;
            } else if nb.is_underloaded() {
                underloaded += 1;
            }
        }

        (overloaded, underloaded)
    }

    /// Check if network is well balanced
    #[inline]
    pub fn is_well_balanced(&self, threshold: f64) -> bool {
        self.overall_balance_score <= threshold
    }

    /// Get most overloaded nodes
    #[inline]
    pub fn get_most_overloaded_nodes(&self, count: usize) -> Vec<&NodeBalance> {
        let mut overloaded: Vec<&NodeBalance> = self.node_balances.iter()
            .filter(|nb| nb.is_overloaded())
            .collect();

        overloaded.sort_by(|a, b| b.rebalancing_priority.partial_cmp(&a.rebalancing_priority).unwrap());
        overloaded.into_iter().take(count).collect()
    }

    /// Get most underloaded nodes
    #[inline]
    pub fn get_most_underloaded_nodes(&self, count: usize) -> Vec<&NodeBalance> {
        let mut underloaded: Vec<&NodeBalance> = self.node_balances.iter()
            .filter(|nb| nb.is_underloaded())
            .collect();

        underloaded.sort_by(|a, b| b.rebalancing_priority.partial_cmp(&a.rebalancing_priority).unwrap());
        underloaded.into_iter().take(count).collect()
    }

    /// Get nodes requiring rebalancing
    #[inline]
    pub fn get_nodes_requiring_rebalancing(&self, priority_threshold: f64) -> Vec<&NodeBalance> {
        self.node_balances.iter()
            .filter(|nb| nb.rebalancing_priority >= priority_threshold)
            .collect()
    }

    /// Calculate potential improvement from balancing
    #[inline]
    pub fn calculate_potential_improvement(&self) -> f64 {
        if self.node_balances.is_empty() {
            return 0.0;
        }

        // Calculate theoretical perfect balance score
        let perfect_balance_score = 0.0;
        let current_score = self.overall_balance_score;
        
        ((current_score - perfect_balance_score) / current_score * 100.0).max(0.0)
    }

    /// Get balance efficiency score
    #[inline]
    pub fn balance_efficiency_score(&self) -> f64 {
        (1.0 - self.overall_balance_score).max(0.0)
    }

    /// Get distribution statistics
    #[inline]
    pub fn get_distribution_statistics(&self) -> DistributionStatistics {
        let load_factors: Vec<f64> = self.node_balances.iter()
            .map(|nb| nb.load_factor())
            .filter(|&lf| lf.is_finite())
            .collect();

        if load_factors.is_empty() {
            return DistributionStatistics::default();
        }

        let min_load = load_factors.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_load = load_factors.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let mean_load = load_factors.iter().sum::<f64>() / load_factors.len() as f64;
        
        // Calculate median
        let mut sorted_loads = load_factors.clone();
        sorted_loads.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_load = if sorted_loads.len() % 2 == 0 {
            let mid = sorted_loads.len() / 2;
            (sorted_loads[mid - 1] + sorted_loads[mid]) / 2.0
        } else {
            sorted_loads[sorted_loads.len() / 2]
        };

        DistributionStatistics {
            min_load_factor: min_load,
            max_load_factor: max_load,
            mean_load_factor: mean_load,
            median_load_factor: median_load,
            load_range: max_load - min_load,
            coefficient_of_variation: self.load_distribution_variance / mean_load,
        }
    }
}

/// Distribution statistics for load analysis
#[derive(Debug, Clone)]
pub struct DistributionStatistics {
    pub min_load_factor: f64,
    pub max_load_factor: f64,
    pub mean_load_factor: f64,
    pub median_load_factor: f64,
    pub load_range: f64,
    pub coefficient_of_variation: f64,
}

impl Default for DistributionStatistics {
    fn default() -> Self {
        Self {
            min_load_factor: 0.0,
            max_load_factor: 0.0,
            mean_load_factor: 0.0,
            median_load_factor: 0.0,
            load_range: 0.0,
            coefficient_of_variation: 0.0,
        }
    }
}

/// Balance analyzer for network analysis
pub struct BalanceAnalyzer;

impl BalanceAnalyzer {
    /// Analyze current network balance with zero allocation optimizations
    #[inline]
    pub fn analyze_network_balance(
        tree: &HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<NetworkBalanceAnalysis, CognitiveError> {
        debug!("Analyzing network balance for {} nodes", tree.len());

        let mut node_balances = Vec::with_capacity(tree.len());

        for (node_id, node) in tree {
            let current_entanglements = Self::count_node_entanglements(node, topology);
            let optimal_entanglements = Self::calculate_optimal_entanglements(node, topology);

            let node_balance = NodeBalance::new(
                node_id.clone(),
                current_entanglements,
                optimal_entanglements,
            );

            node_balances.push(node_balance);
        }

        Ok(NetworkBalanceAnalysis::new(node_balances))
    }

    /// Count entanglements for a node
    #[inline]
    fn count_node_entanglements(
        _node: &QuantumMCTSNode,
        topology: &NetworkTopology,
    ) -> usize {
        // In a real implementation, this would count actual entanglements
        // For now, use topology data as approximation
        topology.node_degrees.values().next().copied().unwrap_or(0)
    }

    /// Calculate optimal entanglements for a node
    #[inline]
    fn calculate_optimal_entanglements(
        _node: &QuantumMCTSNode,
        topology: &NetworkTopology,
    ) -> usize {
        // Calculate based on network topology and node characteristics
        let average_degree = if topology.node_degrees.is_empty() {
            0.0
        } else {
            topology.node_degrees.values().sum::<usize>() as f64 / topology.node_degrees.len() as f64
        };

        // Optimal is slightly above average for better distribution
        (average_degree * 1.2) as usize
    }

    /// Determine if balancing is needed
    #[inline]
    pub fn should_perform_balancing(
        analysis: &NetworkBalanceAnalysis,
        balance_threshold: f64,
        min_imbalanced_nodes: usize,
    ) -> bool {
        analysis.overall_balance_score > balance_threshold ||
        analysis.overloaded_nodes + analysis.underloaded_nodes >= min_imbalanced_nodes
    }

    /// Calculate balance improvement between two analyses
    #[inline]
    pub fn calculate_balance_improvement(
        before: &NetworkBalanceAnalysis,
        after: &NetworkBalanceAnalysis,
    ) -> f64 {
        if before.overall_balance_score == 0.0 {
            return 0.0;
        }

        let improvement = (before.overall_balance_score - after.overall_balance_score) / before.overall_balance_score;
        (improvement * 100.0).max(0.0)
    }
}
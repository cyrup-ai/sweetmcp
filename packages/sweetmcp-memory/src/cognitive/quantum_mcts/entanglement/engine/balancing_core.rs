//! Core balancing structures and main operations
//!
//! This module provides the core balancing functionality with zero-allocation
//! patterns and blazing-fast performance for entanglement distribution.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;
use super::super::{
    analysis::{NetworkTopology, NetworkTopologyAnalyzer},
    metrics::PerformanceTracker,
};
use super::super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};
use super::core::QuantumEntanglementEngine;

/// Balancing operation result with comprehensive analysis
#[derive(Debug, Clone)]
pub struct BalancingResult {
    /// Number of redistributions made
    pub redistributions_made: usize,
    /// Time taken for balancing in milliseconds
    pub balancing_time_ms: u64,
    /// Balance improvement percentage
    pub balance_improvement: f64,
    /// Network efficiency improvement
    pub efficiency_improvement: f64,
    /// Reason for balancing operation
    pub reason: String,
}

impl BalancingResult {
    /// Create new balancing result
    #[inline]
    pub fn new(
        redistributions_made: usize,
        balancing_time_ms: u64,
        balance_improvement: f64,
        efficiency_improvement: f64,
        reason: String,
    ) -> Self {
        Self {
            redistributions_made,
            balancing_time_ms,
            balance_improvement,
            efficiency_improvement,
            reason,
        }
    }

    /// Create result for skipped balancing
    #[inline]
    pub fn skipped(balancing_time_ms: u64, reason: String) -> Self {
        Self {
            redistributions_made: 0,
            balancing_time_ms,
            balance_improvement: 0.0,
            efficiency_improvement: 0.0,
            reason,
        }
    }

    /// Check if balancing was effective
    #[inline]
    pub fn is_effective(&self) -> bool {
        self.redistributions_made > 0 && self.balance_improvement > 1.0
    }

    /// Get total improvement score
    #[inline]
    pub fn total_improvement(&self) -> f64 {
        (self.balance_improvement + self.efficiency_improvement) / 2.0
    }
}

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
    /// Create new node balance
    #[inline]
    pub fn new(
        node_id: String,
        current_entanglements: usize,
        optimal_entanglements: usize,
        balance_score: f64,
        rebalancing_priority: f64,
    ) -> Self {
        Self {
            node_id,
            current_entanglements,
            optimal_entanglements,
            balance_score,
            rebalancing_priority,
        }
    }

    /// Check if node needs rebalancing
    #[inline]
    pub fn needs_rebalancing(&self) -> bool {
        self.balance_score > 0.3
    }

    /// Check if node is critically imbalanced
    #[inline]
    pub fn is_critically_imbalanced(&self) -> bool {
        self.balance_score > 0.7
    }

    /// Get imbalance magnitude
    #[inline]
    pub fn imbalance_magnitude(&self) -> usize {
        if self.current_entanglements > self.optimal_entanglements {
            self.current_entanglements - self.optimal_entanglements
        } else {
            self.optimal_entanglements - self.current_entanglements
        }
    }

    /// Check if node is over-entangled
    #[inline]
    pub fn is_over_entangled(&self) -> bool {
        self.current_entanglements > self.optimal_entanglements
    }

    /// Check if node is under-entangled
    #[inline]
    pub fn is_under_entangled(&self) -> bool {
        self.current_entanglements < self.optimal_entanglements
    }
}

/// Distribution balancing strategy
#[derive(Debug, Clone)]
pub struct BalancingStrategy {
    /// Target balance ratio (optimal entanglements per node)
    pub target_balance_ratio: f64,
    /// Maximum redistributions per operation
    pub max_redistributions: usize,
    /// Minimum improvement threshold to proceed
    pub min_improvement_threshold: f64,
    /// Load balancing factor (0.0 = no balancing, 1.0 = perfect balancing)
    pub load_balancing_factor: f64,
}

impl Default for BalancingStrategy {
    fn default() -> Self {
        Self {
            target_balance_ratio: 0.8,
            max_redistributions: 100,
            min_improvement_threshold: 2.0,
            load_balancing_factor: 0.7,
        }
    }
}

impl BalancingStrategy {
    /// Create new balancing strategy
    #[inline]
    pub fn new(
        target_balance_ratio: f64,
        max_redistributions: usize,
        min_improvement_threshold: f64,
        load_balancing_factor: f64,
    ) -> Self {
        Self {
            target_balance_ratio,
            max_redistributions,
            min_improvement_threshold,
            load_balancing_factor,
        }
    }

    /// Create aggressive balancing strategy
    #[inline]
    pub fn aggressive() -> Self {
        Self {
            target_balance_ratio: 0.9,
            max_redistributions: 200,
            min_improvement_threshold: 1.0,
            load_balancing_factor: 0.9,
        }
    }

    /// Create conservative balancing strategy
    #[inline]
    pub fn conservative() -> Self {
        Self {
            target_balance_ratio: 0.6,
            max_redistributions: 50,
            min_improvement_threshold: 5.0,
            load_balancing_factor: 0.5,
        }
    }

    /// Check if strategy should proceed with balancing
    #[inline]
    pub fn should_proceed(&self, expected_improvement: f64) -> bool {
        expected_improvement >= self.min_improvement_threshold
    }
}

/// Network balance analysis result
#[derive(Debug, Clone)]
pub struct NetworkBalanceAnalysis {
    /// Balance information for each node
    pub node_balances: Vec<NodeBalance>,
    /// Average imbalance across all nodes
    pub average_imbalance: f64,
    /// Total imbalance in the network
    pub total_imbalance: f64,
    /// Distribution statistics
    pub distribution_stats: DistributionStatistics,
    /// Whether balancing is needed
    pub needs_balancing: bool,
}

impl NetworkBalanceAnalysis {
    /// Create new network balance analysis
    #[inline]
    pub fn new(
        node_balances: Vec<NodeBalance>,
        average_imbalance: f64,
        total_imbalance: f64,
        distribution_stats: DistributionStatistics,
        needs_balancing: bool,
    ) -> Self {
        Self {
            node_balances,
            average_imbalance,
            total_imbalance,
            distribution_stats,
            needs_balancing,
        }
    }

    /// Get nodes that need rebalancing
    #[inline]
    pub fn nodes_needing_rebalancing(&self) -> Vec<&NodeBalance> {
        self.node_balances.iter().filter(|nb| nb.needs_rebalancing()).collect()
    }

    /// Get critically imbalanced nodes
    #[inline]
    pub fn critically_imbalanced_nodes(&self) -> Vec<&NodeBalance> {
        self.node_balances.iter().filter(|nb| nb.is_critically_imbalanced()).collect()
    }

    /// Get network health score (0.0 = poor, 1.0 = excellent)
    #[inline]
    pub fn network_health_score(&self) -> f64 {
        (1.0 - self.average_imbalance).max(0.0)
    }
}

/// Distribution statistics
#[derive(Debug, Clone)]
pub struct DistributionStatistics {
    /// Standard deviation of entanglement counts
    pub std_deviation: f64,
    /// Coefficient of variation
    pub coefficient_variation: f64,
    /// Gini coefficient (inequality measure)
    pub gini_coefficient: f64,
    /// Range of entanglement counts
    pub entanglement_range: usize,
}

impl Default for DistributionStatistics {
    fn default() -> Self {
        Self {
            std_deviation: 0.0,
            coefficient_variation: 0.0,
            gini_coefficient: 0.0,
            entanglement_range: 0,
        }
    }
}

impl DistributionStatistics {
    /// Create new distribution statistics
    #[inline]
    pub fn new(
        std_deviation: f64,
        coefficient_variation: f64,
        gini_coefficient: f64,
        entanglement_range: usize,
    ) -> Self {
        Self {
            std_deviation,
            coefficient_variation,
            gini_coefficient,
            entanglement_range,
        }
    }

    /// Check if distribution is well-balanced
    #[inline]
    pub fn is_well_balanced(&self) -> bool {
        self.coefficient_variation < 0.3 && self.gini_coefficient < 0.4
    }

    /// Get distribution quality score (0.0 = poor, 1.0 = excellent)
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let cv_score = (1.0 - self.coefficient_variation.min(1.0)).max(0.0);
        let gini_score = (1.0 - self.gini_coefficient.min(1.0)).max(0.0);
        (cv_score + gini_score) / 2.0
    }
}
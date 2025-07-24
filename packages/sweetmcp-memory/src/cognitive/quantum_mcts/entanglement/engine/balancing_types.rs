//! Balancing data types and structures
//!
//! This module provides comprehensive data structures for entanglement balancing
//! with blazing-fast zero-allocation patterns and efficient memory usage.

use std::time::Duration;

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

    /// Check if balancing was successful
    pub fn was_successful(&self) -> bool {
        self.redistributions_made > 0 && (self.balance_improvement > 0.0 || self.efficiency_improvement > 0.0)
    }
    
    /// Get balancing summary
    pub fn summary(&self) -> String {
        format!(
            "Balanced {} redistributions in {}ms (+{:.1}% balance, +{:.1}% efficiency)",
            self.redistributions_made,
            self.balancing_time_ms,
            self.balance_improvement,
            self.efficiency_improvement
        )
    }
    
    /// Check if balancing had significant impact
    pub fn had_significant_impact(&self) -> bool {
        self.balance_improvement > 10.0 || self.efficiency_improvement > 5.0 || self.redistributions_made > 20
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        format!(
            "=== Balancing Report ===\n\
            Redistributions Made: {}\n\
            Duration: {}ms\n\
            Balance Improvement: {:.1}%\n\
            Efficiency Improvement: {:.1}%\n\
            Reason: {}\n\
            Impact: {}",
            self.redistributions_made,
            self.balancing_time_ms,
            self.balance_improvement,
            self.efficiency_improvement,
            self.reason,
            if self.had_significant_impact() { "SIGNIFICANT" } else { "MODERATE" }
        )
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

    /// Check if node is overloaded
    pub fn is_overloaded(&self) -> bool {
        self.current_entanglements > self.optimal_entanglements
    }

    /// Check if node is underloaded
    pub fn is_underloaded(&self) -> bool {
        self.current_entanglements < self.optimal_entanglements
    }

    /// Get load ratio (current / optimal)
    pub fn load_ratio(&self) -> f64 {
        if self.optimal_entanglements == 0 {
            if self.current_entanglements == 0 { 1.0 } else { f64::INFINITY }
        } else {
            self.current_entanglements as f64 / self.optimal_entanglements as f64
        }
    }

    /// Check if node needs immediate attention
    pub fn needs_immediate_attention(&self) -> bool {
        self.rebalancing_priority > 2.0 || self.balance_score > 0.8
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
    pub fn aggressive() -> Self {
        Self {
            target_balance_ratio: 0.9,
            max_redistributions: 200,
            min_improvement_threshold: 1.0,
            load_balancing_factor: 0.9,
        }
    }

    /// Create conservative balancing strategy
    pub fn conservative() -> Self {
        Self {
            target_balance_ratio: 0.6,
            max_redistributions: 50,
            min_improvement_threshold: 5.0,
            load_balancing_factor: 0.5,
        }
    }

    /// Check if strategy is aggressive
    pub fn is_aggressive(&self) -> bool {
        self.load_balancing_factor > 0.8 && self.max_redistributions > 150
    }
}

/// Network balance analysis result
#[derive(Debug, Clone)]
pub struct NetworkBalanceAnalysis {
    /// Individual node balance information
    pub node_balances: Vec<NodeBalance>,
    /// Average imbalance across all nodes
    pub average_imbalance: f64,
    /// Total imbalance in the network
    pub total_imbalance: f64,
    /// Distribution statistics
    pub distribution_stats: DistributionStatistics,
    /// Whether the network needs balancing
    pub needs_balancing: bool,
}

impl NetworkBalanceAnalysis {
    /// Create new network balance analysis
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

    /// Get high priority nodes count
    pub fn high_priority_nodes_count(&self) -> usize {
        self.node_balances.iter()
            .filter(|nb| nb.rebalancing_priority > 1.0)
            .count()
    }

    /// Get severely imbalanced nodes count
    pub fn severely_imbalanced_count(&self) -> usize {
        self.node_balances.iter()
            .filter(|nb| nb.balance_score > 0.7)
            .count()
    }

    /// Check if network has critical imbalance
    pub fn has_critical_imbalance(&self) -> bool {
        self.average_imbalance > 0.8 || self.severely_imbalanced_count() > 0
    }
}

/// Distribution statistics
#[derive(Debug, Clone)]
pub struct DistributionStatistics {
    /// Mean number of entanglements per node
    pub mean_entanglements: f64,
    /// Standard deviation of entanglement counts
    pub std_dev_entanglements: f64,
    /// Minimum entanglements per node
    pub min_entanglements: usize,
    /// Maximum entanglements per node
    pub max_entanglements: usize,
    /// Mean balance score
    pub mean_balance_score: f64,
    /// Coefficient of variation (std_dev / mean)
    pub coefficient_of_variation: f64,
}

impl Default for DistributionStatistics {
    fn default() -> Self {
        Self {
            mean_entanglements: 0.0,
            std_dev_entanglements: 0.0,
            min_entanglements: 0,
            max_entanglements: 0,
            mean_balance_score: 0.0,
            coefficient_of_variation: 0.0,
        }
    }
}

impl DistributionStatistics {
    /// Create new distribution statistics
    pub fn new(
        mean_entanglements: f64,
        std_dev_entanglements: f64,
        min_entanglements: usize,
        max_entanglements: usize,
        mean_balance_score: f64,
        coefficient_of_variation: f64,
    ) -> Self {
        Self {
            mean_entanglements,
            std_dev_entanglements,
            min_entanglements,
            max_entanglements,
            mean_balance_score,
            coefficient_of_variation,
        }
    }

    /// Get entanglement range
    pub fn entanglement_range(&self) -> usize {
        self.max_entanglements.saturating_sub(self.min_entanglements)
    }

    /// Check if distribution is well balanced
    pub fn is_well_balanced(&self) -> bool {
        self.coefficient_of_variation < 0.3 && self.mean_balance_score < 0.2
    }

    /// Check if distribution has high variance
    pub fn has_high_variance(&self) -> bool {
        self.coefficient_of_variation > 0.8
    }

    /// Get distribution quality score (0.0 = poor, 1.0 = excellent)
    pub fn quality_score(&self) -> f64 {
        let variance_score = (1.0 - self.coefficient_of_variation.min(1.0)).max(0.0);
        let balance_score = (1.0 - self.mean_balance_score.min(1.0)).max(0.0);
        (variance_score + balance_score) / 2.0
    }
}
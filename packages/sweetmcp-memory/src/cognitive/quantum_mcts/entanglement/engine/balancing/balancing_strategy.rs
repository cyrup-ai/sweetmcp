//! Balancing strategies for entanglement distribution
//!
//! This module provides blazing-fast balancing strategies with zero allocation
//! optimizations and elegant ergonomic interfaces for load balancing operations.

use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::{
    balance_analysis::{NetworkBalanceAnalysis, NodeBalance, DistributionStatistics},
    super::super::analysis::NetworkTopology,
};

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
    /// Strategy type for different balancing approaches
    pub strategy_type: BalancingStrategyType,
    /// Priority weighting for different factors
    pub priority_weights: PriorityWeights,
}

impl Default for BalancingStrategy {
    fn default() -> Self {
        Self {
            target_balance_ratio: 0.8,
            max_redistributions: 100,
            min_improvement_threshold: 2.0,
            load_balancing_factor: 0.7,
            strategy_type: BalancingStrategyType::Adaptive,
            priority_weights: PriorityWeights::default(),
        }
    }
}

impl BalancingStrategy {
    /// Create new balancing strategy with zero allocation optimizations
    #[inline]
    pub fn new(
        target_balance_ratio: f64,
        max_redistributions: usize,
        min_improvement_threshold: f64,
        load_balancing_factor: f64,
    ) -> Self {
        Self {
            target_balance_ratio: target_balance_ratio.clamp(0.1, 1.0),
            max_redistributions: max_redistributions.max(1),
            min_improvement_threshold: min_improvement_threshold.max(0.0),
            load_balancing_factor: load_balancing_factor.clamp(0.0, 1.0),
            strategy_type: BalancingStrategyType::Adaptive,
            priority_weights: PriorityWeights::default(),
        }
    }

    /// Create strategy optimized for speed
    #[inline]
    pub fn speed_optimized() -> Self {
        Self {
            target_balance_ratio: 0.6,
            max_redistributions: 50,
            min_improvement_threshold: 5.0,
            load_balancing_factor: 0.5,
            strategy_type: BalancingStrategyType::Greedy,
            priority_weights: PriorityWeights::speed_focused(),
        }
    }

    /// Create strategy optimized for quality
    #[inline]
    pub fn quality_optimized() -> Self {
        Self {
            target_balance_ratio: 0.9,
            max_redistributions: 200,
            min_improvement_threshold: 1.0,
            load_balancing_factor: 0.9,
            strategy_type: BalancingStrategyType::Optimal,
            priority_weights: PriorityWeights::quality_focused(),
        }
    }

    /// Create adaptive strategy based on network characteristics
    #[inline]
    pub fn adaptive(topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> Self {
        let network_size = topology.node_degrees.len();
        let imbalance_severity = analysis.overall_balance_score;
        
        let (target_ratio, max_redistributions, threshold, factor) = if network_size < 50 {
            // Small network - be more aggressive
            (0.8, 150, 1.5, 0.8)
        } else if network_size < 200 {
            // Medium network - balanced approach
            (0.7, 100, 2.0, 0.7)
        } else {
            // Large network - be more conservative
            (0.6, 75, 3.0, 0.6)
        };

        let strategy_type = if imbalance_severity > 0.7 {
            BalancingStrategyType::Aggressive
        } else if imbalance_severity > 0.3 {
            BalancingStrategyType::Adaptive
        } else {
            BalancingStrategyType::Conservative
        };

        Self {
            target_balance_ratio: target_ratio,
            max_redistributions,
            min_improvement_threshold: threshold,
            load_balancing_factor: factor,
            strategy_type,
            priority_weights: PriorityWeights::adaptive(imbalance_severity),
        }
    }

    /// Determine if balancing should proceed
    #[inline]
    pub fn should_proceed(&self, analysis: &NetworkBalanceAnalysis) -> bool {
        let potential_improvement = analysis.calculate_potential_improvement();
        
        potential_improvement >= self.min_improvement_threshold &&
        analysis.overall_balance_score > (1.0 - self.target_balance_ratio)
    }

    /// Calculate redistribution limit based on strategy
    #[inline]
    pub fn calculate_redistribution_limit(&self, analysis: &NetworkBalanceAnalysis) -> usize {
        let base_limit = self.max_redistributions;
        let severity_multiplier = (analysis.overall_balance_score * 2.0).min(2.0);
        
        match self.strategy_type {
            BalancingStrategyType::Conservative => (base_limit as f64 * 0.5) as usize,
            BalancingStrategyType::Adaptive => (base_limit as f64 * severity_multiplier * 0.8) as usize,
            BalancingStrategyType::Aggressive => (base_limit as f64 * severity_multiplier) as usize,
            BalancingStrategyType::Greedy => base_limit / 2,
            BalancingStrategyType::Optimal => (base_limit as f64 * 1.5) as usize,
        }
    }

    /// Prioritize nodes for rebalancing
    #[inline]
    pub fn prioritize_nodes(&self, analysis: &NetworkBalanceAnalysis) -> Vec<NodeRebalancingPriority> {
        let mut priorities = Vec::with_capacity(analysis.node_balances.len());

        for node_balance in &analysis.node_balances {
            let priority_score = self.calculate_node_priority(node_balance, analysis);
            
            priorities.push(NodeRebalancingPriority {
                node_id: node_balance.node_id.clone(),
                priority_score,
                balance_score: node_balance.balance_score,
                deficit: node_balance.entanglement_deficit(),
                is_overloaded: node_balance.is_overloaded(),
                is_underloaded: node_balance.is_underloaded(),
            });
        }

        // Sort by priority score (descending)
        priorities.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());
        priorities
    }

    /// Calculate node priority for rebalancing
    #[inline]
    fn calculate_node_priority(&self, node_balance: &NodeBalance, analysis: &NetworkBalanceAnalysis) -> f64 {
        let weights = &self.priority_weights;
        
        // Base priority from node's own balance score
        let balance_priority = node_balance.balance_score * weights.balance_weight;
        
        // Priority from load factor deviation
        let load_factor = node_balance.load_factor();
        let load_deviation = if load_factor.is_finite() {
            (load_factor - 1.0).abs()
        } else {
            1.0
        };
        let load_priority = load_deviation * weights.load_weight;
        
        // Priority from network impact (larger nodes have higher impact)
        let size_factor = (node_balance.current_entanglements as f64).ln_1p() / 10.0;
        let size_priority = size_factor * weights.size_weight;
        
        // Priority from urgency (how far from optimal)
        let deficit_ratio = if node_balance.optimal_entanglements > 0 {
            node_balance.entanglement_deficit().abs() as f64 / node_balance.optimal_entanglements as f64
        } else {
            0.0
        };
        let urgency_priority = deficit_ratio * weights.urgency_weight;
        
        (balance_priority + load_priority + size_priority + urgency_priority).min(1.0)
    }

    /// Get balancing reason based on analysis
    #[inline]
    pub fn get_balancing_reason(&self, analysis: &NetworkBalanceAnalysis) -> String {
        let stats = analysis.get_distribution_statistics();
        
        if analysis.overloaded_nodes > analysis.underloaded_nodes {
            format!(
                "Load concentration detected: {} overloaded nodes with {:.1}% variance",
                analysis.overloaded_nodes,
                analysis.load_distribution_variance * 100.0
            )
        } else if analysis.underloaded_nodes > analysis.overloaded_nodes {
            format!(
                "Load dispersion detected: {} underloaded nodes with {:.2} load range",
                analysis.underloaded_nodes,
                stats.load_range
            )
        } else if analysis.overall_balance_score > 0.5 {
            format!(
                "General imbalance detected: {:.1}% overall imbalance with {:.2} coefficient of variation",
                analysis.overall_balance_score * 100.0,
                stats.coefficient_of_variation
            )
        } else {
            format!(
                "Proactive balancing: {:.1}% potential improvement available",
                analysis.calculate_potential_improvement()
            )
        }
    }

    /// Adjust strategy based on performance feedback
    #[inline]
    pub fn adjust_for_performance(&mut self, success_rate: f64, efficiency_improvement: f64) {
        if success_rate < 0.7 {
            // Poor success rate - be more conservative
            self.max_redistributions = (self.max_redistributions as f64 * 0.8) as usize;
            self.min_improvement_threshold *= 1.2;
            self.load_balancing_factor *= 0.9;
        } else if success_rate > 0.9 && efficiency_improvement > 5.0 {
            // Excellent performance - be more aggressive
            self.max_redistributions = (self.max_redistributions as f64 * 1.1) as usize;
            self.min_improvement_threshold *= 0.9;
            self.load_balancing_factor = (self.load_balancing_factor * 1.1).min(1.0);
        }

        debug!("Adjusted balancing strategy based on performance: success_rate={:.2}, efficiency_improvement={:.1}%", 
               success_rate, efficiency_improvement);
    }
}

/// Types of balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BalancingStrategyType {
    /// Conservative approach with minimal changes
    Conservative,
    /// Adaptive approach based on network conditions
    Adaptive,
    /// Aggressive approach for severe imbalances
    Aggressive,
    /// Greedy approach for quick improvements
    Greedy,
    /// Optimal approach for best results
    Optimal,
}

/// Priority weights for different balancing factors
#[derive(Debug, Clone)]
pub struct PriorityWeights {
    /// Weight for node balance score
    pub balance_weight: f64,
    /// Weight for load factor deviation
    pub load_weight: f64,
    /// Weight for node size impact
    pub size_weight: f64,
    /// Weight for urgency of rebalancing
    pub urgency_weight: f64,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            balance_weight: 0.4,
            load_weight: 0.3,
            size_weight: 0.2,
            urgency_weight: 0.1,
        }
    }
}

impl PriorityWeights {
    /// Create weights focused on speed
    #[inline]
    pub fn speed_focused() -> Self {
        Self {
            balance_weight: 0.5,
            load_weight: 0.3,
            size_weight: 0.1,
            urgency_weight: 0.1,
        }
    }

    /// Create weights focused on quality
    #[inline]
    pub fn quality_focused() -> Self {
        Self {
            balance_weight: 0.3,
            load_weight: 0.3,
            size_weight: 0.2,
            urgency_weight: 0.2,
        }
    }

    /// Create adaptive weights based on imbalance severity
    #[inline]
    pub fn adaptive(imbalance_severity: f64) -> Self {
        if imbalance_severity > 0.7 {
            // High imbalance - prioritize urgency
            Self {
                balance_weight: 0.3,
                load_weight: 0.2,
                size_weight: 0.2,
                urgency_weight: 0.3,
            }
        } else if imbalance_severity > 0.3 {
            // Medium imbalance - balanced approach
            Self::default()
        } else {
            // Low imbalance - prioritize efficiency
            Self {
                balance_weight: 0.2,
                load_weight: 0.4,
                size_weight: 0.3,
                urgency_weight: 0.1,
            }
        }
    }
}

/// Node rebalancing priority information
#[derive(Debug, Clone)]
pub struct NodeRebalancingPriority {
    /// Node identifier
    pub node_id: String,
    /// Calculated priority score
    pub priority_score: f64,
    /// Balance score from analysis
    pub balance_score: f64,
    /// Entanglement deficit (negative if overloaded)
    pub deficit: i32,
    /// Whether node is overloaded
    pub is_overloaded: bool,
    /// Whether node is underloaded
    pub is_underloaded: bool,
}

impl NodeRebalancingPriority {
    /// Check if node requires immediate attention
    #[inline]
    pub fn requires_immediate_attention(&self, threshold: f64) -> bool {
        self.priority_score >= threshold
    }

    /// Get rebalancing action needed
    #[inline]
    pub fn get_rebalancing_action(&self) -> RebalancingAction {
        if self.is_overloaded {
            RebalancingAction::ReduceLoad
        } else if self.is_underloaded {
            RebalancingAction::IncreaseLoad
        } else {
            RebalancingAction::Maintain
        }
    }

    /// Calculate redistribution amount needed
    #[inline]
    pub fn calculate_redistribution_amount(&self, factor: f64) -> usize {
        (self.deficit.abs() as f64 * factor) as usize
    }
}

/// Rebalancing actions for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebalancingAction {
    /// Reduce load on this node
    ReduceLoad,
    /// Increase load on this node
    IncreaseLoad,
    /// Maintain current load
    Maintain,
}

/// Strategy selector for choosing optimal balancing strategy
pub struct StrategySelector;

impl StrategySelector {
    /// Select optimal strategy based on network conditions
    #[inline]
    pub fn select_optimal_strategy(
        topology: &NetworkTopology,
        analysis: &NetworkBalanceAnalysis,
        performance_history: &PerformanceHistory,
    ) -> BalancingStrategy {
        let network_complexity = Self::assess_network_complexity(topology);
        let imbalance_severity = analysis.overall_balance_score;
        let historical_success = performance_history.average_success_rate();

        match (network_complexity, imbalance_severity, historical_success) {
            (NetworkComplexity::Low, _, _) if imbalance_severity < 0.3 => {
                BalancingStrategy::speed_optimized()
            }
            (NetworkComplexity::High, _, _) if imbalance_severity > 0.7 => {
                BalancingStrategy::quality_optimized()
            }
            (_, _, success_rate) if success_rate < 0.6 => {
                // Poor historical performance - use conservative approach
                let mut strategy = BalancingStrategy::default();
                strategy.strategy_type = BalancingStrategyType::Conservative;
                strategy
            }
            _ => {
                // Use adaptive strategy for most cases
                BalancingStrategy::adaptive(topology, analysis)
            }
        }
    }

    /// Assess network complexity
    #[inline]
    fn assess_network_complexity(topology: &NetworkTopology) -> NetworkComplexity {
        let node_count = topology.node_degrees.len();
        let avg_degree = if node_count > 0 {
            topology.node_degrees.values().sum::<usize>() as f64 / node_count as f64
        } else {
            0.0
        };

        match (node_count, avg_degree) {
            (n, _) if n < 50 => NetworkComplexity::Low,
            (n, d) if n < 200 && d < 10.0 => NetworkComplexity::Medium,
            _ => NetworkComplexity::High,
        }
    }
}

/// Network complexity assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkComplexity {
    Low,
    Medium,
    High,
}

/// Performance history for strategy selection
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    pub recent_operations: Vec<OperationResult>,
    pub success_rates: Vec<f64>,
    pub efficiency_improvements: Vec<f64>,
}

impl PerformanceHistory {
    /// Create new performance history
    #[inline]
    pub fn new() -> Self {
        Self {
            recent_operations: Vec::new(),
            success_rates: Vec::new(),
            efficiency_improvements: Vec::new(),
        }
    }

    /// Add operation result
    #[inline]
    pub fn add_result(&mut self, result: OperationResult) {
        self.recent_operations.push(result.clone());
        self.success_rates.push(if result.success { 1.0 } else { 0.0 });
        self.efficiency_improvements.push(result.efficiency_improvement);

        // Keep only recent history (last 100 operations)
        if self.recent_operations.len() > 100 {
            self.recent_operations.remove(0);
            self.success_rates.remove(0);
            self.efficiency_improvements.remove(0);
        }
    }

    /// Get average success rate
    #[inline]
    pub fn average_success_rate(&self) -> f64 {
        if self.success_rates.is_empty() {
            0.5 // Default neutral value
        } else {
            self.success_rates.iter().sum::<f64>() / self.success_rates.len() as f64
        }
    }

    /// Get average efficiency improvement
    #[inline]
    pub fn average_efficiency_improvement(&self) -> f64 {
        if self.efficiency_improvements.is_empty() {
            0.0
        } else {
            self.efficiency_improvements.iter().sum::<f64>() / self.efficiency_improvements.len() as f64
        }
    }
}

impl Default for PerformanceHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation result for performance tracking
#[derive(Debug, Clone)]
pub struct OperationResult {
    pub success: bool,
    pub efficiency_improvement: f64,
    pub redistributions_made: usize,
    pub operation_time_ms: u64,
}
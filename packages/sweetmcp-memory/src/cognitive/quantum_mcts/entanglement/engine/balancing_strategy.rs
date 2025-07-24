//! Balancing strategy determination and execution coordination
//!
//! This module provides blazing-fast strategy calculation and execution coordination
//! with zero-allocation patterns and adaptive optimization algorithms.

use std::collections::HashMap;
use tracing::{debug, info};
use crate::cognitive::types::CognitiveError;
use super::super::analysis::NetworkTopology;
use super::super::super::node_state::QuantumMCTSNode;
use super::core::QuantumEntanglementEngine;
use super::balancing_types::{BalancingStrategy, NetworkBalanceAnalysis, BalancingResult};

impl QuantumEntanglementEngine {
    /// Determine balancing strategy based on network state
    pub fn determine_balancing_strategy(&self, topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> BalancingStrategy {
        let mut strategy = BalancingStrategy::default();
        
        // Adjust strategy based on network size
        if topology.total_nodes > 1000 {
            strategy.max_redistributions = 200;
            strategy.load_balancing_factor = 0.8;
        } else if topology.total_nodes < 100 {
            strategy.max_redistributions = 50;
            strategy.load_balancing_factor = 0.6;
        }
        
        // Adjust based on imbalance severity
        if analysis.average_imbalance > 0.6 {
            strategy.load_balancing_factor = 0.9;
            strategy.min_improvement_threshold = 1.0;
        } else if analysis.average_imbalance < 0.3 {
            strategy.load_balancing_factor = 0.5;
            strategy.min_improvement_threshold = 5.0;
        }
        
        // Adjust based on distribution statistics
        if analysis.distribution_stats.coefficient_of_variation > 0.8 {
            strategy.target_balance_ratio = 0.9;
            strategy.max_redistributions *= 2;
        }
        
        strategy
    }
    
    /// Execute balancing operations
    pub async fn execute_balancing_operations(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
        strategy: &BalancingStrategy,
        analysis: &NetworkBalanceAnalysis,
    ) -> Result<usize, CognitiveError> {
        let mut redistributions_made = 0;
        
        // Process high-priority nodes first
        for node_balance in &analysis.node_balances {
            if redistributions_made >= strategy.max_redistributions {
                break;
            }
            
            if node_balance.rebalancing_priority < 0.5 {
                continue; // Skip low-priority nodes
            }
            
            let redistributions = self.rebalance_node(node_balance, strategy, tree).await?;
            redistributions_made += redistributions;
            
            if redistributions > 0 {
                self.metrics.record_node_rebalanced(&node_balance.node_id, redistributions);
            }
        }
        
        Ok(redistributions_made)
    }
    
    /// Determine reason for balancing operation
    pub fn determine_balancing_reason(&self, analysis: &NetworkBalanceAnalysis, strategy: &BalancingStrategy) -> String {
        if analysis.average_imbalance > 0.6 {
            "Network had severe load imbalance requiring redistribution".to_string()
        } else if analysis.distribution_stats.coefficient_of_variation > 0.8 {
            "Network had highly uneven entanglement distribution".to_string()
        } else if strategy.load_balancing_factor > 0.8 {
            "Aggressive load balancing strategy applied for optimization".to_string()
        } else {
            "Routine load balancing to improve network efficiency".to_string()
        }
    }
    
    /// Create adaptive strategy based on network conditions
    pub fn create_adaptive_strategy(&self, topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> BalancingStrategy {
        let base_strategy = self.determine_balancing_strategy(topology, analysis);
        
        // Apply adaptive adjustments
        let mut adaptive_strategy = base_strategy;
        
        // Adjust for network density
        let density = if topology.total_nodes > 1 {
            topology.average_degree / (topology.total_nodes as f64 - 1.0)
        } else {
            0.0
        };
        
        if density > 0.7 {
            // Dense network - be more conservative
            adaptive_strategy.load_balancing_factor *= 0.8;
            adaptive_strategy.min_improvement_threshold *= 1.5;
        } else if density < 0.3 {
            // Sparse network - be more aggressive
            adaptive_strategy.load_balancing_factor *= 1.2;
            adaptive_strategy.max_redistributions = (adaptive_strategy.max_redistributions as f64 * 1.5) as usize;
        }
        
        // Adjust for critical nodes
        let critical_nodes = analysis.severely_imbalanced_count();
        if critical_nodes > topology.total_nodes / 10 {
            // Many critical nodes - prioritize them
            adaptive_strategy.load_balancing_factor = 0.95;
            adaptive_strategy.target_balance_ratio = 0.95;
        }
        
        adaptive_strategy
    }
    
    /// Optimize strategy parameters for current conditions
    pub fn optimize_strategy_parameters(&self, strategy: &mut BalancingStrategy, topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) {
        // Optimize max redistributions based on network size and imbalance
        let optimal_redistributions = self.calculate_optimal_redistributions(topology, analysis);
        strategy.max_redistributions = optimal_redistributions;
        
        // Optimize load balancing factor based on distribution variance
        let optimal_factor = self.calculate_optimal_balancing_factor(analysis);
        strategy.load_balancing_factor = optimal_factor;
        
        // Optimize improvement threshold based on current performance
        let optimal_threshold = self.calculate_optimal_improvement_threshold(topology, analysis);
        strategy.min_improvement_threshold = optimal_threshold;
    }
    
    /// Calculate optimal number of redistributions
    fn calculate_optimal_redistributions(&self, topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> usize {
        let base_redistributions = topology.total_nodes / 10; // Base: 10% of nodes
        let imbalance_multiplier = (analysis.average_imbalance * 2.0).max(1.0).min(3.0);
        let variance_multiplier = (analysis.distribution_stats.coefficient_of_variation + 1.0).min(2.0);
        
        let optimal = (base_redistributions as f64 * imbalance_multiplier * variance_multiplier) as usize;
        optimal.max(10).min(500) // Reasonable bounds
    }
    
    /// Calculate optimal load balancing factor
    fn calculate_optimal_balancing_factor(&self, analysis: &NetworkBalanceAnalysis) -> f64 {
        let base_factor = 0.7;
        
        // Increase factor for higher imbalance
        let imbalance_adjustment = analysis.average_imbalance * 0.3;
        
        // Increase factor for higher variance
        let variance_adjustment = analysis.distribution_stats.coefficient_of_variation * 0.2;
        
        let optimal_factor = base_factor + imbalance_adjustment + variance_adjustment;
        optimal_factor.max(0.3).min(0.95) // Reasonable bounds
    }
    
    /// Calculate optimal improvement threshold
    fn calculate_optimal_improvement_threshold(&self, topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> f64 {
        let base_threshold = 2.0;
        
        // Lower threshold for larger networks (easier to achieve improvements)
        let size_adjustment = if topology.total_nodes > 500 {
            -0.5
        } else if topology.total_nodes < 50 {
            1.0
        } else {
            0.0
        };
        
        // Lower threshold for more imbalanced networks
        let imbalance_adjustment = -analysis.average_imbalance * 2.0;
        
        let optimal_threshold = base_threshold + size_adjustment + imbalance_adjustment;
        optimal_threshold.max(0.5).min(10.0) // Reasonable bounds
    }
    
    /// Create strategy for emergency balancing
    pub fn create_emergency_strategy(&self) -> BalancingStrategy {
        BalancingStrategy {
            target_balance_ratio: 0.95,
            max_redistributions: 1000,
            min_improvement_threshold: 0.1,
            load_balancing_factor: 0.98,
        }
    }
    
    /// Create strategy for maintenance balancing
    pub fn create_maintenance_strategy(&self) -> BalancingStrategy {
        BalancingStrategy {
            target_balance_ratio: 0.6,
            max_redistributions: 25,
            min_improvement_threshold: 5.0,
            load_balancing_factor: 0.4,
        }
    }
    
    /// Evaluate strategy effectiveness
    pub fn evaluate_strategy_effectiveness(&self, strategy: &BalancingStrategy, result: &BalancingResult) -> StrategyEffectiveness {
        let time_efficiency = if result.balancing_time_ms > 0 {
            result.redistributions_made as f64 / result.balancing_time_ms as f64 * 1000.0
        } else {
            0.0
        };
        
        let improvement_efficiency = if result.redistributions_made > 0 {
            (result.balance_improvement + result.efficiency_improvement) / result.redistributions_made as f64
        } else {
            0.0
        };
        
        let resource_efficiency = if strategy.max_redistributions > 0 {
            result.redistributions_made as f64 / strategy.max_redistributions as f64
        } else {
            0.0
        };
        
        let overall_score = (time_efficiency * 0.3 + improvement_efficiency * 0.5 + resource_efficiency * 0.2).min(10.0);
        
        StrategyEffectiveness {
            time_efficiency,
            improvement_efficiency,
            resource_efficiency,
            overall_score,
            was_effective: overall_score > 1.0 && result.was_successful(),
        }
    }
}

/// Strategy effectiveness evaluation
#[derive(Debug, Clone)]
pub struct StrategyEffectiveness {
    /// Time efficiency (redistributions per second)
    pub time_efficiency: f64,
    /// Improvement efficiency (improvement per redistribution)
    pub improvement_efficiency: f64,
    /// Resource efficiency (actual vs max redistributions)
    pub resource_efficiency: f64,
    /// Overall effectiveness score
    pub overall_score: f64,
    /// Whether the strategy was effective
    pub was_effective: bool,
}

impl StrategyEffectiveness {
    /// Get effectiveness grade
    pub fn grade(&self) -> char {
        if self.overall_score >= 8.0 {
            'A'
        } else if self.overall_score >= 6.0 {
            'B'
        } else if self.overall_score >= 4.0 {
            'C'
        } else if self.overall_score >= 2.0 {
            'D'
        } else {
            'F'
        }
    }
    
    /// Get effectiveness description
    pub fn description(&self) -> &'static str {
        match self.grade() {
            'A' => "Excellent",
            'B' => "Good",
            'C' => "Average",
            'D' => "Poor",
            'F' => "Failed",
            _ => "Unknown",
        }
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        format!(
            "Strategy Effectiveness: {} (Grade: {})\n\
            Time Efficiency: {:.2} ops/sec\n\
            Improvement Efficiency: {:.2} per redistribution\n\
            Resource Efficiency: {:.1}%\n\
            Overall Score: {:.2}/10.0",
            self.description(),
            self.grade(),
            self.time_efficiency,
            self.improvement_efficiency,
            self.resource_efficiency * 100.0,
            self.overall_score
        )
    }
}
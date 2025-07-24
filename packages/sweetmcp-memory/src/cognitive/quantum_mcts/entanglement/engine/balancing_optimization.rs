//! Balancing optimization strategies and execution
//!
//! This module provides optimization strategies and execution logic for
//! entanglement distribution balancing with zero-allocation patterns.

use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

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
    BalancingResult, NodeBalance, BalancingStrategy, NetworkBalanceAnalysis,
};

impl QuantumEntanglementEngine {
    /// Balance entanglement distribution across the network
    pub async fn balance_entanglement_distribution(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<BalancingResult, CognitiveError> {
        let start_time = Instant::now();
        
        debug!("Starting entanglement distribution balancing for {} nodes", tree.len());
        
        // Analyze current network topology and balance
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        let balance_analysis = self.analyze_current_balance(tree, &topology).await?;
        
        // Determine if balancing is needed
        if !self.should_perform_balancing(&balance_analysis, &topology) {
            debug!("Network balance is acceptable, skipping balancing operation");
            return Ok(BalancingResult::skipped(
                start_time.elapsed().as_millis() as u64,
                "Network balance was already acceptable".to_string(),
            ));
        }
        
        // Determine balancing strategy
        let strategy = self.determine_balancing_strategy(&topology, &balance_analysis);
        
        // Execute balancing operations
        let redistributions_made = self.execute_balancing_operations(tree, &strategy, &balance_analysis).await?;
        
        let balancing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Calculate improvements
        let new_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        let new_balance_analysis = self.analyze_current_balance(tree, &new_topology).await?;
        
        let balance_improvement = self.calculate_balance_improvement(&balance_analysis, &new_balance_analysis);
        let efficiency_improvement = self.calculate_efficiency_improvement(&topology, &new_topology);
        
        // Determine reason for balancing
        let reason = self.determine_balancing_reason(&balance_analysis, &strategy);
        
        // Record balancing metrics
        self.metrics.record_balancing_operation(
            redistributions_made,
            balancing_time_ms,
            balance_improvement,
            efficiency_improvement,
        );
        
        info!(
            "Balancing completed: {} redistributions in {}ms (+{:.1}% balance, +{:.1}% efficiency)",
            redistributions_made,
            balancing_time_ms,
            balance_improvement,
            efficiency_improvement
        );
        
        Ok(BalancingResult::new(
            redistributions_made,
            balancing_time_ms,
            balance_improvement,
            efficiency_improvement,
            reason,
        ))
    }
    
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
    
    /// Execute balancing operations according to strategy
    async fn execute_balancing_operations(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
        strategy: &BalancingStrategy,
        balance_analysis: &NetworkBalanceAnalysis,
    ) -> Result<usize, CognitiveError> {
        let mut redistributions_made = 0;
        let mut operations_performed = 0;
        
        debug!("Executing balancing operations with strategy: max_redistributions={}", strategy.max_redistributions);
        
        // Get nodes that need rebalancing, sorted by priority
        let nodes_to_rebalance = balance_analysis.nodes_needing_rebalancing();
        
        for node_balance in nodes_to_rebalance {
            if operations_performed >= strategy.max_redistributions {
                debug!("Reached maximum redistributions limit: {}", strategy.max_redistributions);
                break;
            }
            
            // Determine redistribution type and amount
            let redistribution_result = if node_balance.is_over_entangled() {
                self.redistribute_from_node(
                    &node_balance.node_id,
                    node_balance.imbalance_magnitude(),
                    tree,
                    strategy,
                ).await?
            } else {
                self.redistribute_to_node(
                    &node_balance.node_id,
                    node_balance.imbalance_magnitude(),
                    tree,
                    strategy,
                ).await?
            };
            
            redistributions_made += redistribution_result;
            operations_performed += 1;
            
            // Check if we've made sufficient improvement
            if operations_performed % 10 == 0 {
                let current_topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
                let current_analysis = self.analyze_current_balance(tree, &current_topology).await?;
                let improvement = self.calculate_balance_improvement(balance_analysis, &current_analysis);
                
                if improvement >= strategy.min_improvement_threshold * 2.0 {
                    debug!("Early termination: sufficient improvement achieved ({:.1}%)", improvement);
                    break;
                }
            }
        }
        
        debug!("Balancing operations completed: {} redistributions across {} operations", 
               redistributions_made, operations_performed);
        
        Ok(redistributions_made)
    }
    
    /// Redistribute entanglements from an over-entangled node
    async fn redistribute_from_node(
        &mut self,
        source_node_id: &str,
        target_redistributions: usize,
        tree: &HashMap<String, QuantumMCTSNode>,
        strategy: &BalancingStrategy,
    ) -> Result<usize, CognitiveError> {
        let mut redistributions_made = 0;
        let max_redistributions = (target_redistributions as f64 * strategy.load_balancing_factor) as usize;
        
        // Get current entanglements for the source node
        let source_entanglements = self.manager.get_node_entanglements(source_node_id).await?;
        
        // Find suitable target nodes (under-entangled nodes)
        let mut target_candidates = Vec::new();
        for (node_id, node) in tree {
            if node_id == source_node_id {
                continue;
            }
            
            let current_entanglements = self.manager.get_node_entanglement_count(node_id).await?;
            let optimal_entanglements = self.calculate_optimal_entanglement_count(node, 0.0); // Will be recalculated
            
            if current_entanglements < optimal_entanglements {
                target_candidates.push((node_id.clone(), optimal_entanglements - current_entanglements));
            }
        }
        
        // Sort target candidates by capacity (descending)
        target_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Redistribute entanglements
        for entanglement_id in source_entanglements.iter().take(max_redistributions) {
            if target_candidates.is_empty() {
                break;
            }
            
            // Select best target node
            let (target_node_id, _) = &target_candidates[0];
            
            // Perform redistribution
            if self.manager.redistribute_entanglement(entanglement_id, source_node_id, target_node_id).await.is_ok() {
                redistributions_made += 1;
                
                // Update target candidate capacity
                target_candidates[0].1 = target_candidates[0].1.saturating_sub(1);
                if target_candidates[0].1 == 0 {
                    target_candidates.remove(0);
                }
            }
        }
        
        Ok(redistributions_made)
    }
    
    /// Redistribute entanglements to an under-entangled node
    async fn redistribute_to_node(
        &mut self,
        target_node_id: &str,
        target_redistributions: usize,
        tree: &HashMap<String, QuantumMCTSNode>,
        strategy: &BalancingStrategy,
    ) -> Result<usize, CognitiveError> {
        let mut redistributions_made = 0;
        let max_redistributions = (target_redistributions as f64 * strategy.load_balancing_factor) as usize;
        
        // Find suitable source nodes (over-entangled nodes)
        let mut source_candidates = Vec::new();
        for (node_id, node) in tree {
            if node_id == target_node_id {
                continue;
            }
            
            let current_entanglements = self.manager.get_node_entanglement_count(node_id).await?;
            let optimal_entanglements = self.calculate_optimal_entanglement_count(node, 0.0); // Will be recalculated
            
            if current_entanglements > optimal_entanglements {
                let excess = current_entanglements - optimal_entanglements;
                let entanglements = self.manager.get_node_entanglements(node_id).await?;
                source_candidates.push((node_id.clone(), excess, entanglements));
            }
        }
        
        // Sort source candidates by excess (descending)
        source_candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Redistribute entanglements
        let mut remaining_redistributions = max_redistributions;
        for (source_node_id, excess, entanglements) in source_candidates {
            if remaining_redistributions == 0 {
                break;
            }
            
            let redistributions_from_source = excess.min(remaining_redistributions);
            
            for entanglement_id in entanglements.iter().take(redistributions_from_source) {
                if self.manager.redistribute_entanglement(entanglement_id, &source_node_id, target_node_id).await.is_ok() {
                    redistributions_made += 1;
                    remaining_redistributions -= 1;
                    
                    if remaining_redistributions == 0 {
                        break;
                    }
                }
            }
        }
        
        Ok(redistributions_made)
    }
}
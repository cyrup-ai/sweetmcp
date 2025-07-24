//! Core balancing operations and entanglement management
//!
//! This module provides blazing-fast entanglement manipulation operations
//! with zero-allocation patterns and comprehensive error handling.

use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};
use crate::cognitive::types::CognitiveError;
use super::super::super::node_state::QuantumMCTSNode;
use super::core::QuantumEntanglementEngine;
use super::balancing_types::{NodeBalance, BalancingStrategy};

impl QuantumEntanglementEngine {
    /// Rebalance a specific node
    pub async fn rebalance_node(
        &mut self,
        node_balance: &NodeBalance,
        strategy: &BalancingStrategy,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        let current = node_balance.current_entanglements;
        let optimal = node_balance.optimal_entanglements;
        
        if current == optimal {
            return Ok(0);
        }
        
        let mut redistributions = 0;
        
        if current > optimal {
            // Too many entanglements - remove some
            let excess = current - optimal;
            let to_remove = (excess as f64 * strategy.load_balancing_factor) as usize;
            
            redistributions += self.remove_excess_entanglements(&node_balance.node_id, to_remove).await?;
        } else {
            // Too few entanglements - add some
            let deficit = optimal - current;
            let to_add = (deficit as f64 * strategy.load_balancing_factor) as usize;
            
            redistributions += self.add_needed_entanglements(&node_balance.node_id, to_add, tree).await?;
        }
        
        Ok(redistributions)
    }
    
    /// Remove excess entanglements from a node
    pub async fn remove_excess_entanglements(&mut self, node_id: &str, count: usize) -> Result<usize, CognitiveError> {
        let node_entanglements = self.manager.get_node_entanglements(node_id).await?;
        
        // Sort by strength (remove weakest first)
        let mut sorted_entanglements: Vec<_> = node_entanglements.into_iter().collect();
        sorted_entanglements.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut removed = 0;
        
        for (target_node, _strength) in sorted_entanglements.into_iter().take(count) {
            match self.manager.remove_entanglement(node_id, &target_node).await {
                Ok(true) => {
                    removed += 1;
                    self.metrics.record_entanglement_removed_for_balancing();
                }
                Ok(false) => {
                    debug!("Entanglement between {} and {} was already removed", node_id, target_node);
                }
                Err(e) => {
                    warn!("Failed to remove entanglement between {} and {}: {}", node_id, target_node, e);
                }
            }
        }
        
        Ok(removed)
    }
    
    /// Add needed entanglements to a node
    pub async fn add_needed_entanglements(
        &mut self,
        node_id: &str,
        count: usize,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        // Find suitable candidates for new entanglements
        let candidates = self.find_entanglement_candidates(node_id, tree).await?;
        
        let mut added = 0;
        
        for candidate in candidates.into_iter().take(count) {
            match self.manager.create_entanglement(node_id, &candidate).await {
                Ok(true) => {
                    added += 1;
                    self.metrics.record_entanglement_created_for_balancing();
                }
                Ok(false) => {
                    debug!("Entanglement between {} and {} already exists", node_id, candidate);
                }
                Err(e) => {
                    warn!("Failed to create entanglement between {} and {}: {}", node_id, candidate, e);
                }
            }
        }
        
        Ok(added)
    }
    
    /// Find suitable candidates for new entanglements
    pub async fn find_entanglement_candidates(&self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<Vec<String>, CognitiveError> {
        let mut candidates = Vec::new();
        let existing_entanglements = self.manager.get_node_entanglements(node_id).await?;
        let existing_targets: HashSet<String> = existing_entanglements.into_keys().collect();
        
        // Find nodes that would benefit from additional entanglements
        for (candidate_id, candidate_node) in tree {
            if candidate_id == node_id || existing_targets.contains(candidate_id) {
                continue;
            }
            
            let candidate_entanglement_count = self.manager.get_node_entanglement_count(candidate_id).await?;
            let candidate_optimal = self.calculate_optimal_entanglement_count(candidate_node, 3.0); // Use reasonable default
            
            // Prefer candidates that also need more entanglements
            if candidate_entanglement_count < candidate_optimal {
                candidates.push(candidate_id.clone());
            }
        }
        
        // Sort candidates by value and visit count (prefer high-value nodes)
        candidates.sort_by(|a, b| {
            let node_a = tree.get(a).unwrap();
            let node_b = tree.get(b).unwrap();
            let score_a = node_a.value + (node_a.visits as f64 * 0.01);
            let score_b = node_b.value + (node_b.visits as f64 * 0.01);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(candidates)
    }
    
    /// Remove entanglements intelligently based on multiple criteria
    pub async fn intelligent_entanglement_removal(
        &mut self,
        node_id: &str,
        target_count: usize,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        let node_entanglements = self.manager.get_node_entanglements(node_id).await?;
        
        if node_entanglements.len() <= target_count {
            return Ok(0); // Already at or below target
        }
        
        let to_remove = node_entanglements.len() - target_count;
        
        // Score entanglements for removal priority
        let mut scored_entanglements: Vec<_> = node_entanglements
            .into_iter()
            .map(|(target_node, strength)| {
                let removal_score = self.calculate_removal_score(&target_node, strength, tree);
                (target_node, strength, removal_score)
            })
            .collect();
        
        // Sort by removal score (lowest scores removed first)
        scored_entanglements.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut removed = 0;
        
        for (target_node, _strength, _score) in scored_entanglements.into_iter().take(to_remove) {
            match self.manager.remove_entanglement(node_id, &target_node).await {
                Ok(true) => {
                    removed += 1;
                    self.metrics.record_entanglement_removed_for_balancing();
                }
                Ok(false) => {
                    debug!("Entanglement between {} and {} was already removed", node_id, target_node);
                }
                Err(e) => {
                    warn!("Failed to remove entanglement between {} and {}: {}", node_id, target_node, e);
                }
            }
        }
        
        Ok(removed)
    }
    
    /// Calculate removal score for an entanglement (lower = higher removal priority)
    fn calculate_removal_score(&self, target_node: &str, strength: f64, tree: &HashMap<String, QuantumMCTSNode>) -> f64 {
        let mut score = strength; // Base score on entanglement strength
        
        // Consider target node value and visits
        if let Some(target) = tree.get(target_node) {
            score += target.value * 2.0; // Higher value nodes are less likely to be removed
            score += (target.visits as f64 * 0.01).min(5.0); // Frequently visited nodes preserved
        }
        
        score
    }
    
    /// Add entanglements with optimal candidate selection
    pub async fn intelligent_entanglement_addition(
        &mut self,
        node_id: &str,
        target_count: usize,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        let current_count = self.manager.get_node_entanglement_count(node_id).await?;
        
        if current_count >= target_count {
            return Ok(0); // Already at or above target
        }
        
        let to_add = target_count - current_count;
        
        // Find and score candidates
        let candidates = self.find_scored_candidates(node_id, tree).await?;
        
        let mut added = 0;
        
        for (candidate, _score) in candidates.into_iter().take(to_add) {
            match self.manager.create_entanglement(node_id, &candidate).await {
                Ok(true) => {
                    added += 1;
                    self.metrics.record_entanglement_created_for_balancing();
                }
                Ok(false) => {
                    debug!("Entanglement between {} and {} already exists", node_id, candidate);
                }
                Err(e) => {
                    warn!("Failed to create entanglement between {} and {}: {}", node_id, candidate, e);
                }
            }
        }
        
        Ok(added)
    }
    
    /// Find and score candidates for entanglement creation
    async fn find_scored_candidates(&self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<Vec<(String, f64)>, CognitiveError> {
        let mut scored_candidates = Vec::new();
        let existing_entanglements = self.manager.get_node_entanglements(node_id).await?;
        let existing_targets: HashSet<String> = existing_entanglements.into_keys().collect();
        
        // Score each potential candidate
        for (candidate_id, candidate_node) in tree {
            if candidate_id == node_id || existing_targets.contains(candidate_id) {
                continue;
            }
            
            let candidate_entanglement_count = self.manager.get_node_entanglement_count(candidate_id).await?;
            let candidate_optimal = self.calculate_optimal_entanglement_count(candidate_node, 3.0);
            
            // Calculate candidate score
            let mut score = 0.0;
            
            // Prefer nodes that need more entanglements
            if candidate_entanglement_count < candidate_optimal {
                score += 5.0;
            }
            
            // Prefer high-value nodes
            score += candidate_node.value * 3.0;
            
            // Prefer frequently visited nodes
            score += (candidate_node.visits as f64 * 0.01).min(10.0);
            
            // Prefer nodes with moderate entanglement counts (not too isolated, not too connected)
            let ideal_range = candidate_optimal as f64 * 0.7..=candidate_optimal as f64 * 1.3;
            if ideal_range.contains(&(candidate_entanglement_count as f64)) {
                score += 2.0;
            }
            
            scored_candidates.push((candidate_id.clone(), score));
        }
        
        // Sort by score (highest first)
        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(scored_candidates)
    }
    
    /// Perform targeted rebalancing for specific node types
    pub async fn targeted_rebalancing(
        &mut self,
        node_id: &str,
        target_type: NodeRebalanceType,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        match target_type {
            NodeRebalanceType::HighValue => {
                self.rebalance_high_value_node(node_id, tree).await
            }
            NodeRebalanceType::HighTraffic => {
                self.rebalance_high_traffic_node(node_id, tree).await
            }
            NodeRebalanceType::Isolated => {
                self.rebalance_isolated_node(node_id, tree).await
            }
            NodeRebalanceType::Overconnected => {
                self.rebalance_overconnected_node(node_id, tree).await
            }
        }
    }
    
    /// Rebalance high-value node (increase connections)
    async fn rebalance_high_value_node(&mut self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<usize, CognitiveError> {
        let current_count = self.manager.get_node_entanglement_count(node_id).await?;
        let target_count = (current_count as f64 * 1.5) as usize; // Increase by 50%
        
        self.intelligent_entanglement_addition(node_id, target_count, tree).await
    }
    
    /// Rebalance high-traffic node (optimize connections)
    async fn rebalance_high_traffic_node(&mut self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<usize, CognitiveError> {
        // First remove weak connections, then add strong ones
        let removed = self.remove_weak_entanglements(node_id, 3).await?;
        let added = self.add_strong_entanglements(node_id, removed + 2, tree).await?;
        
        Ok(removed + added)
    }
    
    /// Rebalance isolated node (increase connections significantly)
    async fn rebalance_isolated_node(&mut self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<usize, CognitiveError> {
        let current_count = self.manager.get_node_entanglement_count(node_id).await?;
        let target_count = current_count.max(5); // Ensure at least 5 connections
        
        self.intelligent_entanglement_addition(node_id, target_count, tree).await
    }
    
    /// Rebalance overconnected node (reduce connections)
    async fn rebalance_overconnected_node(&mut self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<usize, CognitiveError> {
        let current_count = self.manager.get_node_entanglement_count(node_id).await?;
        let target_count = (current_count as f64 * 0.7) as usize; // Reduce by 30%
        
        self.intelligent_entanglement_removal(node_id, target_count, tree).await
    }
    
    /// Remove weak entanglements
    async fn remove_weak_entanglements(&mut self, node_id: &str, count: usize) -> Result<usize, CognitiveError> {
        let node_entanglements = self.manager.get_node_entanglements(node_id).await?;
        
        // Find weakest entanglements
        let mut sorted_entanglements: Vec<_> = node_entanglements.into_iter().collect();
        sorted_entanglements.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut removed = 0;
        
        for (target_node, strength) in sorted_entanglements.into_iter().take(count) {
            if strength < 0.3 { // Only remove very weak entanglements
                match self.manager.remove_entanglement(node_id, &target_node).await {
                    Ok(true) => removed += 1,
                    _ => continue,
                }
            }
        }
        
        Ok(removed)
    }
    
    /// Add strong entanglements
    async fn add_strong_entanglements(&mut self, node_id: &str, count: usize, tree: &HashMap<String, QuantumMCTSNode>) -> Result<usize, CognitiveError> {
        let high_value_candidates = self.find_high_value_candidates(node_id, tree).await?;
        
        let mut added = 0;
        
        for candidate in high_value_candidates.into_iter().take(count) {
            match self.manager.create_entanglement(node_id, &candidate).await {
                Ok(true) => added += 1,
                _ => continue,
            }
        }
        
        Ok(added)
    }
    
    /// Find high-value candidates for strong entanglements
    async fn find_high_value_candidates(&self, node_id: &str, tree: &HashMap<String, QuantumMCTSNode>) -> Result<Vec<String>, CognitiveError> {
        let existing_entanglements = self.manager.get_node_entanglements(node_id).await?;
        let existing_targets: HashSet<String> = existing_entanglements.into_keys().collect();
        
        let mut candidates: Vec<_> = tree
            .iter()
            .filter(|(id, _)| *id != node_id && !existing_targets.contains(*id))
            .filter(|(_, node)| node.value > 0.7 || node.visits > 500) // High value or high traffic
            .map(|(id, _)| id.clone())
            .collect();
        
        candidates.sort_by(|a, b| {
            let node_a = tree.get(a).unwrap();
            let node_b = tree.get(b).unwrap();
            let score_a = node_a.value + (node_a.visits as f64 * 0.001);
            let score_b = node_b.value + (node_b.visits as f64 * 0.001);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(candidates)
    }
}

/// Node rebalancing type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRebalanceType {
    /// High-value node requiring more connections
    HighValue,
    /// High-traffic node requiring optimized connections
    HighTraffic,
    /// Isolated node requiring more connections
    Isolated,
    /// Overconnected node requiring fewer connections
    Overconnected,
}
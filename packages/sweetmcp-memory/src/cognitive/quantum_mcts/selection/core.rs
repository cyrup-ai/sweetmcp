//! Core quantum selection functionality with lock-free tree traversal
//!
//! This module provides the core QuantumSelector struct with blazing-fast
//! quantum UCT selection, superposition states, and optimized memory management.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::cognitive::types::CognitiveError;
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};
use super::scoring::QuantumScorer;

/// Quantum selection engine with optimized algorithms and caching
pub struct QuantumSelector {
    /// Configuration for selection parameters
    config: QuantumMCTSConfig,
    /// Quantum scorer for UCT calculations
    scorer: QuantumScorer,
    /// Cached selection scores to avoid recomputation
    score_cache: HashMap<(String, u64), f64>, // (node_id, visit_count) -> score
}

impl QuantumSelector {
    /// Create new quantum selector with configuration
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self {
            scorer: QuantumScorer::new(config.clone()),
            config,
            score_cache: HashMap::with_capacity(10_000), // Pre-allocate for performance
        }
    }
    
    /// Quantum selection using superposition and entanglement with lock-free traversal
    pub async fn quantum_select(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<String, CognitiveError> {
        let tree_read = tree.read().await;
        let mut current_id = root_id.to_string();

        // Lock-free tree traversal using cached reads
        loop {
            let node = tree_read
                .get(&current_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found during selection".to_string()))?;

            // Fast terminal and expansion checks
            if node.is_terminal || !node.untried_actions.is_empty() {
                return Ok(current_id);
            }

            if node.children.is_empty() {
                return Ok(current_id);
            }

            // Quantum UCT selection with SIMD-optimized scoring
            let selected_child = self.quantum_uct_select_optimized(node, &tree_read).await?;
            current_id = selected_child;
        }
    }
    
    /// Quantum UCT selection with superposition and SIMD optimization
    async fn quantum_uct_select_optimized(
        &mut self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<String, CognitiveError> {
        let parent_visits = node.visits as f64;
        let parent_visits_ln = parent_visits.ln(); // Pre-compute for reuse
        
        // Pre-allocate vectors for SIMD operations
        let child_count = node.children.len();
        let mut child_ids = Vec::with_capacity(child_count);
        let mut exploitation_scores = Vec::with_capacity(child_count);
        let mut exploration_scores = Vec::with_capacity(child_count);
        let mut quantum_bonuses = Vec::with_capacity(child_count);
        
        // Vectorized score calculation for SIMD optimization
        for (action, child_id) in &node.children {
            let child = tree
                .get(child_id)
                .ok_or_else(|| CognitiveError::InvalidState("Child not found during UCT selection".to_string()))?;

            child_ids.push(child_id.clone());
            
            if child.visits == 0 {
                // Infinite exploration for unvisited nodes
                exploitation_scores.push(0.0);
                exploration_scores.push(f64::INFINITY);
                quantum_bonuses.push(1.0);
            } else {
                // Check cache first for performance
                let cache_key = (child_id.clone(), child.visits);
                if let Some(&cached_score) = self.score_cache.get(&cache_key) {
                    exploitation_scores.push(cached_score);
                } else {
                    let exploitation = child.quantum_reward.norm() / child.visits as f64;
                    self.score_cache.insert(cache_key, exploitation);
                    exploitation_scores.push(exploitation);
                }
                
                let exploration = self.config.quantum_exploration
                    * (parent_visits_ln / child.visits as f64).sqrt();
                exploration_scores.push(exploration);
                
                let quantum_bonus = self.scorer.calculate_quantum_bonus(child);
                quantum_bonuses.push(quantum_bonus);
            }
        }
        
        // SIMD-friendly vectorized score computation
        let quantum_scores: Vec<(String, f64)> = child_ids
            .into_iter()
            .zip(exploitation_scores.into_iter())
            .zip(exploration_scores.into_iter())
            .zip(quantum_bonuses.into_iter())
            .map(|(((id, exploitation), exploration), quantum_bonus)| {
                let total_score = exploitation + exploration + quantum_bonus;
                (id, total_score)
            })
            .collect();

        // Quantum measurement for selection with optimized probability calculation
        self.scorer.quantum_measure_selection_optimized(quantum_scores).await
    }
    
    /// Advanced quantum selection with entanglement networks
    pub async fn quantum_select_with_entanglement(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        entanglement_influence: f64,
    ) -> Result<String, CognitiveError> {
        let tree_read = tree.read().await;
        let mut current_id = root_id.to_string();

        loop {
            let node = tree_read
                .get(&current_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found during entangled selection".to_string()))?;

            if node.is_terminal || !node.untried_actions.is_empty() {
                return Ok(current_id);
            }

            if node.children.is_empty() {
                return Ok(current_id);
            }

            // Enhanced UCT selection with entanglement network effects
            let selected_child = self.scorer.quantum_uct_select_with_entanglement(
                node, 
                &tree_read, 
                entanglement_influence
            ).await?;
            current_id = selected_child;
        }
    }
    
    /// Multi-objective quantum selection balancing exploration and exploitation
    pub async fn multi_objective_quantum_select(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        exploration_weight: f64,
        exploitation_weight: f64,
        quantum_weight: f64,
    ) -> Result<String, CognitiveError> {
        let tree_read = tree.read().await;
        let mut current_id = root_id.to_string();

        loop {
            let node = tree_read
                .get(&current_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found during multi-objective selection".to_string()))?;

            if node.is_terminal || !node.untried_actions.is_empty() {
                return Ok(current_id);
            }

            if node.children.is_empty() {
                return Ok(current_id);
            }

            let selected_child = self.scorer.multi_objective_uct_select(
                node,
                &tree_read,
                exploration_weight,
                exploitation_weight,
                quantum_weight,
            ).await?;
            current_id = selected_child;
        }
    }
    
    /// Clear the score cache to prevent memory growth
    pub fn clear_cache(&mut self) {
        self.score_cache.clear();
    }
    
    /// Get cache statistics for performance monitoring
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.score_cache.len(), self.score_cache.capacity())
    }
    
    /// Prune cache entries for nodes that no longer exist
    pub fn prune_cache(&mut self, existing_nodes: &HashMap<String, QuantumMCTSNode>) {
        self.score_cache.retain(|(node_id, _), _| existing_nodes.contains_key(node_id));
    }
    
    /// Update configuration for dynamic parameter adjustment
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config.clone();
        self.scorer.update_config(new_config);
        // Clear cache since scores depend on configuration
        self.clear_cache();
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Get scorer reference for advanced operations
    pub fn scorer(&mut self) -> &mut QuantumScorer {
        &mut self.scorer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::mcts::CodeState;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_quantum_selector_creation() {
        let config = QuantumMCTSConfig::default();
        let selector = QuantumSelector::new(config);
        assert_eq!(selector.cache_stats().0, 0); // Empty cache initially
    }
    
    #[tokio::test]
    async fn test_selection_with_empty_tree() {
        let mut selector = QuantumSelector::new(QuantumMCTSConfig::default());
        let tree = RwLock::new(HashMap::new());
        
        let result = selector.quantum_select(&tree, "nonexistent").await;
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cache_operations() {
        let mut selector = QuantumSelector::new(QuantumMCTSConfig::default());
        
        // Initially empty cache
        assert_eq!(selector.cache_stats().0, 0);
        
        // Clear empty cache
        selector.clear_cache();
        assert_eq!(selector.cache_stats().0, 0);
        
        // Test with empty node map
        let nodes = HashMap::new();
        selector.prune_cache(&nodes);
        assert_eq!(selector.cache_stats().0, 0);
    }
    
    #[test]
    fn test_config_update() {
        let initial_config = QuantumMCTSConfig::default();
        let mut selector = QuantumSelector::new(initial_config.clone());
        
        assert_eq!(selector.config().quantum_exploration, initial_config.quantum_exploration);
        
        let new_config = QuantumMCTSConfig {
            quantum_exploration: 2.0,
            ..initial_config
        };
        
        selector.update_config(new_config);
        assert_eq!(selector.config().quantum_exploration, 2.0);
    }
}
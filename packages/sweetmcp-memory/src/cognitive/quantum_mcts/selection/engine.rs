//! High-level quantum selection engine with comprehensive strategy management
//!
//! This module provides the QuantumSelectionEngine for coordinating selection
//! strategies, performance tracking, and adaptive algorithm switching.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::cognitive::types::CognitiveError;
use super::{
    core::QuantumSelector,
    types::{SelectionStrategy, SelectionResult, SelectionParameters, SelectionStatistics},
};
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// High-level quantum selection interface with strategy management
pub struct QuantumSelectionEngine {
    /// Core quantum selector
    selector: QuantumSelector,
    /// Default selection strategy
    default_strategy: SelectionStrategy,
    /// Performance statistics tracker
    statistics: SelectionStatistics,
    /// Adaptive strategy switching enabled
    adaptive_strategy: bool,
    /// Strategy performance history for adaptation
    strategy_performance: HashMap<SelectionStrategy, f64>,
}

impl QuantumSelectionEngine {
    /// Create new selection engine with configuration
    pub fn new(config: QuantumMCTSConfig, default_strategy: SelectionStrategy) -> Self {
        Self {
            selector: QuantumSelector::new(config),
            default_strategy,
            statistics: SelectionStatistics::new(),
            adaptive_strategy: false,
            strategy_performance: HashMap::new(),
        }
    }
    
    /// Enable adaptive strategy switching based on performance
    pub fn enable_adaptive_strategy(&mut self) {
        self.adaptive_strategy = true;
    }
    
    /// Disable adaptive strategy switching
    pub fn disable_adaptive_strategy(&mut self) {
        self.adaptive_strategy = false;
    }
    
    /// Select node using specified strategy or default
    pub async fn select_node(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        strategy: Option<SelectionStrategy>,
    ) -> Result<SelectionResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        // Determine strategy to use
        let strategy = if let Some(s) = strategy {
            s
        } else if self.adaptive_strategy {
            self.select_adaptive_strategy()
        } else {
            self.default_strategy
        };
        
        // Perform selection based on strategy
        let node_id = match strategy {
            SelectionStrategy::QuantumUCT => {
                self.selector.quantum_select(tree, root_id).await?
            }
            SelectionStrategy::EntanglementAware => {
                self.selector.quantum_select_with_entanglement(tree, root_id, 0.5).await?
            }
            SelectionStrategy::MultiObjective => {
                self.selector.multi_objective_quantum_select(tree, root_id, 1.0, 1.0, 0.5).await?
            }
            SelectionStrategy::FastSelection => {
                // Simplified selection for performance-critical scenarios
                self.selector.quantum_select(tree, root_id).await?
            }
        };
        
        let computation_time = start_time.elapsed();
        
        // Calculate selection confidence and metadata
        let tree_read = tree.read().await;
        let candidates_count = tree_read.get(root_id)
            .map(|node| node.children.len())
            .unwrap_or(0);
        
        let confidence = self.calculate_selection_confidence(&tree_read, &node_id)?;
        let entropy = self.calculate_selection_entropy(&tree_read, root_id)?;
        let selected_unvisited = tree_read.get(&node_id)
            .map(|node| node.visits == 0)
            .unwrap_or(false);
        
        let mut result = SelectionResult::new(
            node_id,
            confidence,
            candidates_count,
            strategy,
            computation_time.as_micros() as u64,
        );
        result.entropy = entropy;
        result.selected_unvisited = selected_unvisited;
        
        // Record statistics and update strategy performance
        self.statistics.record_selection(&result);
        self.update_strategy_performance(strategy, &result);
        
        Ok(result)
    }
    
    /// Select node using custom parameters
    pub async fn select_node_with_parameters(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        params: &SelectionParameters,
    ) -> Result<SelectionResult, CognitiveError> {
        params.validate().map_err(|e| CognitiveError::InvalidState(e))?;
        
        let start_time = std::time::Instant::now();
        
        // Use multi-objective selection with custom parameters
        let node_id = self.selector.multi_objective_quantum_select(
            tree,
            root_id,
            params.exploration_weight,
            params.exploitation_weight,
            params.quantum_weight,
        ).await?;
        
        let computation_time = start_time.elapsed();
        
        // Calculate metadata
        let tree_read = tree.read().await;
        let candidates_count = tree_read.get(root_id)
            .map(|node| node.children.len())
            .unwrap_or(0);
        
        let confidence = self.calculate_selection_confidence(&tree_read, &node_id)?;
        let entropy = self.calculate_selection_entropy(&tree_read, root_id)?;
        let selected_unvisited = tree_read.get(&node_id)
            .map(|node| node.visits == 0)
            .unwrap_or(false);
        
        let mut result = SelectionResult::new(
            node_id,
            confidence,
            candidates_count,
            SelectionStrategy::MultiObjective,
            computation_time.as_micros() as u64,
        );
        result.entropy = entropy;
        result.selected_unvisited = selected_unvisited;
        
        self.statistics.record_selection(&result);
        
        Ok(result)
    }
    
    /// Calculate confidence in the selection
    fn calculate_selection_confidence(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        selected_id: &str,
    ) -> Result<f64, CognitiveError> {
        let node = tree.get(selected_id)
            .ok_or_else(|| CognitiveError::InvalidState("Selected node not found".to_string()))?;
        
        // Confidence based on visits, amplitude, and coherence
        let visit_confidence = (node.visits as f64).sqrt() / (node.visits as f64 + 10.0);
        let amplitude_confidence = node.amplitude.norm();
        let coherence_confidence = 1.0 - node.quantum_state.decoherence;
        
        // Weighted combination
        Ok((visit_confidence * 0.4 + amplitude_confidence * 0.3 + coherence_confidence * 0.3).min(1.0))
    }
    
    /// Calculate selection entropy for the given node
    pub fn calculate_selection_entropy(
        &mut self,
        tree: &HashMap<String, QuantumMCTSNode>,
        node_id: &str,
    ) -> Result<f64, CognitiveError> {
        let root = tree.get(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Root node not found".to_string()))?;
        
        if root.children.is_empty() {
            return Ok(0.0);
        }
        
        // Calculate scores for all children to determine entropy
        let mut scores = Vec::new();
        for (_, child_id) in &root.children {
            if let Some(child) = tree.get(child_id) {
                let score = self.selector.scorer().calculate_fast_score(child, root.visits as f64);
                scores.push((child_id.clone(), score));
            }
        }
        
        Ok(self.selector.scorer().calculate_selection_entropy(&scores))
    }
    
    /// Select best strategy based on recent performance
    fn select_adaptive_strategy(&self) -> SelectionStrategy {
        if self.strategy_performance.is_empty() {
            return self.default_strategy;
        }
        
        // Find strategy with best recent performance
        self.strategy_performance
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(strategy, _)| *strategy)
            .unwrap_or(self.default_strategy)
    }
    
    /// Update performance tracking for strategy adaptation
    fn update_strategy_performance(&mut self, strategy: SelectionStrategy, result: &SelectionResult) {
        // Performance score based on confidence, speed, and entropy
        let speed_score = if result.is_fast() { 1.0 } else { 0.5 };
        let confidence_score = result.confidence;
        let entropy_score = (result.entropy / 2.0).min(1.0); // Normalize entropy
        
        let performance_score = (speed_score + confidence_score + entropy_score) / 3.0;
        
        // Update running average for strategy
        let current_performance = self.strategy_performance.entry(strategy).or_insert(0.5);
        *current_performance = (*current_performance * 0.9 + performance_score * 0.1).max(0.0).min(1.0);
    }
    
    /// Get performance statistics
    pub fn performance_stats(&self) -> (usize, usize) {
        self.selector.cache_stats()
    }
    
    /// Get selection statistics
    pub fn selection_statistics(&self) -> &SelectionStatistics {
        &self.statistics
    }
    
    /// Reset all statistics
    pub fn reset_statistics(&mut self) {
        self.statistics.reset();
        self.strategy_performance.clear();
    }
    
    /// Maintenance operations for long-running systems
    pub fn maintenance(&mut self, tree: &HashMap<String, QuantumMCTSNode>) {
        self.selector.prune_cache(tree);
    }
    
    /// Update configuration and clear caches
    pub fn update_config(&mut self, config: QuantumMCTSConfig) {
        self.selector.update_config(config);
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        self.selector.config()
    }
    
    /// Set default strategy
    pub fn set_default_strategy(&mut self, strategy: SelectionStrategy) {
        self.default_strategy = strategy;
    }
    
    /// Get best performing strategy
    pub fn best_strategy(&self) -> Option<(SelectionStrategy, f64)> {
        self.strategy_performance
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(strategy, performance)| (*strategy, *performance))
    }
    
    /// Get strategy performance history
    pub fn strategy_performance(&self) -> &HashMap<SelectionStrategy, f64> {
        &self.strategy_performance
    }
    
    /// Force strategy performance update
    pub fn update_strategy_score(&mut self, strategy: SelectionStrategy, score: f64) {
        self.strategy_performance.insert(strategy, score.max(0.0).min(1.0));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    use super::*;
    use crate::cognitive::quantum_mcts::{
        node_state::{QuantumNodeState, QuantumMCTSNode},
        config::QuantumMCTSConfig,
    };
    use crate::cognitive::mcts::types::node_types::CodeState;
    use super::super::core::QuantumSelector;
    use super::super::types::{SelectionStrategy, SelectionResult, SelectionParameters, SelectionStatistics};
    use crate::cognitive::types::CognitiveError;
    use crate::cognitive::quantum_mcts::node_state::QuantumNodeState;
    use crate::cognitive::quantum_mcts::node_state::QuantumMCTSNode;
    
    #[tokio::test]
    async fn test_selection_engine_creation() {
        let config = QuantumMCTSConfig::default();
        let engine = QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT);
        
        assert_eq!(engine.default_strategy, SelectionStrategy::QuantumUCT);
        assert!(!engine.adaptive_strategy);
        assert_eq!(engine.statistics.total_selections, 0);
    }
    
    #[tokio::test]
    async fn test_selection_engine() {
        let config = QuantumMCTSConfig::default();
        let mut engine = QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT);
        
        // Create minimal tree for testing
        let mut tree_map = HashMap::new();
        let classical_state = CodeState::new(
            "test".to_string(),
            1.0,  // latency
            1.0,  // memory
            1.0,  // relevance
        );
        
        let quantum_state = QuantumNodeState::new(classical_state, 2);
        let node = QuantumMCTSNode::new(
            "root".to_string(),
            quantum_state,
            vec!["action1".to_string()],
            None,
            0,
        );
        tree_map.insert("root".to_string(), node);
        
        let tree = RwLock::new(tree_map);
        let result = engine.select_node(&tree, "root", None).await;
        assert!(result.is_ok());
        
        let selection_result = result.unwrap();
        assert_eq!(selection_result.node_id, "root");
        assert_eq!(selection_result.strategy, SelectionStrategy::QuantumUCT);
        
        // Check that statistics were updated
        assert_eq!(engine.selection_statistics().total_selections, 1);
    }
    
    #[tokio::test]
    async fn test_selection_with_parameters() {
        let config = QuantumMCTSConfig::default();
        let mut engine = QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT);
        
        let mut tree_map = HashMap::new();
        let classical_state = CodeState::new(
            "test".to_string(),
            1.0,  // latency
            1.0,  // memory
            1.0,  // relevance
        );
        
        let quantum_state = QuantumNodeState::new(classical_state, 2);
        let node = QuantumMCTSNode::new(
            "root".to_string(),
            quantum_state,
            vec!["action1".to_string()],
            None,
            0,
        );
        tree_map.insert("root".to_string(), node);
        
        let tree = RwLock::new(tree_map);
        let params = SelectionParameters::exploration_focused();
        let result = engine.select_node_with_parameters(&tree, "root", &params).await;
        
        assert!(result.is_ok());
        let selection_result = result.unwrap();
        assert_eq!(selection_result.strategy, SelectionStrategy::MultiObjective);
    }
    
    #[test]
    fn test_adaptive_strategy() {
        let config = QuantumMCTSConfig::default();
        let mut engine = QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT);
        
        // Initially should use default strategy
        assert_eq!(engine.select_adaptive_strategy(), SelectionStrategy::QuantumUCT);
        
        // Update strategy performance
        engine.update_strategy_score(SelectionStrategy::FastSelection, 0.9);
        engine.update_strategy_score(SelectionStrategy::QuantumUCT, 0.5);
        
        // Should now prefer FastSelection
        assert_eq!(engine.select_adaptive_strategy(), SelectionStrategy::FastSelection);
        
        let best = engine.best_strategy();
        assert!(best.is_some());
        let (strategy, score) = best.unwrap();
        assert_eq!(strategy, SelectionStrategy::FastSelection);
        assert_eq!(score, 0.9);
    }
    
    #[test]
    fn test_engine_configuration() {
        let config = QuantumMCTSConfig::default();
        let mut engine = QuantumSelectionEngine::new(config.clone(), SelectionStrategy::QuantumUCT);
        
        assert!(!engine.adaptive_strategy);
        
        engine.enable_adaptive_strategy();
        assert!(engine.adaptive_strategy);
        
        engine.disable_adaptive_strategy();
        assert!(!engine.adaptive_strategy);
        
        engine.set_default_strategy(SelectionStrategy::FastSelection);
        assert_eq!(engine.default_strategy, SelectionStrategy::FastSelection);
    }
    
    #[test]
    fn test_statistics_reset() {
        let config = QuantumMCTSConfig::default();
        let mut engine = QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT);
        
        // Simulate some strategy performance
        engine.update_strategy_score(SelectionStrategy::QuantumUCT, 0.8);
        assert!(!engine.strategy_performance.is_empty());
        
        engine.reset_statistics();
        assert_eq!(engine.statistics.total_selections, 0);
        assert!(engine.strategy_performance.is_empty());
    }
}
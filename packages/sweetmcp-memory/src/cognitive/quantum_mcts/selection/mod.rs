//! Quantum selection algorithms with lock-free tree traversal and SIMD optimization
//!
//! This module provides comprehensive quantum selection functionality decomposed
//! into focused submodules for optimal performance and maintainability.

pub mod core;
pub mod scoring;
pub mod types;
pub mod engine;

// Re-export all public types for backward compatibility
pub use core::QuantumSelector;
pub use scoring::QuantumScorer;
pub use types::{
    SelectionStrategy,
    SelectionResult,
    SelectionParameters,
    SelectionStatistics,
};
pub use engine::QuantumSelectionEngine;

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::cognitive::types::CognitiveError;
use super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// High-level selection coordinator providing unified interface
pub struct SelectionCoordinator {
    /// Selection engine with strategy management
    engine: QuantumSelectionEngine,
}

impl SelectionCoordinator {
    /// Create new selection coordinator
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self {
            engine: QuantumSelectionEngine::new(config, SelectionStrategy::QuantumUCT),
        }
    }
    
    /// Create coordinator with specific default strategy
    pub fn with_strategy(config: QuantumMCTSConfig, strategy: SelectionStrategy) -> Self {
        Self {
            engine: QuantumSelectionEngine::new(config, strategy),
        }
    }
    
    /// Perform quantum selection using default strategy
    pub async fn select(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<SelectionResult, CognitiveError> {
        self.engine.select_node(tree, root_id, None).await
    }
    
    /// Perform selection with specific strategy
    pub async fn select_with_strategy(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        strategy: SelectionStrategy,
    ) -> Result<SelectionResult, CognitiveError> {
        self.engine.select_node(tree, root_id, Some(strategy)).await
    }
    
    /// Perform selection with custom parameters
    pub async fn select_with_parameters(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        params: &SelectionParameters,
    ) -> Result<SelectionResult, CognitiveError> {
        self.engine.select_node_with_parameters(tree, root_id, params).await
    }
    
    /// Enable adaptive strategy switching
    pub fn enable_adaptive_strategy(&mut self) {
        self.engine.enable_adaptive_strategy();
    }
    
    /// Get selection statistics
    pub fn statistics(&self) -> &SelectionStatistics {
        self.engine.selection_statistics()
    }
    
    /// Reset all statistics
    pub fn reset_statistics(&mut self) {
        self.engine.reset_statistics();
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: QuantumMCTSConfig) {
        self.engine.update_config(config);
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        self.engine.config()
    }
    
    /// Perform maintenance operations
    pub fn maintenance(&mut self, tree: &HashMap<String, QuantumMCTSNode>) {
        self.engine.maintenance(tree);
    }
    
    /// Get engine reference for advanced operations
    pub fn engine(&mut self) -> &mut QuantumSelectionEngine {
        &mut self.engine
    }
}

/// Factory functions for creating selection coordinators with different configurations
pub mod factory {
    use super::*;
    
    /// Create high-performance selection coordinator
    pub fn create_high_performance_coordinator() -> SelectionCoordinator {
        let config = QuantumMCTSConfig {
            quantum_exploration: 1.8, // Higher exploration
            decoherence_threshold: 0.2, // Lower threshold for better coherence
            amplitude_threshold: 0.1, // Lower threshold for more quantum effects
            ..Default::default()
        };
        
        SelectionCoordinator::with_strategy(config, SelectionStrategy::FastSelection)
    }
    
    /// Create balanced selection coordinator
    pub fn create_balanced_coordinator() -> SelectionCoordinator {
        SelectionCoordinator::new(QuantumMCTSConfig::default())
    }
    
    /// Create exploration-focused coordinator
    pub fn create_exploration_coordinator() -> SelectionCoordinator {
        let config = QuantumMCTSConfig {
            quantum_exploration: 2.0, // High exploration
            ..Default::default()
        };
        
        SelectionCoordinator::with_strategy(config, SelectionStrategy::MultiObjective)
    }
    
    /// Create exploitation-focused coordinator
    pub fn create_exploitation_coordinator() -> SelectionCoordinator {
        let config = QuantumMCTSConfig {
            quantum_exploration: 0.8, // Low exploration
            ..Default::default()
        };
        
        SelectionCoordinator::with_strategy(config, SelectionStrategy::QuantumUCT)
    }
    
    /// Create entanglement-aware coordinator
    pub fn create_entanglement_coordinator() -> SelectionCoordinator {
        let config = QuantumMCTSConfig {
            entanglement_strength: 0.8, // Strong entanglement effects
            max_entanglements_per_node: 15, // More entanglements
            ..Default::default()
        };
        
        SelectionCoordinator::with_strategy(config, SelectionStrategy::EntanglementAware)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::mcts::CodeState;
    use super::super::node_state::{QuantumNodeState, QuantumMCTSNode};
    
    #[tokio::test]
    async fn test_selection_coordinator() {
        let coordinator = SelectionCoordinator::new(QuantumMCTSConfig::default());
        
        assert_eq!(coordinator.statistics().total_selections, 0);
        assert_eq!(coordinator.config().quantum_exploration, QuantumMCTSConfig::default().quantum_exploration);
    }
    
    #[tokio::test]
    async fn test_coordinator_selection() {
        let mut coordinator = SelectionCoordinator::new(QuantumMCTSConfig::default());
        
        // Create test tree
        let mut tree_map = HashMap::new();
        let classical_state = CodeState {
            code: "test".to_string(),
            latency: 1.0,
            memory: 1.0,
            relevance: 1.0,
        };
        
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
        
        // Test default selection
        let result = coordinator.select(&tree, "root").await;
        assert!(result.is_ok());
        
        // Test strategy-specific selection
        let result = coordinator.select_with_strategy(&tree, "root", SelectionStrategy::FastSelection).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().strategy, SelectionStrategy::FastSelection);
        
        // Test parameter-based selection
        let params = SelectionParameters::balanced();
        let result = coordinator.select_with_parameters(&tree, "root", &params).await;
        assert!(result.is_ok());
        
        // Check statistics were updated
        assert_eq!(coordinator.statistics().total_selections, 3);
    }
    
    #[test]
    fn test_factory_functions() {
        let high_perf = factory::create_high_performance_coordinator();
        assert!(high_perf.config().quantum_exploration > 1.5);
        
        let balanced = factory::create_balanced_coordinator();
        assert_eq!(balanced.config().quantum_exploration, QuantumMCTSConfig::default().quantum_exploration);
        
        let exploration = factory::create_exploration_coordinator();
        assert!(exploration.config().quantum_exploration >= 2.0);
        
        let exploitation = factory::create_exploitation_coordinator();
        assert!(exploitation.config().quantum_exploration <= 1.0);
        
        let entanglement = factory::create_entanglement_coordinator();
        assert!(entanglement.config().entanglement_strength >= 0.8);
    }
    
    #[test]
    fn test_coordinator_configuration() {
        let mut coordinator = SelectionCoordinator::new(QuantumMCTSConfig::default());
        
        let new_config = QuantumMCTSConfig {
            quantum_exploration: 2.5,
            ..Default::default()
        };
        
        coordinator.update_config(new_config);
        assert_eq!(coordinator.config().quantum_exploration, 2.5);
    }
    
    #[test]
    fn test_adaptive_strategy() {
        let mut coordinator = SelectionCoordinator::new(QuantumMCTSConfig::default());
        
        coordinator.enable_adaptive_strategy();
        // Adaptive strategy should now be enabled in the engine
        // (This is verified through the fact that the call doesn't panic)
    }
}
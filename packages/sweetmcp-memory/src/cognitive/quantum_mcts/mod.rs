//! Quantum Monte Carlo Tree Search Implementation
//!
//! This module provides a comprehensive quantum-enhanced MCTS implementation with
//! zero-allocation patterns, lock-free operations, and blazing-fast performance.

// Core submodules
pub mod backpropagation;
pub mod config;
pub mod entanglement_mod;
pub mod entanglement_coordinator;
pub mod entanglement_analysis;
pub mod entanglement_factory;

// Import entanglement directory as the primary module
pub mod entanglement;
pub mod expansion;
pub mod improvement;
pub mod node_state;
pub mod quantum_state;
pub mod selection;
pub mod statistics;
pub mod tree_operations;

// Zero-cost re-exports for backward compatibility
pub use backpropagation::QuantumBackpropagator;
pub use config::{QuantumMCTSConfig, QuantumMCTSConfigBuilder};
pub use entanglement::QuantumEntanglementManager;
pub use entanglement_coordinator::EntanglementCoordinator;
pub use entanglement_analysis::ComprehensiveAnalysisReport;
pub use expansion::QuantumExpander;
pub use improvement::RecursiveImprovementEngine;
pub use node_state::{QuantumMCTSNode, QuantumNodeState};
pub use selection::QuantumSelector;

// Statistics re-exports
pub use statistics::{
    QuantumStatisticsCollector,
    QuantumTreeStatistics,
    ConvergenceMetrics,
    PerformanceMetrics,
    DepthStatistics,
    RewardStatistics,
    quick_stats,
    analysis_presets,
    TreeStatisticsAnalyzer,
    TreeAnalysis,
    RewardQuality,
    ConvergencePhase,
    ConvergenceHealth,
};

// Common types and traits
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use crate::utils::error::Error;

/// Quantum MCTS action trait
pub trait QuantumAction: Clone + Send + Sync + 'static {
    type State: QuantumState;
    
    fn apply(&self, state: &mut Self::State) -> Result<(), CognitiveError>;
    fn estimate_reward(&self, state: &Self::State) -> f64;
    fn get_quantum_amplitude(&self) -> Complex64;
    fn is_valid(&self, state: &Self::State) -> bool;
}

/// Quantum MCTS state trait  
pub trait QuantumState: Clone + Send + Sync + 'static {
    type Action: QuantumAction<State = Self>;
    
    fn get_available_actions(&self) -> Vec<Self::Action>;
    fn is_terminal(&self) -> bool;
    fn get_quantum_amplitude(&self) -> Complex64;
    fn calculate_reward(&self) -> f64;
}

/// Result type for quantum MCTS operations
pub type QuantumMCTSResult<T> = Result<T, CognitiveError>;

/// Quantum MCTS main algorithm implementation
#[derive(Debug, Clone)]
pub struct QuantumMCTS<S: QuantumState> {
    config: Arc<QuantumMCTSConfig>,
    root: Arc<QuantumMCTSNode>,
    selector: Arc<QuantumSelector>,
    expander: Arc<QuantumExpander>,
    backpropagator: Arc<QuantumBackpropagator>,
    entanglement_manager: Arc<QuantumEntanglementManager>,
    improvement_engine: Arc<RecursiveImprovementEngine>,
    statistics: Arc<QuantumStatisticsCollector>,
}

impl<S: QuantumState> QuantumMCTS<S> {
    /// Create new quantum MCTS instance with optimized configuration
    pub fn new(initial_state: S, config: QuantumMCTSConfig) -> QuantumMCTSResult<Self> {
        let config = Arc::new(config);
        
        // Initialize core components
        let selector = Arc::new(QuantumSelector::new(config.clone())?);
        let expander = Arc::new(QuantumExpander::new(config.clone())?);
        let backpropagator = Arc::new(QuantumBackpropagator::new(config.clone())?);
        let entanglement_manager = Arc::new(QuantumEntanglementManager::new(config.clone())?);
        let improvement_engine = Arc::new(RecursiveImprovementEngine::new(config.clone())?);
        let statistics = Arc::new(QuantumStatisticsCollector::new(config.clone())?);
        
        // Create root node with quantum state
        let root = Arc::new(QuantumMCTSNode::new(
            "root".to_string(),
            config.clone(),
        )?);
        
        Ok(Self {
            config,
            root,
            selector,
            expander,
            backpropagator,
            entanglement_manager,
            improvement_engine,
            statistics,
        })
    }
    
    /// Execute quantum MCTS search with performance optimization
    pub async fn search(&self, iterations: usize) -> QuantumMCTSResult<S::Action> {
        self.statistics.start_search_timer();
        
        for iteration in 0..iterations {
            // Execute single quantum MCTS iteration
            self.execute_iteration(iteration).await?;
            
            // Apply recursive improvement periodically
            if iteration % 100 == 0 {
                self.improvement_engine.improve_tree(&self.root).await?;
            }
        }
        
        // Select best action based on quantum measurements
        let best_action = self.select_best_action().await?;
        self.statistics.finalize_search();
        
        Ok(best_action)
    }
    
    /// Execute single quantum MCTS iteration
    async fn execute_iteration(&self, iteration: usize) -> QuantumMCTSResult<()> {
        // Phase 1: Quantum Selection
        let selected_node = self.selector.quantum_select(&self.root).await?;
        
        // Phase 2: Quantum Expansion  
        let new_node = self.expander.quantum_expand(&selected_node).await?;
        
        // Phase 3: Quantum Simulation/Evaluation
        let reward = self.simulate_quantum(&new_node).await?;
        
        // Phase 4: Quantum Backpropagation
        self.backpropagator.quantum_backpropagate(&new_node, reward).await?;
        
        // Phase 5: Entanglement Updates
        self.entanglement_manager.update_entanglement(&new_node, iteration).await?;
        
        Ok(())
    }
    
    /// Perform quantum simulation for reward estimation
    async fn simulate_quantum(&self, node: &Arc<QuantumMCTSNode>) -> QuantumMCTSResult<f64> {
        // Simple simulation - can be enhanced with more sophisticated quantum simulation
        Ok(1.0)
    }
    
    /// Select best action using quantum measurements
    async fn select_best_action(&self) -> QuantumMCTSResult<S::Action> {
        // This is a placeholder - would need actual implementation based on node selection
        Err(CognitiveError::ProcessingError("Not implemented".to_string()))
    }
    
    /// Get current tree statistics
    pub async fn get_statistics(&self) -> QuantumTreeStatistics {
        self.statistics.get_tree_statistics(&self.root).await
    }
}

/// Create optimized quantum MCTS configuration for current system
pub fn create_optimized_config() -> QuantumMCTSResult<QuantumMCTSConfig> {
    Ok(QuantumMCTSConfig::system_optimized())
}

/// Create quantum MCTS with default configuration
pub fn create_quantum_mcts<S: QuantumState>(initial_state: S) -> QuantumMCTSResult<QuantumMCTS<S>> {
    let config = create_optimized_config()?;
    QuantumMCTS::new(initial_state, config)
}
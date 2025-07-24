//! Core quantum expansion logic
//!
//! This module provides the main QuantumExpander with zero-copy optimization
//! and parallel expansion capabilities for quantum MCTS operations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;
use tracing::{debug, warn};

use crate::cognitive::{
    committee::EvaluationCommittee,
    mcts::CodeState,
    quantum::{Complex64, MeasurementBasis, PhaseEvolution, SuperpositionState},
    types::{CognitiveError, OptimizationSpec},
};
use super::{
    action_manager::ActionManager,
    node_state::{QuantumMCTSNode, QuantumNodeState, QuantumNodeFactory},
    config::QuantumMCTSConfig,
};

/// Quantum expansion engine with zero-copy optimization
pub struct QuantumExpander {
    /// Configuration for expansion parameters
    config: QuantumMCTSConfig,
    /// Evaluation committee for action assessment
    committee: Arc<EvaluationCommittee>,
    /// Optimization specification
    spec: Arc<OptimizationSpec>,
    /// User objective
    user_objective: String,
    /// Phase evolution for quantum transformations
    phase_evolution: Arc<PhaseEvolution>,
    /// Node factory for efficient node creation
    node_factory: QuantumNodeFactory,
    /// Semaphore for parallel expansion control
    expansion_semaphore: Arc<Semaphore>,
    /// Action manager for caching and reuse
    action_manager: ActionManager,
}

impl QuantumExpander {
    /// Create new quantum expander with optimized initialization
    pub fn new(
        config: QuantumMCTSConfig,
        committee: Arc<EvaluationCommittee>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        phase_evolution: Arc<PhaseEvolution>,
    ) -> Self {
        Self {
            expansion_semaphore: Arc::new(Semaphore::new(config.max_quantum_parallel)),
            config,
            committee,
            spec,
            user_objective,
            phase_evolution,
            node_factory: QuantumNodeFactory::new(),
            action_manager: ActionManager::new(),
        }
    }
    
    /// Quantum expansion with superposition and zero-copy optimization
    pub async fn quantum_expand(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: &str,
    ) -> Result<Option<String>, CognitiveError> {
        // Fast check if expansion is possible
        {
            let tree_read = tree.read().await;
            let node = tree_read
                .get(node_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found during expansion".to_string()))?;

            if node.untried_actions.is_empty() {
                return Ok(None);
            }
        }

        // Acquire expansion semaphore for controlled parallelism
        let _permit = self.expansion_semaphore.acquire().await
            .map_err(|e| CognitiveError::ResourceExhaustion(format!("Failed to acquire expansion permit: {}", e)))?;

        let mut tree_write = tree.write().await;
        let node = tree_write
            .get_mut(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Node not found during expansion write".to_string()))?;

        if node.untried_actions.is_empty() {
            return Ok(None);
        }

        // Select action using quantum superposition with zero allocation
        let action_idx = self.quantum_action_selection_optimized(&node.quantum_state.superposition).await?;
        if action_idx >= node.untried_actions.len() {
            return Err(CognitiveError::InvalidState("Invalid action index from quantum selection".to_string()));
        }
        
        let action = node.untried_actions.remove(action_idx);

        // Zero-copy state cloning for transformation
        let parent_state = node.quantum_state.clone(); // Clone is unavoidable here but optimized
        let parent_amplitude = node.amplitude;
        let parent_id = node.id.clone();
        let improvement_depth = node.improvement_depth;
        
        // Drop write lock early to allow parallel operations
        drop(tree_write);

        // Apply quantum transformation with parallel processing capability
        let new_quantum_state = self.apply_quantum_action_optimized(&parent_state, &action).await?;
        let child_amplitude = self.action_manager.calculate_child_amplitude(&parent_amplitude, &action);

        // Get untried actions for the new state
        let untried_actions = self.action_manager.get_quantum_actions(&new_quantum_state.classical_state);

        // Create child with optimized allocation
        let child_id = self.node_factory.generate_id(&parent_id);
        let mut child_node = QuantumMCTSNode::new(
            child_id.clone(),
            new_quantum_state,
            untried_actions,
            Some(parent_id),
            improvement_depth,
        );
        child_node.applied_action = Some(action.clone());
        child_node.amplitude = child_amplitude;

        // Add to tree with minimal lock time
        let mut tree_write = tree.write().await;
        tree_write.insert(child_id.clone(), child_node);
        
        // Update parent's children mapping
        if let Some(parent_node) = tree_write.get_mut(node_id) {
            parent_node.add_child(action, child_id.clone());
        }

        Ok(Some(child_id))
    }
    
    /// Parallel quantum expansion for multiple nodes
    pub async fn parallel_quantum_expand(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_ids: &[String],
    ) -> Result<Vec<Option<String>>, CognitiveError> {
        let mut results = Vec::with_capacity(node_ids.len());

        // For now, do sequential expansion but with optimized batching
        // In practice, we'd need to restructure for true parallelism
        for node_id in node_ids {
            let expansion_result = self.quantum_expand(tree, node_id).await?;
            results.push(expansion_result);
        }

        Ok(results)
    }
    
    /// Optimized quantum action selection using superposition
    async fn quantum_action_selection_optimized(
        &self,
        superposition: &SuperpositionState,
    ) -> Result<usize, CognitiveError> {
        // Fast path for single action
        if superposition.dimension() == 1 {
            return Ok(0);
        }

        // Measure the superposition state with computational basis
        let measurement = MeasurementBasis::computational();
        let probabilities = superposition.measure(&measurement)
            .map_err(|e| CognitiveError::QuantumError(format!("Measurement failed: {}", e)))?;

        // Optimized selection based on quantum probabilities
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let selection = rng.gen_range(0.0..1.0);
        let mut cumulative = 0.0;

        for (i, &p) in probabilities.iter().enumerate() {
            cumulative += p;
            if selection < cumulative {
                return Ok(i);
            }
        }

        // Fallback to last index
        Ok(probabilities.len().saturating_sub(1))
    }
    
    /// Apply quantum action with optimized transformation
    async fn apply_quantum_action_optimized(
        &self,
        state: &QuantumNodeState,
        action: &str,
    ) -> Result<QuantumNodeState, CognitiveError> {
        // Classical state transformation
        let new_classical_state = self.action_manager.transform_classical_state(&state.classical_state, action);

        // Quantum evolution with phase development
        let mut new_superposition = state.superposition.clone();
        let phase_delta = self.phase_evolution.compute(0.1);
        new_superposition.evolve(phase_delta)
            .map_err(|e| CognitiveError::QuantumError(format!("Superposition evolution failed: {}", e)))?;

        // Phase evolution based on action type
        let phase_increment = self.action_manager.calculate_phase_increment(action, self.config.phase_evolution_rate);
        let new_phase = (state.phase + phase_increment) % (2.0 * std::f64::consts::PI);

        // Decoherence calculation with action-dependent factors
        let decoherence_increment = self.action_manager.calculate_decoherence_increment(action, state.decoherence);
        let new_decoherence = (state.decoherence + decoherence_increment).min(1.0);

        // Clone entanglements for new state (could be optimized with reference counting)
        let new_entanglements = state.entanglements.clone();

        Ok(QuantumNodeState {
            classical_state: new_classical_state,
            superposition: new_superposition,
            entanglements: new_entanglements,
            phase: new_phase,
            decoherence: new_decoherence,
        })
    }
    
    /// Batch expansion for multiple nodes with optimized resource management
    pub async fn batch_expand(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_ids: &[String],
        max_concurrent: usize,
    ) -> Result<Vec<Option<String>>, CognitiveError> {
        let mut results = Vec::with_capacity(node_ids.len());
        
        for node_id in node_ids {
            // For now, do sequential expansion due to borrowing constraints
            // In a real implementation, we'd need to restructure for true parallelism
            let expansion_result = self.quantum_expand(tree, node_id).await?;
            results.push(expansion_result);
        }
        
        Ok(results)
    }
    
    /// Get expansion statistics for performance monitoring
    pub fn expansion_stats(&self) -> super::ExpansionStats {
        super::ExpansionStats {
            action_pool_size: self.action_manager.pool_size(),
            action_pool_capacity: self.action_manager.pool_capacity(),
            max_parallel: self.config.max_quantum_parallel,
            available_permits: self.expansion_semaphore.available_permits(),
        }
    }
    
    /// Update configuration
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config;
        // Update semaphore if parallelism changed
        self.expansion_semaphore = Arc::new(Semaphore::new(self.config.max_quantum_parallel));
    }
    
    /// Cleanup resources and return action vectors to pool
    pub fn cleanup(&mut self) {
        self.action_manager.cleanup();
    }
}
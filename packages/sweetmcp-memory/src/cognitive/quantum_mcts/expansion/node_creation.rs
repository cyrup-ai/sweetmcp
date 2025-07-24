//! Node creation and initialization for quantum MCTS
//!
//! This module provides optimized node creation with efficient memory allocation,
//! quantum state initialization, and factory patterns for high-performance
//! MCTS tree construction.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use rand::Rng;

use crate::cognitive::{
    committee::EvaluationCommittee,
    mcts::CodeState,
    quantum::{Complex64, SuperpositionState, EntanglementMap},
    types::{CognitiveError, OptimizationSpec},
};
use super::super::{
    node_state::{QuantumMCTSNode, QuantumNodeState},
    config::QuantumMCTSConfig,
};

/// Factory for creating quantum MCTS nodes with optimized allocation
pub struct QuantumNodeFactory {
    /// Node ID counter for unique identification
    node_counter: AtomicU64,
    
    /// Configuration for node creation
    config: QuantumMCTSConfig,
    
    /// Evaluation committee for initial assessment
    committee: Arc<EvaluationCommittee>,
    
    /// Optimization specification
    spec: Arc<OptimizationSpec>,
    
    /// Pre-allocated node pool for reuse
    node_pool: Vec<QuantumMCTSNode>,
    
    /// Pre-allocated action vectors for reuse
    action_pool: Vec<Vec<String>>,
}

impl QuantumNodeFactory {
    /// Create new quantum node factory
    pub fn new() -> Self {
        Self {
            node_counter: AtomicU64::new(0),
            config: QuantumMCTSConfig::default(),
            committee: Arc::new(EvaluationCommittee::new()),
            spec: Arc::new(OptimizationSpec::default()),
            node_pool: Vec::with_capacity(64),
            action_pool: Vec::with_capacity(32),
        }
    }

    /// Create new factory with configuration
    pub fn with_config(
        config: QuantumMCTSConfig,
        committee: Arc<EvaluationCommittee>,
        spec: Arc<OptimizationSpec>,
    ) -> Self {
        Self {
            node_counter: AtomicU64::new(0),
            config,
            committee,
            spec,
            node_pool: Vec::with_capacity(64),
            action_pool: Vec::with_capacity(32),
        }
    }

    /// Generate unique node ID
    pub fn generate_id(&self, parent_id: &str) -> String {
        let counter = self.node_counter.fetch_add(1, Ordering::Relaxed);
        format!("{}_{}", parent_id, counter)
    }

    /// Create root node with quantum initialization
    pub fn create_root_node(
        &mut self,
        initial_state: CodeState,
        user_objective: &str,
    ) -> Result<QuantumMCTSNode, CognitiveError> {
        let node_id = "root_0".to_string();
        
        // Initialize quantum state for root
        let quantum_state = self.initialize_quantum_state(&initial_state, user_objective)?;
        
        // Get initial actions
        let untried_actions = self.get_initial_actions(&initial_state);
        
        // Create root node
        let mut root_node = QuantumMCTSNode::new(
            node_id,
            quantum_state,
            untried_actions,
            None, // No parent for root
            0,    // Initial improvement depth
        );
        
        // Set initial amplitude for root (normalized)
        root_node.amplitude = Complex64::new(1.0, 0.0);
        
        Ok(root_node)
    }

    /// Create child node with optimized allocation
    pub fn create_child_node(
        &mut self,
        parent_id: &str,
        parent_state: &QuantumNodeState,
        action: &str,
        improvement_depth: usize,
    ) -> Result<QuantumMCTSNode, CognitiveError> {
        let child_id = self.generate_id(parent_id);
        
        // Apply action to create new quantum state
        let new_quantum_state = self.apply_action_to_state(parent_state, action)?;
        
        // Get untried actions for new state
        let untried_actions = self.get_actions_for_state(&new_quantum_state.classical_state);
        
        // Create child node
        let mut child_node = QuantumMCTSNode::new(
            child_id,
            new_quantum_state,
            untried_actions,
            Some(parent_id.to_string()),
            improvement_depth,
        );
        
        // Set applied action
        child_node.applied_action = Some(action.to_string());
        
        Ok(child_node)
    }

    /// Initialize quantum state for given classical state
    fn initialize_quantum_state(
        &self,
        classical_state: &CodeState,
        user_objective: &str,
    ) -> Result<QuantumNodeState, CognitiveError> {
        // Create superposition based on state complexity
        let dimension = self.calculate_superposition_dimension(classical_state);
        let superposition = SuperpositionState::uniform(dimension)
            .map_err(|e| CognitiveError::QuantumError(format!("Failed to create superposition: {}", e)))?;
        
        // Initialize entanglement map
        let entanglement_map = EntanglementMap::new();
        
        // Apply user objective influence to quantum state
        let influenced_superposition = self.apply_objective_influence(superposition, user_objective)?;
        
        Ok(QuantumNodeState {
            classical_state: classical_state.clone(),
            superposition: influenced_superposition,
            entanglement_map,
        })
    }

    /// Calculate optimal superposition dimension based on state complexity
    fn calculate_superposition_dimension(&self, state: &CodeState) -> usize {
        // Base dimension on code complexity and available actions
        let complexity_factor = (state.complexity_score / 10.0).ceil() as usize;
        let function_factor = (state.functions.len() / 5).max(1);
        
        // Clamp to reasonable range
        (complexity_factor * function_factor).clamp(2, 16)
    }

    /// Apply user objective influence to superposition
    fn apply_objective_influence(
        &self,
        mut superposition: SuperpositionState,
        user_objective: &str,
    ) -> Result<SuperpositionState, CognitiveError> {
        // Parse objective to determine quantum bias
        let bias_vector = self.parse_objective_bias(user_objective);
        
        // Apply bias to superposition amplitudes
        superposition.apply_bias(&bias_vector)
            .map_err(|e| CognitiveError::QuantumError(format!("Failed to apply objective bias: {}", e)))?;
        
        Ok(superposition)
    }

    /// Parse user objective into quantum bias vector
    fn parse_objective_bias(&self, objective: &str) -> Vec<f64> {
        let mut bias = vec![1.0; 8]; // Default uniform bias
        
        // Adjust bias based on objective keywords
        if objective.contains("performance") || objective.contains("speed") {
            bias[0] *= 1.5; // Boost performance-related states
            bias[1] *= 1.3;
        }
        
        if objective.contains("memory") || objective.contains("allocation") {
            bias[2] *= 1.4; // Boost memory-related states
            bias[3] *= 1.2;
        }
        
        if objective.contains("parallel") || objective.contains("concurrent") {
            bias[4] *= 1.6; // Boost parallelism states
            bias[5] *= 1.3;
        }
        
        if objective.contains("quantum") || objective.contains("superposition") {
            bias[6] *= 1.8; // Boost quantum states
            bias[7] *= 1.5;
        }
        
        // Normalize bias vector
        let sum: f64 = bias.iter().sum();
        if sum > 0.0 {
            for b in &mut bias {
                *b /= sum;
            }
        }
        
        bias
    }

    /// Get initial actions for root state
    fn get_initial_actions(&mut self, state: &CodeState) -> Vec<String> {
        let mut actions = self.get_actions_from_pool();
        self.populate_initial_actions(&mut actions, state);
        actions
    }

    /// Get actions for given state
    fn get_actions_for_state(&mut self, state: &CodeState) -> Vec<String> {
        let mut actions = self.get_actions_from_pool();
        self.populate_state_actions(&mut actions, state);
        actions
    }

    /// Get action vector from pool or create new
    fn get_actions_from_pool(&mut self) -> Vec<String> {
        self.action_pool.pop().unwrap_or_else(|| Vec::with_capacity(20))
    }

    /// Return action vector to pool for reuse
    pub fn return_actions_to_pool(&mut self, mut actions: Vec<String>) {
        if actions.capacity() >= 16 && self.action_pool.len() < 32 {
            actions.clear();
            self.action_pool.push(actions);
        }
    }

    /// Populate initial actions for root node
    fn populate_initial_actions(&self, actions: &mut Vec<String>, state: &CodeState) {
        // Core optimization actions
        actions.extend([
            "analyze_performance_bottlenecks",
            "identify_memory_leaks",
            "optimize_critical_paths",
            "improve_algorithm_efficiency",
            "reduce_computational_complexity",
        ].iter().map(|s| s.to_string()));

        // Quantum-enhanced actions
        actions.extend([
            "quantum_superposition_analysis",
            "entangle_related_optimizations",
            "quantum_phase_estimation",
            "amplitude_amplification_search",
        ].iter().map(|s| s.to_string()));

        // State-specific actions
        if state.functions.len() > 10 {
            actions.push("decompose_large_functions".to_string());
        }
        
        if state.complexity_score > 15.0 {
            actions.push("simplify_complex_logic".to_string());
        }
        
        if state.performance_score < 0.7 {
            actions.push("profile_performance_issues".to_string());
        }
    }

    /// Populate actions based on current state
    fn populate_state_actions(&self, actions: &mut Vec<String>, state: &CodeState) {
        // Clear existing actions
        actions.clear();

        // Performance-focused actions
        if state.performance_score < 0.8 {
            actions.extend([
                "optimize_hot_paths",
                "reduce_allocations",
                "improve_cache_locality",
                "vectorize_loops",
                "inline_critical_functions",
            ].iter().map(|s| s.to_string()));
        }

        // Memory-focused actions
        if state.memory_usage > 0.7 {
            actions.extend([
                "optimize_memory_layout",
                "implement_object_pooling",
                "reduce_memory_fragmentation",
                "use_stack_allocation",
            ].iter().map(|s| s.to_string()));
        }

        // Parallelism actions
        if state.parallelism_potential > 0.5 {
            actions.extend([
                "parallelize_independent_work",
                "implement_work_stealing",
                "optimize_thread_synchronization",
                "use_lock_free_data_structures",
            ].iter().map(|s| s.to_string()));
        }

        // Quantum actions (always available)
        actions.extend([
            "quantum_optimize_superposition",
            "entangle_parallel_paths",
            "quantum_phase_shift",
            "amplitude_amplification",
            "quantum_error_correction",
        ].iter().map(|s| s.to_string()));

        // Shuffle for variety
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        actions.shuffle(&mut rng);
    }

    /// Apply action to quantum state to create new state
    fn apply_action_to_state(
        &self,
        parent_state: &QuantumNodeState,
        action: &str,
    ) -> Result<QuantumNodeState, CognitiveError> {
        // Clone parent state as starting point
        let mut new_classical_state = parent_state.classical_state.clone();
        let mut new_superposition = parent_state.superposition.clone();
        let new_entanglement_map = parent_state.entanglement_map.clone();

        // Apply action-specific transformations
        match action {
            "optimize_hot_paths" => {
                new_classical_state.performance_score *= 1.1;
                new_classical_state.complexity_score *= 0.95;
            }
            "reduce_allocations" => {
                new_classical_state.memory_usage *= 0.9;
                new_classical_state.performance_score *= 1.05;
            }
            "improve_cache_locality" => {
                new_classical_state.performance_score *= 1.08;
                new_classical_state.cache_efficiency *= 1.15;
            }
            "parallelize_independent_work" => {
                new_classical_state.parallelism_potential *= 1.2;
                new_classical_state.performance_score *= 1.15;
            }
            "quantum_optimize_superposition" => {
                // Quantum action - evolve superposition
                new_superposition = new_superposition.evolve_phase(0.1)
                    .map_err(|e| CognitiveError::QuantumError(format!("Phase evolution failed: {}", e)))?;
            }
            "entangle_parallel_paths" => {
                // Create entanglement between optimization paths
                new_superposition = new_superposition.create_entanglement(vec![0, 1])
                    .map_err(|e| CognitiveError::QuantumError(format!("Entanglement creation failed: {}", e)))?;
            }
            "amplitude_amplification" => {
                // Amplify promising amplitudes
                new_superposition = new_superposition.amplify_amplitudes(1.1)
                    .map_err(|e| CognitiveError::QuantumError(format!("Amplitude amplification failed: {}", e)))?;
            }
            _ => {
                // Generic improvement for unknown actions
                new_classical_state.complexity_score *= 0.98;
            }
        }

        Ok(QuantumNodeState {
            classical_state: new_classical_state,
            superposition: new_superposition,
            entanglement_map: new_entanglement_map,
        })
    }

    /// Get node from pool or create new
    pub fn get_node_from_pool(&mut self) -> Option<QuantumMCTSNode> {
        self.node_pool.pop()
    }

    /// Return node to pool for reuse
    pub fn return_node_to_pool(&mut self, mut node: QuantumMCTSNode) {
        if self.node_pool.len() < 64 {
            // Reset node for reuse
            node.reset_for_reuse();
            self.node_pool.push(node);
        }
    }

    /// Pre-warm the factory with allocated nodes
    pub fn prewarm(&mut self, count: usize) {
        for _ in 0..count {
            let dummy_state = CodeState::default();
            if let Ok(node) = self.create_root_node(dummy_state, "prewarm") {
                self.node_pool.push(node);
            }
        }
    }
}

impl Default for QuantumNodeFactory {
    fn default() -> Self {
        Self::new()
    }
}
//! Node creation factory for optimized node allocation with zero-allocation patterns
//!
//! This module provides efficient factory functions for creating quantum MCTS nodes
//! with minimal memory allocation and optimal performance.

use std::sync::atomic::{AtomicU64, Ordering};
use crate::cognitive::mcts::types::node_types::CodeState;
use super::{core::QuantumNodeState, node::QuantumMCTSNode};

/// Node creation factory for optimized node allocation
pub struct QuantumNodeFactory {
    /// Atomic counter for unique node IDs - thread-safe
    id_counter: AtomicU64,
    /// Factory identifier for distributed systems
    factory_id: String,
}

impl QuantumNodeFactory {
    /// Create new node factory with default settings
    pub fn new() -> Self {
        Self { 
            id_counter: AtomicU64::new(0),
            factory_id: "default".to_string(),
        }
    }
    
    /// Create node factory with custom factory ID
    pub fn with_factory_id(factory_id: String) -> Self {
        Self {
            id_counter: AtomicU64::new(0),
            factory_id,
        }
    }
    
    /// Create node factory with custom starting counter
    pub fn with_start_counter(start_counter: u64) -> Self {
        Self {
            id_counter: AtomicU64::new(start_counter),
            factory_id: "default".to_string(),
        }
    }
    
    /// Generate unique node ID with minimal allocation - thread-safe
    #[inline]
    pub fn generate_id(&self, prefix: &str) -> String {
        let counter = self.id_counter.fetch_add(1, Ordering::Relaxed);
        format!("{}-{}-q{}", prefix, self.factory_id, counter)
    }
    
    /// Generate simple numeric ID for performance-critical paths
    #[inline]
    pub fn generate_numeric_id(&self) -> String {
        let counter = self.id_counter.fetch_add(1, Ordering::Relaxed);
        counter.to_string()
    }
    
    /// Get current counter value
    #[inline]
    pub fn current_counter(&self) -> u64 {
        self.id_counter.load(Ordering::Relaxed)
    }
    
    /// Reset counter (use with caution in multi-threaded environments)
    #[inline]
    pub fn reset_counter(&self) {
        self.id_counter.store(0, Ordering::Relaxed);
    }
    
    /// Create root node with optimized initialization
    #[inline]
    pub fn create_root_node(
        &self,
        classical_state: CodeState,
        untried_actions: Vec<String>,
    ) -> QuantumMCTSNode {
        let id = self.generate_id("root");
        let quantum_state = QuantumNodeState::new(classical_state, untried_actions.len());
        
        QuantumMCTSNode::new_root(id, quantum_state, untried_actions)
    }
    
    /// Create root node with custom quantum state configuration
    #[inline]
    pub fn create_root_node_with_config(
        &self,
        classical_state: CodeState,
        untried_actions: Vec<String>,
        superposition_size: usize,
        entanglement_capacity: usize,
    ) -> QuantumMCTSNode {
        let id = self.generate_id("root");
        let quantum_state = QuantumNodeState::with_entanglement_capacity(
            classical_state, 
            superposition_size, 
            entanglement_capacity
        );
        
        QuantumMCTSNode::new_root(id, quantum_state, untried_actions)
    }
    
    /// Create child node with parent relationship
    #[inline]
    pub fn create_child_node(
        &self,
        parent_id: &str,
        classical_state: CodeState,
        untried_actions: Vec<String>,
        applied_action: String,
        improvement_depth: u32,
    ) -> QuantumMCTSNode {
        let id = self.generate_id(parent_id);
        let quantum_state = QuantumNodeState::new(classical_state, untried_actions.len());
        
        QuantumMCTSNode::new_child(
            id,
            quantum_state,
            untried_actions,
            parent_id.to_string(),
            applied_action,
            improvement_depth,
        )
    }
    
    /// Create child node with inherited quantum state
    #[inline]
    pub fn create_child_with_inheritance(
        &self,
        parent_id: &str,
        parent_quantum_state: &QuantumNodeState,
        classical_state: CodeState,
        untried_actions: Vec<String>,
        applied_action: String,
        improvement_depth: u32,
        phase_shift: f64,
    ) -> QuantumMCTSNode {
        let id = self.generate_id(parent_id);
        
        // Create quantum state with inherited properties
        let quantum_state = parent_quantum_state.clone_with_modifications(|state| {
            state.classical_state = classical_state;
            state.evolve_phase(phase_shift);
            // Slightly increase decoherence for child nodes
            state.update_decoherence(0.01);
        });
        
        QuantumMCTSNode::new_child(
            id,
            quantum_state,
            untried_actions,
            parent_id.to_string(),
            applied_action,
            improvement_depth,
        )
    }
    
    /// Create node with custom quantum initialization
    #[inline]
    pub fn create_custom_node(
        &self,
        prefix: &str,
        quantum_state: QuantumNodeState,
        untried_actions: Vec<String>,
        parent: Option<String>,
        improvement_depth: u32,
    ) -> QuantumMCTSNode {
        let id = self.generate_id(prefix);
        QuantumMCTSNode::new(id, quantum_state, untried_actions, parent, improvement_depth)
    }
    
    /// Create minimal node for testing or prototyping
    #[inline]
    pub fn create_minimal_node(&self, classical_state: CodeState) -> QuantumMCTSNode {
        let id = self.generate_numeric_id();
        let quantum_state = QuantumNodeState::new(classical_state, 1);
        let untried_actions = Vec::new();
        
        QuantumMCTSNode::new(id, quantum_state, untried_actions, None, 0)
    }
    
    /// Batch create multiple child nodes efficiently
    #[inline]
    pub fn create_child_batch(
        &self,
        parent_id: &str,
        parent_quantum_state: &QuantumNodeState,
        child_specs: Vec<ChildSpec>,
    ) -> Vec<QuantumMCTSNode> {
        child_specs
            .into_iter()
            .enumerate()
            .map(|(index, spec)| {
                let phase_shift = (index as f64) * 0.1; // Small phase variations
                self.create_child_with_inheritance(
                    parent_id,
                    parent_quantum_state,
                    spec.classical_state,
                    spec.untried_actions,
                    spec.applied_action,
                    spec.improvement_depth,
                    phase_shift,
                )
            })
            .collect()
    }
    
    /// Clone node with new ID (for branching scenarios)
    #[inline]
    pub fn clone_node_with_new_id(
        &self,
        original: &QuantumMCTSNode,
        prefix: &str,
    ) -> QuantumMCTSNode {
        let new_id = self.generate_id(prefix);
        let mut cloned = QuantumMCTSNode::new(
            new_id,
            original.quantum_state.clone(),
            original.untried_actions.clone(),
            original.parent.clone(),
            original.improvement_depth,
        );
        
        // Copy statistics
        cloned.visits = original.visits;
        cloned.amplitude = original.amplitude;
        cloned.quantum_reward = original.quantum_reward;
        cloned.is_terminal = original.is_terminal;
        cloned.applied_action = original.applied_action.clone();
        
        // Clone children mapping
        cloned.children = original.children.clone();
        
        cloned
    }
    
    /// Create node from template with customization
    #[inline]
    pub fn create_from_template<F>(
        &self,
        template: &QuantumMCTSNode,
        prefix: &str,
        customizer: F,
    ) -> QuantumMCTSNode 
    where
        F: FnOnce(&mut QuantumMCTSNode),
    {
        let mut node = self.clone_node_with_new_id(template, prefix);
        customizer(&mut node);
        node
    }
    
    /// Get factory statistics
    #[inline]
    pub fn statistics(&self) -> FactoryStatistics {
        FactoryStatistics {
            factory_id: self.factory_id.clone(),
            nodes_created: self.current_counter(),
        }
    }
}

impl Default for QuantumNodeFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Specification for creating child nodes in batch operations
#[derive(Debug, Clone)]
pub struct ChildSpec {
    pub classical_state: CodeState,
    pub untried_actions: Vec<String>,
    pub applied_action: String,
    pub improvement_depth: u32,
}

impl ChildSpec {
    /// Create new child specification
    pub fn new(
        classical_state: CodeState,
        untried_actions: Vec<String>,
        applied_action: String,
        improvement_depth: u32,
    ) -> Self {
        Self {
            classical_state,
            untried_actions,
            applied_action,
            improvement_depth,
        }
    }
}

/// Factory statistics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct FactoryStatistics {
    pub factory_id: String,
    pub nodes_created: u64,
}

/// Global factory instance for convenience
static GLOBAL_FACTORY: std::sync::OnceLock<QuantumNodeFactory> = std::sync::OnceLock::new();

/// Get global node factory instance
pub fn global_factory() -> &'static QuantumNodeFactory {
    GLOBAL_FACTORY.get_or_init(QuantumNodeFactory::new)
}

/// Quick factory functions using global instance
pub mod quick {
    use super::*;

    /// Create root node using global factory
    #[inline]
    pub fn root_node(
        classical_state: CodeState,
        untried_actions: Vec<String>,
    ) -> QuantumMCTSNode {
        global_factory().create_root_node(classical_state, untried_actions)
    }
    
    /// Create child node using global factory
    #[inline]
    pub fn child_node(
        parent_id: &str,
        classical_state: CodeState,
        untried_actions: Vec<String>,
        applied_action: String,
        improvement_depth: u32,
    ) -> QuantumMCTSNode {
        global_factory().create_child_node(
            parent_id,
            classical_state,
            untried_actions,
            applied_action,
            improvement_depth,
        )
    }
    
    /// Create minimal node using global factory
    #[inline]
    pub fn minimal_node(classical_state: CodeState) -> QuantumMCTSNode {
        global_factory().create_minimal_node(classical_state)
    }
    
    /// Generate unique ID using global factory
    #[inline]
    pub fn generate_id(prefix: &str) -> String {
        global_factory().generate_id(prefix)
    }
    
    /// Clone node with new ID using global factory
    #[inline]
    pub fn clone_with_new_id(
        original: &QuantumMCTSNode,
        prefix: &str,
    ) -> QuantumMCTSNode {
        global_factory().clone_node_with_new_id(original, prefix)
    }
}

/// Pool-based factory for high-performance scenarios
pub struct PooledNodeFactory {
    base_factory: QuantumNodeFactory,
    node_pool: std::sync::Mutex<Vec<QuantumMCTSNode>>,
    quantum_state_pool: std::sync::Mutex<Vec<QuantumNodeState>>,
}

impl PooledNodeFactory {
    /// Create new pooled factory with initial capacity
    pub fn with_capacity(initial_capacity: usize) -> Self {
        Self {
            base_factory: QuantumNodeFactory::new(),
            node_pool: std::sync::Mutex::new(Vec::with_capacity(initial_capacity)),
            quantum_state_pool: std::sync::Mutex::new(Vec::with_capacity(initial_capacity)),
        }
    }
    
    /// Get node from pool or create new one
    pub fn get_or_create_node(
        &self,
        quantum_state: QuantumNodeState,
        untried_actions: Vec<String>,
        parent: Option<String>,
        improvement_depth: u32,
    ) -> QuantumMCTSNode {
        if let Ok(mut pool) = self.node_pool.try_lock() {
            if let Some(mut node) = pool.pop() {
                // Reuse existing node by resetting its state
                let id = self.base_factory.generate_numeric_id();
                node.id = id;
                node.quantum_state = quantum_state;
                node.untried_actions = untried_actions;
                node.parent = parent;
                node.improvement_depth = improvement_depth;
                node.reset_statistics();
                return node;
            }
        }
        
        // Fall back to creating new node
        let id = self.base_factory.generate_numeric_id();
        QuantumMCTSNode::new(id, quantum_state, untried_actions, parent, improvement_depth)
    }
    
    /// Return node to pool for reuse
    pub fn return_node(&self, mut node: QuantumMCTSNode) {
        // Clean up node for reuse
        node.children.clear();
        node.optimize_memory();
        
        if let Ok(mut pool) = self.node_pool.try_lock() {
            if pool.len() < 1000 { // Limit pool size
                pool.push(node);
            }
        }
    }
    
    /// Get pool statistics
    pub fn pool_statistics(&self) -> PoolStatistics {
        let node_pool_size = self.node_pool.try_lock()
            .map(|pool| pool.len())
            .unwrap_or(0);
        let quantum_state_pool_size = self.quantum_state_pool.try_lock()
            .map(|pool| pool.len())
            .unwrap_or(0);
            
        PoolStatistics {
            node_pool_size,
            quantum_state_pool_size,
            nodes_created: self.base_factory.current_counter(),
        }
    }
}

/// Pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub node_pool_size: usize,
    pub quantum_state_pool_size: usize,
    pub nodes_created: u64,
}
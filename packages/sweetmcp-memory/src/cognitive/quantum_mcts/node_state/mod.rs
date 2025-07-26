//! Quantum node state management with zero-allocation optimization
//!
//! This module provides comprehensive quantum node state management for MCTS
//! with blazing-fast state transitions and cache-optimized memory layouts.

pub mod core;
pub mod node;

// Re-export core types for backward compatibility
pub use core::{QuantumNodeState, transitions};
pub use node::QuantumMCTSNode;

use std::sync::OnceLock;
use crate::cognitive::quantum_mcts::expansion::QuantumNodeFactory;

/// Node state coordinator for centralized management
pub struct NodeStateCoordinator {
    factory: QuantumNodeFactory,
}

static NODE_COORDINATOR: OnceLock<NodeStateCoordinator> = OnceLock::new();

impl NodeStateCoordinator {
    /// Get global node state coordinator instance
    pub fn global() -> &'static NodeStateCoordinator {
        NODE_COORDINATOR.get_or_init(|| Self::new())
    }

    /// Create new node state coordinator
    pub fn new() -> Self {
        Self {
            factory: QuantumNodeFactory::new(),
        }
    }

    /// Create coordinator with custom factory
    pub fn with_factory(factory: QuantumNodeFactory) -> Self {
        Self { factory }
    }

    /// Get reference to the node factory
    pub fn factory(&self) -> &QuantumNodeFactory {
        &self.factory
    }

    /// Create optimized root node for tree initialization
    pub fn create_root(
        &self,
        classical_state: crate::cognitive::mcts::CodeState,
        available_actions: Vec<String>,
    ) -> QuantumMCTSNode {
        // Optimize superposition size based on action count
        let superposition_size = (available_actions.len() as f64).sqrt().ceil() as usize;
        let superposition_size = superposition_size.clamp(2, 16); // Reasonable bounds
        
        self.factory.create_root_node_with_config(
            classical_state,
            available_actions,
            superposition_size,
            16, // Standard entanglement capacity
        )
    }

    /// Create child node with optimized inheritance
    pub fn create_child(
        &self,
        parent: &QuantumMCTSNode,
        classical_state: crate::cognitive::mcts::CodeState,
        available_actions: Vec<String>,
        applied_action: String,
    ) -> QuantumMCTSNode {
        let improvement_depth = parent.improvement_depth + 1;
        let phase_shift = self.calculate_phase_shift(&applied_action, improvement_depth);
        
        self.factory.create_child_with_inheritance(
            &parent.id,
            &parent.quantum_state,
            classical_state,
            available_actions,
            applied_action,
            improvement_depth,
            phase_shift,
        )
    }

    /// Create batch of child nodes efficiently
    pub fn create_child_batch(
        &self,
        parent: &QuantumMCTSNode,
        child_specs: Vec<(crate::cognitive::mcts::CodeState, Vec<String>, String)>,
    ) -> Vec<QuantumMCTSNode> {
        let factory_specs = child_specs
            .into_iter()
            .map(|(classical_state, untried_actions, applied_action)| {
                ChildSpec::new(
                    classical_state,
                    untried_actions,
                    applied_action,
                    parent.improvement_depth + 1,
                )
            })
            .collect();

        self.factory.create_child_batch(&parent.id, &parent.quantum_state, factory_specs)
    }

    /// Calculate phase shift based on action characteristics
    fn calculate_phase_shift(&self, applied_action: &str, depth: u32) -> f64 {
        // Hash-based phase shift for deterministic but varied phases
        let hash = self.simple_hash(applied_action);
        let base_shift = (hash as f64) / (u32::MAX as f64) * std::f64::consts::TAU;
        
        // Add depth-dependent phase evolution
        let depth_factor = (depth as f64) * 0.1;
        
        (base_shift + depth_factor) % std::f64::consts::TAU
    }

    /// Simple hash function for phase calculation
    fn simple_hash(&self, s: &str) -> u32 {
        s.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32))
    }

    /// Validate node tree consistency
    pub fn validate_tree(&self, root: &QuantumMCTSNode, nodes: &std::collections::HashMap<String, QuantumMCTSNode>) -> Result<(), String> {
        // Validate root node
        root.validate()?;
        
        // Recursive validation of tree structure
        self.validate_node_recursive(root, nodes, 0)?;
        
        Ok(())
    }

    /// Recursive node validation helper
    fn validate_node_recursive(
        &self,
        node: &QuantumMCTSNode,
        nodes: &std::collections::HashMap<String, QuantumMCTSNode>,
        depth: usize,
    ) -> Result<(), String> {
        // Prevent infinite recursion
        if depth > 1000 {
            return Err("Tree depth exceeds maximum limit (1000)".to_string());
        }

        // Validate current node
        node.validate()?;

        // Check parent-child relationships
        if let Some(parent_id) = &node.parent {
            if !nodes.contains_key(parent_id) {
                return Err(format!("Parent node '{}' not found in tree", parent_id));
            }
        }

        // Validate all children
        for (action, child_id) in &node.children {
            if let Some(child) = nodes.get(child_id) {
                // Check parent reference consistency
                if child.parent.as_ref() != Some(&node.id) {
                    return Err(format!(
                        "Child node '{}' has incorrect parent reference", 
                        child_id
                    ));
                }

                // Check applied action consistency
                if child.applied_action.as_ref() != Some(action) {
                    return Err(format!(
                        "Child node '{}' has incorrect applied action", 
                        child_id
                    ));
                }

                // Recursively validate child
                self.validate_node_recursive(child, nodes, depth + 1)?;
            } else {
                return Err(format!("Child node '{}' not found in tree", child_id));
            }
        }

        Ok(())
    }

    /// Calculate tree statistics
    pub fn calculate_tree_stats(&self, root: &QuantumMCTSNode, nodes: &std::collections::HashMap<String, QuantumMCTSNode>) -> TreeStatistics {
        let mut stats = TreeStatistics::default();
        
        self.collect_stats_recursive(root, nodes, &mut stats, 0);
        
        stats.avg_depth = if stats.leaf_count > 0 {
            stats.total_depth as f64 / stats.leaf_count as f64
        } else {
            0.0
        };

        stats.avg_branching_factor = if stats.internal_node_count > 0 {
            stats.total_children as f64 / stats.internal_node_count as f64
        } else {
            0.0
        };

        stats
    }

    /// Recursive statistics collection helper
    fn collect_stats_recursive(
        &self,
        node: &QuantumMCTSNode,
        nodes: &std::collections::HashMap<String, QuantumMCTSNode>,
        stats: &mut TreeStatistics,
        depth: usize,
    ) {
        stats.total_nodes += 1;
        stats.total_visits += node.visits;
        stats.total_quantum_reward += node.quantum_reward.norm();
        
        if depth > stats.max_depth {
            stats.max_depth = depth;
        }

        if node.is_leaf() {
            stats.leaf_count += 1;
            stats.total_depth += depth;
        } else {
            stats.internal_node_count += 1;
            stats.total_children += node.children_count();
        }

        if node.is_terminal {
            stats.terminal_count += 1;
        }

        // Recursively process children
        for child_id in node.children.values() {
            if let Some(child) = nodes.get(child_id) {
                self.collect_stats_recursive(child, nodes, stats, depth + 1);
            }
        }
    }
}

impl Default for NodeStateCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree statistics for analysis and debugging
#[derive(Debug, Clone, Default)]
pub struct TreeStatistics {
    pub total_nodes: usize,
    pub leaf_count: usize,
    pub internal_node_count: usize,
    pub terminal_count: usize,
    pub max_depth: usize,
    pub total_depth: usize,
    pub avg_depth: f64,
    pub total_children: usize,
    pub avg_branching_factor: f64,
    pub total_visits: u64,
    pub total_quantum_reward: f64,
}

impl TreeStatistics {
    /// Get human-readable statistics summary
    pub fn summary(&self) -> String {
        format!(
            "Tree Stats: {} nodes ({} leaves, {} internal, {} terminal), max depth: {}, avg depth: {:.2}, avg branching: {:.2}, total visits: {}, total reward: {:.4}",
            self.total_nodes,
            self.leaf_count,
            self.internal_node_count,
            self.terminal_count,
            self.max_depth,
            self.avg_depth,
            self.avg_branching_factor,
            self.total_visits,
            self.total_quantum_reward
        )
    }
}

/// Quick access functions for common operations
pub fn create_root_node(
    classical_state: crate::cognitive::mcts::CodeState,
    available_actions: Vec<String>,
) -> QuantumMCTSNode {
    NodeStateCoordinator::global().create_root(classical_state, available_actions)
}

pub fn create_child_node(
    parent: &QuantumMCTSNode,
    classical_state: crate::cognitive::mcts::CodeState,
    available_actions: Vec<String>,
    applied_action: String,
) -> QuantumMCTSNode {
    NodeStateCoordinator::global().create_child(parent, classical_state, available_actions, applied_action)
}

pub fn validate_tree(
    root: &QuantumMCTSNode,
    nodes: &std::collections::HashMap<String, QuantumMCTSNode>,
) -> Result<(), String> {
    NodeStateCoordinator::global().validate_tree(root, nodes)
}

pub fn tree_statistics(
    root: &QuantumMCTSNode,
    nodes: &std::collections::HashMap<String, QuantumMCTSNode>,
) -> TreeStatistics {
    NodeStateCoordinator::global().calculate_tree_stats(root, nodes)
}
//! Quantum-enhanced MCTS node with cache-optimized layout and zero-allocation patterns
//!
//! This module provides the QuantumMCTSNode struct with blazing-fast node operations
//! and memory-efficient storage patterns.

use std::collections::HashMap;
use crate::cognitive::quantum::Complex64;
use super::core::QuantumNodeState;

/// Quantum-enhanced MCTS node with cache-optimized layout
#[derive(Debug)]
#[repr(align(64))] // Cache-line aligned for optimal memory access patterns
pub struct QuantumMCTSNode {
    /// Node identifier - interned for fast comparison
    pub id: String,
    /// Visit count - atomic-friendly u64 for lock-free updates
    pub visits: u64,
    /// Quantum amplitude for this path - complex number for interference
    pub amplitude: Complex64,
    /// Total quantum reward - accumulated complex rewards
    pub quantum_reward: Complex64,
    /// Children mapping - pre-allocated HashMap for zero allocation
    pub children: HashMap<String, String>,
    /// Parent node reference - Option for root node handling  
    pub parent: Option<String>,
    /// Quantum state - embedded for cache locality
    pub quantum_state: QuantumNodeState,
    /// Untried actions - pre-allocated vector for action selection
    pub untried_actions: Vec<String>,
    /// Terminal flag - bool for fast terminal check
    pub is_terminal: bool,
    /// Applied action - cached for backpropagation optimization
    pub applied_action: Option<String>,
    /// Recursive improvement depth - u32 for memory efficiency
    pub improvement_depth: u32,
}

impl QuantumMCTSNode {
    /// Create new quantum MCTS node with optimized allocation patterns
    #[inline]
    pub fn new(
        id: String,
        quantum_state: QuantumNodeState,
        untried_actions: Vec<String>,
        parent: Option<String>,
        improvement_depth: u32,
    ) -> Self {
        // Pre-allocate children HashMap with expected capacity
        let mut children = HashMap::new();
        children.reserve(untried_actions.len().min(32)); // Reasonable upper bound
        
        Self {
            id,
            visits: 0,
            amplitude: Complex64::new(1.0, 0.0), // Start with unit amplitude
            quantum_reward: Complex64::new(0.0, 0.0), // Initialize to zero reward
            children,
            parent,
            quantum_state,
            untried_actions,
            is_terminal: false,
            applied_action: None,
            improvement_depth,
        }
    }
    
    /// Create root node with specialized initialization
    #[inline]
    pub fn new_root(
        id: String,
        quantum_state: QuantumNodeState,
        untried_actions: Vec<String>,
    ) -> Self {
        Self::new(id, quantum_state, untried_actions, None, 0)
    }
    
    /// Create child node with parent relationship
    #[inline]
    pub fn new_child(
        id: String,
        quantum_state: QuantumNodeState,
        untried_actions: Vec<String>,
        parent_id: String,
        applied_action: String,
        improvement_depth: u32,
    ) -> Self {
        let mut node = Self::new(id, quantum_state, untried_actions, Some(parent_id), improvement_depth);
        node.applied_action = Some(applied_action);
        node
    }
    
    /// Check if node can be expanded (has untried actions)
    #[inline(always)]
    pub fn can_expand(&self) -> bool {
        !self.untried_actions.is_empty() && !self.is_terminal
    }
    
    /// Check if node is fully expanded (no untried actions)
    #[inline(always)]
    pub fn is_fully_expanded(&self) -> bool {
        self.untried_actions.is_empty()
    }
    
    /// Check if node is a leaf (no children)
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    
    /// Check if node is root (no parent)
    #[inline(always)]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
    
    /// Get action count for performance analysis
    #[inline(always)]
    pub fn action_count(&self) -> usize {
        self.untried_actions.len() + self.children.len()
    }
    
    /// Get total possible actions (tried + untried)
    #[inline(always)]
    pub fn total_actions(&self) -> usize {
        self.action_count()
    }
    
    /// Get completion ratio (tried actions / total actions)
    #[inline(always)]
    pub fn completion_ratio(&self) -> f64 {
        let total = self.action_count();
        if total == 0 {
            1.0 // Fully completed if no actions
        } else {
            self.children.len() as f64 / total as f64
        }
    }
    
    /// Update visit count with overflow protection
    #[inline(always)]
    pub fn increment_visits(&mut self) {
        self.visits = self.visits.saturating_add(1);
    }
    
    /// Add multiple visits efficiently
    #[inline(always)]
    pub fn add_visits(&mut self, count: u64) {
        self.visits = self.visits.saturating_add(count);
    }
    
    /// Reset visit count
    #[inline(always)]
    pub fn reset_visits(&mut self) {
        self.visits = 0;
    }
    
    /// Add quantum reward with numerical stability
    #[inline(always)]
    pub fn add_quantum_reward(&mut self, reward: Complex64) {
        self.quantum_reward += reward;
        
        // Prevent numerical overflow by normalizing large rewards
        let norm = self.quantum_reward.norm();
        if norm > 1e10 {
            self.quantum_reward /= norm / 1e6; // Scale down while preserving direction
        }
    }
    
    /// Set quantum reward directly
    #[inline(always)]
    pub fn set_quantum_reward(&mut self, reward: Complex64) {
        self.quantum_reward = reward;
    }
    
    /// Reset quantum reward to zero
    #[inline(always)]
    pub fn reset_quantum_reward(&mut self) {
        self.quantum_reward = Complex64::new(0.0, 0.0);
    }
    
    /// Calculate average quantum reward for this node
    #[inline(always)]
    pub fn average_quantum_reward(&self) -> Complex64 {
        if self.visits > 0 {
            self.quantum_reward / self.visits as f64
        } else {
            Complex64::new(0.0, 0.0)
        }
    }
    
    /// Calculate real-valued average reward (ignoring imaginary component)
    #[inline(always)]
    pub fn average_real_reward(&self) -> f64 {
        if self.visits > 0 {
            self.quantum_reward.re / self.visits as f64
        } else {
            0.0
        }
    }
    
    /// Get quantum reward magnitude
    #[inline(always)]
    pub fn reward_magnitude(&self) -> f64 {
        self.quantum_reward.norm()
    }
    
    /// Get quantum reward phase
    #[inline(always)]
    pub fn reward_phase(&self) -> f64 {
        self.quantum_reward.arg()
    }
    
    /// Get quantum confidence based on visits and amplitude
    #[inline(always)]
    pub fn quantum_confidence(&self) -> f64 {
        let visit_confidence = (self.visits as f64).sqrt() / (self.visits as f64 + 1.0);
        let amplitude_confidence = self.amplitude.norm();
        let coherence_confidence = 1.0 - self.quantum_state.decoherence;
        
        visit_confidence * amplitude_confidence * coherence_confidence
    }
    
    /// Get statistical confidence based on visit count
    #[inline(always)]
    pub fn statistical_confidence(&self) -> f64 {
        (self.visits as f64).sqrt() / (self.visits as f64 + 1.0)
    }
    
    /// Update quantum amplitude with normalization
    #[inline(always)]
    pub fn update_amplitude(&mut self, delta: Complex64) {
        self.amplitude += delta;
        
        // Normalize amplitude if it becomes too large
        let norm = self.amplitude.norm();
        if norm > 10.0 {
            self.amplitude /= norm;
        }
    }
    
    /// Set quantum amplitude directly
    #[inline(always)]
    pub fn set_amplitude(&mut self, amplitude: Complex64) {
        self.amplitude = amplitude;
    }
    
    /// Normalize quantum amplitude to unit magnitude
    #[inline(always)]
    pub fn normalize_amplitude(&mut self) {
        let norm = self.amplitude.norm();
        if norm > f64::EPSILON {
            self.amplitude /= norm;
        } else {
            self.amplitude = Complex64::new(1.0, 0.0); // Reset to unit amplitude
        }
    }
    
    /// Mark node as terminal with cleanup
    #[inline]
    pub fn mark_terminal(&mut self) {
        self.is_terminal = true;
        // Clear untried actions to save memory
        self.untried_actions.clear();
        self.untried_actions.shrink_to_fit();
    }
    
    /// Unmark node as terminal (rarely used)
    #[inline]
    pub fn unmark_terminal(&mut self) {
        self.is_terminal = false;
    }
    
    /// Add child node with action mapping
    #[inline]
    pub fn add_child(&mut self, action: String, child_id: String) {
        self.children.insert(action, child_id);
    }
    
    /// Remove child node with cleanup
    #[inline]
    pub fn remove_child(&mut self, action: &str) -> Option<String> {
        self.children.remove(action)
    }
    
    /// Get child node ID by action
    #[inline(always)]
    pub fn get_child(&self, action: &str) -> Option<&String> {
        self.children.get(action)
    }
    
    /// Check if node has children
    #[inline(always)]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
    
    /// Get children count
    #[inline(always)]
    pub fn children_count(&self) -> usize {
        self.children.len()
    }
    
    /// Get all child actions
    #[inline]
    pub fn child_actions(&self) -> Vec<&String> {
        self.children.keys().collect()
    }
    
    /// Get all child node IDs
    #[inline]
    pub fn child_node_ids(&self) -> Vec<&String> {
        self.children.values().collect()
    }
    
    /// Remove an untried action (when expanding)
    #[inline]
    pub fn consume_untried_action(&mut self, action: &str) -> Option<String> {
        if let Some(pos) = self.untried_actions.iter().position(|a| a == action) {
            Some(self.untried_actions.swap_remove(pos))
        } else {
            None
        }
    }
    
    /// Get random untried action for expansion
    #[inline]
    pub fn random_untried_action(&self) -> Option<&String> {
        if self.untried_actions.is_empty() {
            None
        } else {
            // Use simple deterministic selection based on visits for reproducibility
            let index = (self.visits as usize) % self.untried_actions.len();
            self.untried_actions.get(index)
        }
    }
    
    /// Get first untried action
    #[inline(always)]
    pub fn first_untried_action(&self) -> Option<&String> {
        self.untried_actions.first()
    }
    
    /// Add untried action back (rarely used for backtracking)
    #[inline]
    pub fn add_untried_action(&mut self, action: String) {
        if !self.untried_actions.contains(&action) {
            self.untried_actions.push(action);
        }
    }
    
    /// Get memory usage estimation for this node
    #[inline]
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.id.capacity() +
        self.children.capacity() * (std::mem::size_of::<String>() * 2) +
        self.children.iter().map(|(k, v)| k.capacity() + v.capacity()).sum::<usize>() +
        self.untried_actions.capacity() * std::mem::size_of::<String>() +
        self.untried_actions.iter().map(|s| s.capacity()).sum::<usize>() +
        self.applied_action.as_ref().map_or(0, |s| s.capacity()) +
        self.parent.as_ref().map_or(0, |s| s.capacity()) +
        self.quantum_state.memory_usage()
    }
    
    /// Optimize memory usage by shrinking collections
    #[inline]
    pub fn optimize_memory(&mut self) {
        self.children.shrink_to_fit();
        self.untried_actions.shrink_to_fit();
        self.quantum_state.optimize_memory();
    }
    
    /// Reset node to initial state (keep structure but reset statistics)
    #[inline]
    pub fn reset_statistics(&mut self) {
        self.visits = 0;
        self.quantum_reward = Complex64::new(0.0, 0.0);
        self.amplitude = Complex64::new(1.0, 0.0);
        self.quantum_state.reset();
    }
    
    /// Validate node consistency
    #[inline]
    pub fn validate(&self) -> Result<(), String> {
        // Check for valid amplitude
        if self.amplitude.norm().is_nan() || self.amplitude.norm().is_infinite() {
            return Err(format!("Invalid amplitude: {:?}", self.amplitude));
        }
        
        // Check for valid reward
        if self.quantum_reward.norm().is_nan() || self.quantum_reward.norm().is_infinite() {
            return Err(format!("Invalid quantum reward: {:?}", self.quantum_reward));
        }
        
        // Check for terminal state consistency
        if self.is_terminal && !self.untried_actions.is_empty() {
            return Err("Terminal node should not have untried actions".to_string());
        }
        
        // Check for action consistency
        for (action, _) in &self.children {
            if self.untried_actions.contains(action) {
                return Err(format!("Action '{}' appears in both children and untried", action));
            }
        }
        
        // Validate quantum state
        self.quantum_state.validate()?;
        
        Ok(())
    }
}
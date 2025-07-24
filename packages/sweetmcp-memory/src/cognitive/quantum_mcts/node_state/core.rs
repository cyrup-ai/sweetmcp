//! Core quantum node state with zero-allocation optimization
//!
//! This module provides the foundational QuantumNodeState struct with
//! blazing-fast state transitions and cache-aligned memory layout.

use crate::cognitive::{
    mcts::CodeState,
    quantum::SuperpositionState,
};

/// Quantum state for MCTS nodes with zero-allocation state transitions
#[derive(Clone, Debug)]
#[repr(align(64))] // Cache-line aligned for SIMD operations
pub struct QuantumNodeState {
    /// Classical code state - preserved for compatibility
    pub classical_state: CodeState,
    /// Quantum superposition of possible improvements
    pub superposition: SuperpositionState,
    /// Entanglement connections to other nodes - pre-allocated for zero allocation
    pub entanglements: Vec<String>,
    /// Quantum phase for interference effects - optimized for fast computation
    pub phase: f64,
    /// Decoherence factor - tracks quantum coherence loss over time
    pub decoherence: f64,
}

impl QuantumNodeState {
    /// Create new quantum node state with optimized initialization
    #[inline(always)]
    pub fn new(
        classical_state: CodeState,
        superposition_size: usize,
    ) -> Self {
        // Pre-allocate entanglements vector to avoid reallocation
        let mut entanglements = Vec::new();
        entanglements.reserve(16); // Reserve space for typical entanglement count
        
        Self {
            classical_state,
            superposition: SuperpositionState::new(superposition_size),
            entanglements,
            phase: 0.0,
            decoherence: 0.0,
        }
    }
    
    /// Create quantum state with custom entanglement capacity
    #[inline]
    pub fn with_entanglement_capacity(
        classical_state: CodeState,
        superposition_size: usize,
        entanglement_capacity: usize,
    ) -> Self {
        let mut entanglements = Vec::new();
        entanglements.reserve(entanglement_capacity);
        
        Self {
            classical_state,
            superposition: SuperpositionState::new(superposition_size),
            entanglements,
            phase: 0.0,
            decoherence: 0.0,
        }
    }
    
    /// Create quantum state from existing state with phase copy
    #[inline]
    pub fn from_state_with_phase(other: &Self, new_phase: f64) -> Self {
        let mut new_state = other.clone();
        new_state.phase = new_phase;
        new_state
    }
    
    /// Apply quantum evolution with zero allocation
    #[inline(always)]
    pub fn evolve_phase(&mut self, delta: f64) {
        self.phase += delta;
        // Normalize phase to [0, 2π) to maintain numerical stability
        if self.phase >= std::f64::consts::TAU {
            self.phase -= std::f64::consts::TAU;
        } else if self.phase < 0.0 {
            self.phase += std::f64::consts::TAU;
        }
    }
    
    /// Set phase directly with normalization
    #[inline(always)]
    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase % std::f64::consts::TAU;
        if self.phase < 0.0 {
            self.phase += std::f64::consts::TAU;
        }
    }
    
    /// Get normalized phase in [0, 2π)
    #[inline(always)]
    pub fn normalized_phase(&self) -> f64 {
        let normalized = self.phase % std::f64::consts::TAU;
        if normalized < 0.0 {
            normalized + std::f64::consts::TAU
        } else {
            normalized
        }
    }
    
    /// Update decoherence with bounds checking
    #[inline(always)]
    pub fn update_decoherence(&mut self, delta: f64) {
        self.decoherence = (self.decoherence + delta).clamp(0.0, 1.0);
    }
    
    /// Set decoherence directly with bounds checking
    #[inline(always)]
    pub fn set_decoherence(&mut self, decoherence: f64) {
        self.decoherence = decoherence.clamp(0.0, 1.0);
    }
    
    /// Reset decoherence to perfect coherence
    #[inline(always)]
    pub fn reset_decoherence(&mut self) {
        self.decoherence = 0.0;
    }
    
    /// Apply exponential decoherence decay
    #[inline(always)]
    pub fn decay_coherence(&mut self, decay_rate: f64, time_delta: f64) {
        let decay_factor = (-decay_rate * time_delta).exp();
        self.decoherence = 1.0 - (1.0 - self.decoherence) * decay_factor;
        self.decoherence = self.decoherence.clamp(0.0, 1.0);
    }
    
    /// Check if quantum state is coherent
    #[inline(always)]
    pub fn is_coherent(&self, threshold: f64) -> bool {
        self.decoherence < threshold
    }
    
    /// Get coherence level (1.0 - decoherence)
    #[inline(always)]
    pub fn coherence(&self) -> f64 {
        1.0 - self.decoherence
    }
    
    /// Check if state is maximally coherent (no decoherence)
    #[inline(always)]
    pub fn is_maximally_coherent(&self) -> bool {
        self.decoherence == 0.0
    }
    
    /// Check if state is maximally decoherent (complete decoherence)
    #[inline(always)]
    pub fn is_maximally_decoherent(&self) -> bool {
        self.decoherence >= 1.0
    }
    
    /// Add entanglement with capacity management
    #[inline]
    pub fn add_entanglement(&mut self, node_id: String) {
        // Prevent duplicate entanglements with fast lookup
        if !self.entanglements.contains(&node_id) {
            self.entanglements.push(node_id);
        }
    }
    
    /// Add multiple entanglements efficiently
    #[inline]
    pub fn add_entanglements(&mut self, node_ids: Vec<String>) {
        for node_id in node_ids {
            self.add_entanglement(node_id);
        }
    }
    
    /// Remove entanglement with O(n) search but maintained order
    #[inline]  
    pub fn remove_entanglement(&mut self, node_id: &str) {
        self.entanglements.retain(|id| id != node_id);
    }
    
    /// Remove all entanglements efficiently
    #[inline]
    pub fn clear_entanglements(&mut self) {
        self.entanglements.clear();
    }
    
    /// Check if node is entangled with specific node
    #[inline]
    pub fn is_entangled_with(&self, node_id: &str) -> bool {
        self.entanglements.contains(&node_id.to_string())
    }
    
    /// Get entanglement count for performance monitoring
    #[inline(always)]
    pub fn entanglement_count(&self) -> usize {
        self.entanglements.len()
    }
    
    /// Check if node has any entanglements
    #[inline(always)]
    pub fn has_entanglements(&self) -> bool {
        !self.entanglements.is_empty()
    }
    
    /// Get entanglements as slice for iteration
    #[inline(always)]
    pub fn entanglements(&self) -> &[String] {
        &self.entanglements
    }
    
    /// Get mutable reference to entanglements for advanced operations
    #[inline]
    pub fn entanglements_mut(&mut self) -> &mut Vec<String> {
        &mut self.entanglements
    }
    
    /// Calculate quantum fidelity with another state
    #[inline]
    pub fn fidelity_with(&self, other: &Self) -> f64 {
        // Simplified fidelity calculation based on phase and coherence
        let phase_diff = (self.phase - other.phase).abs();
        let phase_similarity = (phase_diff / std::f64::consts::PI).cos().abs();
        
        let coherence_product = self.coherence() * other.coherence();
        let coherence_factor = (coherence_product + 0.1) / 1.1; // Avoid division by zero
        
        phase_similarity * coherence_factor
    }
    
    /// Calculate quantum distance from another state
    #[inline]
    pub fn distance_from(&self, other: &Self) -> f64 {
        1.0 - self.fidelity_with(other)
    }
    
    /// Reset quantum state to initial conditions
    #[inline]
    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.decoherence = 0.0;
        self.clear_entanglements();
        // Keep classical_state and superposition unchanged
    }
    
    /// Clone state with specific modifications
    #[inline]
    pub fn clone_with_modifications<F>(&self, modifier: F) -> Self 
    where
        F: FnOnce(&mut Self),
    {
        let mut cloned = self.clone();
        modifier(&mut cloned);
        cloned
    }
    
    /// Get memory usage estimation for this state
    #[inline]
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.entanglements.capacity() * std::mem::size_of::<String>() +
        self.entanglements.iter().map(|s| s.capacity()).sum::<usize>() +
        self.superposition.memory_usage()
    }
    
    /// Optimize memory usage by shrinking vectors
    #[inline]
    pub fn optimize_memory(&mut self) {
        self.entanglements.shrink_to_fit();
        self.superposition.optimize_memory();
    }
    
    /// Validate state consistency
    #[inline]
    pub fn validate(&self) -> Result<(), String> {
        if self.decoherence < 0.0 || self.decoherence > 1.0 {
            return Err(format!("Invalid decoherence value: {}", self.decoherence));
        }
        
        if self.phase.is_nan() || self.phase.is_infinite() {
            return Err(format!("Invalid phase value: {}", self.phase));
        }
        
        // Check for duplicate entanglements
        let mut sorted_entanglements = self.entanglements.clone();
        sorted_entanglements.sort();
        for window in sorted_entanglements.windows(2) {
            if window[0] == window[1] {
                return Err(format!("Duplicate entanglement found: {}", window[0]));
            }
        }
        
        self.superposition.validate()?;
        
        Ok(())
    }
}

impl Default for QuantumNodeState {
    fn default() -> Self {
        Self::new(
            CodeState::default(),
            4, // Default superposition size
        )
    }
}

impl PartialEq for QuantumNodeState {
    fn eq(&self, other: &Self) -> bool {
        self.classical_state == other.classical_state &&
        self.superposition == other.superposition &&
        self.entanglements == other.entanglements &&
        (self.phase - other.phase).abs() < f64::EPSILON &&
        (self.decoherence - other.decoherence).abs() < f64::EPSILON
    }
}

/// State transition helpers for common quantum operations
pub mod transitions {
    use super::*;

    /// Apply quantum interference between two states
    #[inline]
    pub fn apply_interference(state1: &mut QuantumNodeState, state2: &QuantumNodeState, strength: f64) {
        let phase_diff = state2.phase - state1.phase;
        let interference_phase = strength * phase_diff;
        state1.evolve_phase(interference_phase);
        
        // Coherence mixing based on interference strength
        let mixed_decoherence = state1.decoherence * (1.0 - strength) + state2.decoherence * strength;
        state1.set_decoherence(mixed_decoherence);
    }
    
    /// Create superposition of two quantum states
    #[inline]
    pub fn create_superposition(state1: &QuantumNodeState, state2: &QuantumNodeState, weight: f64) -> QuantumNodeState {
        let weight = weight.clamp(0.0, 1.0);
        let anti_weight = 1.0 - weight;
        
        let mixed_phase = state1.phase * anti_weight + state2.phase * weight;
        let mixed_decoherence = state1.decoherence * anti_weight + state2.decoherence * weight;
        
        let mut result = state1.clone();
        result.set_phase(mixed_phase);
        result.set_decoherence(mixed_decoherence);
        
        // Merge entanglements from both states
        for entanglement in &state2.entanglements {
            result.add_entanglement(entanglement.clone());
        }
        
        result
    }
    
    /// Apply quantum tunneling effect (sudden phase jump)
    #[inline]
    pub fn apply_tunneling(state: &mut QuantumNodeState, tunnel_probability: f64) {
        if tunnel_probability > 0.5 {
            // Apply phase jump proportional to tunneling probability
            let phase_jump = std::f64::consts::PI * tunnel_probability;
            state.evolve_phase(phase_jump);
            
            // Tunneling may temporarily increase coherence
            let coherence_boost = (1.0 - tunnel_probability) * 0.1;
            state.update_decoherence(-coherence_boost);
        }
    }
}
//! Action management for quantum MCTS expansion
//!
//! This module provides optimized action selection, transformation, and caching
//! with zero-allocation patterns and efficient resource pooling.

use crate::cognitive::{mcts::CodeState, quantum::Complex64};
use rand::seq::SliceRandom;
use std::collections::VecDeque;

/// Action manager with pooling and optimization
pub struct ActionManager {
    /// Pool of reusable action vectors
    action_pool: VecDeque<Vec<String>>,
    /// Maximum pool size to prevent unbounded growth
    max_pool_size: usize,
}

impl ActionManager {
    /// Create new action manager with optimized pooling
    pub fn new() -> Self {
        let mut action_pool = VecDeque::with_capacity(100);
        
        // Pre-allocate action vectors for zero allocation during expansion
        for _ in 0..50 {
            let actions = Vec::with_capacity(20); // Typical action count
            action_pool.push_back(actions);
        }
        
        Self {
            action_pool,
            max_pool_size: 100,
        }
    }

    /// Get quantum-enhanced actions with caching for performance
    pub fn get_quantum_actions(&mut self, state: &CodeState) -> Vec<String> {
        // Try to reuse from action pool
        if let Some(mut actions) = self.action_pool.pop_front() {
            actions.clear();
            self.populate_quantum_actions(&mut actions, state);
            actions
        } else {
            // Create new if pool is empty
            let mut actions = Vec::with_capacity(20);
            self.populate_quantum_actions(&mut actions, state);
            actions
        }
    }
    
    /// Populate actions vector with quantum-enhanced actions
    fn populate_quantum_actions(&self, actions: &mut Vec<String>, _state: &CodeState) {
        // Quantum-specific actions
        actions.extend([
            "quantum_optimize_superposition",
            "entangle_parallel_paths", 
            "quantum_phase_shift",
            "amplitude_amplification",
            "quantum_error_correction",
            "decoherence_mitigation",
            "quantum_annealing",
            "quantum_gradient_descent",
            "quantum_fourier_transform",
            "quantum_circuit_optimization",
        ].iter().map(|s| s.to_string()));

        // Classical performance actions
        actions.extend([
            "optimize_hot_paths",
            "reduce_allocations",
            "improve_cache_locality", 
            "parallelize_independent_work",
            "vectorize_loops",
            "inline_critical_functions",
            "prefetch_data",
            "optimize_branch_prediction",
        ].iter().map(|s| s.to_string()));
        
        // Shuffle for variety in action selection
        let mut rng = rand::thread_rng();
        actions.shuffle(&mut rng);
    }
    
    /// Return action vector to pool for reuse
    pub fn return_action_vector(&mut self, actions: Vec<String>) {
        if self.action_pool.len() < self.max_pool_size {
            self.action_pool.push_back(actions);
        }
    }

    /// Transform classical state based on action
    pub fn transform_classical_state(&self, state: &CodeState, action: &str) -> CodeState {
        // Action-specific transformations with performance optimizations
        let (latency_factor, memory_factor, relevance_factor) = match action {
            "quantum_optimize_superposition" => (0.92, 0.95, 1.05),
            "entangle_parallel_paths" => (0.88, 1.02, 1.08),
            "quantum_phase_shift" => (0.95, 0.98, 1.03),
            "amplitude_amplification" => (0.90, 0.97, 1.10),
            "quantum_error_correction" => (1.05, 1.10, 1.15), // Overhead but better quality
            "decoherence_mitigation" => (0.97, 1.03, 1.12),
            "quantum_annealing" => (0.85, 0.93, 1.08),
            "quantum_gradient_descent" => (0.93, 0.96, 1.06),
            "quantum_fourier_transform" => (0.89, 0.94, 1.07),
            "quantum_circuit_optimization" => (0.87, 0.91, 1.09),
            "optimize_hot_paths" => (0.80, 0.95, 1.05),
            "reduce_allocations" => (0.95, 0.75, 1.03),
            "improve_cache_locality" => (0.85, 0.98, 1.04),
            "parallelize_independent_work" => (0.70, 1.15, 1.08),
            _ => (0.98, 0.99, 1.01), // Default minimal improvement
        };

        CodeState {
            code: format!("// Quantum: {}\n{}", action, state.code),
            latency: (state.latency * latency_factor).max(0.001), // Prevent zero latency
            memory: (state.memory * memory_factor).max(0.001),   // Prevent zero memory
            relevance: (state.relevance * relevance_factor).min(2.0), // Cap relevance
        }
    }
    
    /// Calculate phase increment for action
    #[inline(always)]
    pub fn calculate_phase_increment(&self, action: &str, base_rate: f64) -> f64 {
        // Action-specific phase shifts for quantum interference
        let action_factor = match action {
            "quantum_phase_shift" => 2.0,
            "quantum_fourier_transform" => 1.8,
            "amplitude_amplification" => 1.5,
            "quantum_annealing" => 1.3,
            "quantum_circuit_optimization" => 1.4,
            "entangle_parallel_paths" => 1.2,
            _ => 1.0,
        };
        
        base_rate * action_factor
    }
    
    /// Calculate decoherence increment for action
    #[inline(always)]
    pub fn calculate_decoherence_increment(&self, action: &str, current_decoherence: f64) -> f64 {
        let base_increment = 0.01;
        
        // Some actions increase decoherence more than others
        let decoherence_factor = match action {
            "decoherence_mitigation" => -0.5, // Actually reduces decoherence
            "quantum_error_correction" => -0.3, // Reduces decoherence
            "quantum_optimize_superposition" => 0.5,
            "amplitude_amplification" => 0.8,
            "entangle_parallel_paths" => 1.2, // Entanglement increases decoherence
            "parallelize_independent_work" => 1.5, // Parallelism increases decoherence
            _ => 1.0,
        };
        
        // Decoherence increase slows down as we approach maximum
        let saturation_factor = 1.0 - current_decoherence;
        base_increment * decoherence_factor * saturation_factor
    }
    
    /// Calculate child amplitude with optimized complex arithmetic
    pub fn calculate_child_amplitude(&self, parent_amplitude: &Complex64, action: &str) -> Complex64 {
        // Action-dependent phase shift for quantum interference
        let phase_shift = match action {
            "optimize_hot_paths" => 0.1,
            "reduce_allocations" => 0.15,
            "improve_cache_locality" => 0.2,
            "quantum_optimize_superposition" => 0.25,
            "amplitude_amplification" => 0.3,
            "quantum_phase_shift" => 0.35,
            "entangle_parallel_paths" => 0.18,
            "quantum_annealing" => 0.22,
            "quantum_fourier_transform" => 0.28,
            _ => 0.05,
        };

        // Amplitude decay factor for realistic quantum evolution
        let decay_factor = match action {
            "quantum_error_correction" => 0.98, // Minimal decay due to error correction
            "decoherence_mitigation" => 0.95,
            "amplitude_amplification" => 1.05, // Actually amplifies
            _ => 0.90, // Standard decay
        };

        // Complex phase rotation: e^(i * phase_shift)
        let phase_rotation = Complex64::new(0.0, phase_shift).exp();
        
        // Apply phase rotation and amplitude decay
        *parent_amplitude * phase_rotation * decay_factor
    }

    /// Get pool statistics
    pub fn pool_size(&self) -> usize {
        self.action_pool.len()
    }

    /// Get pool capacity
    pub fn pool_capacity(&self) -> usize {
        self.action_pool.capacity()
    }

    /// Cleanup resources
    pub fn cleanup(&mut self) {
        // Keep some vectors for future use, but limit memory usage
        while self.action_pool.len() > 25 {
            self.action_pool.pop_back();
        }
    }
}
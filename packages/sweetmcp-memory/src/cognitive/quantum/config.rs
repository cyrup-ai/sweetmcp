//! Quantum orchestration configuration extracted from orchestrator

use serde::{Deserialize, Serialize};

/// Quantum orchestration configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumOrchestrationConfig {
    /// Maximum recursive depth
    pub max_recursive_depth: u32,
    /// Improvement threshold
    pub improvement_threshold: f64,
    /// Quantum coherence time (ms)
    pub coherence_time_ms: u64,
    /// Parallel quantum circuits
    pub parallel_circuits: usize,
    /// Convergence epsilon
    pub convergence_epsilon: f64,
    /// Max iterations per depth
    pub max_iterations_per_depth: u32,
}

impl Default for QuantumOrchestrationConfig {
    fn default() -> Self {
        Self {
            max_recursive_depth: 5,
            improvement_threshold: 0.05,
            coherence_time_ms: 1000,
            parallel_circuits: 4,
            convergence_epsilon: 0.001,
            max_iterations_per_depth: 100,
        }
    }
}

/// Recursive improvement state
#[derive(Debug, Clone, Serialize)]
pub struct RecursiveState {
    pub depth: u32,
    pub improvement: f64,
    pub quantum_fidelity: f64,
    pub decoherence_level: f64,
    pub entanglement_strength: f64,
}
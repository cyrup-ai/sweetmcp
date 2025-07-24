//! Quantum orchestrator for managing recursive improvement loops

use std::sync::Arc;
use tokio::sync::mpsc;

use crate::cognitive::{
    committee::CommitteeEvent,
    mcts::CodeState,
    performance::PerformanceAnalyzer,
    quantum_mcts::QuantumMCTSConfig,
    types::{CognitiveError, OptimizationOutcome, OptimizationSpec},
};

// Import from quantum module
use crate::cognitive::quantum::{
    recursive_improvement::RecursiveImprovement,
};

// Re-export main types for compatibility
pub use crate::cognitive::quantum::config::{QuantumOrchestrationConfig, RecursiveState};
pub use crate::cognitive::quantum::mcts_integration::QuantumOrchestrator;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quantum_orchestration() {
        // Test implementation
    }
}
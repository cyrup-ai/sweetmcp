// src/cognitive/mod.rs
//! Cognitive enhancement system for quantum memory optimization
//!
//! This module provides self-optimizing capabilities through committee-based
//! evaluation and Monte Carlo Tree Search (MCTS).

pub mod committee;
pub mod compiler;
pub mod evolution;
pub mod evolution_manager;
pub mod llm_integration;
pub mod manager;
pub mod mcts;
pub mod mesh;
pub mod orchestrator;
pub mod performance;
pub mod subsystem_coordinator;
pub mod types;

// Core cognitive modules from existing implementation
pub mod attention;
pub mod state;

// Quantum-specific cognitive modules
pub mod quantum;
pub mod quantum_mcts;
pub mod quantum_orchestrator;

// Re-exports for convenience
pub use committee::{CommitteeEvent, EvaluationCommittee};
pub use evolution::{CodeEvolution, CognitiveCodeEvolution, EvolutionEngine};
pub use mcts::{CodeState, MCTS};
pub use orchestrator::InfiniteOrchestrator;
pub use quantum_mcts::{QuantumMCTS, QuantumMCTSConfig, QuantumNodeState, QuantumTreeStatistics};
pub use quantum_orchestrator::{QuantumOrchestrationConfig, QuantumOrchestrator, RecursiveState};
pub use types::{
    CognitiveError, CognitiveMemoryNode, CognitiveSettings, EvolutionMetadata, ImpactFactors,
    Model, ModelType, OptimizationOutcome, OptimizationSpec, OptimizationType,
    PendingOptimizationResult, QuantumSignature,
};

// Re-export existing cognitive components
pub use attention::AttentionMechanism;
pub use manager::CognitiveMemoryManager;
pub use state::CognitiveState;

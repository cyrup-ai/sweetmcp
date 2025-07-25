//! Evaluation module coordination
//!
//! This module coordinates all evaluation components with optimized access patterns
//! and zero-allocation re-exports for blazing-fast performance.

pub mod agent_orchestration;
pub mod agent_simulation_batch;
pub mod agent_simulation_core;
pub mod agent_simulation_scores;
pub mod builder;
pub mod consensus;
pub mod consensus_calculation;
pub mod consensus_metrics;
pub mod decision_building;
pub mod evaluation_simulation;
pub mod execution;
pub mod management;
pub mod timeout_handling;

// Re-export key types for convenient access
pub use consensus::ConsensusCalculator;
pub use consensus_metrics::AdvancedConsensusMetrics;
pub use execution::EvaluationExecutor;
pub use evaluation_simulation::AgentSimulator;
pub use management::{ExtendedCommitteeEvent, EvaluationStatistics};
pub use builder::CommitteeBuilder;
//! Evaluation module coordination
//!
//! This module coordinates all evaluation components with optimized access patterns
//! and zero-allocation re-exports for blazing-fast performance.

pub mod consensus;
pub mod consensus_metrics;
pub mod execution;
pub mod evaluation_simulation;
pub mod management;
pub mod builder;

// Re-export key types for convenient access
pub use consensus::ConsensusCalculator;
pub use consensus_metrics::AdvancedConsensusMetrics;
pub use execution::EvaluationExecutor;
pub use evaluation_simulation::AgentSimulator;
pub use management::{ExtendedCommitteeEvent, EvaluationStatistics};
pub use builder::CommitteeBuilder;
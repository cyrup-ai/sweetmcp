//! Consensus-based decision making for committee evaluation
//!
//! This module provides high-performance consensus algorithms with LRU caching,
//! multi-phase evaluation, and event-driven architecture for optimal decision making.

pub mod calculation;
pub mod committee;
pub mod evaluation_phases;
pub mod events;
pub mod steering;

// Re-export all public types for external use
pub use calculation::{ConsensusCalculator, ConsensusQuality};
pub use committee::Committee;
pub use evaluation_phases::{EvaluationPhase, EvaluationRound, RoundStatistics, PhaseExecutor};
pub use events::{CommitteeEvent, EventBus, LoggingListener};
pub use steering::{SteeringSystem, SteeringFeedback};
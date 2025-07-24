//! Committee-based evaluation system decomposition
//!
//! This module provides the decomposed committee evaluation functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod core;
pub mod consensus;
pub mod evaluation;

// Re-export key types and functions for backward compatibility
pub use core::{
    ConsensusDecision, AgentEvaluation, EvaluationRubric, CommitteeAgent, AgentPerspective,
    CommitteeConfig, EvaluationContext
};

// Re-export types from cognitive::types for committee modules
pub use crate::cognitive::types::{Model, ModelType};

pub use consensus::{
    committee::Committee,
    evaluation_phases::{EvaluationPhase, EvaluationRound, RoundStatistics},
    events::CommitteeEvent
};

pub use evaluation::{
    EvaluationStatistics, ExtendedCommitteeEvent, CommitteeBuilder
};

// Type alias for backward compatibility
pub type EvaluationCommittee = consensus::committee::Committee;
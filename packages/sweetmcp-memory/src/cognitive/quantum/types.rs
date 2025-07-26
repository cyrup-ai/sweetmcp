//! Core types and error definitions for the quantum cognitive system

use std::fmt;
use thiserror::Error;

// Re-export CognitiveError and CognitiveResult from the more comprehensive types module
pub use crate::cognitive::types::{CognitiveError, CognitiveResult};

/// Query intent for routing decisions
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    Retrieval,
    Association,
    Prediction,
    Reasoning,
    Exploration,
    Creation,
}

/// Enhanced query with cognitive context
#[derive(Debug, Clone)]
pub struct EnhancedQuery {
    pub original: String,
    pub intent: QueryIntent,
    pub context_embedding: Vec<f32>,
    pub temporal_context: Option<TemporalContext>,
    pub cognitive_hints: Vec<String>,
    pub expected_complexity: f64,
}

/// Temporal context for queries
#[derive(Debug, Clone)]
pub struct TemporalContext {
    pub timestamp: std::time::Instant,
    pub duration: std::time::Duration,
    pub temporal_type: TemporalType,
}

#[derive(Debug, Clone)]
pub enum TemporalType {
    Past,
    Present,
    Future,
    Timeless,
}

/// Routing strategy for query processing
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    Quantum,
    Attention,
    Causal,
    Emergent,
    Hybrid(Vec<RoutingStrategy>),
}

/// Routing decision with alternatives and reasoning
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub strategy: RoutingStrategy,
    pub target_context: String,
    pub confidence: f64,
    pub alternatives: Vec<AlternativeRoute>,
    pub reasoning: String,
}

/// Alternative routing option
#[derive(Debug, Clone)]
pub struct AlternativeRoute {
    pub strategy: RoutingStrategy,
    pub confidence: f64,
    pub estimated_quality: f64,
}

/// Types of entanglement between quantum states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuantumEntanglementType {
    Bell,
    GHZ,
    Werner,
    Cluster,
    Custom,
}

// Re-export for backward compatibility
pub use QuantumEntanglementType as EntanglementType;

impl fmt::Display for QuantumEntanglementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantumEntanglementType::Bell => write!(f, "Bell"),
            QuantumEntanglementType::GHZ => write!(f, "GHZ"),
            QuantumEntanglementType::Werner => write!(f, "Werner"),
            QuantumEntanglementType::Cluster => write!(f, "Cluster"),
            QuantumEntanglementType::Custom => write!(f, "Custom"),
        }
    }
}

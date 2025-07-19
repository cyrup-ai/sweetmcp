//! Core types and error definitions for the quantum cognitive system

use std::fmt;
use thiserror::Error;

/// Result type for cognitive operations
pub type CognitiveResult<T> = Result<T, CognitiveError>;

/// Errors that can occur in cognitive operations
#[derive(Error, Debug)]
pub enum CognitiveError {
    #[error("Quantum decoherence detected: {0}")]
    QuantumDecoherence(String),

    #[error("Context processing error: {0}")]
    ContextProcessingError(String),

    #[error("System capacity exceeded: {0}")]
    CapacityExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Memory allocation error: {0}")]
    MemoryAllocationError(String),

    #[error("Invalid quantum state: {0}")]
    InvalidQuantumState(String),

    #[error("Measurement failed: {0}")]
    MeasurementError(String),

    #[error("Entanglement error: {0}")]
    EntanglementError(String),

    #[error("Error correction failed: {0}")]
    ErrorCorrectionFailed(String),

    #[error("Hardware backend error: {0}")]
    HardwareBackendError(String),

    #[error("LLM integration error: {0}")]
    LLMIntegrationError(String),

    #[error("Evolution error: {0}")]
    EvolutionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

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
pub enum EntanglementType {
    Bell,
    GHZ,
    Werner,
    Cluster,
    Custom,
}

impl fmt::Display for EntanglementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntanglementType::Bell => write!(f, "Bell"),
            EntanglementType::GHZ => write!(f, "GHZ"),
            EntanglementType::Werner => write!(f, "Werner"),
            EntanglementType::Cluster => write!(f, "Cluster"),
            EntanglementType::Custom => write!(f, "Custom"),
        }
    }
}

//! Core cognitive types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Cognitive state representing the current understanding and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub activation_pattern: Vec<f32>,
    pub attention_weights: Vec<f32>,
    pub temporal_context: TemporalContext,
    pub uncertainty: f32,
    pub confidence: f32,
    pub meta_awareness: f32,
}

/// Temporal context and dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContext {
    pub history_embedding: Vec<f32>,
    pub prediction_horizon: Vec<f32>,
    pub causal_dependencies: Vec<CausalLink>,
    pub temporal_decay: f32,
}

/// Causal relationship between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub source_id: String,
    pub target_id: String,
    pub causal_strength: f32,
    pub temporal_distance: i64, // milliseconds
}

/// Quantum signature for superposition routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSignature {
    pub coherence_fingerprint: Vec<f32>,
    pub entanglement_bonds: Vec<EntanglementBond>,
    pub superposition_contexts: Vec<String>,
    pub collapse_probability: f32,
}

/// Cognitive memory node with enhanced capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMemoryNode {
    pub base_memory: crate::memory::MemoryNode,
    pub cognitive_state: CognitiveState,
    pub quantum_signature: Option<QuantumSignature>,
    pub evolution_metadata: Option<EvolutionMetadata>,
    pub attention_weights: Vec<f32>,
    pub semantic_relationships: Vec<String>,
}

/// Configuration settings for cognitive memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSettings {
    pub enable_quantum_routing: bool,
    pub enable_evolution: bool,
    pub enable_attention_mechanism: bool,
    pub max_cognitive_load: f32,
    pub quantum_coherence_threshold: f32,
    pub evolution_mutation_rate: f32,
    pub attention_decay_rate: f32,
    pub meta_awareness_level: f32,
}

impl Default for CognitiveSettings {
    fn default() -> Self {
        Self {
            enable_quantum_routing: true,
            enable_evolution: true,
            enable_attention_mechanism: true,
            max_cognitive_load: 1.0,
            quantum_coherence_threshold: 0.8,
            evolution_mutation_rate: 0.1,
            attention_decay_rate: 0.95,
            meta_awareness_level: 0.7,
        }
    }
}

/// Quantum entanglement between memories or agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntanglementBond {
    pub target_id: String,
    pub bond_strength: f32,
    pub entanglement_type: EntanglementType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntanglementType {
    Semantic,
    Temporal,
    Causal,
    Emergent,
}

/// Evolution metadata tracking system development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionMetadata {
    pub generation: u32,
    pub fitness_score: f32,
    pub mutation_history: Vec<MutationEvent>,
    pub specialization_domains: Vec<SpecializationDomain>,
    pub adaptation_rate: f32,
}

/// A mutation event in system evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub mutation_type: MutationType,
    pub impact_score: f32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    AttentionWeightAdjustment,
    RoutingStrategyModification,
    ContextualUnderstandingEvolution,
    QuantumCoherenceOptimization,
    EmergentPatternRecognition,
}

/// Specialization domains for agent evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecializationDomain {
    SemanticProcessing,
    TemporalAnalysis,
    CausalReasoning,
    PatternRecognition,
    ContextualUnderstanding,
    PredictiveModeling,
    MetaCognition,
}

/// Routing decision with confidence and alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub strategy: RoutingStrategy,
    pub target_context: String,
    pub confidence: f32,
    pub alternatives: Vec<AlternativeRoute>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    Quantum,
    Attention,
    Causal,
    Emergent,
    Hybrid(Vec<RoutingStrategy>),
}

/// Alternative routing option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeRoute {
    pub strategy: RoutingStrategy,
    pub confidence: f32,
    pub estimated_quality: f32,
}

/// Enhanced query with cognitive understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuery {
    pub original: String,
    pub intent: QueryIntent,
    pub context_embedding: Vec<f32>,
    pub temporal_context: Option<TemporalContext>,
    pub cognitive_hints: Vec<String>,
    pub expected_complexity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryIntent {
    Retrieval,
    Association,
    Prediction,
    Reasoning,
    Exploration,
    Creation,
}

/// Emergent pattern discovered by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergentPattern {
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub strength: f32,
    pub affected_memories: Vec<String>,
    pub discovery_timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Temporal,
    Semantic,
    Causal,
    Behavioral,
    Structural,
}

/// Cognitive error types
#[derive(Debug, thiserror::Error)]
pub enum CognitiveError {
    #[error("Quantum decoherence occurred: {0}")]
    QuantumDecoherence(String),

    #[error("Attention overflow: {0}")]
    AttentionOverflow(String),

    #[error("Evolution failure: {0}")]
    EvolutionFailure(String),

    #[error("Meta-consciousness error: {0}")]
    MetaConsciousnessError(String),

    #[error("Context processing error: {0}")]
    ContextProcessingError(String),

    #[error("Routing error: {0}")]
    RoutingError(String),

    #[error("Cognitive capacity exceeded: {0}")]
    CapacityExceeded(String),
}

pub type CognitiveResult<T> = Result<T, CognitiveError>;

/// Specification for optimization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSpec {
    pub objective: String,
    pub constraints: Vec<String>,
    pub success_criteria: Vec<String>,
    pub optimization_type: OptimizationType,
    pub timeout_ms: Option<u64>,
    pub max_iterations: Option<u32>,
    pub target_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    Performance,
    Quality,
    Efficiency,
    Accuracy,
    Custom(String),
}

/// Impact measurement structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactFactors {
    pub performance_impact: f32,
    pub quality_impact: f32,
    pub user_satisfaction_impact: f32,
    pub system_stability_impact: f32,
    pub maintainability_impact: f32,
    pub overall_score: f32,
}

impl ImpactFactors {
    pub fn new() -> Self {
        Self {
            performance_impact: 0.0,
            quality_impact: 0.0,
            user_satisfaction_impact: 0.0,
            system_stability_impact: 0.0,
            maintainability_impact: 0.0,
            overall_score: 0.0,
        }
    }

    pub fn calculate_overall_score(&mut self) {
        self.overall_score = (self.performance_impact
            + self.quality_impact
            + self.user_satisfaction_impact
            + self.system_stability_impact
            + self.maintainability_impact)
            / 5.0;
    }
}

/// Result of optimization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationOutcome {
    Success {
        improvements: Vec<String>,
        performance_gain: f32,
        quality_score: f32,
        metadata: HashMap<String, serde_json::Value>,
    },
    PartialSuccess {
        improvements: Vec<String>,
        issues: Vec<String>,
        performance_gain: f32,
        quality_score: f32,
    },
    Failure {
        errors: Vec<String>,
        root_cause: String,
        suggestions: Vec<String>,
    },
}

/// Async optimization result wrapper
pub struct PendingOptimizationResult {
    rx: tokio::sync::oneshot::Receiver<CognitiveResult<OptimizationOutcome>>,
}

impl PendingOptimizationResult {
    pub fn new(rx: tokio::sync::oneshot::Receiver<CognitiveResult<OptimizationOutcome>>) -> Self {
        Self { rx }
    }
}

impl std::future::Future for PendingOptimizationResult {
    type Output = CognitiveResult<OptimizationOutcome>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match std::pin::Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => std::task::Poll::Ready(Err(
                CognitiveError::ContextProcessingError("Channel closed".to_string()),
            )),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

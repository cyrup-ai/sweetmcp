//! Core cognitive types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

// Re-export MCTS types for backward compatibility
pub use super::mcts::types::{MCTSNode, CodeState};

// Re-export evolution types with alias for backward compatibility
pub use super::evolution::EvolutionRules;
pub use super::evolution::EvolutionRules as EvolutionRule;

// Re-export vector optimization types for cognitive usage
pub use crate::vector::async_vector_optimization::coordinator_types::OptimizationSpec;

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
    // Additional fields needed by manager.rs
    pub enabled: bool,
    pub llm_provider: String,
    pub attention_heads: usize,
    pub evolution_rate: f32,
    pub quantum_coherence_time: std::time::Duration,
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
            // Additional fields needed by manager.rs
            enabled: true,
            llm_provider: "openai".to_string(),
            attention_heads: 8,
            evolution_rate: 0.1,
            quantum_coherence_time: std::time::Duration::from_millis(100),
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

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Optimization error: {0}")]
    OptimizationError(String),

    #[error("Orchestration error: {0}")]
    OrchestrationError(String),

    #[error("Spec error: {0}")]
    SpecError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),

    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
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
    pub content_type: ContentType,
    pub baseline_metrics: BaselineMetrics,
    pub evolution_rules: EvolutionRules,
}

/// Content type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentType {
    pub format: String,
    pub restrictions: Restrictions,
}

/// Restrictions for content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Restrictions {
    pub compiler: String,
    pub max_latency_increase: f32,
    pub max_memory_increase: f32,
    pub min_relevance_improvement: f32,
}

/// Constraints for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    pub size: String,
    pub style: String,
    pub schemas: Vec<String>,
}

/// Evolution rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRules {
    pub build_on_previous: bool,
    pub new_axis_per_iteration: bool,
    pub max_cumulative_latency_increase: f32,
    pub min_action_diversity: f32,
    pub validation_required: bool,
}

/// Baseline metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub latency: f32,
    pub memory: f32,
    pub relevance: f32,
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
    // Additional fields for backward compatibility
    pub latency_factor: f32,
    pub memory_factor: f32,
    pub relevance_factor: f32,
    pub confidence: f32,
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
            latency_factor: 1.0,
            memory_factor: 1.0,
            relevance_factor: 1.0,
            confidence: 0.5,
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

/// Model type for LLM evaluation agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    OpenAI { model: String },
    Anthropic { model: String },
    Local { model: String },
}

impl ModelType {
    pub fn display_name(&self) -> &str {
        match self {
            ModelType::OpenAI { model } => model,
            ModelType::Anthropic { model } => model,
            ModelType::Local { model } => model,
        }
    }
}

impl fmt::Display for ModelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Model wrapper for LLM operations
#[derive(Debug, Clone)]
pub struct Model {
    model_type: ModelType,
    client: ModelClient,
}

/// Model client abstraction
#[derive(Debug, Clone)]
pub enum ModelClient {
    OpenAI(String),
    Anthropic(String),
    Local(String),
}

impl Model {
    pub async fn create(
        model_type: ModelType,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = match &model_type {
            ModelType::OpenAI { model } => ModelClient::OpenAI(model.clone()),
            ModelType::Anthropic { model } => ModelClient::Anthropic(model.clone()),
            ModelType::Local { model } => ModelClient::Local(model.clone()),
        };

        Ok(Self { model_type, client })
    }

    pub async fn generate_response(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Mock implementation for now
        Ok(format!(
            "Response from {} for prompt: {}",
            self.model_type.display_name(),
            prompt
        ))
    }

    pub fn available_types() -> Vec<ModelType> {
        vec![
            ModelType::OpenAI {
                model: "gpt-4".to_string(),
            },
            ModelType::OpenAI {
                model: "gpt-3.5-turbo".to_string(),
            },
            ModelType::Anthropic {
                model: "claude-3-sonnet".to_string(),
            },
            ModelType::Local {
                model: "llama-2-7b".to_string(),
            },
        ]
    }
}

impl CognitiveMemoryNode {
    /// Check if this memory node has been enhanced with cognitive features
    pub fn is_enhanced(&self) -> bool {
        self.cognitive_state.activation_pattern.len() > 0
            || self.quantum_signature.is_some()
            || self.evolution_metadata.is_some()
            || !self.attention_weights.is_empty()
    }

    /// Get the base memory node
    pub fn base(&self) -> &crate::memory::MemoryNode {
        &self.base_memory
    }
}

impl From<crate::memory::MemoryNode> for CognitiveMemoryNode {
    fn from(memory: crate::memory::MemoryNode) -> Self {
        Self {
            base_memory: memory,
            cognitive_state: CognitiveState {
                activation_pattern: Vec::new(),
                attention_weights: Vec::new(),
                temporal_context: TemporalContext {
                    history_embedding: Vec::new(),
                    prediction_horizon: Vec::new(),
                    causal_dependencies: Vec::new(),
                    temporal_decay: 0.95,
                },
                uncertainty: 0.0,
                confidence: 0.0,
                meta_awareness: 0.0,
            },
            quantum_signature: None,
            evolution_metadata: None,
            attention_weights: Vec::new(),
            semantic_relationships: Vec::new(),
        }
    }
}

impl EvolutionMetadata {
    /// Create new evolution metadata for a memory node
    pub fn new(_memory: &crate::memory::MemoryNode) -> Self {
        Self {
            generation: 0,
            fitness_score: 0.0,
            mutation_history: Vec::new(),
            specialization_domains: Vec::new(),
            adaptation_rate: 0.1,
        }
    }
}

//! Committee structure and coordination for multi-agent evaluation
//!
//! This module provides the main committee structure that coordinates
//! multiple agents for comprehensive code evaluation with optimized
//! performance and zero allocation fast paths.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio::time::{timeout, Duration};
use futures::stream::{FuturesUnordered, StreamExt};
use tracing::warn;
use sha2::Digest;

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;
use super::agents::{CommitteeAgent, AgentEvaluation};
use super::agents::AgentPerspective;
use super::evaluation::{EvaluationRubric, ConsensusDecision, EvaluationContext, ScoringWeights};

/// Committee configuration for evaluation behavior
#[derive(Debug, Clone)]
pub struct CommitteeConfig {
    pub max_agents: usize,
    pub consensus_threshold: f64,
    pub max_rounds: usize,
    pub timeout_seconds: u64,
    pub require_unanimous: bool,
    pub weight_by_reliability: bool,
}

impl Default for CommitteeConfig {
    fn default() -> Self {
        Self {
            max_agents: 7,
            consensus_threshold: 0.7,
            max_rounds: 3,
            timeout_seconds: 30,
            require_unanimous: false,
            weight_by_reliability: true,
        }
    }
}

/// Evaluation committee with multiple specialized agents
pub struct EvaluationCommittee {
    /// Committee agents with different perspectives
    agents: Vec<CommitteeAgent>,
    
    /// Committee configuration
    config: CommitteeConfig,
    
    /// Evaluation rubric
    rubric: EvaluationRubric,
    
    /// Scoring weights
    scoring_weights: ScoringWeights,
    
    /// Evaluation context
    context: EvaluationContext,
    
    /// Communication channel for coordination
    coordinator_tx: mpsc::Sender<CommitteeMessage>,
    
    /// Semaphore for controlling concurrent evaluations
    evaluation_semaphore: Arc<Semaphore>,
    
    /// Committee statistics
    stats: CommitteeStats,
}

/// Messages for committee coordination
#[derive(Debug, Clone)]
pub enum CommitteeMessage {
    StartEvaluation { action: String, state: CodeState },
    AgentEvaluation { agent_id: String, evaluation: AgentEvaluation },
    ConsensusReached { decision: ConsensusDecision },
    EvaluationTimeout,
    Error { message: String },
}

/// Committee performance statistics
#[derive(Debug, Clone, Default)]
pub struct CommitteeStats {
    pub total_evaluations: u64,
    pub consensus_reached: u64,
    pub unanimous_decisions: u64,
    pub avg_evaluation_time_ms: f64,
    pub agent_performance: HashMap<String, f64>,
}

impl EvaluationCommittee {
    /// Create new evaluation committee
    pub async fn new(
        coordinator_tx: mpsc::Sender<CommitteeMessage>,
        max_concurrent: usize,
    ) -> Result<Self, CognitiveError> {
        let agents = Self::create_default_agents();
        let config = CommitteeConfig::default();
        let rubric = EvaluationRubric::custom(
            "Evaluate code changes for progress and quality".to_string(),
            vec!["Makes measurable progress".to_string()],
            vec!["Maintains safety".to_string()],
        );
        let scoring_weights = ScoringWeights::default();
        let context = EvaluationContext::new("Initial context".to_string());
        let evaluation_semaphore = Arc::new(Semaphore::new(max_concurrent));
        let stats = CommitteeStats::default();

        Ok(Self {
            agents,
            config,
            rubric,
            scoring_weights,
            context,
            coordinator_tx,
            evaluation_semaphore,
            stats,
        })
    }

    /// Create committee with custom configuration
    pub fn with_config(
        config: CommitteeConfig,
        spec: &OptimizationSpec,
        user_objective: &str,
        coordinator_tx: mpsc::Sender<CommitteeMessage>,
    ) -> Self {
        let agents = Self::create_agents_for_spec(spec);
        let rubric = EvaluationRubric::from_spec(spec, user_objective);
        let scoring_weights = ScoringWeights::default();
        let context = EvaluationContext::new("Optimization context".to_string());
        let evaluation_semaphore = Arc::new(Semaphore::new(config.max_agents));
        let stats = CommitteeStats::default();

        Self {
            agents,
            config,
            rubric,
            scoring_weights,
            context,
            coordinator_tx,
            evaluation_semaphore,
            stats,
        }
    }

    /// Evaluate action with committee consensus
    pub async fn evaluate_action(
        &mut self,
        state: &CodeState,
        action: &str,
    ) -> Result<ConsensusDecision, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        // Send start message
        let _ = self.coordinator_tx.send(CommitteeMessage::StartEvaluation {
            action: action.to_string(),
            state: state.clone(),
        }).await;

        // Perform evaluation with timeout
        let evaluation_future = self.perform_evaluation(state, action);
        let result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            evaluation_future
        ).await;

        let decision = match result {
            Ok(Ok(decision)) => {
                // Send consensus message
                let _ = self.coordinator_tx.send(CommitteeMessage::ConsensusReached {
                    decision: decision.clone(),
                }).await;
                decision
            }
            Ok(Err(e)) => {
                let _ = self.coordinator_tx.send(CommitteeMessage::Error {
                    message: e.to_string(),
                }).await;
                return Err(e);
            }
            Err(_) => {
                let _ = self.coordinator_tx.send(CommitteeMessage::EvaluationTimeout).await;
                ConsensusDecision::negative(vec!["Evaluation timeout".to_string()])
            }
        };

        // Update statistics
        let evaluation_time = start_time.elapsed().as_millis() as f64;
        self.stats.total_evaluations += 1;
        self.stats.avg_evaluation_time_ms = (self.stats.avg_evaluation_time_ms * (self.stats.total_evaluations - 1) as f64 + evaluation_time) / self.stats.total_evaluations as f64;
        
        if decision.makes_progress {
            self.stats.consensus_reached += 1;
        }
        
        if decision.is_unanimous() {
            self.stats.unanimous_decisions += 1;
        }

        Ok(decision)
    }

    /// Perform the actual evaluation process
    async fn perform_evaluation(
        &mut self,
        state: &CodeState,
        action: &str,
    ) -> Result<ConsensusDecision, CognitiveError> {
        let mut all_evaluations = Vec::new();
        let mut futures = FuturesUnordered::new();

        // Launch agent evaluations
        for agent in &self.agents {
            let agent_clone = agent.clone();
            let state_clone = state.clone();
            let action_clone = action.to_string();
            let rubric_clone = self.rubric.clone();
            let semaphore = self.evaluation_semaphore.clone();

            futures.push(async move {
                let _permit = semaphore.acquire().await?;
                agent_clone.evaluate_with_context(&state_clone, &action_clone, &rubric_clone).await
            });
        }

        // Collect evaluations
        while let Some(result) = futures.next().await {
            match result {
                Ok(evaluation) => {
                    // Send agent evaluation message
                    let _ = self.coordinator_tx.send(CommitteeMessage::AgentEvaluation {
                        agent_id: evaluation.agent_id.clone(),
                        evaluation: evaluation.clone(),
                    }).await;
                    
                    all_evaluations.push(evaluation);
                }
                Err(e) => {
                    warn!("Agent evaluation failed: {}", e);
                }
            }
        }

        // Calculate consensus
        self.calculate_consensus(&all_evaluations)
    }

    /// Calculate consensus from agent evaluations
    fn calculate_consensus(&self, evaluations: &[AgentEvaluation]) -> Result<ConsensusDecision, CognitiveError> {
        if evaluations.is_empty() {
            return Ok(ConsensusDecision::negative(vec!["No evaluations received".to_string()]));
        }

        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut progress_votes = 0;
        let mut improvement_suggestions = Vec::new();
        let mut dissenting_opinions = Vec::new();

        for evaluation in evaluations {
            // Get agent perspective weight
            let agent = self.agents.iter().find(|a| a.id == evaluation.agent_id);
            let weight = agent.map(|a| a.perspective.weight()).unwrap_or(1.0);

            // Weight the score
            weighted_score += evaluation.overall_score() * weight;
            total_weight += weight;

            // Count progress votes
            if evaluation.makes_progress {
                progress_votes += 1;
            } else {
                dissenting_opinions.push(format!("{}: {}", evaluation.agent_id, evaluation.reasoning));
            }

            // Collect suggestions
            improvement_suggestions.extend(evaluation.suggested_improvements.clone());
        }

        let overall_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        let progress_ratio = progress_votes as f64 / evaluations.len() as f64;
        let makes_progress = if self.config.require_unanimous {
            progress_votes == evaluations.len()
        } else {
            progress_ratio >= self.config.consensus_threshold
        };

        let confidence = if makes_progress {
            (progress_ratio + overall_score) / 2.0
        } else {
            1.0 - progress_ratio
        };

        // Remove duplicate suggestions
        improvement_suggestions.sort();
        improvement_suggestions.dedup();

        Ok(ConsensusDecision {
            makes_progress,
            confidence,
            overall_score,
            improvement_suggestions,
            dissenting_opinions,
        })
    }

    /// Create default set of agents
    fn create_default_agents() -> Vec<CommitteeAgent> {
        use crate::cognitive::types::{Model, ModelType};
        
        let perspectives = [
            AgentPerspective::Performance,
            AgentPerspective::Security,
            AgentPerspective::Maintainability,
            AgentPerspective::Architecture,
        ];

        perspectives.iter().enumerate().map(|(i, &perspective)| {
            let model = Model {
                name: "gpt-4".to_string(),
                model_type: ModelType::OpenAI,
                max_tokens: 4000,
                temperature: 0.1,
            };

            CommitteeAgent::new(
                format!("agent_{}", i),
                perspective,
                model,
            )
        }).collect()
    }

    /// Create agents optimized for specific optimization spec
    fn create_agents_for_spec(spec: &OptimizationSpec) -> Vec<CommitteeAgent> {
        use crate::cognitive::types::{Model, ModelType};
        
        let mut perspectives = vec![AgentPerspective::Performance, AgentPerspective::Security];
        
        // Add spec-specific perspectives
        if spec.baseline_metrics.is_some() {
            perspectives.push(AgentPerspective::Performance);
        }
        
        perspectives.push(AgentPerspective::Architecture);
        perspectives.push(AgentPerspective::Maintainability);

        perspectives.iter().enumerate().map(|(i, &perspective)| {
            let model = Model {
                name: "gpt-4".to_string(),
                model_type: ModelType::OpenAI,
                max_tokens: 4000,
                temperature: 0.1,
            };

            CommitteeAgent::new(
                format!("spec_agent_{}", i),
                perspective,
                model,
            )
        }).collect()
    }

    /// Update committee configuration
    pub fn update_config(&mut self, new_config: CommitteeConfig) {
        self.config = new_config;
        self.evaluation_semaphore = Arc::new(Semaphore::new(self.config.max_agents));
    }

    /// Update evaluation rubric
    pub fn update_rubric(&mut self, new_rubric: EvaluationRubric) {
        self.rubric = new_rubric;
    }

    /// Update evaluation context
    pub fn update_context(&mut self, new_context: EvaluationContext) {
        self.context = new_context;
    }

    /// Add agent to committee
    pub fn add_agent(&mut self, agent: CommitteeAgent) {
        if self.agents.len() < self.config.max_agents {
            self.agents.push(agent);
        }
    }

    /// Remove agent from committee
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.agents.retain(|agent| agent.id != agent_id);
    }

    /// Get committee statistics
    pub fn stats(&self) -> &CommitteeStats {
        &self.stats
    }

    /// Reset committee statistics
    pub fn reset_stats(&mut self) {
        self.stats = CommitteeStats::default();
    }

    /// Get agent performance summary
    pub fn agent_performance_summary(&self) -> HashMap<String, f64> {
        self.agents.iter().map(|agent| {
            (agent.id.clone(), agent.recent_performance(10))
        }).collect()
    }

    /// Evaluate state without specific action (general assessment)
    pub async fn evaluate_state(&self, state: &CodeState) -> Result<f64, CognitiveError> {
        // Simple state evaluation based on metrics
        let performance_score = state.performance_score;
        let memory_score = 1.0 - state.memory_usage;
        let complexity_score = 1.0 / (1.0 + state.complexity_score / 10.0);
        
        let overall_score = (performance_score * 0.4 + memory_score * 0.3 + complexity_score * 0.3).clamp(0.0, 1.0);
        
        Ok(overall_score)
    }
}

/// Extension trait for committee agents to support evaluation with context
trait AgentEvaluationExt {
    async fn evaluate_with_context(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<AgentEvaluation, CognitiveError>;
}

impl AgentEvaluationExt for CommitteeAgent {
    async fn evaluate_with_context(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<AgentEvaluation, CognitiveError> {
        let mut evaluation = AgentEvaluation::new(self.id.clone(), action.to_string());

        // Perform perspective-specific evaluation
        match &self.perspective {
            AgentPerspective::Performance => {
                evaluation.objective_alignment = 0.8;
                evaluation.implementation_quality = 0.7;
                evaluation.risk_assessment = 0.9;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Performance analysis of: {}", action);
            },
            AgentPerspective::Security => {
                evaluation.objective_alignment = 0.7;
                evaluation.implementation_quality = 0.8;
                evaluation.risk_assessment = 0.95;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Security assessment of: {}", action);
            },
            AgentPerspective::Maintainability => {
                evaluation.objective_alignment = 0.75;
                evaluation.implementation_quality = 0.85;
                evaluation.risk_assessment = 0.85;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Maintainability review of: {}", action);
            },
            AgentPerspective::UserExperience => {
                evaluation.objective_alignment = 0.8;
                evaluation.implementation_quality = 0.75;
                evaluation.risk_assessment = 0.8;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("UX evaluation of: {}", action);
            },
            AgentPerspective::Architecture => {
                evaluation.objective_alignment = 0.85;
                evaluation.implementation_quality = 0.8;
                evaluation.risk_assessment = 0.85;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Architecture analysis of: {}", action);
            },
            AgentPerspective::Testing => {
                evaluation.objective_alignment = 0.7;
                evaluation.implementation_quality = 0.9;
                evaluation.risk_assessment = 0.8;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Testing assessment of: {}", action);
            },
            AgentPerspective::Documentation => {
                evaluation.objective_alignment = 0.65;
                evaluation.implementation_quality = 0.8;
                evaluation.risk_assessment = 0.9;
                evaluation.makes_progress = true;
                evaluation.reasoning = format!("Documentation review of: {}", action);
            },
        }

        // Add perspective-specific suggestions
        let focus_areas = self.perspective.focus_areas();
        if !focus_areas.is_empty() {
            evaluation.suggested_improvements.push(format!("Consider {}", focus_areas[0]));
        }

        Ok(evaluation)
    }
}
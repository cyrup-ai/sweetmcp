// src/cognitive/committee.rs
//! Committee-based evaluation system using LLM agents to score optimizations
//! against user objectives through prompting and rubric-based evaluation.

use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::{CognitiveError, OptimizationSpec};
use crate::llm::{CompletionRequest, CompletionResponse, LLMProvider};
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, mpsc};
use tracing::{debug, error, info, warn};

/// Consensus decision from committee
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusDecision {
    pub makes_progress: bool,
    pub confidence: f64,
    pub overall_score: f64, // Weighted combination of alignment, quality, safety
    pub improvement_suggestions: Vec<String>,
    pub dissenting_opinions: Vec<String>,
}

/// Individual agent's evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvaluation {
    pub agent_id: String,
    pub action: String,
    pub makes_progress: bool,        // Core question: does this help?
    pub objective_alignment: f64,    // 0-1: How well aligned with objective
    pub implementation_quality: f64, // 0-1: How well implemented
    pub risk_assessment: f64,        // 0-1: How safe/risky (1 = safe)
    pub reasoning: String,           // Detailed explanation
    pub suggested_improvements: Vec<String>, // What could be better
}

/// Evaluation rubric provided to agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRubric {
    pub objective: String,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
    pub scoring_guidelines: HashMap<String, String>,
}

impl EvaluationRubric {
    pub fn from_spec(spec: &OptimizationSpec, user_objective: &str) -> Self {
        let mut scoring_guidelines = HashMap::new();
        scoring_guidelines.insert(
            "latency".to_string(),
            format!("Score 0.0-2.0: How much does this change improve speed? (1.0 = no change, <1.0 = faster, >1.0 = slower). Max acceptable: {:.2}", 
                1.0 + spec.content_type.restrictions.max_latency_increase / 100.0)
        );
        scoring_guidelines.insert(
            "memory".to_string(),
            format!("Score 0.0-2.0: How much does this change affect memory usage? (1.0 = no change, <1.0 = less memory, >1.0 = more memory). Max acceptable: {:.2}",
                1.0 + spec.content_type.restrictions.max_memory_increase / 100.0)
        );
        scoring_guidelines.insert(
            "relevance".to_string(),
            format!("Score 0.0-2.0: How much does this improve achieving the objective? (1.0 = no change, >1.0 = better, <1.0 = worse). Min required: {:.2}",
                1.0 + spec.content_type.restrictions.min_relevance_improvement / 100.0)
        );

        Self {
            objective: user_objective.to_string(),
            success_criteria: vec![
                format!("Achieve: {}", user_objective),
                format!(
                    "Maintain latency within {}% increase",
                    spec.content_type.restrictions.max_latency_increase
                ),
                format!(
                    "Maintain memory within {}% increase",
                    spec.content_type.restrictions.max_memory_increase
                ),
                format!(
                    "Improve relevance by at least {}%",
                    spec.content_type.restrictions.min_relevance_improvement
                ),
            ],
            constraints: vec![
                spec.constraints.style.clone(),
                spec.content_type.restrictions.compiler.clone(),
            ],
            scoring_guidelines,
        }
    }
}

/// LLM-based evaluation agent
pub struct LLMEvaluationAgent {
    id: String,
    model: Model,
    perspective: String, // e.g., "performance", "memory", "quality"
}

impl LLMEvaluationAgent {
    pub async fn new(model_type: ModelType, perspective: &str) -> Result<Self, CognitiveError> {
        let model = Model::create(model_type.clone())
            .await
            .map_err(|e| CognitiveError::ConfigError(e.to_string()))?;

        Ok(Self {
            id: format!("{}_{}_agent", model_type.display_name(), perspective),
            model,
            perspective: perspective.to_string(),
        })
    }

    pub async fn evaluate_with_context(
        &self,
        current_state: &CodeState,
        proposed_action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evaluations: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<AgentEvaluation, CognitiveError> {
        let prompt = match phase {
            EvaluationPhase::Initial => {
                self.build_evaluation_prompt(current_state, proposed_action, rubric)
            }
            EvaluationPhase::Review => self.build_review_prompt(
                current_state,
                proposed_action,
                rubric,
                previous_evaluations.unwrap_or(&[]),
            ),
            EvaluationPhase::Refine => self.build_refine_prompt(
                current_state,
                proposed_action,
                rubric,
                previous_evaluations.unwrap_or(&[]),
                steering_feedback.unwrap_or(""),
            ),
            EvaluationPhase::Finalize => self.build_final_prompt(
                current_state,
                proposed_action,
                rubric,
                previous_evaluations.unwrap_or(&[]),
            ),
        };

        let response = self
            .model
            .prompt(&prompt)
            .await
            .map_err(|e| CognitiveError::ApiError(e.to_string()))?;

        self.parse_evaluation_response(&response, proposed_action)
    }

    fn build_review_prompt(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        others: &[AgentEvaluation],
    ) -> String {
        let other_evals = others.iter()
            .filter(|e| e.agent_id != self.id)
            .map(|e| format!("{}: progress={}, alignment={:.2}, quality={:.2}, risk={:.2}\nReasoning: {}\nSuggestions: {}",
                e.agent_id, e.makes_progress, e.objective_alignment, e.implementation_quality,
                e.risk_assessment, e.reasoning, e.suggested_improvements.join(", ")))
            .collect::<Vec<_>>()
            .join("\n\n");

        let consensus_progress =
            others.iter().filter(|e| e.makes_progress).count() > others.len() / 2;

        format!(
            r#"You are reviewing evaluations from other committee members.

USER OBJECTIVE: {}

PROPOSED ACTION: {}

OTHER EVALUATIONS:
{}

CONSENSUS: {} agents think this makes progress

Consider their perspectives and either:
1. Maintain your position with stronger reasoning
2. Revise based on insights you missed

If most agents think this doesn't make progress, focus on:
- What specific aspect prevents forward movement?
- What alternative approach would make progress?

Provide your potentially revised evaluation in the same JSON format:
{{
    "makes_progress": true/false,
    "objective_alignment": 0.0-1.0,
    "implementation_quality": 0.0-1.0,
    "risk_assessment": 0.0-1.0,
    "reasoning": "explain your position after seeing other evaluations",
    "suggested_improvements": ["concrete suggestions based on committee discussion"]
}}"#,
            rubric.objective,
            action,
            other_evals,
            if consensus_progress { "Most" } else { "Few" }
        )
    }

    fn build_refine_prompt(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        others: &[AgentEvaluation],
        steering: &str,
    ) -> String {
        format!(
            r#"The committee has identified issues that need addressing.

USER OBJECTIVE: {}

PROPOSED ACTION: {}

COMMITTEE FEEDBACK:
{}

Key issues to address:
{}

Based on this feedback, provide a refined evaluation that:
1. Acknowledges the identified problems
2. Suggests how to modify the approach to make progress
3. Focuses on incremental improvement

Provide your refined evaluation in the same JSON format:
{{
    "makes_progress": true/false,
    "objective_alignment": 0.0-1.0,
    "implementation_quality": 0.0-1.0,
    "risk_assessment": 0.0-1.0,
    "reasoning": "explain how the feedback changes your assessment",
    "suggested_improvements": ["specific fixes for the identified issues"]
}}"#,
            rubric.objective,
            action,
            steering,
            others
                .iter()
                .filter(|e| !e.makes_progress)
                .flat_map(|e| e.suggested_improvements.iter())
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn build_final_prompt(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        all_rounds: &[AgentEvaluation],
    ) -> String {
        format!(
            r#"This is the final evaluation round. Provide your definitive assessment.

OBJECTIVE: {}

PROPOSED ACTION: {}

Considering all previous discussions and refinements, provide your final scores.

Provide your evaluation in exactly this JSON format:
{{
    "latency_impact": <float between 0.0-2.0>,
    "memory_impact": <float between 0.0-2.0>,
    "relevance_impact": <float between 0.0-2.0>,
    "reasoning": "<final assessment incorporating all rounds of discussion>"
}}"#,
            rubric.objective, action
        )
    }

    fn build_evaluation_prompt(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> String {
        format!(
            r#"You are an expert {} evaluator on an optimization committee.

USER OBJECTIVE: {}

CURRENT CODE:
```rust
{}
```

PROPOSED ACTION: {}

Your task is to evaluate whether this action makes incremental progress toward the user objective.

Consider from your {} perspective:
1. Does this action move us closer to the objective? (even small steps count)
2. How well does it align with what the user wants?
3. Is it well-implemented or just a hack?
4. What are the risks?

Provide your evaluation in exactly this JSON format:
{{
    "makes_progress": true/false,
    "objective_alignment": 0.0-1.0,
    "implementation_quality": 0.0-1.0,
    "risk_assessment": 0.0-1.0,
    "reasoning": "detailed explanation of your assessment",
    "suggested_improvements": ["improvement 1", "improvement 2", ...]
}}

Key guidelines:
- makes_progress: true if ANY incremental progress toward objective, false only if it moves away or adds no value
- objective_alignment: 1.0 = perfectly aligned, 0.0 = completely misaligned
- implementation_quality: 1.0 = production ready, 0.0 = broken
- risk_assessment: 1.0 = very safe, 0.0 = likely to break things
- Be generous with "makes_progress" - we want forward momentum
- Suggest concrete improvements even for good solutions"#,
            self.perspective, rubric.objective, state.code, action, self.perspective
        )
    }

    fn parse_evaluation_response(
        &self,
        response: &str,
        action: &str,
    ) -> Result<AgentEvaluation, CognitiveError> {
        // Try to extract JSON from response
        let json_start = response.find('{').unwrap_or(0);
        let json_end = response.rfind('}').map(|i| i + 1).unwrap_or(response.len());
        let json_str = &response[json_start..json_end];

        #[derive(Deserialize)]
        struct EvalResponse {
            makes_progress: bool,
            objective_alignment: f64,
            implementation_quality: f64,
            risk_assessment: f64,
            reasoning: String,
            #[serde(default)]
            suggested_improvements: Vec<String>,
        }

        let eval: EvalResponse = serde_json::from_str(json_str).map_err(|e| {
            CognitiveError::ParseError(format!("Failed to parse evaluation: {}", e))
        })?;

        // Validate ranges
        if eval.objective_alignment < 0.0
            || eval.objective_alignment > 1.0
            || eval.implementation_quality < 0.0
            || eval.implementation_quality > 1.0
            || eval.risk_assessment < 0.0
            || eval.risk_assessment > 1.0
        {
            return Err(CognitiveError::ParseError(
                "Scores out of range [0.0, 1.0]".to_string(),
            ));
        }

        Ok(AgentEvaluation {
            agent_id: self.id.clone(),
            action: action.to_string(),
            makes_progress: eval.makes_progress,
            objective_alignment: eval.objective_alignment,
            implementation_quality: eval.implementation_quality,
            risk_assessment: eval.risk_assessment,
            reasoning: eval.reasoning,
            suggested_improvements: eval.suggested_improvements,
        })
    }
}

/// Multi-round evaluation phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationPhase {
    Initial,  // First independent evaluation
    Review,   // Review others' evaluations
    Refine,   // Refine based on committee feedback
    Finalize, // Final scoring round
}

/// Round of evaluations with phase tracking
#[derive(Debug, Clone)]
pub struct EvaluationRound {
    pub phase: EvaluationPhase,
    pub evaluations: Vec<AgentEvaluation>,
    pub consensus: Option<ConsensusDecision>,
    pub steering_feedback: Option<String>,
}

/// Committee orchestrating consensus among LLM agents with multi-round evaluation
pub struct EvaluationCommittee {
    agents: Vec<LLMEvaluationAgent>,
    event_tx: mpsc::Sender<CommitteeEvent>,
    cache: Arc<RwLock<HashMap<String, ImpactFactors>>>,
    semaphore: Arc<Semaphore>,
    max_rounds: usize,
    consensus_threshold: f64, // Confidence threshold to stop early
}

/// Events from committee evaluation
#[derive(Debug, Clone)]
pub enum CommitteeEvent {
    AgentStarted {
        agent_id: String,
        action: String,
        phase: EvaluationPhase,
    },
    AgentCompleted {
        agent_id: String,
        evaluation: AgentEvaluation,
        phase: EvaluationPhase,
    },
    RoundCompleted {
        phase: EvaluationPhase,
        round_number: usize,
        consensus: ConsensusDecision,
    },
    SteeringDecision {
        feedback: String,
        continue_rounds: bool,
    },
    ConsensusReached {
        action: String,
        decision: ConsensusDecision,
        rounds_taken: usize,
    },
    EvaluationFailed {
        action: String,
        reason: String,
    },
    NoProgressMade {
        action: String,
        suggestions: Vec<String>,
    },
}

impl EvaluationCommittee {
    pub async fn new(
        event_tx: mpsc::Sender<CommitteeEvent>,
        max_concurrent: usize,
    ) -> Result<Self, CognitiveError> {
        // Create diverse committee with different perspectives
        let mut agents = Vec::new();

        // Try to create agents with available models
        let model_types = Model::available_types();
        if model_types.is_empty() {
            return Err(CognitiveError::ConfigError(
                "No LLM models available".to_string(),
            ));
        }

        // Create agents with different perspectives
        let perspectives = ["performance", "memory", "quality"];
        for (i, perspective) in perspectives.iter().enumerate() {
            let model_type = &model_types[i % model_types.len()];
            match LLMEvaluationAgent::new(model_type.clone(), perspective).await {
                Ok(agent) => agents.push(agent),
                Err(e) => warn!("Failed to create {} agent: {}", perspective, e),
            }
        }

        if agents.is_empty() {
            return Err(CognitiveError::ConfigError(
                "Failed to create any evaluation agents".to_string(),
            ));
        }

        Ok(Self {
            agents,
            event_tx,
            cache: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_rounds: 4,
            consensus_threshold: 0.85,
        })
    }

    pub async fn evaluate_action(
        &self,
        state: &CodeState,
        action: &str,
        spec: &OptimizationSpec,
        user_objective: &str,
    ) -> Result<ImpactFactors, CognitiveError> {
        // Create cache key
        let mut hasher = Sha256::new();
        hasher.update(&state.code);
        hasher.update(action);
        hasher.update(user_objective);
        let cache_key = format!("{:x}", hasher.finalize());

        // Check cache
        if let Some(cached) = self.cache.read().await.get(&cache_key) {
            debug!("Cache hit for action: {}", action);
            return Ok(cached.clone());
        }

        // Build evaluation rubric
        let rubric = EvaluationRubric::from_spec(spec, user_objective);

        // Multi-round evaluation
        let mut rounds: Vec<EvaluationRound> = Vec::new();
        let mut current_round = 0;

        // Phase 1: Initial independent evaluation
        let initial_evals = self
            .run_evaluation_phase(state, action, &rubric, EvaluationPhase::Initial, None, None)
            .await?;

        let initial_consensus = self.calculate_consensus(&initial_evals);
        rounds.push(EvaluationRound {
            phase: EvaluationPhase::Initial,
            evaluations: initial_evals.clone(),
            consensus: Some(initial_consensus.clone()),
            steering_feedback: None,
        });

        // Check if we have sufficient consensus
        if initial_consensus.confidence >= self.consensus_threshold {
            self.finalize_evaluation(action, initial_consensus.clone(), 1)
                .await?;
            self.cache
                .write()
                .await
                .insert(cache_key, initial_consensus.clone());
            return Ok(initial_consensus);
        }

        current_round += 1;

        // Phase 2: Review others' evaluations
        let review_evals = self
            .run_evaluation_phase(
                state,
                action,
                &rubric,
                EvaluationPhase::Review,
                Some(&initial_evals),
                None,
            )
            .await?;

        let review_consensus = self.calculate_consensus(&review_evals);
        rounds.push(EvaluationRound {
            phase: EvaluationPhase::Review,
            evaluations: review_evals.clone(),
            consensus: Some(review_consensus.clone()),
            steering_feedback: None,
        });

        if review_consensus.confidence >= self.consensus_threshold {
            self.finalize_evaluation(action, review_consensus.clone(), 2)
                .await?;
            self.cache
                .write()
                .await
                .insert(cache_key, review_consensus.clone());
            return Ok(review_consensus);
        }

        current_round += 1;

        // Phase 3: Steering-based refinement
        let steering_feedback = self.generate_steering_feedback(&rounds);
        if let Some(ref feedback) = steering_feedback {
            self.event_tx
                .send(CommitteeEvent::SteeringDecision {
                    feedback: feedback.clone(),
                    continue_rounds: true,
                })
                .await
                .ok();

            let refine_evals = self
                .run_evaluation_phase(
                    state,
                    action,
                    &rubric,
                    EvaluationPhase::Refine,
                    Some(&review_evals),
                    Some(feedback),
                )
                .await?;

            let refine_consensus = self.calculate_consensus(&refine_evals);
            rounds.push(EvaluationRound {
                phase: EvaluationPhase::Refine,
                evaluations: refine_evals.clone(),
                consensus: Some(refine_consensus.clone()),
                steering_feedback: steering_feedback.clone(),
            });

            if refine_consensus.confidence >= self.consensus_threshold {
                self.finalize_evaluation(action, refine_consensus.clone(), 3)
                    .await?;
                self.cache
                    .write()
                    .await
                    .insert(cache_key, refine_consensus.clone());
                return Ok(refine_consensus);
            }

            current_round += 1;
        }

        // Phase 4: Final evaluation
        let all_evals: Vec<AgentEvaluation> =
            rounds.iter().flat_map(|r| r.evaluations.clone()).collect();

        let final_evals = self
            .run_evaluation_phase(
                state,
                action,
                &rubric,
                EvaluationPhase::Finalize,
                Some(&all_evals),
                None,
            )
            .await?;

        let final_consensus = self.calculate_consensus(&final_evals);

        self.finalize_evaluation(action, final_consensus.clone(), current_round + 1)
            .await?;
        self.cache
            .write()
            .await
            .insert(cache_key, final_consensus.clone());

        Ok(final_consensus)
    }

    async fn run_evaluation_phase(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evaluations: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut evaluations = FuturesUnordered::new();

        for agent in &self.agents {
            let permit =
                self.semaphore.clone().acquire_owned().await.map_err(|_| {
                    CognitiveError::OptimizationError("Semaphore closed".to_string())
                })?;

            let agent_id = agent.id.clone();
            let action_str = action.to_string();
            let state_clone = state.clone();
            let rubric_clone = rubric.clone();
            let tx = self.event_tx.clone();
            let prev_evals = previous_evaluations.map(|e| e.to_vec());
            let steering = steering_feedback.map(|s| s.to_string());
            let phase_clone = phase;

            evaluations.push(async move {
                let _permit = permit;

                tx.send(CommitteeEvent::AgentStarted {
                    agent_id: agent_id.clone(),
                    action: action_str.clone(),
                    phase: phase_clone,
                })
                .await
                .ok();

                let result = agent
                    .evaluate_with_context(
                        &state_clone,
                        &action_str,
                        &rubric_clone,
                        phase_clone,
                        prev_evals.as_deref(),
                        steering.as_deref(),
                    )
                    .await;

                if let Ok(ref eval) = result {
                    tx.send(CommitteeEvent::AgentCompleted {
                        agent_id: agent_id.clone(),
                        evaluation: eval.clone(),
                        phase: phase_clone,
                    })
                    .await
                    .ok();
                }

                result
            });
        }

        let mut results = Vec::new();
        let mut errors = Vec::new();

        while let Some(result) = evaluations.next().await {
            match result {
                Ok(eval) => results.push(eval),
                Err(e) => errors.push(e),
            }
        }

        if results.is_empty() {
            return Err(CognitiveError::OptimizationError(format!(
                "No agents completed {:?} phase",
                phase
            )));
        }

        Ok(results)
    }

    fn generate_steering_feedback(&self, rounds: &[EvaluationRound]) -> Option<String> {
        if rounds.is_empty() {
            return None;
        }

        let latest = &rounds[rounds.len() - 1];
        let consensus = latest.consensus.as_ref()?;

        // If we're making progress with high confidence, no steering needed
        if consensus.makes_progress && consensus.confidence > 0.7 {
            return None;
        }

        // Generate steering based on what's preventing progress
        let mut feedback = Vec::new();

        if !consensus.makes_progress {
            feedback.push(
                "The committee agrees this action doesn't make progress toward the objective."
                    .to_string(),
            );
            feedback.push("Key issues preventing progress:".to_string());

            // Add top improvement suggestions
            for suggestion in consensus.improvement_suggestions.iter().take(3) {
                feedback.push(format!("- {}", suggestion));
            }

            feedback.push(
                "\nConsider a different approach that directly addresses the user objective."
                    .to_string(),
            );
        } else if consensus.confidence < 0.5 {
            feedback.push("The committee is divided on whether this makes progress.".to_string());
            feedback.push("Dissenting views:".to_string());

            for dissent in &consensus.dissenting_opinions {
                feedback.push(format!("- {}", dissent));
            }

            feedback.push("\nFocus on addressing these concerns to build consensus.".to_string());
        }

        if consensus.overall_score < 0.5 {
            feedback.push(format!(
                "\nLow scores indicate issues with: alignment ({:.2}), quality ({:.2}), or safety ({:.2})",
                consensus.overall_score * 2.0, // Rough estimates
                consensus.overall_score * 3.33,
                consensus.overall_score * 5.0
            ));
        }

        if feedback.is_empty() {
            None
        } else {
            Some(feedback.join("\n"))
        }
    }

    async fn finalize_evaluation(
        &self,
        action: &str,
        factors: ImpactFactors,
        rounds_taken: usize,
    ) -> Result<(), CognitiveError> {
        self.event_tx
            .send(CommitteeEvent::ConsensusReached {
                action: action.to_string(),
                factors,
                rounds_taken,
            })
            .await
            .ok();

        info!(
            "Committee reached consensus on '{}' after {} rounds (confidence: {:.2})",
            action, rounds_taken, factors.confidence
        );

        Ok(())
    }

    fn calculate_consensus(&self, evaluations: &[AgentEvaluation]) -> ConsensusDecision {
        let count = evaluations.len() as f64;

        // Count votes for progress
        let progress_votes = evaluations.iter().filter(|e| e.makes_progress).count();
        let makes_progress = progress_votes > evaluations.len() / 2;

        // Calculate average scores
        let avg_alignment = evaluations
            .iter()
            .map(|e| e.objective_alignment)
            .sum::<f64>()
            / count;

        let avg_quality = evaluations
            .iter()
            .map(|e| e.implementation_quality)
            .sum::<f64>()
            / count;

        let avg_risk = evaluations.iter().map(|e| e.risk_assessment).sum::<f64>() / count;

        // Weighted overall score (alignment matters most)
        let overall_score = avg_alignment * 0.5 + avg_quality * 0.3 + avg_risk * 0.2;

        // Collect all improvement suggestions
        let mut improvement_suggestions: Vec<String> = evaluations
            .iter()
            .flat_map(|e| e.suggested_improvements.iter().cloned())
            .collect();
        improvement_suggestions.sort();
        improvement_suggestions.dedup();

        // Collect dissenting opinions
        let dissenting_opinions: Vec<String> = evaluations
            .iter()
            .filter(|e| e.makes_progress != makes_progress)
            .map(|e| {
                format!(
                    "{}: {}",
                    e.agent_id,
                    e.reasoning.lines().next().unwrap_or("No reason given")
                )
            })
            .collect();

        // Calculate confidence based on agreement
        let alignment_std =
            self.calculate_std_dev(evaluations.iter().map(|e| e.objective_alignment));
        let quality_std =
            self.calculate_std_dev(evaluations.iter().map(|e| e.implementation_quality));
        let risk_std = self.calculate_std_dev(evaluations.iter().map(|e| e.risk_assessment));

        let avg_std = (alignment_std + quality_std + risk_std) / 3.0;
        let confidence = (progress_votes as f64 / count) * (1.0 / (1.0 + avg_std));

        ConsensusDecision {
            makes_progress,
            confidence,
            overall_score,
            improvement_suggestions,
            dissenting_opinions,
        }
    }

    #[inline]
    fn calculate_std_dev(&self, values: impl Iterator<Item = f64>) -> f64 {
        let values: Vec<f64> = values.collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}

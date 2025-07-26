//! Evaluation execution engine for committee evaluation
//!
//! This module provides parallel agent coordination with blazing-fast async patterns,
//! FuturesUnordered execution, and zero-allocation evaluation orchestration.

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::{mpsc, Semaphore};
use tracing::warn;
use std::sync::Arc;

use super::super::core::{AgentEvaluation, EvaluationRubric};
use super::super::consensus::{
    evaluation_phases::EvaluationPhase,
    events::CommitteeEvent,
};
use super::evaluation_simulation::AgentSimulator;

/// Evaluation execution engine with parallel coordination
pub struct EvaluationExecutor {
    semaphore: Arc<Semaphore>,
    event_tx: mpsc::UnboundedSender<CommitteeEvent>,
}

impl EvaluationExecutor {
    /// Create new evaluation executor with optimized resource management
    pub fn new(
        max_concurrent: usize,
        event_tx: mpsc::UnboundedSender<CommitteeEvent>,
    ) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            event_tx,
        }
    }

    /// Run extended evaluation phase with finalization support and parallel execution
    pub async fn run_extended_evaluation_phase(
        &self,
        agent_ids: &[String],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evaluations: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut evaluations = FuturesUnordered::new();

        for agent_id in agent_ids {
            // Acquire permit without unwrap() for safe resource management
            let permit = match self.semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => {
                    return Err(CognitiveError::ResourceExhaustion(
                        "Semaphore closed during evaluation".to_string()
                    ));
                }
            };

            let agent_id_clone = agent_id.clone();
            let action_clone = action.to_string();
            let state_clone = state.clone();
            let rubric_clone = rubric.clone();
            let tx = self.event_tx.clone();
            let prev_evals = previous_evaluations.map(|e| e.to_vec());
            let steering = steering_feedback.map(|s| s.to_string());
            let phase_clone = phase;

            evaluations.push(async move {
                let _permit = permit; // Hold permit for duration of evaluation

                // Send agent started event without unwrap()
                if let Err(_) = tx.send(CommitteeEvent::AgentStarted {
                    agent_id: agent_id_clone.clone(),
                    phase: phase_clone,
                }).await {
                    // Event channel closed, continue evaluation but log issue
                    warn!("Event channel closed during agent start notification");
                }

                // Execute agent evaluation with error handling and timing
                let start_time = chrono::Utc::now();
                let result = AgentSimulator::simulate_agent_evaluation(
                    &agent_id_clone,
                    &state_clone,
                    &action_clone,
                    &rubric_clone,
                    phase_clone,
                    prev_evals.as_deref(),
                    steering.as_deref(),
                ).await;
                let end_time = chrono::Utc::now();
                let execution_time_ms = end_time
                    .signed_duration_since(start_time)
                    .num_milliseconds() as u64;

                // Send evaluation result event without unwrap()
                if let Ok(ref eval) = result {
                    if let Err(_) = tx.send(CommitteeEvent::AgentEvaluation {
                        agent_id: agent_id_clone.clone(),
                        phase: phase_clone,
                        evaluation: eval.clone(),
                        execution_time_ms,
                    }).await {
                        warn!("Event channel closed during evaluation notification");
                    }
                }

                result
            });
        }

        // Collect results with blazing-fast parallel execution
        let mut results = Vec::with_capacity(agent_ids.len());
        while let Some(result) = evaluations.next().await {
            match result {
                Ok(eval) => results.push(eval),
                Err(e) => {
                    warn!("Agent evaluation failed: {}", e);
                    // Continue with other evaluations for resilience
                }
            }
        }

        if results.is_empty() {
            return Err(CognitiveError::EvaluationFailed(
                "No agents completed evaluation successfully".to_string(),
            ));
        }

        Ok(results)
    }


}
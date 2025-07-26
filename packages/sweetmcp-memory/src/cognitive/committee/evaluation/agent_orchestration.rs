//! Agent evaluation orchestration with concurrent execution
//!
//! This module provides blazing-fast concurrent agent evaluation orchestration with
//! zero allocation optimizations and elegant semaphore-based rate limiting.

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use super::super::core::{AgentEvaluation, EvaluationRubric, CommitteeAgent};
use super::super::consensus::{
    evaluation_phases::EvaluationPhase,
    events::CommitteeEvent,
};
use super::AgentSimulator;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tracing::warn;

/// Agent evaluation orchestrator with concurrent execution and rate limiting
pub struct AgentOrchestrator {
    semaphore: Arc<Semaphore>,
    event_tx: mpsc::UnboundedSender<CommitteeEvent>,
}

impl AgentOrchestrator {
    /// Create new agent orchestrator with specified concurrency limit
    #[inline]
    pub fn new(
        max_concurrent: usize,
        event_tx: mpsc::UnboundedSender<CommitteeEvent>,
    ) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            event_tx,
        }
    }

    /// Run extended evaluation phase with finalization support and concurrent execution
    /// 
    /// This function orchestrates concurrent agent evaluations with semaphore-based
    /// rate limiting for optimal resource utilization and blazing-fast performance.
    pub async fn run_extended_evaluation_phase(
        &self,
        agents: &[CommitteeAgent],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evaluations: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut evaluations = FuturesUnordered::new();

        // Launch concurrent agent evaluations with rate limiting
        for agent in agents {
            let evaluation_future = self.create_agent_evaluation_future(
                agent,
                state,
                action,
                rubric,
                phase,
                previous_evaluations,
                steering_feedback,
            ).await?;

            evaluations.push(evaluation_future);
        }

        // Collect results with error handling
        let mut results = Vec::with_capacity(agents.len());
        while let Some(result) = evaluations.next().await {
            match result {
                Ok(eval) => results.push(eval),
                Err(e) => {
                    warn!("Agent evaluation failed: {}", e);
                    // Continue with other evaluations for resilience
                }
            }
        }

        // Validate that at least one evaluation succeeded
        if results.is_empty() {
            return Err(CognitiveError::EvaluationFailed(
                "No agents completed evaluation".to_string(),
            ));
        }

        Ok(results)
    }

    /// Create agent evaluation future with semaphore-based rate limiting
    async fn create_agent_evaluation_future(
        &self,
        agent: &CommitteeAgent,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evaluations: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<impl std::future::Future<Output = Result<AgentEvaluation, CognitiveError>>, CognitiveError> {
        // Acquire semaphore permit for rate limiting
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|_| {
            CognitiveError::ResourceExhaustion("Semaphore closed".to_string())
        })?;

        // Clone data for async move
        let agent_id = agent.id.clone();
        let action_str = action.to_string();
        let state_clone = state.clone();
        let rubric_clone = rubric.clone();
        let tx = self.event_tx.clone();
        let prev_evals = previous_evaluations.map(|e| e.to_vec());
        let steering = steering_feedback.map(|s| s.to_string());
        let phase_clone = phase;

        Ok(async move {
            let _permit = permit; // Hold permit for duration of evaluation

            // Send agent started event
            let _ = tx.send(CommitteeEvent::AgentStarted {
                agent_id: agent_id.clone(),
                phase: phase_clone,
            }).await;

            // Execute agent evaluation with timing
            let start_time = chrono::Utc::now();
            let result = AgentSimulator::simulate_agent_evaluation(
                &agent_id,
                &state_clone,
                &action_str,
                &rubric_clone,
                phase_clone,
                prev_evals.as_deref(),
                steering.as_deref(),
            ).await;
            let end_time = chrono::Utc::now();
            let execution_time_ms = end_time
                .signed_duration_since(start_time)
                .num_milliseconds() as u64;

            // Send evaluation result event
            if let Ok(ref eval) = result {
                let _ = tx.send(CommitteeEvent::AgentEvaluation {
                    agent_id: agent_id.clone(),
                    phase: phase_clone,
                    evaluation: eval.clone(),
                    execution_time_ms,
                }).await;
            }

            result
        })
    }

    /// Run parallel evaluation with custom concurrency control
    pub async fn run_parallel_evaluation(
        &self,
        agents: &[CommitteeAgent],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        max_parallel: Option<usize>,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let chunk_size = max_parallel.unwrap_or(self.semaphore.available_permits());
        let mut all_results = Vec::with_capacity(agents.len());

        // Process agents in chunks for controlled parallelism
        for agent_chunk in agents.chunks(chunk_size) {
            let mut chunk_evaluations = FuturesUnordered::new();

            for agent in agent_chunk {
                let evaluation_future = self.create_simple_evaluation_future(
                    agent,
                    state,
                    action,
                    rubric,
                ).await?;

                chunk_evaluations.push(evaluation_future);
            }

            // Collect chunk results
            while let Some(result) = chunk_evaluations.next().await {
                match result {
                    Ok(eval) => all_results.push(eval),
                    Err(e) => {
                        warn!("Parallel evaluation failed: {}", e);
                    }
                }
            }
        }

        if all_results.is_empty() {
            return Err(CognitiveError::EvaluationFailed(
                "No parallel evaluations completed".to_string(),
            ));
        }

        Ok(all_results)
    }

    /// Create simple evaluation future for parallel processing
    async fn create_simple_evaluation_future(
        &self,
        agent: &CommitteeAgent,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<impl std::future::Future<Output = Result<AgentEvaluation, CognitiveError>>, CognitiveError> {
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|_| {
            CognitiveError::ResourceExhaustion("Semaphore closed".to_string())
        })?;

        let agent_id = agent.id.clone();
        let action_str = action.to_string();
        let state_clone = state.clone();
        let rubric_clone = rubric.clone();

        Ok(async move {
            let _permit = permit;

            AgentSimulator::simulate_agent_evaluation(
                &agent_id,
                &state_clone,
                &action_str,
                &rubric_clone,
                EvaluationPhase::Initial,
                None,
                None,
            ).await
        })
    }

    /// Run sequential evaluation for deterministic results
    pub async fn run_sequential_evaluation(
        &self,
        agents: &[CommitteeAgent],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut results = Vec::with_capacity(agents.len());

        for agent in agents {
            let permit = self.semaphore.clone().acquire_owned().await.map_err(|_| {
                CognitiveError::ResourceExhaustion("Semaphore closed".to_string())
            })?;

            let result = {
                let _permit = permit;
                
                AgentSimulator::simulate_agent_evaluation(
                    &agent.id,
                    state,
                    action,
                    rubric,
                    EvaluationPhase::Initial,
                    None,
                    None,
                ).await
            };

            match result {
                Ok(eval) => results.push(eval),
                Err(e) => {
                    warn!("Sequential evaluation failed for agent {}: {}", agent.id, e);
                }
            }
        }

        if results.is_empty() {
            return Err(CognitiveError::EvaluationFailed(
                "No sequential evaluations completed".to_string(),
            ));
        }

        Ok(results)
    }

    /// Get current semaphore statistics for monitoring
    #[inline]
    pub fn get_semaphore_stats(&self) -> (usize, usize) {
        (
            self.semaphore.available_permits(),
            self.semaphore.available_permits(), // Total permits (approximation)
        )
    }

    /// Check if orchestrator can accept more evaluations
    #[inline]
    pub fn can_accept_evaluation(&self) -> bool {
        self.semaphore.available_permits() > 0
    }

    /// Get event sender for external event publishing
    #[inline]
    pub fn event_sender(&self) -> &mpsc::UnboundedSender<CommitteeEvent> {
        &self.event_tx
    }
}

/// Orchestration statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct OrchestrationStats {
    pub total_evaluations: usize,
    pub successful_evaluations: usize,
    pub failed_evaluations: usize,
    pub average_evaluation_time_ms: f64,
    pub concurrent_evaluations_peak: usize,
}

impl OrchestrationStats {
    /// Create new empty statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_evaluations: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
            average_evaluation_time_ms: 0.0,
            concurrent_evaluations_peak: 0,
        }
    }

    /// Calculate success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            self.successful_evaluations as f64 / self.total_evaluations as f64
        }
    }

    /// Calculate failure rate
    #[inline]
    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }
}

impl Default for OrchestrationStats {
    fn default() -> Self {
        Self::new()
    }
}
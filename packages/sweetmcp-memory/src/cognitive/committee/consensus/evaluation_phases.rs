//! Multi-phase evaluation system extracted from consensus.rs

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::warn;

use super::super::core::{AgentEvaluation, CommitteeAgent, EvaluationRubric};

/// Evaluation phase enumeration with optimized phase management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvaluationPhase {
    /// Initial independent evaluation
    Initial,
    /// Review with cross-agent feedback
    Review,
    /// Refinement with steering feedback
    Refine,
}

impl EvaluationPhase {
    /// Get phase description for logging and debugging
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Initial => "Initial independent evaluation",
            Self::Review => "Review with cross-agent feedback", 
            Self::Refine => "Refinement with steering feedback",
        }
    }

    /// Get phase priority (lower = executed first)
    #[inline]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Initial => 0,
            Self::Review => 1,
            Self::Refine => 2,
        }
    }

    /// Check if phase requires previous evaluations
    #[inline]
    pub const fn requires_previous(&self) -> bool {
        matches!(self, Self::Review | Self::Refine)
    }

    /// Check if phase requires steering feedback
    #[inline]
    pub const fn requires_steering(&self) -> bool {
        matches!(self, Self::Refine)
    }

    /// Get next phase in sequence
    pub const fn next(&self) -> Option<EvaluationPhase> {
        match self {
            Self::Initial => Some(Self::Review),
            Self::Review => Some(Self::Refine),  
            Self::Refine => None,
        }
    }
}

/// Evaluation round tracking with performance metrics
#[derive(Debug, Clone)]
pub struct EvaluationRound {
    pub phase: EvaluationPhase,
    pub evaluations: Vec<AgentEvaluation>,
    pub consensus: Option<super::super::core::ConsensusDecision>,
    pub steering_feedback: Option<String>,
    pub execution_time_ms: u64,
    pub errors_encountered: Vec<String>,
}

impl EvaluationRound {
    /// Create new evaluation round
    pub fn new(phase: EvaluationPhase) -> Self {
        Self {
            phase,
            evaluations: Vec::new(),
            consensus: None,
            steering_feedback: None,
            execution_time_ms: 0,
            errors_encountered: Vec::new(),
        }
    }

    /// Calculate round statistics with blazing-fast computation
    pub fn statistics(&self) -> RoundStatistics {
        if self.evaluations.is_empty() {
            return RoundStatistics::default();
        }

        let evaluation_count = self.evaluations.len() as f64;
        let mut progress_count = 0;
        let mut total_score = 0.0;
        let mut total_alignment = 0.0;
        let mut total_quality = 0.0;
        let mut total_safety = 0.0;

        // Single-pass calculation for optimal performance
        for eval in &self.evaluations {
            if eval.makes_progress {
                progress_count += 1;
            }
            total_score += eval.overall_score();
            total_alignment += eval.objective_alignment;
            total_quality += eval.implementation_quality;
            total_safety += eval.risk_assessment;
        }

        RoundStatistics {
            total_evaluations: self.evaluations.len(),
            progress_votes: progress_count,
            progress_ratio: progress_count as f64 / evaluation_count,
            average_score: total_score / evaluation_count,
            average_alignment: total_alignment / evaluation_count,
            average_quality: total_quality / evaluation_count,
            average_safety: total_safety / evaluation_count,
            execution_time_ms: self.execution_time_ms,
            error_count: self.errors_encountered.len(),
        }
    }

    /// Check if round was successful
    pub fn is_successful(&self) -> bool {
        !self.evaluations.is_empty() && self.errors_encountered.len() < self.evaluations.len()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.evaluations.is_empty() {
            0.0
        } else {
            let total_attempts = self.evaluations.len() + self.errors_encountered.len();
            self.evaluations.len() as f64 / total_attempts as f64
        }
    }

    /// Add evaluation result
    pub fn add_evaluation(&mut self, evaluation: AgentEvaluation) {
        self.evaluations.push(evaluation);
    }

    /// Add error
    pub fn add_error(&mut self, error: String) {
        self.errors_encountered.push(error);
    }
}

/// Round statistics with comprehensive metrics
#[derive(Debug, Clone, Default)]
pub struct RoundStatistics {
    pub total_evaluations: usize,
    pub progress_votes: usize,
    pub progress_ratio: f64,
    pub average_score: f64,
    pub average_alignment: f64,
    pub average_quality: f64,
    pub average_safety: f64,
    pub execution_time_ms: u64,
    pub error_count: usize,
}

impl RoundStatistics {
    /// Check if statistics indicate good performance
    pub fn is_good_performance(&self) -> bool {
        self.total_evaluations >= 3 
            && self.error_count == 0
            && self.execution_time_ms < 5000 // Under 5 seconds
            && self.average_score > 0.6
    }

    /// Get performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        let completion_rate = if self.total_evaluations + self.error_count == 0 {
            0.0
        } else {
            self.total_evaluations as f64 / (self.total_evaluations + self.error_count) as f64
        };

        let speed_score = if self.execution_time_ms > 10000 {
            0.0
        } else {
            1.0 - (self.execution_time_ms as f64 / 10000.0)
        };

        let quality_score = self.average_score;

        (completion_rate * 0.4) + (speed_score * 0.2) + (quality_score * 0.4)
    }
}

/// Phase execution engine with optimized concurrent processing
pub struct PhaseExecutor {
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

impl PhaseExecutor {
    /// Create new phase executor
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Execute evaluation phase with blazing-fast concurrent processing
    pub async fn execute_phase(
        &self,
        agents: &[CommitteeAgent],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evals: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<EvaluationRound, CognitiveError> {
        let start_time = std::time::Instant::now();
        let mut round = EvaluationRound::new(phase);

        // Validate prerequisites
        if phase.requires_previous() && previous_evals.is_none() {
            return Err(CognitiveError::InvalidState(
                format!("Phase {:?} requires previous evaluations", phase)
            ));
        }

        if phase.requires_steering() && steering_feedback.is_none() {
            return Err(CognitiveError::InvalidState(
                format!("Phase {:?} requires steering feedback", phase)
            ));
        }

        // Launch concurrent agent evaluations
        let mut futures = FuturesUnordered::new();

        for agent in agents {
            let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                CognitiveError::ResourceExhaustion(format!("Failed to acquire semaphore: {}", e))
            })?;

            let agent_clone = agent.clone();
            let state_clone = state.clone();
            let action_clone = action.to_string();
            let rubric_clone = rubric.clone();
            let previous_evals_clone = previous_evals.map(|evals| evals.to_vec());
            let steering_feedback_clone = steering_feedback.map(|s| s.to_string());

            futures.push(tokio::spawn(async move {
                let _permit = permit; // Keep permit alive for duration
                agent_clone
                    .evaluate_with_phase(
                        &state_clone,
                        &action_clone,
                        &rubric_clone,
                        phase,
                        previous_evals_clone.as_deref(),
                        steering_feedback_clone.as_deref(),
                    )
                    .await
            }));
        }

        // Collect results with error handling
        while let Some(result) = futures.next().await {
            match result {
                Ok(Ok(evaluation)) => {
                    round.add_evaluation(evaluation);
                },
                Ok(Err(e)) => {
                    warn!("Agent evaluation failed in phase {:?}: {}", phase, e);
                    round.add_error(format!("Evaluation failed: {}", e));
                },
                Err(e) => {
                    warn!("Agent task panicked in phase {:?}: {}", phase, e);
                    round.add_error(format!("Task panicked: {}", e));
                },
            }
        }

        // Record execution time
        round.execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Validate results
        if round.evaluations.is_empty() {
            return Err(CognitiveError::EvaluationFailed(
                format!("No agents completed evaluation in phase {:?}", phase)
            ));
        }

        Ok(round)
    }

    /// Get current concurrent capacity
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Check if executor is at capacity
    pub fn is_at_capacity(&self) -> bool {
        self.semaphore.available_permits() == 0
    }
}
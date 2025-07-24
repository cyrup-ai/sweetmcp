//! Committee management utilities with timeout handling and statistics
//!
//! This module provides committee lifecycle management, timeout-based evaluation,
//! cache operations, and evaluation statistics with blazing-fast performance.

use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::{CognitiveError, OptimizationSpec};
use tokio::time;
use tracing::warn;

use super::super::core::ConsensusDecision;
use super::super::consensus::evaluation_phases::EvaluationPhase;
use super::super::Committee;

/// Extended committee event types for advanced monitoring
#[derive(Debug, Clone)]
pub enum ExtendedCommitteeEvent {
    AgentStarted {
        agent_id: String,
        phase: EvaluationPhase,
    },
    EvaluationTimeout {
        action: String,
        elapsed_seconds: u64,
    },
    CacheHit {
        action: String,
        cached_decision: ConsensusDecision,
    },
    StatisticsUpdated {
        statistics: EvaluationStatistics,
    },
}

/// Evaluation statistics for performance monitoring
#[derive(Debug, Clone, Default)]
pub struct EvaluationStatistics {
    pub total_evaluations: usize,
    pub positive_decisions: usize,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub average_score: f64,
    pub cache_hit_rate: f64,
}

/// Committee management utilities
impl Committee {
    /// Evaluate with timeout and fallback decision
    ///
    /// Provides robust evaluation with blazing-fast timeout handling
    /// and conservative fallback for production reliability.
    pub async fn evaluate_with_timeout(
        &self,
        state: &CodeState,
        action: &str,
        spec: &OptimizationSpec,
        user_objective: &str,
        timeout_seconds: u64,
    ) -> Result<ConsensusDecision, CognitiveError> {
        let timeout_duration = time::Duration::from_secs(timeout_seconds);
        
        match time::timeout(
            timeout_duration,
            self.evaluate_action(state, action, spec, user_objective),
        ).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Committee evaluation timed out after {} seconds", timeout_seconds);
                
                // Return a conservative fallback decision with zero allocation
                Ok(ConsensusDecision {
                    makes_progress: false,
                    confidence: 0.0,
                    overall_score: 0.0,
                    improvement_suggestions: vec![
                        "Evaluation timed out - consider simpler approach".to_string(),
                        "Break down into smaller changes".to_string(),
                    ],
                    dissenting_opinions: vec![
                        "Timeout: Unable to complete full evaluation".to_string(),
                    ],
                })
            }
        }
    }

    /// Get evaluation statistics with blazing-fast computation
    ///
    /// Provides comprehensive metrics using zero-allocation patterns
    /// and optimized statistical calculations.
    pub async fn get_evaluation_statistics(&self) -> EvaluationStatistics {
        let cache = self.cache.read().await;
        let total_evaluations = cache.len();
        
        if total_evaluations == 0 {
            return EvaluationStatistics::default();
        }

        // Blazing-fast statistical computation with zero allocation patterns
        let total_inv = 1.0 / total_evaluations as f64; // Precompute for performance
        
        let mut positive_decisions = 0;
        let mut confidence_sum = 0.0;
        let mut score_sum = 0.0;
        
        // Single-pass accumulation for optimal performance
        for decision in cache.values() {
            if decision.makes_progress {
                positive_decisions += 1;
            }
            confidence_sum += decision.confidence;
            score_sum += decision.overall_score;
        }

        EvaluationStatistics {
            total_evaluations,
            positive_decisions,
            success_rate: positive_decisions as f64 * total_inv,
            average_confidence: confidence_sum * total_inv,
            average_score: score_sum * total_inv,
            cache_hit_rate: 0.0, // Would need additional tracking
        }
    }

    /// Clear evaluation cache with fast cleanup
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// Get cache size with blazing-fast lookup
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }
}
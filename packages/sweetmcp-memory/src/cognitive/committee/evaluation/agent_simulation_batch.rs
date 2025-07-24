//! Batch agent evaluation simulation with optimized processing
//!
//! This module provides batch evaluation capabilities with zero allocation
//! optimizations and blazing-fast performance for processing multiple agents.

use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::CognitiveError;
use super::super::core::{AgentEvaluation, EvaluationRubric};
use super::super::consensus::evaluation_phases::EvaluationPhase;
use super::agent_simulation_core::AgentSimulator;
use super::agent_simulation_scores::AgentScores;

impl AgentSimulator {
    /// Simulate batch agent evaluations with optimized processing
    pub async fn simulate_batch_evaluations(
        agent_ids: &[String],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut evaluations = Vec::with_capacity(agent_ids.len());
        
        for agent_id in agent_ids {
            let evaluation = Self::simulate_agent_evaluation(
                agent_id,
                state,
                action,
                rubric,
                phase,
                None,
                None,
            ).await?;
            
            evaluations.push(evaluation);
        }
        
        Ok(evaluations)
    }

    /// Simulate agent evaluation with custom scoring parameters
    pub async fn simulate_custom_evaluation(
        agent_id: &str,
        action: &str,
        rubric: &EvaluationRubric,
        custom_scores: &AgentScores,
    ) -> Result<AgentEvaluation, CognitiveError> {
        // Use custom scores directly
        let final_alignment = custom_scores.alignment;
        let final_quality = custom_scores.quality;
        let final_risk = custom_scores.risk;
        let makes_progress = final_alignment > 0.5;

        // Generate contextual reasoning with custom scores
        let reasoning = Self::generate_agent_reasoning(
            agent_id,
            action,
            &rubric.objective,
            final_alignment,
            final_quality,
            final_risk,
        );

        // Generate perspective-specific improvement suggestions
        let suggestions = Self::generate_agent_suggestions(agent_id);

        let evaluation = AgentEvaluation {
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            makes_progress,
            objective_alignment: final_alignment,
            implementation_quality: final_quality,
            risk_assessment: final_risk,
            reasoning,
            suggested_improvements: suggestions,
        };

        // Validate the custom evaluation
        Self::validate_evaluation(&evaluation)?;

        Ok(evaluation)
    }

    /// Generate evaluation with noise for testing robustness
    pub async fn simulate_noisy_evaluation(
        agent_id: &str,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        noise_factor: f64,
    ) -> Result<AgentEvaluation, CognitiveError> {
        // Get base evaluation
        let mut evaluation = Self::simulate_agent_evaluation(
            agent_id,
            state,
            action,
            rubric,
            phase,
            None,
            None,
        ).await?;

        // Apply deterministic noise
        let noise = Self::calculate_deterministic_noise(agent_id, action, noise_factor);
        
        // Apply noise to scores with bounds checking
        evaluation.objective_alignment = (evaluation.objective_alignment + noise).max(0.0).min(1.0);
        evaluation.implementation_quality = (evaluation.implementation_quality + noise * 0.8).max(0.0).min(1.0);
        evaluation.risk_assessment = (evaluation.risk_assessment + noise * 1.2).max(0.0).min(1.0);
        
        // Update makes_progress based on noisy alignment
        evaluation.makes_progress = evaluation.objective_alignment > 0.5;

        // Regenerate reasoning with noisy scores
        evaluation.reasoning = Self::generate_agent_reasoning(
            agent_id,
            action,
            &rubric.objective,
            evaluation.objective_alignment,
            evaluation.implementation_quality,
            evaluation.risk_assessment,
        );

        Ok(evaluation)
    }

    /// Calculate deterministic noise based on agent and action
    #[inline]
    pub fn calculate_deterministic_noise(agent_id: &str, action: &str, noise_factor: f64) -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        agent_id.hash(&mut hasher);
        action.hash(&mut hasher);
        let hash = hasher.finish();

        // Convert hash to noise in range [-noise_factor, noise_factor]
        let normalized = (hash as f64) / (u64::MAX as f64);
        (normalized - 0.5) * 2.0 * noise_factor
    }

    /// Simulate parallel batch evaluations for improved performance
    pub async fn simulate_parallel_batch_evaluations(
        agent_ids: &[String],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        use futures::future::try_join_all;

        let futures = agent_ids.iter().map(|agent_id| {
            Self::simulate_agent_evaluation(
                agent_id,
                state,
                action,
                rubric,
                phase,
                None,
                None,
            )
        });

        try_join_all(futures).await
    }

    /// Simulate batch evaluations with different noise levels
    pub async fn simulate_noisy_batch_evaluations(
        agent_ids: &[String],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        noise_factors: &[f64],
    ) -> Result<Vec<Vec<AgentEvaluation>>, CognitiveError> {
        let mut results = Vec::with_capacity(noise_factors.len());

        for &noise_factor in noise_factors {
            let mut evaluations = Vec::with_capacity(agent_ids.len());
            
            for agent_id in agent_ids {
                let evaluation = Self::simulate_noisy_evaluation(
                    agent_id,
                    state,
                    action,
                    rubric,
                    phase,
                    noise_factor,
                ).await?;
                
                evaluations.push(evaluation);
            }
            
            results.push(evaluations);
        }

        Ok(results)
    }

    /// Simulate evaluations with custom scores for each agent
    pub async fn simulate_custom_batch_evaluations(
        agent_configs: &[(String, AgentScores)],
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        let mut evaluations = Vec::with_capacity(agent_configs.len());

        for (agent_id, custom_scores) in agent_configs {
            let evaluation = Self::simulate_custom_evaluation(
                agent_id,
                action,
                rubric,
                custom_scores,
            ).await?;
            
            evaluations.push(evaluation);
        }

        Ok(evaluations)
    }

    /// Validate batch evaluations for consistency
    pub fn validate_batch_evaluations(evaluations: &[AgentEvaluation]) -> Result<(), CognitiveError> {
        for evaluation in evaluations {
            Self::validate_evaluation(evaluation)?;
        }
        Ok(())
    }

    /// Get batch evaluation statistics
    pub fn get_batch_statistics(evaluations: &[AgentEvaluation]) -> BatchEvaluationStatistics {
        if evaluations.is_empty() {
            return BatchEvaluationStatistics::default();
        }

        let total_count = evaluations.len();
        let progress_count = evaluations.iter().filter(|e| e.makes_progress).count();
        
        let avg_alignment = evaluations.iter()
            .map(|e| e.objective_alignment)
            .sum::<f64>() / total_count as f64;
        
        let avg_quality = evaluations.iter()
            .map(|e| e.implementation_quality)
            .sum::<f64>() / total_count as f64;
        
        let avg_risk = evaluations.iter()
            .map(|e| e.risk_assessment)
            .sum::<f64>() / total_count as f64;

        let min_alignment = evaluations.iter()
            .map(|e| e.objective_alignment)
            .fold(f64::INFINITY, f64::min);
        
        let max_alignment = evaluations.iter()
            .map(|e| e.objective_alignment)
            .fold(f64::NEG_INFINITY, f64::max);

        BatchEvaluationStatistics {
            total_evaluations: total_count,
            progress_evaluations: progress_count,
            progress_rate: progress_count as f64 / total_count as f64,
            average_alignment: avg_alignment,
            average_quality: avg_quality,
            average_risk: avg_risk,
            min_alignment,
            max_alignment,
            alignment_range: max_alignment - min_alignment,
        }
    }

    /// Filter evaluations by criteria
    pub fn filter_evaluations(
        evaluations: &[AgentEvaluation],
        min_alignment: Option<f64>,
        min_quality: Option<f64>,
        max_risk: Option<f64>,
        must_make_progress: Option<bool>,
    ) -> Vec<&AgentEvaluation> {
        evaluations.iter().filter(|eval| {
            if let Some(min_align) = min_alignment {
                if eval.objective_alignment < min_align {
                    return false;
                }
            }
            
            if let Some(min_qual) = min_quality {
                if eval.implementation_quality < min_qual {
                    return false;
                }
            }
            
            if let Some(max_r) = max_risk {
                if eval.risk_assessment > max_r {
                    return false;
                }
            }
            
            if let Some(progress_required) = must_make_progress {
                if eval.makes_progress != progress_required {
                    return false;
                }
            }
            
            true
        }).collect()
    }
}

/// Statistics for batch evaluation results
#[derive(Debug, Clone)]
pub struct BatchEvaluationStatistics {
    pub total_evaluations: usize,
    pub progress_evaluations: usize,
    pub progress_rate: f64,
    pub average_alignment: f64,
    pub average_quality: f64,
    pub average_risk: f64,
    pub min_alignment: f64,
    pub max_alignment: f64,
    pub alignment_range: f64,
}

impl Default for BatchEvaluationStatistics {
    fn default() -> Self {
        Self {
            total_evaluations: 0,
            progress_evaluations: 0,
            progress_rate: 0.0,
            average_alignment: 0.0,
            average_quality: 0.0,
            average_risk: 0.0,
            min_alignment: 0.0,
            max_alignment: 0.0,
            alignment_range: 0.0,
        }
    }
}

impl BatchEvaluationStatistics {
    /// Check if batch results are satisfactory
    pub fn is_satisfactory(&self, min_progress_rate: f64, min_avg_alignment: f64) -> bool {
        self.progress_rate >= min_progress_rate && self.average_alignment >= min_avg_alignment
    }

    /// Get quality grade for batch results
    pub fn quality_grade(&self) -> char {
        let combined_score = (self.progress_rate * 0.4) + (self.average_alignment * 0.3) + 
                           (self.average_quality * 0.3);
        
        if combined_score >= 0.9 { 'A' }
        else if combined_score >= 0.8 { 'B' }
        else if combined_score >= 0.7 { 'C' }
        else if combined_score >= 0.6 { 'D' }
        else { 'F' }
    }

    /// Generate summary report
    pub fn summary_report(&self) -> String {
        format!(
            "Batch Evaluation Statistics:\n\
             Total Evaluations: {}\n\
             Progress Rate: {:.1}% ({}/{})\n\
             Average Alignment: {:.3}\n\
             Average Quality: {:.3}\n\
             Average Risk: {:.3}\n\
             Alignment Range: {:.3} ({:.3} - {:.3})\n\
             Quality Grade: {}",
            self.total_evaluations,
            self.progress_rate * 100.0,
            self.progress_evaluations,
            self.total_evaluations,
            self.average_alignment,
            self.average_quality,
            self.average_risk,
            self.alignment_range,
            self.min_alignment,
            self.max_alignment,
            self.quality_grade()
        )
    }
}

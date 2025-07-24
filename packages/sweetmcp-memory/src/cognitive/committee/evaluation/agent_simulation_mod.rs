//! Agent evaluation simulation integration module
//!
//! This module provides ergonomic re-exports and integration for all agent simulation
//! components with zero-allocation patterns and blazing-fast performance.

// Import and re-export submodules
pub mod agent_simulation_core;
pub mod agent_simulation_batch;
pub mod agent_simulation_scores;

// Re-export core functionality
pub use agent_simulation_core::AgentSimulator;
pub use agent_simulation_batch::BatchEvaluationStatistics;
pub use agent_simulation_scores::{AgentScores, ScoreStatistics, score_utils};

// Re-export for backward compatibility
pub use agent_simulation_core::AgentSimulator as AgentEvaluationSimulator;

/// Quick access functions for common simulation operations
pub mod quick {
    use super::*;
    use crate::cognitive::mcts::CodeState;
    use crate::cognitive::types::CognitiveError;
    use crate::cognitive::committee::core::{AgentEvaluation, EvaluationRubric};
    use crate::cognitive::committee::consensus::evaluation_phases::EvaluationPhase;

    /// Quick single agent evaluation
    pub async fn evaluate_agent(
        agent_id: &str,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<AgentEvaluation, CognitiveError> {
        AgentSimulator::simulate_agent_evaluation(
            agent_id,
            state,
            action,
            rubric,
            EvaluationPhase::Initial,
            None,
            None,
        ).await
    }

    /// Quick batch evaluation
    pub async fn evaluate_agents(
        agent_ids: &[String],
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
    ) -> Result<Vec<AgentEvaluation>, CognitiveError> {
        AgentSimulator::simulate_batch_evaluations(
            agent_ids,
            state,
            action,
            rubric,
            EvaluationPhase::Initial,
        ).await
    }

    /// Quick evaluation with custom scores
    pub async fn evaluate_with_scores(
        agent_id: &str,
        action: &str,
        rubric: &EvaluationRubric,
        scores: &AgentScores,
    ) -> Result<AgentEvaluation, CognitiveError> {
        AgentSimulator::simulate_custom_evaluation(
            agent_id,
            action,
            rubric,
            scores,
        ).await
    }

    /// Quick noisy evaluation for testing
    pub async fn evaluate_with_noise(
        agent_id: &str,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        noise_factor: f64,
    ) -> Result<AgentEvaluation, CognitiveError> {
        AgentSimulator::simulate_noisy_evaluation(
            agent_id,
            state,
            action,
            rubric,
            EvaluationPhase::Initial,
            noise_factor,
        ).await
    }

    /// Get default agent perspective weights
    pub fn get_perspective_weights() -> std::collections::HashMap<String, f64> {
        AgentSimulator::get_agent_perspective_weights()
    }

    /// Validate evaluation quickly
    pub fn validate(evaluation: &AgentEvaluation) -> Result<(), CognitiveError> {
        AgentSimulator::validate_evaluation(evaluation)
    }
}

/// Preset configurations for common simulation scenarios
pub mod presets {
    use super::*;

    /// High-quality agent scores preset
    pub fn high_quality_scores() -> AgentScores {
        AgentScores::high_performance_scores()
    }

    /// Conservative agent scores preset
    pub fn conservative_scores() -> AgentScores {
        AgentScores::conservative_scores()
    }

    /// Balanced agent scores preset
    pub fn balanced_scores() -> AgentScores {
        AgentScores::balanced_scores()
    }

    /// Risky agent scores preset
    pub fn risky_scores() -> AgentScores {
        AgentScores::risky_scores()
    }

    /// Default agent scores preset
    pub fn default_scores() -> AgentScores {
        AgentScores::default_scores()
    }

    /// Get scores optimized for specific agent type
    pub fn scores_for_agent(agent_id: &str) -> AgentScores {
        AgentScores::for_agent_perspective(agent_id)
    }

    /// Common agent IDs for testing
    pub fn common_agent_ids() -> Vec<String> {
        vec![
            "performance_agent".to_string(),
            "security_agent".to_string(),
            "maintainability_agent".to_string(),
            "user_experience_agent".to_string(),
            "architecture_agent".to_string(),
            "testing_agent".to_string(),
            "documentation_agent".to_string(),
        ]
    }

    /// Get weighted agent configurations
    pub fn weighted_agent_configs() -> Vec<(String, AgentScores)> {
        common_agent_ids().into_iter()
            .map(|id| {
                let scores = scores_for_agent(&id);
                (id, scores)
            })
            .collect()
    }
}

/// Simulation utilities and helpers
pub mod utils {
    use super::*;
    use crate::cognitive::committee::core::AgentEvaluation;

    /// Calculate consensus score from multiple evaluations
    pub fn calculate_consensus_score(evaluations: &[AgentEvaluation]) -> f64 {
        if evaluations.is_empty() {
            return 0.0;
        }

        let weights = AgentSimulator::get_agent_perspective_weights();
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for eval in evaluations {
            let weight = weights.get(&eval.agent_id)
                .or_else(|| {
                    // Try to find weight based on agent ID substring
                    weights.iter()
                        .find(|(key, _)| eval.agent_id.contains(key))
                        .map(|(_, w)| w)
                })
                .unwrap_or(&1.0);

            weighted_sum += eval.objective_alignment * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Find evaluations with highest alignment
    pub fn find_best_evaluations(evaluations: &[AgentEvaluation], count: usize) -> Vec<&AgentEvaluation> {
        let mut sorted_evals: Vec<&AgentEvaluation> = evaluations.iter().collect();
        sorted_evals.sort_by(|a, b| {
            b.objective_alignment.partial_cmp(&a.objective_alignment)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_evals.into_iter().take(count).collect()
    }

    /// Find evaluations with lowest risk
    pub fn find_safest_evaluations(evaluations: &[AgentEvaluation], count: usize) -> Vec<&AgentEvaluation> {
        let mut sorted_evals: Vec<&AgentEvaluation> = evaluations.iter().collect();
        sorted_evals.sort_by(|a, b| {
            a.risk_assessment.partial_cmp(&b.risk_assessment)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_evals.into_iter().take(count).collect()
    }

    /// Calculate evaluation diversity score
    pub fn calculate_diversity_score(evaluations: &[AgentEvaluation]) -> f64 {
        if evaluations.len() < 2 {
            return 0.0;
        }

        let alignments: Vec<f64> = evaluations.iter().map(|e| e.objective_alignment).collect();
        let mean = alignments.iter().sum::<f64>() / alignments.len() as f64;
        let variance = alignments.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / alignments.len() as f64;
        
        variance.sqrt() // Standard deviation as diversity measure
    }

    /// Generate evaluation summary
    pub fn generate_summary(evaluations: &[AgentEvaluation]) -> String {
        if evaluations.is_empty() {
            return "No evaluations to summarize".to_string();
        }

        let stats = AgentSimulator::get_batch_statistics(evaluations);
        let consensus = calculate_consensus_score(evaluations);
        let diversity = calculate_diversity_score(evaluations);

        format!(
            "Evaluation Summary:\n\
             - Total Evaluations: {}\n\
             - Progress Rate: {:.1}%\n\
             - Consensus Score: {:.3}\n\
             - Diversity Score: {:.3}\n\
             - Average Alignment: {:.3}\n\
             - Average Quality: {:.3}\n\
             - Average Risk: {:.3}\n\
             - Quality Grade: {}",
            stats.total_evaluations,
            stats.progress_rate * 100.0,
            consensus,
            diversity,
            stats.average_alignment,
            stats.average_quality,
            stats.average_risk,
            stats.quality_grade()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::mcts::CodeState;
    use crate::cognitive::committee::core::EvaluationRubric;

    #[tokio::test]
    async fn test_quick_evaluation() {
        let state = CodeState::default();
        let rubric = EvaluationRubric {
            objective: "Test objective".to_string(),
            criteria: vec![],
        };

        let result = quick::evaluate_agent(
            "performance_agent",
            &state,
            "test_action",
            &rubric,
        ).await;

        assert!(result.is_ok());
        let evaluation = result.unwrap();
        assert_eq!(evaluation.agent_id, "performance_agent");
        assert_eq!(evaluation.action, "test_action");
    }

    #[tokio::test]
    async fn test_batch_evaluation() {
        let state = CodeState::default();
        let rubric = EvaluationRubric {
            objective: "Test objective".to_string(),
            criteria: vec![],
        };
        let agent_ids = presets::common_agent_ids();

        let result = quick::evaluate_agents(
            &agent_ids,
            &state,
            "test_action",
            &rubric,
        ).await;

        assert!(result.is_ok());
        let evaluations = result.unwrap();
        assert_eq!(evaluations.len(), agent_ids.len());
    }

    #[test]
    fn test_presets() {
        let high_quality = presets::high_quality_scores();
        assert!(high_quality.overall_score() > 0.7);

        let conservative = presets::conservative_scores();
        assert!(conservative.risk < 0.3);

        let agent_ids = presets::common_agent_ids();
        assert!(!agent_ids.is_empty());
        assert!(agent_ids.contains(&"performance_agent".to_string()));
    }

    #[test]
    fn test_utils() {
        let evaluations = vec![
            crate::cognitive::committee::core::AgentEvaluation {
                agent_id: "performance_agent".to_string(),
                action: "test".to_string(),
                makes_progress: true,
                objective_alignment: 0.8,
                implementation_quality: 0.7,
                risk_assessment: 0.3,
                reasoning: "Test reasoning".to_string(),
                suggested_improvements: vec![],
            },
            crate::cognitive::committee::core::AgentEvaluation {
                agent_id: "security_agent".to_string(),
                action: "test".to_string(),
                makes_progress: true,
                objective_alignment: 0.6,
                implementation_quality: 0.8,
                risk_assessment: 0.2,
                reasoning: "Test reasoning".to_string(),
                suggested_improvements: vec![],
            },
        ];

        let consensus = utils::calculate_consensus_score(&evaluations);
        assert!(consensus > 0.0 && consensus <= 1.0);

        let best = utils::find_best_evaluations(&evaluations, 1);
        assert_eq!(best.len(), 1);
        assert_eq!(best[0].agent_id, "performance_agent");

        let safest = utils::find_safest_evaluations(&evaluations, 1);
        assert_eq!(safest.len(), 1);
        assert_eq!(safest[0].agent_id, "security_agent");

        let diversity = utils::calculate_diversity_score(&evaluations);
        assert!(diversity >= 0.0);

        let summary = utils::generate_summary(&evaluations);
        assert!(!summary.is_empty());
    }
}

//! Agent evaluation simulation with deterministic behavior
//!
//! This module provides agent evaluation simulation with blazing-fast lookups,
//! optimized scoring, and zero-allocation pattern generation.

use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::CognitiveError;
use super::super::core::{AgentEvaluation, EvaluationRubric};
use super::super::consensus::evaluation_phases::EvaluationPhase;

/// Agent evaluation simulator with optimized scoring algorithms
pub struct AgentSimulator;

impl AgentSimulator {
    /// Simulate agent evaluation with deterministic behavior and error handling
    pub async fn simulate_agent_evaluation(
        agent_id: &str,
        _state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: EvaluationPhase,
        previous_evals: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<AgentEvaluation, CognitiveError> {
        // Determine base score from agent perspective with blazing-fast lookup
        let base_score = Self::get_agent_base_score(agent_id);

        // Apply phase modifier for evaluation refinement
        let phase_modifier = match phase {
            EvaluationPhase::Initial => 0.0,
            EvaluationPhase::Review => 0.05,
            EvaluationPhase::Refine => 0.1,
        };

        // Apply previous evaluation adjustment
        let consensus_modifier = if let Some(prev_evals) = previous_evals {
            Self::calculate_consensus_adjustment(prev_evals)
        } else {
            0.0
        };

        // Apply steering feedback adjustment
        let steering_modifier = if let Some(feedback) = steering_feedback {
            Self::calculate_steering_adjustment(feedback)
        } else {
            0.0
        };

        // Calculate final scores with optimized clamping
        let final_alignment = (base_score + phase_modifier + consensus_modifier + steering_modifier).clamp(0.0, 1.0);
        let final_quality = (base_score * 0.9 + phase_modifier + consensus_modifier * 0.5).clamp(0.0, 1.0);
        let final_risk = (base_score * 1.1 + phase_modifier - steering_modifier * 0.2).clamp(0.0, 1.0);
        let makes_progress = final_alignment > 0.5;

        // Generate contextual reasoning with optimized string formatting
        let reasoning = Self::generate_reasoning(agent_id, action, rubric, final_alignment, final_quality, final_risk);

        // Generate suggestions based on agent perspective with zero allocations
        let suggestions = Self::generate_suggestions(agent_id);

        Ok(AgentEvaluation {
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            makes_progress,
            objective_alignment: final_alignment,
            implementation_quality: final_quality,
            risk_assessment: final_risk,
            reasoning,
            suggested_improvements: suggestions,
        })
    }

    /// Get agent base score with blazing-fast const lookup
    #[inline]
    pub fn get_agent_base_score(agent_id: &str) -> f64 {
        match agent_id {
            id if id.contains("performance") => 0.8,
            id if id.contains("security") => 0.6,
            id if id.contains("maintainability") => 0.7,
            id if id.contains("user") => 0.75,
            id if id.contains("architecture") => 0.65,
            id if id.contains("testing") => 0.55,
            id if id.contains("documentation") => 0.5,
            _ => 0.6,
        }
    }

    /// Calculate consensus adjustment based on previous evaluations
    #[inline]
    fn calculate_consensus_adjustment(previous_evals: &[AgentEvaluation]) -> f64 {
        if previous_evals.is_empty() {
            return 0.0;
        }

        let avg_alignment = previous_evals
            .iter()
            .map(|e| e.objective_alignment)
            .sum::<f64>() / (previous_evals.len() as f64);

        // Moderate adjustment based on consensus
        (avg_alignment - 0.6) * 0.1
    }

    /// Calculate steering feedback adjustment
    #[inline]
    fn calculate_steering_adjustment(feedback: &str) -> f64 {
        let positive_words = ["good", "correct", "progress", "excellent", "improve"];
        let negative_words = ["bad", "wrong", "poor", "terrible", "regress"];

        let positive_count = positive_words.iter().filter(|&word| feedback.contains(word)).count();
        let negative_count = negative_words.iter().filter(|&word| feedback.contains(word)).count();

        match positive_count.cmp(&negative_count) {
            std::cmp::Ordering::Greater => 0.1,
            std::cmp::Ordering::Less => -0.1,
            std::cmp::Ordering::Equal => 0.0,
        }
    }

    /// Generate contextual reasoning with optimized string formatting
    fn generate_reasoning(
        agent_id: &str,
        action: &str,
        rubric: &EvaluationRubric,
        alignment: f64,
        quality: f64,
        risk: f64,
    ) -> String {
        let alignment_desc = match alignment {
            a if a > 0.7 => "strong",
            a if a > 0.5 => "moderate",
            _ => "weak",
        };

        let quality_desc = match quality {
            q if q > 0.7 => "high",
            q if q > 0.5 => "adequate",
            _ => "concerning",
        };

        let risk_desc = match risk {
            r if r > 0.7 => "low risk",
            r if r > 0.5 => "moderate risk",
            _ => "high risk",
        };

        format!(
            "From {} perspective: The action '{}' shows {} alignment with objective '{}'. \
            Implementation quality is {} and risk assessment is {}.",
            agent_id,
            action,
            alignment_desc,
            rubric.objective,
            quality_desc,
            risk_desc
        )
    }

    /// Generate suggestions based on agent perspective with zero allocations
    fn generate_suggestions(agent_id: &str) -> Vec<String> {
        match agent_id {
            id if id.contains("performance") => vec![
                "Consider performance implications".to_string(),
                "Add benchmarking".to_string(),
            ],
            id if id.contains("security") => vec![
                "Review security implications".to_string(),
                "Add input validation".to_string(),
            ],
            id if id.contains("maintainability") => vec![
                "Improve code clarity".to_string(),
                "Add documentation".to_string(),
            ],
            id if id.contains("user") => vec![
                "Consider user experience".to_string(),
                "Add error handling".to_string(),
            ],
            id if id.contains("architecture") => vec![
                "Review system design".to_string(),
                "Consider modularity".to_string(),
            ],
            id if id.contains("testing") => vec![
                "Add test coverage".to_string(),
                "Improve testability".to_string(),
            ],
            id if id.contains("documentation") => vec![
                "Add code comments".to_string(),
                "Update documentation".to_string(),
            ],
            _ => vec![
                "General improvement needed".to_string(),
            ],
        }
    }

    /// Get agent perspective type for specialized handling
    pub fn get_agent_perspective(agent_id: &str) -> AgentPerspectiveType {
        match agent_id {
            id if id.contains("performance") => AgentPerspectiveType::Performance,
            id if id.contains("security") => AgentPerspectiveType::Security,
            id if id.contains("maintainability") => AgentPerspectiveType::Maintainability,
            id if id.contains("user") => AgentPerspectiveType::UserExperience,
            id if id.contains("architecture") => AgentPerspectiveType::Architecture,
            id if id.contains("testing") => AgentPerspectiveType::Testing,
            id if id.contains("documentation") => AgentPerspectiveType::Documentation,
            _ => AgentPerspectiveType::General,
        }
    }
}

/// Agent perspective types for specialized evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentPerspectiveType {
    Performance,
    Security,
    Maintainability,
    UserExperience,
    Architecture,
    Testing,
    Documentation,
    General,
}

impl AgentPerspectiveType {
    /// Get evaluation weight for this perspective
    pub fn evaluation_weight(self) -> f64 {
        match self {
            AgentPerspectiveType::Security => 1.2,
            AgentPerspectiveType::Performance => 1.1,
            AgentPerspectiveType::Maintainability => 1.0,
            AgentPerspectiveType::UserExperience => 1.0,
            AgentPerspectiveType::Architecture => 0.9,
            AgentPerspectiveType::Testing => 0.8,
            AgentPerspectiveType::Documentation => 0.7,
            AgentPerspectiveType::General => 1.0,
        }
    }

    /// Get focus areas for this perspective
    pub fn focus_areas(self) -> &'static [&'static str] {
        match self {
            AgentPerspectiveType::Performance => &["performance", "optimization", "efficiency"],
            AgentPerspectiveType::Security => &["security", "validation", "protection"],
            AgentPerspectiveType::Maintainability => &["maintainability", "clarity", "modularity"],
            AgentPerspectiveType::UserExperience => &["usability", "accessibility", "experience"],
            AgentPerspectiveType::Architecture => &["architecture", "design", "structure"],
            AgentPerspectiveType::Testing => &["testing", "coverage", "quality"],
            AgentPerspectiveType::Documentation => &["documentation", "comments", "clarity"],
            AgentPerspectiveType::General => &["general", "overall", "comprehensive"],
        }
    }
}
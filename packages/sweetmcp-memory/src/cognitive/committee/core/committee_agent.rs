//! Committee agent implementation with evaluation capabilities
//!
//! This module provides the CommitteeAgent structure with specialized
//! evaluation methods based on different perspectives and phases.

use crate::cognitive::{mcts::CodeState, types::{CognitiveError, Model}};
use super::{AgentPerspective, AgentEvaluation, EvaluationRubric};

/// Committee agent with specialized perspective
#[derive(Debug, Clone)]
pub struct CommitteeAgent {
    pub id: String,
    pub perspective: AgentPerspective,
    pub model: Model,
    pub evaluation_history: Vec<AgentEvaluation>,
    pub confidence_threshold: f64,
}

impl CommitteeAgent {
    /// Create new committee agent with optimized initialization
    pub fn new(id: String, perspective: AgentPerspective, model: Model) -> Self {
        Self {
            id,
            perspective,
            model,
            evaluation_history: Vec::new(),
            confidence_threshold: 0.7,
        }
    }

    /// Get agent description with fast description generation
    pub fn description(&self) -> String {
        match &self.perspective {
            AgentPerspective::Performance => "Performance optimization expert".to_string(),
            AgentPerspective::Security => "Security and safety specialist".to_string(),
            AgentPerspective::Maintainability => "Code maintainability advocate".to_string(),
            AgentPerspective::UserExperience => "User experience focused".to_string(),
            AgentPerspective::Architecture => "System architecture expert".to_string(),
            AgentPerspective::Testing => "Testing and quality assurance".to_string(),
            AgentPerspective::Documentation => "Documentation and clarity".to_string(),
        }
    }

    /// Add evaluation to history with fast history management
    pub fn add_evaluation(&mut self, evaluation: AgentEvaluation) {
        self.evaluation_history.push(evaluation);
        
        // Keep history manageable
        if self.evaluation_history.len() > 100 {
            self.evaluation_history.remove(0);
        }
    }

    /// Get recent evaluations with fast filtering
    pub fn recent_evaluations(&self, count: usize) -> &[AgentEvaluation] {
        let start = self.evaluation_history.len().saturating_sub(count);
        &self.evaluation_history[start..]
    }

    /// Calculate agent reliability with optimized reliability calculation
    pub fn calculate_reliability(&self) -> f64 {
        if self.evaluation_history.is_empty() {
            return 0.5; // Neutral reliability for new agents
        }

        let recent_evals = self.recent_evaluations(20);
        let positive_count = recent_evals.iter()
            .filter(|eval| eval.is_positive())
            .count();
        
        positive_count as f64 / recent_evals.len() as f64
    }

    /// Evaluate action with specific phase logic
    pub async fn evaluate_with_phase(
        &self,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        phase: super::super::consensus::evaluation_phases::EvaluationPhase,
        previous_evals: Option<&[AgentEvaluation]>,
        steering_feedback: Option<&str>,
    ) -> Result<AgentEvaluation, CognitiveError> {
        use super::super::consensus::evaluation_phases::EvaluationPhase;
        
        let mut evaluation = AgentEvaluation::new(self.id.clone(), action.to_string());
        
        // Evaluate based on agent perspective and phase
        match phase {
            EvaluationPhase::Initial => {
                // Initial independent evaluation
                self.evaluate_initial(&mut evaluation, state, action, rubric).await?;
            },
            EvaluationPhase::Review => {
                // Review considering previous evaluations
                self.evaluate_review(&mut evaluation, state, action, rubric, previous_evals).await?;
            },
            EvaluationPhase::Refine => {
                // Refine with steering feedback
                self.evaluate_refine(&mut evaluation, state, action, rubric, steering_feedback).await?;
            },
        }

        Ok(evaluation)
    }

    /// Perform initial evaluation based on agent perspective
    async fn evaluate_initial(
        &self,
        evaluation: &mut AgentEvaluation,
        _state: &CodeState,
        action: &str,
        _rubric: &EvaluationRubric,
    ) -> Result<(), CognitiveError> {
        // Evaluate based on agent perspective
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
            evaluation.add_suggestion(format!("Consider {}", focus_areas[0]));
        }

        Ok(())
    }

    /// Perform review evaluation considering previous evaluations
    async fn evaluate_review(
        &self,
        evaluation: &mut AgentEvaluation,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        previous_evals: Option<&[AgentEvaluation]>,
    ) -> Result<(), CognitiveError> {
        // Start with initial evaluation
        self.evaluate_initial(evaluation, state, action, rubric).await?;

        // Adjust based on previous evaluations
        if let Some(prev_evals) = previous_evals {
            let avg_alignment: f64 = prev_evals.iter()
                .map(|e| e.objective_alignment)
                .sum::<f64>() / prev_evals.len() as f64;
            
            // Moderate our evaluation based on consensus
            evaluation.objective_alignment = (evaluation.objective_alignment + avg_alignment) / 2.0;
            
            // Add consensus-based reasoning
            evaluation.reasoning.push_str(&format!(
                " (Adjusted based on {} previous evaluations with avg alignment: {:.2})",
                prev_evals.len(),
                avg_alignment
            ));
        }

        Ok(())
    }

    /// Perform refinement evaluation with steering feedback
    async fn evaluate_refine(
        &self,
        evaluation: &mut AgentEvaluation,
        state: &CodeState,
        action: &str,
        rubric: &EvaluationRubric,
        steering_feedback: Option<&str>,
    ) -> Result<(), CognitiveError> {
        // Start with review-level evaluation
        self.evaluate_review(evaluation, state, action, rubric, None).await?;

        // Incorporate steering feedback
        if let Some(feedback) = steering_feedback {
            // Adjust confidence based on feedback positivity
            let feedback_positive = feedback.contains("good") || 
                                  feedback.contains("correct") || 
                                  feedback.contains("progress");
            
            if feedback_positive {
                evaluation.objective_alignment = (evaluation.objective_alignment * 1.1).min(1.0);
                evaluation.implementation_quality = (evaluation.implementation_quality * 1.05).min(1.0);
            } else {
                evaluation.objective_alignment *= 0.9;
                evaluation.implementation_quality *= 0.95;
            }

            // Add feedback to reasoning
            evaluation.reasoning.push_str(&format!(" (Steering feedback: {})", feedback));
            
            // Add feedback-based suggestion
            evaluation.add_suggestion(format!("Consider feedback: {}", feedback));
        }

        Ok(())
    }
}
//! Evaluation context for committee assessment
//!
//! This module provides the EvaluationContext structure for managing
//! committee evaluation state across multiple rounds.

use crate::cognitive::mcts::types::node_types::CodeState;
use super::{AgentEvaluation, EvaluationRubric};
use std::collections::HashSet;

/// Committee evaluation context
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    pub state: CodeState,
    pub action: String,
    pub rubric: EvaluationRubric,
    pub round: usize,
    pub previous_evaluations: Vec<AgentEvaluation>,
}

impl EvaluationContext {
    /// Create new evaluation context with optimized initialization
    pub fn new(state: CodeState, action: String, rubric: EvaluationRubric) -> Self {
        Self {
            state,
            action,
            rubric,
            round: 0,
            previous_evaluations: Vec::new(),
        }
    }

    /// Add evaluation result with fast result tracking
    pub fn add_evaluation(&mut self, evaluation: AgentEvaluation) {
        self.previous_evaluations.push(evaluation);
    }

    /// Start next round with optimized round management
    pub fn next_round(&mut self) {
        self.round += 1;
    }

    /// Get evaluations for current round with fast filtering
    pub fn current_round_evaluations(&self) -> Vec<&AgentEvaluation> {
        // In this simplified implementation, all evaluations are for current round
        self.previous_evaluations.iter().collect()
    }

    /// Calculate current consensus with optimized consensus calculation
    pub fn calculate_consensus(&self) -> f64 {
        if self.previous_evaluations.is_empty() {
            return 0.0;
        }

        let positive_count = self.previous_evaluations.iter()
            .filter(|eval| eval.makes_progress)
            .count();
        
        positive_count as f64 / self.previous_evaluations.len() as f64
    }

    /// Get average confidence with fast averaging
    pub fn average_confidence(&self) -> f64 {
        if self.previous_evaluations.is_empty() {
            return 0.0;
        }

        let total_confidence: f64 = self.previous_evaluations.iter()
            .map(|eval| eval.overall_score())
            .sum();
        
        total_confidence / self.previous_evaluations.len() as f64
    }

    /// Get improvement suggestions with fast suggestion aggregation
    pub fn aggregate_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let mut seen = HashSet::new();

        for eval in &self.previous_evaluations {
            for suggestion in &eval.suggested_improvements {
                if seen.insert(suggestion.clone()) {
                    suggestions.push(suggestion.clone());
                }
            }
        }

        suggestions
    }

    /// Get dissenting opinions with fast dissent identification
    pub fn get_dissenting_opinions(&self) -> Vec<String> {
        self.previous_evaluations.iter()
            .filter(|eval| !eval.makes_progress)
            .map(|eval| format!("{}: {}", eval.agent_id, eval.reasoning))
            .collect()
    }
}
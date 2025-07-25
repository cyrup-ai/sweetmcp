//! Core evaluation structures for committee consensus
//!
//! This module provides consensus decisions, agent evaluations, and evaluation
//! rubrics with optimized performance and zero-allocation patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::vector::async_vector_optimization::OptimizationSpec;

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

impl AgentEvaluation {
    /// Create new agent evaluation with optimized initialization
    pub fn new(agent_id: String, action: String) -> Self {
        Self {
            agent_id,
            action,
            makes_progress: false,
            objective_alignment: 0.0,
            implementation_quality: 0.0,
            risk_assessment: 0.0,
            reasoning: String::new(),
            suggested_improvements: Vec::new(),
        }
    }

    /// Calculate overall score with optimized scoring
    pub fn overall_score(&self) -> f64 {
        // Weighted combination of factors
        let alignment_weight = 0.4;
        let quality_weight = 0.3;
        let safety_weight = 0.3;
        
        self.objective_alignment * alignment_weight +
        self.implementation_quality * quality_weight +
        self.risk_assessment * safety_weight
    }

    /// Check if evaluation is positive with fast threshold check
    pub fn is_positive(&self) -> bool {
        self.makes_progress && self.overall_score() > 0.6
    }

    /// Add improvement suggestion with zero allocation
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggested_improvements.push(suggestion);
    }
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
    /// Create rubric from optimization spec with optimized construction
    pub fn from_spec(spec: &OptimizationSpec, user_objective: &str) -> Self {
        let mut scoring_guidelines = HashMap::new();
        
        // Add default scoring guidelines with fast insertion
        scoring_guidelines.insert("alignment".to_string(), 
            "Rate 0-1 how well the action aligns with the user objective".to_string());
        scoring_guidelines.insert("quality".to_string(), 
            "Rate 0-1 the implementation quality and correctness".to_string());
        scoring_guidelines.insert("safety".to_string(), 
            "Rate 0-1 how safe the change is (1 = very safe)".to_string());

        // Add spec-specific guidelines
        if let Some(baseline) = &spec.baseline_metrics {
            scoring_guidelines.insert("performance".to_string(),
                format!("Consider baseline metrics: {:?}", baseline));
        }

        Self {
            objective: user_objective.to_string(),
            success_criteria: vec![
                "Makes measurable progress toward objective".to_string(),
                "Maintains code quality and safety".to_string(),
                "Follows established constraints".to_string(),
            ],
            constraints: vec![
                "No breaking changes to public APIs".to_string(),
                "Maintain existing functionality".to_string(),
                "Follow coding standards".to_string(),
            ],
            scoring_guidelines,
        }
    }

    /// Add success criterion with zero allocation
    pub fn add_success_criterion(&mut self, criterion: String) {
        self.success_criteria.push(criterion);
    }

    /// Add constraint with zero allocation
    pub fn add_constraint(&mut self, constraint: String) {
        self.constraints.push(constraint);
    }

    /// Add scoring guideline with fast insertion
    pub fn add_scoring_guideline(&mut self, key: String, guideline: String) {
        self.scoring_guidelines.insert(key, guideline);
    }

    /// Get formatted rubric for prompts with optimized formatting
    pub fn format_for_prompt(&self) -> String {
        let mut formatted = format!("OBJECTIVE: {}\n\n", self.objective);
        
        formatted.push_str("SUCCESS CRITERIA:\n");
        for (i, criterion) in self.success_criteria.iter().enumerate() {
            formatted.push_str(&format!("{}. {}\n", i + 1, criterion));
        }
        
        formatted.push_str("\nCONSTRAINTS:\n");
        for (i, constraint) in self.constraints.iter().enumerate() {
            formatted.push_str(&format!("{}. {}\n", i + 1, constraint));
        }
        
        formatted.push_str("\nSCORING GUIDELINES:\n");
        for (key, guideline) in &self.scoring_guidelines {
            formatted.push_str(&format!("- {}: {}\n", key, guideline));
        }
        
        formatted
    }
}
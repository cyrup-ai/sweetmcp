//! Evaluation rubric and scoring systems for committee-based assessment
//!
//! This module provides evaluation rubrics, scoring guidelines, and assessment
//! frameworks for committee-based code evaluation with optimized performance
//! and zero allocation fast paths.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::cognitive::types::OptimizationSpec;

/// Evaluation rubric provided to agents for consistent assessment
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

        if let Some(content_type) = &spec.content_type {
            scoring_guidelines.insert("content_type".to_string(),
                format!("Consider content type requirements: {}", content_type));
        }

        if let Some(evolution_rules) = &spec.evolution_rules {
            scoring_guidelines.insert("evolution".to_string(),
                format!("Follow evolution rules: {:?}", evolution_rules));
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

    /// Create custom rubric with specific criteria
    pub fn custom(
        objective: String,
        success_criteria: Vec<String>,
        constraints: Vec<String>,
    ) -> Self {
        let mut scoring_guidelines = HashMap::new();
        
        // Default scoring guidelines
        scoring_guidelines.insert("alignment".to_string(), 
            "Rate 0-1 how well the action aligns with the user objective".to_string());
        scoring_guidelines.insert("quality".to_string(), 
            "Rate 0-1 the implementation quality and correctness".to_string());
        scoring_guidelines.insert("safety".to_string(), 
            "Rate 0-1 how safe the change is (1 = very safe)".to_string());

        Self {
            objective,
            success_criteria,
            constraints,
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

    /// Get compact rubric for quick reference
    pub fn format_compact(&self) -> String {
        format!(
            "Objective: {} | Criteria: {} | Constraints: {} | Guidelines: {}",
            self.objective,
            self.success_criteria.len(),
            self.constraints.len(),
            self.scoring_guidelines.len()
        )
    }

    /// Validate rubric completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.objective.is_empty() {
            return Err("Objective cannot be empty".to_string());
        }

        if self.success_criteria.is_empty() {
            return Err("At least one success criterion is required".to_string());
        }

        if self.scoring_guidelines.is_empty() {
            return Err("At least one scoring guideline is required".to_string());
        }

        // Check for required scoring guidelines
        let required_guidelines = ["alignment", "quality", "safety"];
        for guideline in &required_guidelines {
            if !self.scoring_guidelines.contains_key(*guideline) {
                return Err(format!("Missing required scoring guideline: {}", guideline));
            }
        }

        Ok(())
    }

    /// Update objective while preserving other settings
    pub fn update_objective(&mut self, new_objective: String) {
        self.objective = new_objective;
    }

    /// Merge with another rubric (combines criteria and constraints)
    pub fn merge(&mut self, other: &EvaluationRubric) {
        // Extend success criteria
        for criterion in &other.success_criteria {
            if !self.success_criteria.contains(criterion) {
                self.success_criteria.push(criterion.clone());
            }
        }

        // Extend constraints
        for constraint in &other.constraints {
            if !self.constraints.contains(constraint) {
                self.constraints.push(constraint.clone());
            }
        }

        // Merge scoring guidelines (other takes precedence)
        for (key, guideline) in &other.scoring_guidelines {
            self.scoring_guidelines.insert(key.clone(), guideline.clone());
        }
    }

    /// Get rubric statistics
    pub fn stats(&self) -> RubricStats {
        RubricStats {
            objective_length: self.objective.len(),
            success_criteria_count: self.success_criteria.len(),
            constraints_count: self.constraints.len(),
            scoring_guidelines_count: self.scoring_guidelines.len(),
            total_text_length: self.format_for_prompt().len(),
        }
    }
}

/// Statistics about a rubric
#[derive(Debug, Clone)]
pub struct RubricStats {
    pub objective_length: usize,
    pub success_criteria_count: usize,
    pub constraints_count: usize,
    pub scoring_guidelines_count: usize,
    pub total_text_length: usize,
}

/// Consensus decision from committee evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusDecision {
    pub makes_progress: bool,
    pub confidence: f64,
    pub overall_score: f64, // Weighted combination of alignment, quality, safety
    pub improvement_suggestions: Vec<String>,
    pub dissenting_opinions: Vec<String>,
}

impl ConsensusDecision {
    /// Create new consensus decision
    pub fn new() -> Self {
        Self {
            makes_progress: false,
            confidence: 0.0,
            overall_score: 0.0,
            improvement_suggestions: Vec::new(),
            dissenting_opinions: Vec::new(),
        }
    }

    /// Create positive consensus decision
    pub fn positive(score: f64, confidence: f64) -> Self {
        Self {
            makes_progress: true,
            confidence,
            overall_score: score,
            improvement_suggestions: Vec::new(),
            dissenting_opinions: Vec::new(),
        }
    }

    /// Create negative consensus decision
    pub fn negative(reasons: Vec<String>) -> Self {
        Self {
            makes_progress: false,
            confidence: 0.8, // High confidence in rejection
            overall_score: 0.0,
            improvement_suggestions: reasons,
            dissenting_opinions: Vec::new(),
        }
    }

    /// Add improvement suggestion
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.improvement_suggestions.push(suggestion);
    }

    /// Add dissenting opinion
    pub fn add_dissent(&mut self, opinion: String) {
        self.dissenting_opinions.push(opinion);
    }

    /// Check if decision is strong (high confidence)
    pub fn is_strong(&self) -> bool {
        self.confidence > 0.8
    }

    /// Check if decision is unanimous (no dissenting opinions)
    pub fn is_unanimous(&self) -> bool {
        self.dissenting_opinions.is_empty()
    }

    /// Get decision summary
    pub fn summary(&self) -> String {
        let status = if self.makes_progress { "PROGRESS" } else { "NO PROGRESS" };
        let confidence_level = if self.confidence > 0.8 { "HIGH" } else if self.confidence > 0.5 { "MEDIUM" } else { "LOW" };
        
        format!(
            "{} (Score: {:.2}, Confidence: {} {:.2})",
            status,
            self.overall_score,
            confidence_level,
            self.confidence
        )
    }

    /// Merge with another consensus decision (weighted average)
    pub fn merge(&mut self, other: &ConsensusDecision, weight: f64) {
        let self_weight = 1.0 - weight;
        
        // Weighted average of scores and confidence
        self.overall_score = self.overall_score * self_weight + other.overall_score * weight;
        self.confidence = self.confidence * self_weight + other.confidence * weight;
        
        // Progress requires both to agree
        self.makes_progress = self.makes_progress && other.makes_progress;
        
        // Combine suggestions and dissents
        self.improvement_suggestions.extend(other.improvement_suggestions.clone());
        self.dissenting_opinions.extend(other.dissenting_opinions.clone());
        
        // Remove duplicates
        self.improvement_suggestions.sort();
        self.improvement_suggestions.dedup();
        self.dissenting_opinions.sort();
        self.dissenting_opinions.dedup();
    }
}

impl Default for ConsensusDecision {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoring weights for different evaluation aspects
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub alignment_weight: f64,
    pub quality_weight: f64,
    pub safety_weight: f64,
    pub performance_weight: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            alignment_weight: 0.4,
            quality_weight: 0.3,
            safety_weight: 0.2,
            performance_weight: 0.1,
        }
    }
}

impl ScoringWeights {
    /// Create custom scoring weights
    pub fn custom(alignment: f64, quality: f64, safety: f64, performance: f64) -> Self {
        let total = alignment + quality + safety + performance;
        
        Self {
            alignment_weight: alignment / total,
            quality_weight: quality / total,
            safety_weight: safety / total,
            performance_weight: performance / total,
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let total = self.alignment_weight + self.quality_weight + self.safety_weight + self.performance_weight;
        
        if total > 0.0 {
            self.alignment_weight /= total;
            self.quality_weight /= total;
            self.safety_weight /= total;
            self.performance_weight /= total;
        }
    }

    /// Calculate weighted score from individual components
    pub fn calculate_score(
        &self,
        alignment: f64,
        quality: f64,
        safety: f64,
        performance: f64,
    ) -> f64 {
        (alignment * self.alignment_weight +
         quality * self.quality_weight +
         safety * self.safety_weight +
         performance * self.performance_weight).clamp(0.0, 1.0)
    }
}

/// Evaluation context for providing additional information to agents
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current code state
    pub code_context: String,
    
    /// Previous actions taken
    pub action_history: Vec<String>,
    
    /// Performance metrics
    pub performance_data: HashMap<String, f64>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl EvaluationContext {
    /// Create new evaluation context
    pub fn new(code_context: String) -> Self {
        Self {
            code_context,
            action_history: Vec::new(),
            performance_data: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add action to history
    pub fn add_action(&mut self, action: String) {
        self.action_history.push(action);
        
        // Keep history manageable
        if self.action_history.len() > 50 {
            self.action_history.remove(0);
        }
    }

    /// Add performance metric
    pub fn add_metric(&mut self, key: String, value: f64) {
        self.performance_data.insert(key, value);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Format context for prompt inclusion
    pub fn format_for_prompt(&self) -> String {
        let mut formatted = format!("CODE CONTEXT:\n{}\n\n", self.code_context);
        
        if !self.action_history.is_empty() {
            formatted.push_str("RECENT ACTIONS:\n");
            for (i, action) in self.action_history.iter().rev().take(5).enumerate() {
                formatted.push_str(&format!("{}. {}\n", i + 1, action));
            }
            formatted.push('\n');
        }
        
        if !self.performance_data.is_empty() {
            formatted.push_str("PERFORMANCE METRICS:\n");
            for (key, value) in &self.performance_data {
                formatted.push_str(&format!("- {}: {:.3}\n", key, value));
            }
            formatted.push('\n');
        }
        
        formatted
    }
}
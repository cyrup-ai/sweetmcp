//! Core committee data structures and types
//!
//! This module provides the fundamental data structures for the committee-based
//! evaluation system with zero allocation patterns and blazing-fast performance.

use serde::{Deserialize, Serialize};

/// Consensus decision from committee evaluation
/// 
/// Represents the final decision reached by the committee after evaluating
/// an action or proposal, with comprehensive scoring and feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusDecision {
    /// Whether the committee believes the action makes progress toward the objective
    pub makes_progress: bool,
    /// Confidence level in the decision (0.0 to 1.0)
    pub confidence: f64,
    /// Overall weighted score combining alignment, quality, and safety
    pub overall_score: f64,
    /// Constructive suggestions for improvement
    pub improvement_suggestions: Vec<String>,
    /// Dissenting opinions from minority committee members
    pub dissenting_opinions: Vec<String>,
}

impl ConsensusDecision {
    /// Create a new consensus decision with default values
    /// 
    /// # Returns
    /// ConsensusDecision with conservative defaults
    pub fn new() -> Self {
        Self {
            makes_progress: false,
            confidence: 0.0,
            overall_score: 0.0,
            improvement_suggestions: Vec::new(),
            dissenting_opinions: Vec::new(),
        }
    }

    /// Create a positive consensus decision
    /// 
    /// # Arguments
    /// * `confidence` - Confidence level (0.0 to 1.0)
    /// * `overall_score` - Overall score (0.0 to 1.0)
    /// 
    /// # Returns
    /// ConsensusDecision indicating progress
    pub fn positive(confidence: f64, overall_score: f64) -> Self {
        Self {
            makes_progress: true,
            confidence: confidence.clamp(0.0, 1.0),
            overall_score: overall_score.clamp(0.0, 1.0),
            improvement_suggestions: Vec::new(),
            dissenting_opinions: Vec::new(),
        }
    }

    /// Create a negative consensus decision
    /// 
    /// # Arguments
    /// * `confidence` - Confidence level (0.0 to 1.0)
    /// * `reasons` - Reasons for the negative decision
    /// 
    /// # Returns
    /// ConsensusDecision indicating no progress
    pub fn negative(confidence: f64, reasons: Vec<String>) -> Self {
        Self {
            makes_progress: false,
            confidence: confidence.clamp(0.0, 1.0),
            overall_score: 0.0,
            improvement_suggestions: reasons,
            dissenting_opinions: Vec::new(),
        }
    }

    /// Add an improvement suggestion
    /// 
    /// # Arguments
    /// * `suggestion` - The improvement suggestion to add
    pub fn add_suggestion(&mut self, suggestion: String) {
        if !suggestion.is_empty() && !self.improvement_suggestions.contains(&suggestion) {
            self.improvement_suggestions.push(suggestion);
        }
    }

    /// Add a dissenting opinion
    /// 
    /// # Arguments
    /// * `opinion` - The dissenting opinion to add
    pub fn add_dissenting_opinion(&mut self, opinion: String) {
        if !opinion.is_empty() && !self.dissenting_opinions.contains(&opinion) {
            self.dissenting_opinions.push(opinion);
        }
    }

    /// Check if the decision is unanimous
    /// 
    /// # Returns
    /// true if there are no dissenting opinions, false otherwise
    pub fn is_unanimous(&self) -> bool {
        self.dissenting_opinions.is_empty()
    }

    /// Get the strength of consensus (inverse of dissent)
    /// 
    /// # Returns
    /// Consensus strength as a value between 0.0 and 1.0
    pub fn consensus_strength(&self) -> f64 {
        if self.dissenting_opinions.is_empty() {
            1.0
        } else {
            // Reduce strength based on number of dissenting opinions
            let dissent_factor = self.dissenting_opinions.len() as f64 * 0.1;
            (1.0 - dissent_factor).max(0.0)
        }
    }
}

impl Default for ConsensusDecision {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual agent's evaluation of an action or proposal
/// 
/// Represents a single agent's assessment with detailed scoring across
/// multiple dimensions and comprehensive reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEvaluation {
    /// Unique identifier for the evaluating agent
    pub agent_id: String,
    /// The action or proposal being evaluated
    pub action: String,
    /// Core assessment: does this help achieve the objective?
    pub makes_progress: bool,
    /// How well aligned with the stated objective (0.0 to 1.0)
    pub objective_alignment: f64,
    /// Quality of implementation or approach (0.0 to 1.0)
    pub implementation_quality: f64,
    /// Safety and risk assessment (0.0 = high risk, 1.0 = safe)
    pub risk_assessment: f64,
    /// Detailed explanation of the evaluation reasoning
    pub reasoning: String,
    /// Specific suggestions for improvement
    pub suggested_improvements: Vec<String>,
}

impl AgentEvaluation {
    /// Create new agent evaluation with optimized initialization
    /// 
    /// # Arguments
    /// * `agent_id` - Unique identifier for the agent
    /// * `action` - The action being evaluated
    /// 
    /// # Returns
    /// New AgentEvaluation with default values
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

    /// Calculate overall score with weighted combination
    /// 
    /// Combines alignment, quality, and safety scores with appropriate weights
    /// to produce a single overall assessment score.
    /// 
    /// # Returns
    /// Overall score as a weighted average (0.0 to 1.0)
    pub fn overall_score(&self) -> f64 {
        // Weighted combination: alignment (40%), quality (30%), safety (30%)
        let weighted_score = (self.objective_alignment * 0.4) +
                           (self.implementation_quality * 0.3) +
                           (self.risk_assessment * 0.3);
        
        weighted_score.clamp(0.0, 1.0)
    }

    /// Set the progress assessment with validation
    /// 
    /// # Arguments
    /// * `makes_progress` - Whether the action makes progress
    /// * `reasoning` - Explanation for the assessment
    pub fn set_progress(&mut self, makes_progress: bool, reasoning: String) {
        self.makes_progress = makes_progress;
        if !reasoning.is_empty() {
            self.reasoning = reasoning;
        }
    }

    /// Set objective alignment score with validation
    /// 
    /// # Arguments
    /// * `score` - Alignment score (will be clamped to 0.0-1.0)
    pub fn set_alignment(&mut self, score: f64) {
        self.objective_alignment = score.clamp(0.0, 1.0);
    }

    /// Set implementation quality score with validation
    /// 
    /// # Arguments
    /// * `score` - Quality score (will be clamped to 0.0-1.0)
    pub fn set_quality(&mut self, score: f64) {
        self.implementation_quality = score.clamp(0.0, 1.0);
    }

    /// Set risk assessment score with validation
    /// 
    /// # Arguments
    /// * `score` - Risk score (will be clamped to 0.0-1.0)
    pub fn set_risk(&mut self, score: f64) {
        self.risk_assessment = score.clamp(0.0, 1.0);
    }

    /// Add an improvement suggestion
    /// 
    /// # Arguments
    /// * `suggestion` - The improvement suggestion to add
    pub fn add_suggestion(&mut self, suggestion: String) {
        if !suggestion.is_empty() && !self.suggested_improvements.contains(&suggestion) {
            self.suggested_improvements.push(suggestion);
        }
    }

    /// Check if the evaluation is complete
    /// 
    /// An evaluation is considered complete if it has reasoning and
    /// at least one non-zero score.
    /// 
    /// # Returns
    /// true if the evaluation appears complete, false otherwise
    pub fn is_complete(&self) -> bool {
        !self.reasoning.is_empty() && 
        (self.objective_alignment > 0.0 || 
         self.implementation_quality > 0.0 || 
         self.risk_assessment > 0.0)
    }

    /// Get a summary of the evaluation
    /// 
    /// # Returns
    /// Brief summary string of the evaluation
    pub fn summary(&self) -> String {
        format!(
            "Agent {}: {} (Overall: {:.2}, Progress: {})",
            self.agent_id,
            if self.reasoning.len() > 50 {
                format!("{}...", &self.reasoning[..47])
            } else {
                self.reasoning.clone()
            },
            self.overall_score(),
            self.makes_progress
        )
    }
}
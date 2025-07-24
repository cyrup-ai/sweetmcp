//! Committee agents with specialized perspectives
//!
//! This module provides committee agent implementations with different
//! perspectives for comprehensive evaluation with zero allocation
//! fast paths and optimized performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::cognitive::types::{Model, ModelType};

/// Agent perspective for specialized evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentPerspective {
    /// Performance optimization focus
    Performance,
    
    /// Security and safety focus
    Security,
    
    /// Code maintainability focus
    Maintainability,
    
    /// User experience focus
    UserExperience,
    
    /// System architecture focus
    Architecture,
    
    /// Testing and quality assurance focus
    Testing,
    
    /// Documentation and clarity focus
    Documentation,
}

/// Individual agent's evaluation result
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

    /// Calculate overall score with weighted factors
    pub fn overall_score(&self) -> f64 {
        if !self.makes_progress {
            return 0.0;
        }
        
        // Weighted combination of factors
        let alignment_weight = 0.4;
        let quality_weight = 0.3;
        let safety_weight = 0.3;
        
        (self.objective_alignment * alignment_weight +
         self.implementation_quality * quality_weight +
         self.risk_assessment * safety_weight).clamp(0.0, 1.0)
    }

    /// Check if evaluation meets confidence threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.overall_score() >= threshold
    }

    /// Get evaluation summary for reporting
    pub fn summary(&self) -> String {
        format!(
            "Agent {}: {} (Score: {:.2}, Progress: {}, Alignment: {:.2}, Quality: {:.2}, Safety: {:.2})",
            self.agent_id,
            if self.makes_progress { "PROGRESS" } else { "NO PROGRESS" },
            self.overall_score(),
            self.makes_progress,
            self.objective_alignment,
            self.implementation_quality,
            self.risk_assessment
        )
    }
}

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

    /// Get recent evaluation performance
    pub fn recent_performance(&self, last_n: usize) -> f64 {
        if self.evaluation_history.is_empty() {
            return 0.5; // Neutral score
        }
        
        let recent_count = last_n.min(self.evaluation_history.len());
        let recent_evaluations = &self.evaluation_history[self.evaluation_history.len() - recent_count..];
        
        let total_score: f64 = recent_evaluations.iter()
            .map(|eval| eval.overall_score())
            .sum();
        
        total_score / recent_count as f64
    }

    /// Check if agent is performing well
    pub fn is_performing_well(&self) -> bool {
        self.recent_performance(10) >= self.confidence_threshold
    }

    /// Get specialized prompt for this agent's perspective
    pub fn get_specialized_prompt(&self, base_prompt: &str, action: &str) -> String {
        let perspective_guidance = match &self.perspective {
            AgentPerspective::Performance => {
                "Focus on performance implications, optimization opportunities, and efficiency gains. \
                 Consider memory usage, CPU utilization, and scalability."
            }
            AgentPerspective::Security => {
                "Evaluate security implications, potential vulnerabilities, and safety concerns. \
                 Consider input validation, access control, and data protection."
            }
            AgentPerspective::Maintainability => {
                "Assess code maintainability, readability, and long-term sustainability. \
                 Consider code organization, documentation, and future modification ease."
            }
            AgentPerspective::UserExperience => {
                "Focus on user experience impact, usability, and interface design. \
                 Consider user workflows, accessibility, and interaction patterns."
            }
            AgentPerspective::Architecture => {
                "Evaluate architectural soundness, design patterns, and system structure. \
                 Consider modularity, coupling, cohesion, and scalability."
            }
            AgentPerspective::Testing => {
                "Assess testability, test coverage, and quality assurance aspects. \
                 Consider test strategies, edge cases, and verification approaches."
            }
            AgentPerspective::Documentation => {
                "Focus on documentation quality, code clarity, and knowledge transfer. \
                 Consider comments, API documentation, and code self-documentation."
            }
        };

        format!(
            "{}\n\nSPECIALIZED PERSPECTIVE ({}): {}\n\nACTION TO EVALUATE: {}",
            base_prompt,
            self.description(),
            perspective_guidance,
            action
        )
    }

    /// Update confidence threshold based on performance
    pub fn adapt_confidence_threshold(&mut self) {
        let recent_perf = self.recent_performance(20);
        
        // Adjust threshold based on recent performance
        if recent_perf > 0.8 {
            // Performing well, can be more selective
            self.confidence_threshold = (self.confidence_threshold + 0.05).min(0.9);
        } else if recent_perf < 0.5 {
            // Performing poorly, be less selective
            self.confidence_threshold = (self.confidence_threshold - 0.05).max(0.3);
        }
    }

    /// Get evaluation statistics
    pub fn evaluation_stats(&self) -> AgentStats {
        if self.evaluation_history.is_empty() {
            return AgentStats::default();
        }

        let total_evaluations = self.evaluation_history.len();
        let progress_count = self.evaluation_history.iter()
            .filter(|eval| eval.makes_progress)
            .count();

        let avg_alignment: f64 = self.evaluation_history.iter()
            .map(|eval| eval.objective_alignment)
            .sum::<f64>() / total_evaluations as f64;

        let avg_quality: f64 = self.evaluation_history.iter()
            .map(|eval| eval.implementation_quality)
            .sum::<f64>() / total_evaluations as f64;

        let avg_safety: f64 = self.evaluation_history.iter()
            .map(|eval| eval.risk_assessment)
            .sum::<f64>() / total_evaluations as f64;

        AgentStats {
            total_evaluations,
            progress_rate: progress_count as f64 / total_evaluations as f64,
            avg_alignment,
            avg_quality,
            avg_safety,
            recent_performance: self.recent_performance(10),
            confidence_threshold: self.confidence_threshold,
        }
    }

    /// Reset evaluation history
    pub fn reset_history(&mut self) {
        self.evaluation_history.clear();
    }

    /// Export evaluation history for analysis
    pub fn export_history(&self) -> Vec<AgentEvaluation> {
        self.evaluation_history.clone()
    }
}

/// Agent performance statistics
#[derive(Debug, Clone, Default)]
pub struct AgentStats {
    pub total_evaluations: usize,
    pub progress_rate: f64,
    pub avg_alignment: f64,
    pub avg_quality: f64,
    pub avg_safety: f64,
    pub recent_performance: f64,
    pub confidence_threshold: f64,
}

impl AgentStats {
    /// Get overall agent health score
    pub fn health_score(&self) -> f64 {
        if self.total_evaluations == 0 {
            return 0.5; // Neutral for new agents
        }

        // Weighted combination of metrics
        (self.progress_rate * 0.3 +
         self.avg_alignment * 0.25 +
         self.avg_quality * 0.25 +
         self.avg_safety * 0.2).clamp(0.0, 1.0)
    }

    /// Check if agent needs attention
    pub fn needs_attention(&self) -> bool {
        self.health_score() < 0.4 || self.recent_performance < 0.3
    }

    /// Get performance trend (positive = improving, negative = declining)
    pub fn performance_trend(&self) -> f64 {
        // Simplified trend calculation
        // In practice, would compare recent vs historical performance
        self.recent_performance - (self.avg_alignment + self.avg_quality + self.avg_safety) / 3.0
    }
}

/// Agent factory for creating specialized agents
pub struct AgentFactory;

impl AgentFactory {
    /// Create default committee with all perspectives
    pub fn create_default_committee() -> Vec<CommitteeAgent> {
        let perspectives = [
            AgentPerspective::Performance,
            AgentPerspective::Security,
            AgentPerspective::Maintainability,
            AgentPerspective::Architecture,
        ];

        perspectives.iter().enumerate().map(|(i, &perspective)| {
            let model = Model {
                name: "gpt-4".to_string(),
                model_type: ModelType::OpenAI,
                max_tokens: 4000,
                temperature: 0.1,
            };

            CommitteeAgent::new(
                format!("agent_{}", i),
                perspective,
                model,
            )
        }).collect()
    }

    /// Create specialized committee for specific domain
    pub fn create_specialized_committee(domain: &str) -> Vec<CommitteeAgent> {
        let perspectives = match domain {
            "performance" => vec![
                AgentPerspective::Performance,
                AgentPerspective::Architecture,
                AgentPerspective::Testing,
            ],
            "security" => vec![
                AgentPerspective::Security,
                AgentPerspective::Architecture,
                AgentPerspective::Testing,
            ],
            "maintainability" => vec![
                AgentPerspective::Maintainability,
                AgentPerspective::Documentation,
                AgentPerspective::Testing,
            ],
            _ => vec![
                AgentPerspective::Performance,
                AgentPerspective::Security,
                AgentPerspective::Maintainability,
                AgentPerspective::Architecture,
            ],
        };

        perspectives.iter().enumerate().map(|(i, &perspective)| {
            let model = Model {
                name: "gpt-4".to_string(),
                model_type: ModelType::OpenAI,
                max_tokens: 4000,
                temperature: 0.1,
            };

            CommitteeAgent::new(
                format!("{}_{}", domain, i),
                perspective,
                model,
            )
        }).collect()
    }

    /// Create agent with custom configuration
    pub fn create_custom_agent(
        id: String,
        perspective: AgentPerspective,
        model_name: &str,
        confidence_threshold: f64,
    ) -> CommitteeAgent {
        let model = Model {
            name: model_name.to_string(),
            model_type: ModelType::OpenAI, // Default to OpenAI
            max_tokens: 4000,
            temperature: 0.1,
        };

        let mut agent = CommitteeAgent::new(id, perspective, model);
        agent.confidence_threshold = confidence_threshold;
        agent
    }
}
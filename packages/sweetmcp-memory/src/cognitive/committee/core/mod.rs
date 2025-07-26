//! Committee core module coordination
//!
//! This module coordinates all aspects of committee-based evaluation including
//! agent management, evaluation rubrics, and committee coordination with
//! optimized performance and zero allocation fast paths.

pub mod agents;
pub mod agent_config;
pub mod agent_perspectives;
pub mod committee;
pub mod committee_agent;
pub mod committee_config;
pub mod config;
pub mod evaluation;
pub mod evaluation_context;
pub mod evaluation_structures;
pub mod evaluators;
pub mod metrics;
pub mod types;

// Re-export key types and functionality
pub use agents::{
    CommitteeAgent, AgentEvaluation, AgentStats, AgentFactory
};
pub use agent_perspectives::AgentPerspective;
pub use evaluation::{
    EvaluationRubric, ConsensusDecision, EvaluationContext, ScoringWeights, RubricStats
};
pub use committee::{
    EvaluationCommittee, CommitteeConfig, CommitteeMessage, CommitteeStats
};

use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::cognitive::mcts::types::node_types::CodeState;
use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;

/// High-level committee coordinator for simplified usage
pub struct CommitteeCoordinator {
    /// Main evaluation committee
    committee: EvaluationCommittee,
    
    /// Message receiver for coordination
    coordinator_rx: mpsc::Receiver<CommitteeMessage>,
    
    /// Coordinator statistics
    stats: CoordinatorStats,
}

/// Coordinator statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct CoordinatorStats {
    /// Total coordination operations
    pub total_operations: u64,
    
    /// Successful evaluations
    pub successful_evaluations: u64,
    
    /// Failed evaluations
    pub failed_evaluations: u64,
    
    /// Average response time (milliseconds)
    pub avg_response_time_ms: f64,
    
    /// Message processing statistics
    pub messages_processed: u64,
}

impl CommitteeCoordinator {
    /// Create new committee coordinator
    pub async fn new(
        config: CommitteeConfig,
        spec: &OptimizationSpec,
        user_objective: &str,
    ) -> Result<Self, CognitiveError> {
        let (coordinator_tx, coordinator_rx) = mpsc::channel(100);
        
        let committee = EvaluationCommittee::with_config(
            config,
            spec,
            user_objective,
            coordinator_tx,
        );
        
        let stats = CoordinatorStats::default();

        Ok(Self {
            committee,
            coordinator_rx,
            stats,
        })
    }

    /// Create coordinator with default configuration
    pub async fn default(user_objective: &str) -> Result<Self, CognitiveError> {
        let config = CommitteeConfig::default();
        let spec = OptimizationSpec::default();
        
        Self::new(config, &spec, user_objective).await
    }

    /// Evaluate action with committee
    pub async fn evaluate_action(
        &mut self,
        state: &CodeState,
        action: &str,
    ) -> Result<ConsensusDecision, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        let result = self.committee.evaluate_action(state, action).await;
        
        // Update statistics
        let response_time = start_time.elapsed().as_millis() as f64;
        self.stats.total_operations += 1;
        self.stats.avg_response_time_ms = (self.stats.avg_response_time_ms * (self.stats.total_operations - 1) as f64 + response_time) / self.stats.total_operations as f64;
        
        match &result {
            Ok(_) => self.stats.successful_evaluations += 1,
            Err(_) => self.stats.failed_evaluations += 1,
        }

        result
    }

    /// Process coordination messages
    pub async fn process_messages(&mut self) -> Result<Vec<CommitteeMessage>, CognitiveError> {
        let mut messages = Vec::new();
        
        // Process all available messages
        while let Ok(message) = self.coordinator_rx.try_recv() {
            match &message {
                CommitteeMessage::StartEvaluation { action, state: _ } => {
                    debug!("Committee evaluation started for action: {}", action);
                }
                CommitteeMessage::AgentEvaluation { agent_id, evaluation } => {
                    debug!("Agent {} completed evaluation: {}", agent_id, evaluation.summary());
                }
                CommitteeMessage::ConsensusReached { decision } => {
                    info!("Committee consensus reached: {}", decision.summary());
                }
                CommitteeMessage::EvaluationTimeout => {
                    warn!("Committee evaluation timed out");
                }
                CommitteeMessage::Error { message } => {
                    warn!("Committee error: {}", message);
                }
            }
            
            messages.push(message);
            self.stats.messages_processed += 1;
        }

        Ok(messages)
    }

    /// Get comprehensive statistics
    pub fn stats(&self) -> CoordinatorStats {
        self.stats.clone()
    }

    /// Get committee statistics
    pub fn committee_stats(&self) -> &CommitteeStats {
        self.committee.stats()
    }

    /// Update committee configuration
    pub fn update_config(&mut self, new_config: CommitteeConfig) {
        self.committee.update_config(new_config);
    }

    /// Update evaluation rubric
    pub fn update_rubric(&mut self, new_rubric: EvaluationRubric) {
        self.committee.update_rubric(new_rubric);
    }

    /// Add agent to committee
    pub fn add_agent(&mut self, agent: CommitteeAgent) {
        self.committee.add_agent(agent);
    }

    /// Remove agent from committee
    pub fn remove_agent(&mut self, agent_id: &str) {
        self.committee.remove_agent(agent_id);
    }

    /// Get agent performance summary
    pub fn agent_performance(&self) -> HashMap<String, f64> {
        self.committee.agent_performance_summary()
    }

    /// Health check for the committee system
    pub fn health_check(&self) -> CommitteeHealth {
        let mut issues = Vec::new();
        
        // Check success rate
        let total_evaluations = self.stats.successful_evaluations + self.stats.failed_evaluations;
        if total_evaluations > 0 {
            let success_rate = self.stats.successful_evaluations as f64 / total_evaluations as f64;
            if success_rate < 0.8 {
                issues.push(format!("Low evaluation success rate: {:.2}%", success_rate * 100.0));
            }
        }

        // Check response time
        if self.stats.avg_response_time_ms > 5000.0 {
            issues.push("High average response time".to_string());
        }

        // Check committee stats
        let committee_stats = self.committee.stats();
        if committee_stats.total_evaluations > 0 {
            let consensus_rate = committee_stats.consensus_reached as f64 / committee_stats.total_evaluations as f64;
            if consensus_rate < 0.5 {
                issues.push(format!("Low consensus rate: {:.2}%", consensus_rate * 100.0));
            }
        }

        CommitteeHealth {
            healthy: issues.is_empty(),
            issues,
            coordinator_stats: self.stats.clone(),
            committee_stats: committee_stats.clone(),
        }
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.stats = CoordinatorStats::default();
        self.committee.reset_stats();
    }
}

/// Health status for the committee system
#[derive(Debug, Clone)]
pub struct CommitteeHealth {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub coordinator_stats: CoordinatorStats,
    pub committee_stats: CommitteeStats,
}

/// Builder for creating customized committees
pub struct CommitteeBuilder {
    config: CommitteeConfig,
    agents: Vec<CommitteeAgent>,
    rubric: Option<EvaluationRubric>,
    scoring_weights: Option<ScoringWeights>,
}

impl CommitteeBuilder {
    /// Create new committee builder
    pub fn new() -> Self {
        Self {
            config: CommitteeConfig::default(),
            agents: Vec::new(),
            rubric: None,
            scoring_weights: None,
        }
    }

    /// Set committee configuration
    pub fn with_config(mut self, config: CommitteeConfig) -> Self {
        self.config = config;
        self
    }

    /// Add agent to committee
    pub fn with_agent(mut self, agent: CommitteeAgent) -> Self {
        self.agents.push(agent);
        self
    }

    /// Add multiple agents
    pub fn with_agents(mut self, agents: Vec<CommitteeAgent>) -> Self {
        self.agents.extend(agents);
        self
    }

    /// Set evaluation rubric
    pub fn with_rubric(mut self, rubric: EvaluationRubric) -> Self {
        self.rubric = Some(rubric);
        self
    }

    /// Set scoring weights
    pub fn with_scoring_weights(mut self, weights: ScoringWeights) -> Self {
        self.scoring_weights = Some(weights);
        self
    }

    /// Add default agents for common perspectives
    pub fn with_default_agents(mut self) -> Self {
        self.agents = AgentFactory::create_default_committee();
        self
    }

    /// Add specialized agents for specific domain
    pub fn with_specialized_agents(mut self, domain: &str) -> Self {
        self.agents = AgentFactory::create_specialized_committee(domain);
        self
    }

    /// Build the committee coordinator
    pub async fn build(self, user_objective: &str) -> Result<CommitteeCoordinator, CognitiveError> {
        let (coordinator_tx, coordinator_rx) = mpsc::channel(100);
        
        let spec = OptimizationSpec::default();
        let rubric = self.rubric.unwrap_or_else(|| {
            EvaluationRubric::from_spec(&spec, user_objective)
        });

        let mut committee = EvaluationCommittee::with_config(
            self.config.clone(),
            &spec,
            user_objective,
            coordinator_tx,
        );

        // Update committee with custom settings
        committee.update_rubric(rubric);

        // Add custom agents if provided
        for agent in self.agents {
            committee.add_agent(agent);
        }

        let stats = CoordinatorStats::default();

        Ok(CommitteeCoordinator {
            committee,
            coordinator_rx,
            stats,
        })
    }
}

impl Default for CommitteeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for committee management
pub mod utils {
    use super::*;

    /// Create a performance-focused committee
    pub async fn create_performance_committee(user_objective: &str) -> Result<CommitteeCoordinator, CognitiveError> {
        CommitteeBuilder::new()
            .with_specialized_agents("performance")
            .with_config(CommitteeConfig {
                consensus_threshold: 0.8,
                max_rounds: 2,
                ..Default::default()
            })
            .build(user_objective)
            .await
    }

    /// Create a security-focused committee
    pub async fn create_security_committee(user_objective: &str) -> Result<CommitteeCoordinator, CognitiveError> {
        CommitteeBuilder::new()
            .with_specialized_agents("security")
            .with_config(CommitteeConfig {
                consensus_threshold: 0.9,
                require_unanimous: true,
                ..Default::default()
            })
            .build(user_objective)
            .await
    }

    /// Create a balanced committee for general use
    pub async fn create_balanced_committee(user_objective: &str) -> Result<CommitteeCoordinator, CognitiveError> {
        CommitteeBuilder::new()
            .with_default_agents()
            .build(user_objective)
            .await
    }

    /// Analyze committee performance and suggest improvements
    pub fn analyze_committee_performance(health: &CommitteeHealth) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !health.healthy {
            for issue in &health.issues {
                if issue.contains("success rate") {
                    suggestions.push("Consider adjusting consensus threshold or agent configuration".to_string());
                } else if issue.contains("response time") {
                    suggestions.push("Consider reducing timeout or increasing parallelism".to_string());
                } else if issue.contains("consensus rate") {
                    suggestions.push("Consider reviewing evaluation rubric or agent perspectives".to_string());
                }
            }
        }

        if health.committee_stats.unanimous_decisions == 0 && health.committee_stats.total_evaluations > 10 {
            suggestions.push("Consider reviewing agent diversity or evaluation criteria".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("Committee is performing well".to_string());
        }

        suggestions
    }
}

/// Re-export utilities for convenience
pub use utils::*;
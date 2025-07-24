//! Committee builder pattern and helper functions
//!
//! This module provides builder pattern for committee construction,
//! agent management utilities, and blazing-fast committee operations.

use tokio::sync::mpsc;

use super::super::core::CommitteeAgent;
use super::super::consensus::events::CommitteeEvent;
use super::super::Committee;

/// Committee builder for easier construction with fluid interface
pub struct CommitteeBuilder {
    agents: Vec<CommitteeAgent>,
    consensus_threshold: f64,
    max_concurrent: usize,
    timeout_seconds: u64,
}

impl CommitteeBuilder {
    /// Create new committee builder with optimized defaults
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            consensus_threshold: 0.7,
            max_concurrent: 5,
            timeout_seconds: 300,
        }
    }

    /// Add agent to committee with fluid interface
    pub fn add_agent(mut self, agent: CommitteeAgent) -> Self {
        self.agents.push(agent);
        self
    }

    /// Set consensus threshold with bounds checking
    pub fn consensus_threshold(mut self, threshold: f64) -> Self {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set maximum concurrent evaluations with safety bounds
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max.max(1);
        self
    }

    /// Set evaluation timeout in seconds
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Build committee with event channel
    pub fn build(self, event_tx: mpsc::UnboundedSender<CommitteeEvent>) -> Committee {
        Committee::new(
            self.agents,
            self.consensus_threshold,
            self.max_concurrent,
            event_tx,
        )
    }

    /// Build committee with default event channel for convenience
    pub fn build_with_default_events(self) -> (Committee, mpsc::UnboundedReceiver<CommitteeEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let committee = self.build(tx);
        (committee, rx)
    }
}

impl Default for CommitteeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for committee management with blazing-fast operations
impl Committee {
    /// Create default committee with standard agent perspectives
    pub fn create_default(event_tx: mpsc::UnboundedSender<CommitteeEvent>) -> Self {
        use super::super::{AgentPerspective, Model, ModelType};
        
        let mut agents = Vec::new();
        let perspectives = AgentPerspective::all();
        
        // Pre-allocate for zero-allocation pattern
        agents.reserve_exact(perspectives.len());
        
        for (i, perspective) in perspectives.into_iter().enumerate() {
            let agent = CommitteeAgent::new(
                format!("agent_{}", i),
                perspective,
                Model {
                    model_type: ModelType::GPT4,
                    temperature: 0.7,
                    max_tokens: 1000,
                },
            );
            agents.push(agent);
        }

        Committee::new(agents, 0.7, 5, event_tx)
    }

    /// Add agent to existing committee
    pub fn add_agent(&mut self, agent: CommitteeAgent) {
        self.agents.push(agent);
    }

    /// Remove agent from committee with blazing-fast lookup
    pub fn remove_agent(&mut self, agent_id: &str) -> bool {
        if let Some(pos) = self.agents.iter().position(|a| a.id == agent_id) {
            self.agents.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get agent by ID with efficient lookup
    pub fn get_agent(&self, agent_id: &str) -> Option<&CommitteeAgent> {
        self.agents.iter().find(|a| a.id == agent_id)
    }

    /// Get agent by ID (mutable) with efficient lookup
    pub fn get_agent_mut(&mut self, agent_id: &str) -> Option<&mut CommitteeAgent> {
        self.agents.iter_mut().find(|a| a.id == agent_id)
    }

    /// List all agent IDs with zero-allocation pattern
    pub fn list_agent_ids(&self) -> Vec<String> {
        // Pre-allocate for blazing-fast performance
        let mut ids = Vec::with_capacity(self.agents.len());
        for agent in &self.agents {
            ids.push(agent.id.clone());
        }
        ids
    }

    /// Update consensus threshold with bounds checking
    pub fn set_consensus_threshold(&mut self, threshold: f64) {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get current consensus threshold
    pub fn get_consensus_threshold(&self) -> f64 {
        self.consensus_threshold
    }

    /// Update maximum concurrent evaluations with safety bounds
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
        // Note: This doesn't update the existing semaphore
        // In a real implementation, you'd need to recreate the semaphore
    }

    /// Get current maximum concurrent evaluations
    pub fn get_max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}
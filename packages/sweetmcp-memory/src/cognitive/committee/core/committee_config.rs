//! Committee configuration with builder pattern
//!
//! This module provides the CommitteeConfig structure with
//! flexible configuration options and optimized defaults.

/// Committee configuration
#[derive(Debug, Clone)]
pub struct CommitteeConfig {
    pub max_agents: usize,
    pub consensus_threshold: f64,
    pub max_rounds: usize,
    pub timeout_seconds: u64,
    pub require_unanimous: bool,
    pub weight_by_reliability: bool,
}

impl Default for CommitteeConfig {
    fn default() -> Self {
        Self {
            max_agents: 7,
            consensus_threshold: 0.7,
            max_rounds: 3,
            timeout_seconds: 300,
            require_unanimous: false,
            weight_by_reliability: true,
        }
    }
}

impl CommitteeConfig {
    /// Create new committee config with optimized defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of agents
    pub fn with_max_agents(mut self, max_agents: usize) -> Self {
        self.max_agents = max_agents;
        self
    }

    /// Set consensus threshold
    pub fn with_consensus_threshold(mut self, threshold: f64) -> Self {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set maximum rounds
    pub fn with_max_rounds(mut self, rounds: usize) -> Self {
        self.max_rounds = rounds;
        self
    }

    /// Set timeout in seconds
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Require unanimous decision
    pub fn with_unanimous_requirement(mut self, require: bool) -> Self {
        self.require_unanimous = require;
        self
    }

    /// Enable reliability weighting
    pub fn with_reliability_weighting(mut self, enable: bool) -> Self {
        self.weight_by_reliability = enable;
        self
    }
}
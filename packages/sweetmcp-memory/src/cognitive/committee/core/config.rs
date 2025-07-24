//! Committee configuration and setup
//!
//! This module provides configuration management for the committee-based
//! evaluation system with validation and optimization for blazing-fast performance.

use super::agent_config::AgentConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for committee-based evaluation
/// 
/// Defines the parameters and settings for how the committee operates,
/// including agent configuration, consensus thresholds, and performance tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeConfig {
    /// Number of agents in the committee
    pub agent_count: usize,
    /// Minimum consensus threshold (0.0 to 1.0)
    pub consensus_threshold: f64,
    /// Maximum number of evaluation rounds
    pub max_rounds: usize,
    /// Timeout for individual agent evaluations (in milliseconds)
    pub agent_timeout_ms: u64,
    /// Whether to enable parallel agent execution
    pub parallel_execution: bool,
    /// Weight for objective alignment in scoring (0.0 to 1.0)
    pub alignment_weight: f64,
    /// Weight for implementation quality in scoring (0.0 to 1.0)
    pub quality_weight: f64,
    /// Weight for risk assessment in scoring (0.0 to 1.0)
    pub risk_weight: f64,
    /// Custom agent configurations
    pub agent_configs: HashMap<String, AgentConfig>,
}

impl CommitteeConfig {
    /// Create a new committee configuration with default values
    /// 
    /// # Returns
    /// CommitteeConfig with production-ready defaults
    pub fn new() -> Self {
        Self {
            agent_count: 5,
            consensus_threshold: 0.6,
            max_rounds: 3,
            agent_timeout_ms: 5000,
            parallel_execution: true,
            alignment_weight: 0.4,
            quality_weight: 0.3,
            risk_weight: 0.3,
            agent_configs: HashMap::new(),
        }
    }

    /// Create a fast configuration optimized for speed
    /// 
    /// # Returns
    /// CommitteeConfig optimized for minimal latency
    pub fn fast() -> Self {
        Self {
            agent_count: 3,
            consensus_threshold: 0.7,
            max_rounds: 2,
            agent_timeout_ms: 2000,
            parallel_execution: true,
            alignment_weight: 0.5,
            quality_weight: 0.3,
            risk_weight: 0.2,
            agent_configs: HashMap::new(),
        }
    }

    /// Create a thorough configuration for comprehensive evaluation
    /// 
    /// # Returns
    /// CommitteeConfig optimized for thorough analysis
    pub fn thorough() -> Self {
        Self {
            agent_count: 7,
            consensus_threshold: 0.5,
            max_rounds: 5,
            agent_timeout_ms: 10000,
            parallel_execution: true,
            alignment_weight: 0.35,
            quality_weight: 0.35,
            risk_weight: 0.3,
            agent_configs: HashMap::new(),
        }
    }

    /// Validate the configuration parameters
    /// 
    /// Ensures all configuration values are within acceptable ranges
    /// and the configuration is internally consistent.
    /// 
    /// # Returns
    /// Result indicating validation success or error message
    pub fn validate(&self) -> Result<(), String> {
        if self.agent_count == 0 {
            return Err("Agent count must be greater than 0".to_string());
        }

        if self.agent_count > 20 {
            return Err("Agent count should not exceed 20 for performance reasons".to_string());
        }

        if !(0.0..=1.0).contains(&self.consensus_threshold) {
            return Err("Consensus threshold must be between 0.0 and 1.0".to_string());
        }

        if self.max_rounds == 0 {
            return Err("Max rounds must be greater than 0".to_string());
        }

        if self.max_rounds > 10 {
            return Err("Max rounds should not exceed 10 for performance reasons".to_string());
        }

        if self.agent_timeout_ms == 0 {
            return Err("Agent timeout must be greater than 0".to_string());
        }

        // Validate weight normalization
        let total_weight = self.alignment_weight + self.quality_weight + self.risk_weight;
        if (total_weight - 1.0).abs() > 0.01 {
            return Err("Scoring weights must sum to approximately 1.0".to_string());
        }

        if !(0.0..=1.0).contains(&self.alignment_weight) ||
           !(0.0..=1.0).contains(&self.quality_weight) ||
           !(0.0..=1.0).contains(&self.risk_weight) {
            return Err("All weights must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }

    /// Normalize the scoring weights to sum to 1.0
    /// 
    /// Adjusts the weights proportionally so they sum to exactly 1.0
    /// while maintaining their relative proportions.
    pub fn normalize_weights(&mut self) {
        let total = self.alignment_weight + self.quality_weight + self.risk_weight;
        if total > 0.0 {
            self.alignment_weight /= total;
            self.quality_weight /= total;
            self.risk_weight /= total;
        }
    }

    /// Set agent count with validation
    /// 
    /// # Arguments
    /// * `count` - Number of agents (will be clamped to reasonable range)
    pub fn set_agent_count(&mut self, count: usize) {
        self.agent_count = count.clamp(1, 20);
    }

    /// Set consensus threshold with validation
    /// 
    /// # Arguments
    /// * `threshold` - Consensus threshold (will be clamped to 0.0-1.0)
    pub fn set_consensus_threshold(&mut self, threshold: f64) {
        self.consensus_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Add or update an agent configuration
    /// 
    /// # Arguments
    /// * `agent_id` - Unique identifier for the agent
    /// * `config` - Configuration for the agent
    pub fn set_agent_config(&mut self, agent_id: String, config: AgentConfig) {
        self.agent_configs.insert(agent_id, config);
    }

    /// Get agent configuration by ID
    /// 
    /// # Arguments
    /// * `agent_id` - Unique identifier for the agent
    /// 
    /// # Returns
    /// Agent configuration if found, default configuration otherwise
    pub fn get_agent_config(&self, agent_id: &str) -> AgentConfig {
        self.agent_configs.get(agent_id).cloned().unwrap_or_default()
    }

    /// Remove agent configuration
    /// 
    /// # Arguments
    /// * `agent_id` - Unique identifier for the agent to remove
    /// 
    /// # Returns
    /// Removed configuration if it existed
    pub fn remove_agent_config(&mut self, agent_id: &str) -> Option<AgentConfig> {
        self.agent_configs.remove(agent_id)
    }

    /// Get all configured agent IDs
    /// 
    /// # Returns
    /// Vector of all agent IDs with custom configurations
    pub fn configured_agent_ids(&self) -> Vec<String> {
        self.agent_configs.keys().cloned().collect()
    }

    /// Calculate expected evaluation time in milliseconds
    /// 
    /// Estimates the total time for committee evaluation based on configuration.
    /// 
    /// # Returns
    /// Estimated evaluation time in milliseconds
    pub fn estimated_evaluation_time_ms(&self) -> u64 {
        if self.parallel_execution {
            // Parallel execution: max agent timeout * max rounds
            self.agent_timeout_ms * self.max_rounds as u64
        } else {
            // Sequential execution: sum of all agent timeouts * max rounds
            self.agent_timeout_ms * self.agent_count as u64 * self.max_rounds as u64
        }
    }

    /// Get memory usage estimate for the configuration
    /// 
    /// Estimates memory usage based on agent count and configuration complexity.
    /// 
    /// # Returns
    /// Estimated memory usage in bytes
    pub fn estimated_memory_usage(&self) -> usize {
        // Base memory for committee structure
        let base_size = std::mem::size_of::<Self>();
        
        // Memory for agent configurations
        let agent_config_size = self.agent_configs.len() * std::mem::size_of::<AgentConfig>();
        
        // Memory for evaluation results (estimated)
        let evaluation_size = self.agent_count * self.max_rounds * 1024; // 1KB per evaluation
        
        base_size + agent_config_size + evaluation_size
    }

    /// Create a configuration optimized for memory usage
    /// 
    /// # Returns
    /// CommitteeConfig optimized for minimal memory footprint
    pub fn memory_optimized() -> Self {
        Self {
            agent_count: 3,
            consensus_threshold: 0.6,
            max_rounds: 2,
            agent_timeout_ms: 3000,
            parallel_execution: true,
            alignment_weight: 0.4,
            quality_weight: 0.3,
            risk_weight: 0.3,
            agent_configs: HashMap::new(),
        }
    }

    /// Check if configuration is suitable for high-load scenarios
    /// 
    /// # Returns
    /// true if configuration can handle high load, false otherwise
    pub fn is_high_load_suitable(&self) -> bool {
        self.parallel_execution &&
        self.agent_count <= 10 &&
        self.max_rounds <= 5 &&
        self.agent_timeout_ms <= 10000
    }

    /// Get configuration summary for logging
    /// 
    /// # Returns
    /// Human-readable summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "Committee: {} agents, {:.1}% consensus, {} rounds max, {}ms timeout, parallel={}",
            self.agent_count,
            self.consensus_threshold * 100.0,
            self.max_rounds,
            self.agent_timeout_ms,
            self.parallel_execution
        )
    }
}

impl Default for CommitteeConfig {
    fn default() -> Self {
        Self::new()
    }
}
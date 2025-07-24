//! Agent configuration and specialization types
//!
//! This module provides configuration structures for individual agents
//! within the committee evaluation system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for individual agents
/// 
/// Defines specialized settings and behavior for specific agents
/// within the committee evaluation system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent specialization type
    pub specialization: AgentSpecialization,
    /// Custom timeout for this agent (overrides committee default)
    pub timeout_ms: Option<u64>,
    /// Weight multiplier for this agent's evaluation (default 1.0)
    pub weight_multiplier: f64,
    /// Whether this agent can veto decisions
    pub can_veto: bool,
    /// Custom parameters for the agent
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl AgentConfig {
    /// Create a new agent configuration
    /// 
    /// # Arguments
    /// * `specialization` - The agent's area of specialization
    /// 
    /// # Returns
    /// New AgentConfig with default settings
    pub fn new(specialization: AgentSpecialization) -> Self {
        Self {
            specialization,
            timeout_ms: None,
            weight_multiplier: 1.0,
            can_veto: false,
            custom_params: HashMap::new(),
        }
    }

    /// Set custom parameter for the agent
    /// 
    /// # Arguments
    /// * `key` - Parameter name
    /// * `value` - Parameter value
    pub fn set_param(&mut self, key: String, value: serde_json::Value) {
        self.custom_params.insert(key, value);
    }

    /// Get custom parameter value
    /// 
    /// # Arguments
    /// * `key` - Parameter name
    /// 
    /// # Returns
    /// Parameter value if found, None otherwise
    pub fn get_param(&self, key: &str) -> Option<&serde_json::Value> {
        self.custom_params.get(key)
    }

    /// Enable veto power for this agent
    /// 
    /// Allows this agent to block committee decisions even if
    /// consensus threshold is otherwise met.
    pub fn enable_veto(&mut self) {
        self.can_veto = true;
    }

    /// Set weight multiplier with validation
    /// 
    /// # Arguments
    /// * `multiplier` - Weight multiplier (will be clamped to positive values)
    pub fn set_weight_multiplier(&mut self, multiplier: f64) {
        self.weight_multiplier = multiplier.max(0.0);
    }

    /// Set custom timeout with validation
    /// 
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds (None to use committee default)
    pub fn set_timeout(&mut self, timeout_ms: Option<u64>) {
        self.timeout_ms = timeout_ms;
    }

    /// Get effective timeout considering agent and committee settings
    /// 
    /// # Arguments
    /// * `committee_timeout` - Default timeout from committee configuration
    /// 
    /// # Returns
    /// Effective timeout in milliseconds
    pub fn effective_timeout(&self, committee_timeout: u64) -> u64 {
        self.timeout_ms.unwrap_or(committee_timeout)
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new(AgentSpecialization::General)
    }
}

/// Agent specialization types
/// 
/// Defines the different areas of expertise that agents can specialize in
/// for more targeted and effective evaluation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentSpecialization {
    /// General purpose evaluation
    General,
    /// Focus on objective alignment
    Alignment,
    /// Focus on implementation quality
    Quality,
    /// Focus on risk and safety assessment
    Safety,
    /// Focus on performance optimization
    Performance,
    /// Focus on security considerations
    Security,
    /// Focus on maintainability and code quality
    Maintainability,
}

impl AgentSpecialization {
    /// Get the display name for the specialization
    /// 
    /// # Returns
    /// Human-readable name for the specialization
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::General => "General Evaluation",
            Self::Alignment => "Objective Alignment",
            Self::Quality => "Implementation Quality",
            Self::Safety => "Risk & Safety",
            Self::Performance => "Performance Optimization",
            Self::Security => "Security Assessment",
            Self::Maintainability => "Code Maintainability",
        }
    }

    /// Get the primary focus area for scoring
    /// 
    /// # Returns
    /// The scoring dimension this specialization focuses on
    pub fn primary_focus(&self) -> ScoringDimension {
        match self {
            Self::General => ScoringDimension::Overall,
            Self::Alignment => ScoringDimension::Alignment,
            Self::Quality | Self::Maintainability => ScoringDimension::Quality,
            Self::Safety | Self::Security => ScoringDimension::Risk,
            Self::Performance => ScoringDimension::Quality,
        }
    }

    /// Get all available specializations
    /// 
    /// # Returns
    /// Vector of all specialization types
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::Alignment,
            Self::Quality,
            Self::Safety,
            Self::Performance,
            Self::Security,
            Self::Maintainability,
        ]
    }

    /// Get recommended weight multiplier for this specialization
    /// 
    /// # Returns
    /// Suggested weight multiplier based on specialization importance
    pub fn recommended_weight(&self) -> f64 {
        match self {
            Self::General => 1.0,
            Self::Alignment => 1.2,  // Higher weight for alignment specialists
            Self::Quality => 1.1,
            Self::Safety => 1.3,     // Highest weight for safety specialists
            Self::Performance => 1.0,
            Self::Security => 1.2,
            Self::Maintainability => 0.9,
        }
    }
}

/// Scoring dimensions for evaluation
/// 
/// Represents the different aspects that can be evaluated and scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoringDimension {
    /// Overall combined score
    Overall,
    /// Objective alignment score
    Alignment,
    /// Implementation quality score
    Quality,
    /// Risk assessment score
    Risk,
}

impl ScoringDimension {
    /// Get the display name for the scoring dimension
    /// 
    /// # Returns
    /// Human-readable name for the dimension
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Overall => "Overall Score",
            Self::Alignment => "Objective Alignment",
            Self::Quality => "Implementation Quality",
            Self::Risk => "Risk Assessment",
        }
    }

    /// Get all scoring dimensions
    /// 
    /// # Returns
    /// Vector of all scoring dimensions
    pub fn all() -> Vec<Self> {
        vec![Self::Overall, Self::Alignment, Self::Quality, Self::Risk]
    }
}
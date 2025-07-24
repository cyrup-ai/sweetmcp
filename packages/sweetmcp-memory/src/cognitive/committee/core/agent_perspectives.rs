//! Agent perspective definitions for committee evaluation
//!
//! This module provides the AgentPerspective enum with specialized
//! evaluation criteria and focus areas for different viewpoints.

use serde::{Deserialize, Serialize};

/// Agent perspective enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentPerspective {
    Performance,
    Security,
    Maintainability,
    UserExperience,
    Architecture,
    Testing,
    Documentation,
}

impl AgentPerspective {
    /// Get all available perspectives
    pub fn all() -> Vec<Self> {
        vec![
            Self::Performance,
            Self::Security,
            Self::Maintainability,
            Self::UserExperience,
            Self::Architecture,
            Self::Testing,
            Self::Documentation,
        ]
    }

    /// Get perspective weight for scoring
    pub fn weight(&self) -> f64 {
        match self {
            Self::Performance => 1.2,
            Self::Security => 1.3,
            Self::Maintainability => 1.0,
            Self::UserExperience => 1.1,
            Self::Architecture => 1.0,
            Self::Testing => 0.9,
            Self::Documentation => 0.8,
        }
    }

    /// Get focus areas for this perspective
    pub fn focus_areas(&self) -> Vec<String> {
        match self {
            Self::Performance => vec![
                "Execution speed".to_string(),
                "Memory usage".to_string(),
                "Scalability".to_string(),
                "Resource efficiency".to_string(),
            ],
            Self::Security => vec![
                "Vulnerability assessment".to_string(),
                "Input validation".to_string(),
                "Access control".to_string(),
                "Data protection".to_string(),
            ],
            Self::Maintainability => vec![
                "Code clarity".to_string(),
                "Modularity".to_string(),
                "Refactoring ease".to_string(),
                "Technical debt".to_string(),
            ],
            Self::UserExperience => vec![
                "Usability".to_string(),
                "Accessibility".to_string(),
                "Error handling".to_string(),
                "User feedback".to_string(),
            ],
            Self::Architecture => vec![
                "System design".to_string(),
                "Component coupling".to_string(),
                "Extensibility".to_string(),
                "Design patterns".to_string(),
            ],
            Self::Testing => vec![
                "Test coverage".to_string(),
                "Test quality".to_string(),
                "Testability".to_string(),
                "Regression prevention".to_string(),
            ],        
            Self::Documentation => vec![
                "Code comments".to_string(),
                "API documentation".to_string(),
                "Usage examples".to_string(),
                "Clarity of intent".to_string(),
            ],
        }
    }
}
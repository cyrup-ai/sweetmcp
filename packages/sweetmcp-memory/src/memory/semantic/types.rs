//! Core semantic memory types and enumerations
//!
//! This module provides the fundamental types used throughout the semantic memory system,
//! including confidence levels, item types, and relationship types.

use serde::{Deserialize, Serialize};
use crate::utils::{Result, error::Error};

/// Confidence level enum for semantic items and relationships
/// 
/// Represents the confidence level associated with semantic information,
/// providing a standardized way to express certainty about knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Very low confidence (0.1)
    VeryLow,
    /// Low confidence (0.3)
    Low,
    /// Medium confidence (0.5)
    Medium,
    /// High confidence (0.7)
    High,
    /// Very high confidence (0.9)
    VeryHigh,
}

impl ConfidenceLevel {
    /// Convert confidence level to float value
    /// 
    /// # Returns
    /// Float value between 0.1 and 0.9 representing confidence
    pub fn to_float(&self) -> f32 {
        match self {
            ConfidenceLevel::VeryLow => 0.1,
            ConfidenceLevel::Low => 0.3,
            ConfidenceLevel::Medium => 0.5,
            ConfidenceLevel::High => 0.7,
            ConfidenceLevel::VeryHigh => 0.9,
        }
    }

    /// Convert float value to confidence level
    /// 
    /// # Arguments
    /// * `value` - Float value to convert
    /// 
    /// # Returns
    /// Corresponding ConfidenceLevel
    pub fn from_float(value: f32) -> Self {
        if value < 0.2 {
            ConfidenceLevel::VeryLow
        } else if value < 0.4 {
            ConfidenceLevel::Low
        } else if value < 0.6 {
            ConfidenceLevel::Medium
        } else if value < 0.8 {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::VeryHigh
        }
    }

    /// Get the display name for the confidence level
    /// 
    /// # Returns
    /// String representation of the confidence level
    pub fn display_name(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryLow => "Very Low",
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::VeryHigh => "Very High",
        }
    }

    /// Check if confidence level is above a threshold
    /// 
    /// # Arguments
    /// * `threshold` - Threshold confidence level
    /// 
    /// # Returns
    /// True if this confidence level is above the threshold
    pub fn is_above(&self, threshold: ConfidenceLevel) -> bool {
        self >= &threshold
    }

    /// Check if confidence level is below a threshold
    /// 
    /// # Arguments
    /// * `threshold` - Threshold confidence level
    /// 
    /// # Returns
    /// True if this confidence level is below the threshold
    pub fn is_below(&self, threshold: ConfidenceLevel) -> bool {
        self < &threshold
    }
}

/// Semantic item type enumeration
/// 
/// Defines the different types of semantic items that can be stored
/// in the semantic memory system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticItemType {
    /// Concept (entity, object, idea)
    Concept,
    /// Fact (statement about concepts)
    Fact,
    /// Rule (logical rule or pattern)
    Rule,
    /// Category (classification or grouping)
    Category,
}

impl SemanticItemType {
    /// Convert semantic item type to string
    /// 
    /// # Returns
    /// String representation of the item type
    pub fn to_string(&self) -> String {
        match self {
            SemanticItemType::Concept => "concept".to_string(),
            SemanticItemType::Fact => "fact".to_string(),
            SemanticItemType::Rule => "rule".to_string(),
            SemanticItemType::Category => "category".to_string(),
        }
    }

    /// Convert string to semantic item type
    /// 
    /// # Arguments
    /// * `s` - String to convert
    /// 
    /// # Returns
    /// Result containing the SemanticItemType or error
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "concept" => Ok(SemanticItemType::Concept),
            "fact" => Ok(SemanticItemType::Fact),
            "rule" => Ok(SemanticItemType::Rule),
            "category" => Ok(SemanticItemType::Category),
            _ => Err(Error::ConversionError(format!(
                "Invalid semantic item type: {}",
                s
            ))),
        }
    }

    /// Get the display name for the item type
    /// 
    /// # Returns
    /// String representation suitable for display
    pub fn display_name(&self) -> &'static str {
        match self {
            SemanticItemType::Concept => "Concept",
            SemanticItemType::Fact => "Fact",
            SemanticItemType::Rule => "Rule",
            SemanticItemType::Category => "Category",
        }
    }

    /// Check if the item type represents knowledge
    /// 
    /// # Returns
    /// True if the item type represents factual knowledge
    pub fn is_knowledge(&self) -> bool {
        matches!(self, SemanticItemType::Fact | SemanticItemType::Rule)
    }

    /// Check if the item type represents structure
    /// 
    /// # Returns
    /// True if the item type represents structural information
    pub fn is_structural(&self) -> bool {
        matches!(self, SemanticItemType::Category)
    }

    /// Get priority weight for the item type
    /// 
    /// # Returns
    /// Float value representing the priority weight
    pub fn priority_weight(&self) -> f32 {
        match self {
            SemanticItemType::Concept => 0.8,
            SemanticItemType::Fact => 0.9,
            SemanticItemType::Rule => 0.7,
            SemanticItemType::Category => 0.6,
        }
    }
}

/// Semantic relationship type enumeration
/// 
/// Defines the different types of relationships that can exist
/// between semantic items in the memory system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRelationshipType {
    /// Is-a relationship (inheritance)
    IsA,
    /// Has-a relationship (composition)
    HasA,
    /// Part-of relationship
    PartOf,
    /// Related-to relationship
    RelatedTo,
    /// Causes relationship
    Causes,
    /// Custom relationship with specific name
    Custom(String),
}

impl SemanticRelationshipType {
    /// Convert relationship type to string
    /// 
    /// # Returns
    /// String representation of the relationship type
    pub fn to_string(&self) -> String {
        match self {
            SemanticRelationshipType::IsA => "is_a".to_string(),
            SemanticRelationshipType::HasA => "has_a".to_string(),
            SemanticRelationshipType::PartOf => "part_of".to_string(),
            SemanticRelationshipType::RelatedTo => "related_to".to_string(),
            SemanticRelationshipType::Causes => "causes".to_string(),
            SemanticRelationshipType::Custom(name) => name.clone(),
        }
    }

    /// Convert string to relationship type
    /// 
    /// # Arguments
    /// * `s` - String to convert
    /// 
    /// # Returns
    /// SemanticRelationshipType corresponding to the string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "is_a" => SemanticRelationshipType::IsA,
            "has_a" => SemanticRelationshipType::HasA,
            "part_of" => SemanticRelationshipType::PartOf,
            "related_to" => SemanticRelationshipType::RelatedTo,
            "causes" => SemanticRelationshipType::Causes,
            _ => SemanticRelationshipType::Custom(s.to_string()),
        }
    }

    /// Get the display name for the relationship type
    /// 
    /// # Returns
    /// String representation suitable for display
    pub fn display_name(&self) -> String {
        match self {
            SemanticRelationshipType::IsA => "Is A".to_string(),
            SemanticRelationshipType::HasA => "Has A".to_string(),
            SemanticRelationshipType::PartOf => "Part Of".to_string(),
            SemanticRelationshipType::RelatedTo => "Related To".to_string(),
            SemanticRelationshipType::Causes => "Causes".to_string(),
            SemanticRelationshipType::Custom(name) => name.clone(),
        }
    }

    /// Check if the relationship type is hierarchical
    /// 
    /// # Returns
    /// True if the relationship represents a hierarchy
    pub fn is_hierarchical(&self) -> bool {
        matches!(self, SemanticRelationshipType::IsA | SemanticRelationshipType::PartOf)
    }

    /// Check if the relationship type is compositional
    /// 
    /// # Returns
    /// True if the relationship represents composition
    pub fn is_compositional(&self) -> bool {
        matches!(self, SemanticRelationshipType::HasA | SemanticRelationshipType::PartOf)
    }

    /// Check if the relationship type is causal
    /// 
    /// # Returns
    /// True if the relationship represents causation
    pub fn is_causal(&self) -> bool {
        matches!(self, SemanticRelationshipType::Causes)
    }

    /// Get the inverse relationship type if applicable
    /// 
    /// # Returns
    /// Option containing the inverse relationship type
    pub fn inverse(&self) -> Option<Self> {
        match self {
            SemanticRelationshipType::IsA => None, // No clear inverse
            SemanticRelationshipType::HasA => Some(SemanticRelationshipType::PartOf),
            SemanticRelationshipType::PartOf => Some(SemanticRelationshipType::HasA),
            SemanticRelationshipType::RelatedTo => Some(SemanticRelationshipType::RelatedTo),
            SemanticRelationshipType::Causes => None, // Causation is directional
            SemanticRelationshipType::Custom(_) => None, // Cannot determine inverse for custom
        }
    }

    /// Get strength weight for the relationship type
    /// 
    /// # Returns
    /// Float value representing the relationship strength weight
    pub fn strength_weight(&self) -> f32 {
        match self {
            SemanticRelationshipType::IsA => 0.9,
            SemanticRelationshipType::HasA => 0.8,
            SemanticRelationshipType::PartOf => 0.8,
            SemanticRelationshipType::RelatedTo => 0.6,
            SemanticRelationshipType::Causes => 0.7,
            SemanticRelationshipType::Custom(_) => 0.5,
        }
    }
}
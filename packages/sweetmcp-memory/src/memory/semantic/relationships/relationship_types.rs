//! Semantic relationship types and directions
//!
//! This module provides blazing-fast semantic relationship type definitions with zero allocation
//! optimizations and elegant ergonomic interfaces for relationship type management.

use serde::{Deserialize, Serialize};

/// Semantic relationship type enum with optimized operations
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
    /// Custom relationship with type name
    Custom(String),
}

impl SemanticRelationshipType {
    /// Convert to string with zero allocation for standard types
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            SemanticRelationshipType::IsA => "is_a",
            SemanticRelationshipType::HasA => "has_a",
            SemanticRelationshipType::PartOf => "part_of",
            SemanticRelationshipType::RelatedTo => "related_to",
            SemanticRelationshipType::Causes => "causes",
            SemanticRelationshipType::Custom(name) => name.as_str(),
        }
    }

    /// Convert to string (for compatibility)
    #[inline]
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Convert from string with optimized matching
    #[inline]
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "is_a" | "isa" | "is-a" => SemanticRelationshipType::IsA,
            "has_a" | "hasa" | "has-a" => SemanticRelationshipType::HasA,
            "part_of" | "partof" | "part-of" => SemanticRelationshipType::PartOf,
            "related_to" | "relatedto" | "related-to" => SemanticRelationshipType::RelatedTo,
            "causes" => SemanticRelationshipType::Causes,
            _ => SemanticRelationshipType::Custom(s.to_string()),
        }
    }

    /// Get all standard relationship types
    #[inline]
    pub fn standard_types() -> [SemanticRelationshipType; 5] {
        [
            SemanticRelationshipType::IsA,
            SemanticRelationshipType::HasA,
            SemanticRelationshipType::PartOf,
            SemanticRelationshipType::RelatedTo,
            SemanticRelationshipType::Causes,
        ]
    }

    /// Get relationship type description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            SemanticRelationshipType::IsA => "Inheritance or classification relationship",
            SemanticRelationshipType::HasA => "Composition or ownership relationship",
            SemanticRelationshipType::PartOf => "Part-whole relationship",
            SemanticRelationshipType::RelatedTo => "General association relationship",
            SemanticRelationshipType::Causes => "Causal relationship",
            SemanticRelationshipType::Custom(_) => "Custom relationship type",
        }
    }

    /// Check if relationship is hierarchical
    #[inline]
    pub fn is_hierarchical(&self) -> bool {
        matches!(
            self,
            SemanticRelationshipType::IsA | 
            SemanticRelationshipType::HasA | 
            SemanticRelationshipType::PartOf
        )
    }

    /// Check if relationship is associative
    #[inline]
    pub fn is_associative(&self) -> bool {
        matches!(
            self,
            SemanticRelationshipType::RelatedTo | 
            SemanticRelationshipType::Causes
        )
    }

    /// Check if relationship is custom
    #[inline]
    pub fn is_custom(&self) -> bool {
        matches!(self, SemanticRelationshipType::Custom(_))
    }

    /// Check if relationship is directional
    #[inline]
    pub fn is_directional(&self) -> bool {
        match self {
            SemanticRelationshipType::IsA => true,
            SemanticRelationshipType::HasA => true,
            SemanticRelationshipType::PartOf => true,
            SemanticRelationshipType::Causes => true,
            SemanticRelationshipType::RelatedTo => false,
            SemanticRelationshipType::Custom(_) => true, // Assume custom relationships are directional
        }
    }

    /// Get inverse relationship type
    #[inline]
    pub fn inverse(&self) -> Option<SemanticRelationshipType> {
        match self {
            SemanticRelationshipType::IsA => None, // No direct inverse
            SemanticRelationshipType::HasA => Some(SemanticRelationshipType::PartOf),
            SemanticRelationshipType::PartOf => Some(SemanticRelationshipType::HasA),
            SemanticRelationshipType::RelatedTo => Some(SemanticRelationshipType::RelatedTo), // Symmetric
            SemanticRelationshipType::Causes => None, // No standard inverse
            SemanticRelationshipType::Custom(_) => None, // Custom relationships don't have automatic inverses
        }
    }

    /// Get relationship strength weight (higher = stronger relationship)
    #[inline]
    pub fn strength_weight(&self) -> f32 {
        match self {
            SemanticRelationshipType::IsA => 0.9,      // Strong hierarchical
            SemanticRelationshipType::HasA => 0.8,     // Strong compositional
            SemanticRelationshipType::PartOf => 0.8,   // Strong compositional
            SemanticRelationshipType::Causes => 0.7,   // Strong causal
            SemanticRelationshipType::RelatedTo => 0.5, // Weaker associative
            SemanticRelationshipType::Custom(_) => 0.6, // Medium strength for custom
        }
    }

    /// Get default confidence level for relationship type
    #[inline]
    pub fn default_confidence(&self) -> super::super::confidence::ConfidenceLevel {
        use super::super::confidence::ConfidenceLevel;
        
        match self {
            SemanticRelationshipType::IsA => ConfidenceLevel::High,
            SemanticRelationshipType::HasA => ConfidenceLevel::High,
            SemanticRelationshipType::PartOf => ConfidenceLevel::High,
            SemanticRelationshipType::Causes => ConfidenceLevel::Medium,
            SemanticRelationshipType::RelatedTo => ConfidenceLevel::Medium,
            SemanticRelationshipType::Custom(_) => ConfidenceLevel::Low,
        }
    }

    /// Check if relationship type is compatible with item types
    #[inline]
    pub fn is_compatible_with_item_types(
        &self,
        source_type: &super::super::item_types::SemanticItemType,
        target_type: &super::super::item_types::SemanticItemType,
    ) -> bool {
        use super::super::item_types::SemanticItemType;
        
        match self {
            SemanticRelationshipType::IsA => {
                // IsA relationships: concept -> category, category -> category
                matches!(
                    (source_type, target_type),
                    (SemanticItemType::Concept, SemanticItemType::Category) |
                    (SemanticItemType::Category, SemanticItemType::Category)
                )
            }
            SemanticRelationshipType::HasA => {
                // HasA relationships: concept -> concept, category -> concept
                matches!(
                    (source_type, target_type),
                    (SemanticItemType::Concept, SemanticItemType::Concept) |
                    (SemanticItemType::Category, SemanticItemType::Concept)
                )
            }
            SemanticRelationshipType::PartOf => {
                // PartOf relationships: concept -> concept, concept -> category
                matches!(
                    (source_type, target_type),
                    (SemanticItemType::Concept, SemanticItemType::Concept) |
                    (SemanticItemType::Concept, SemanticItemType::Category)
                )
            }
            SemanticRelationshipType::Causes => {
                // Causes relationships: fact -> fact, rule -> fact
                matches!(
                    (source_type, target_type),
                    (SemanticItemType::Fact, SemanticItemType::Fact) |
                    (SemanticItemType::Rule, SemanticItemType::Fact)
                )
            }
            SemanticRelationshipType::RelatedTo => {
                // RelatedTo relationships: any -> any (most flexible)
                true
            }
            SemanticRelationshipType::Custom(_) => {
                // Custom relationships: allow any combination
                true
            }
        }
    }
}

impl Default for SemanticRelationshipType {
    fn default() -> Self {
        SemanticRelationshipType::RelatedTo
    }
}

impl std::fmt::Display for SemanticRelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SemanticRelationshipType {
    type Err = (); // Never fails, creates Custom for unknown types

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_string(s))
    }
}

/// Relationship direction for navigation and queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipDirection {
    /// Outgoing relationships (from source)
    Outgoing,
    /// Incoming relationships (to target)
    Incoming,
    /// Both directions
    Both,
}

impl RelationshipDirection {
    /// Get all directions
    #[inline]
    pub fn all() -> [RelationshipDirection; 3] {
        [
            RelationshipDirection::Outgoing,
            RelationshipDirection::Incoming,
            RelationshipDirection::Both,
        ]
    }

    /// Check if direction includes outgoing
    #[inline]
    pub fn includes_outgoing(&self) -> bool {
        matches!(self, RelationshipDirection::Outgoing | RelationshipDirection::Both)
    }

    /// Check if direction includes incoming
    #[inline]
    pub fn includes_incoming(&self) -> bool {
        matches!(self, RelationshipDirection::Incoming | RelationshipDirection::Both)
    }
}

impl Default for RelationshipDirection {
    fn default() -> Self {
        RelationshipDirection::Both
    }
}
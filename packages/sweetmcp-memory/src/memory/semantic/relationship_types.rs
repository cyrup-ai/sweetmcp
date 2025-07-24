//! Semantic relationship types and type-specific functionality
//!
//! This module provides the SemanticRelationshipType enum and related functionality
//! for managing different types of semantic relationships with zero allocation,
//! blazing-fast performance, and ergonomic API design.

use serde::{Deserialize, Serialize};

/// Semantic relationship type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRelationshipType {
    /// Is-a relationship (inheritance)
    IsA,
    /// Part-of relationship (composition)
    PartOf,
    /// Related-to relationship (association)
    RelatedTo,
    /// Causes relationship (causation)
    Causes,
    /// Enables relationship (enablement)
    Enables,
    /// Requires relationship (dependency)
    Requires,
    /// Contradicts relationship (contradiction)
    Contradicts,
    /// Similar-to relationship (similarity)
    SimilarTo,
    /// Custom relationship type
    Custom(String),
}

impl SemanticRelationshipType {
    /// Get the string representation of the relationship type
    pub fn as_str(&self) -> &str {
        match self {
            SemanticRelationshipType::IsA => "is_a",
            SemanticRelationshipType::PartOf => "part_of",
            SemanticRelationshipType::RelatedTo => "related_to",
            SemanticRelationshipType::Causes => "causes",
            SemanticRelationshipType::Enables => "enables",
            SemanticRelationshipType::Requires => "requires",
            SemanticRelationshipType::Contradicts => "contradicts",
            SemanticRelationshipType::SimilarTo => "similar_to",
            SemanticRelationshipType::Custom(name) => name,
        }
    }

    /// Create a relationship type from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "is_a" => SemanticRelationshipType::IsA,
            "part_of" => SemanticRelationshipType::PartOf,
            "related_to" => SemanticRelationshipType::RelatedTo,
            "causes" => SemanticRelationshipType::Causes,
            "enables" => SemanticRelationshipType::Enables,
            "requires" => SemanticRelationshipType::Requires,
            "contradicts" => SemanticRelationshipType::Contradicts,
            "similar_to" => SemanticRelationshipType::SimilarTo,
            custom => SemanticRelationshipType::Custom(custom.to_string()),
        }
    }

    /// Check if this is a bidirectional relationship
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self,
            SemanticRelationshipType::RelatedTo
                | SemanticRelationshipType::SimilarTo
                | SemanticRelationshipType::Contradicts
        )
    }

    /// Get the inverse relationship type if applicable
    pub fn inverse(&self) -> Option<Self> {
        match self {
            SemanticRelationshipType::IsA => None, // No clear inverse
            SemanticRelationshipType::PartOf => None, // Could be "has_part" but not standard
            SemanticRelationshipType::RelatedTo => Some(SemanticRelationshipType::RelatedTo),
            SemanticRelationshipType::Causes => None, // Could be "caused_by" but not standard
            SemanticRelationshipType::Enables => None, // Could be "enabled_by" but not standard
            SemanticRelationshipType::Requires => None, // Could be "required_by" but not standard
            SemanticRelationshipType::Contradicts => Some(SemanticRelationshipType::Contradicts),
            SemanticRelationshipType::SimilarTo => Some(SemanticRelationshipType::SimilarTo),
            SemanticRelationshipType::Custom(_) => None, // Cannot determine inverse for custom
        }
    }

    /// Get relationship strength weight for scoring
    pub fn strength_weight(&self) -> f64 {
        match self {
            SemanticRelationshipType::IsA => 0.9,
            SemanticRelationshipType::PartOf => 0.8,
            SemanticRelationshipType::Causes => 0.7,
            SemanticRelationshipType::Requires => 0.7,
            SemanticRelationshipType::Enables => 0.6,
            SemanticRelationshipType::SimilarTo => 0.5,
            SemanticRelationshipType::RelatedTo => 0.4,
            SemanticRelationshipType::Contradicts => 0.3,
            SemanticRelationshipType::Custom(_) => 0.5, // Default for custom
        }
    }

    /// Get all standard relationship types
    pub fn all_standard_types() -> Vec<Self> {
        vec![
            SemanticRelationshipType::IsA,
            SemanticRelationshipType::PartOf,
            SemanticRelationshipType::RelatedTo,
            SemanticRelationshipType::Causes,
            SemanticRelationshipType::Enables,
            SemanticRelationshipType::Requires,
            SemanticRelationshipType::Contradicts,
            SemanticRelationshipType::SimilarTo,
        ]
    }

    /// Check if this is a standard (non-custom) relationship type
    pub fn is_standard(&self) -> bool {
        !matches!(self, SemanticRelationshipType::Custom(_))
    }

    /// Get relationship type category for grouping
    pub fn category(&self) -> &'static str {
        match self {
            SemanticRelationshipType::IsA => "hierarchical",
            SemanticRelationshipType::PartOf => "compositional",
            SemanticRelationshipType::RelatedTo => "associative",
            SemanticRelationshipType::Causes => "causal",
            SemanticRelationshipType::Enables => "causal",
            SemanticRelationshipType::Requires => "dependency",
            SemanticRelationshipType::Contradicts => "logical",
            SemanticRelationshipType::SimilarTo => "similarity",
            SemanticRelationshipType::Custom(_) => "custom",
        }
    }

    /// Get relationship directionality
    pub fn directionality(&self) -> RelationshipDirectionality {
        match self {
            SemanticRelationshipType::IsA => RelationshipDirectionality::Unidirectional,
            SemanticRelationshipType::PartOf => RelationshipDirectionality::Unidirectional,
            SemanticRelationshipType::RelatedTo => RelationshipDirectionality::Bidirectional,
            SemanticRelationshipType::Causes => RelationshipDirectionality::Unidirectional,
            SemanticRelationshipType::Enables => RelationshipDirectionality::Unidirectional,
            SemanticRelationshipType::Requires => RelationshipDirectionality::Unidirectional,
            SemanticRelationshipType::Contradicts => RelationshipDirectionality::Bidirectional,
            SemanticRelationshipType::SimilarTo => RelationshipDirectionality::Bidirectional,
            SemanticRelationshipType::Custom(_) => RelationshipDirectionality::Unknown,
        }
    }

    /// Check if relationship type is transitive
    pub fn is_transitive(&self) -> bool {
        matches!(
            self,
            SemanticRelationshipType::IsA
                | SemanticRelationshipType::PartOf
                | SemanticRelationshipType::SimilarTo
        )
    }

    /// Check if relationship type is symmetric
    pub fn is_symmetric(&self) -> bool {
        matches!(
            self,
            SemanticRelationshipType::RelatedTo
                | SemanticRelationshipType::SimilarTo
                | SemanticRelationshipType::Contradicts
        )
    }

    /// Check if relationship type is reflexive
    pub fn is_reflexive(&self) -> bool {
        matches!(self, SemanticRelationshipType::SimilarTo)
    }

    /// Get relationship type priority for conflict resolution
    pub fn priority(&self) -> u8 {
        match self {
            SemanticRelationshipType::IsA => 10,
            SemanticRelationshipType::PartOf => 9,
            SemanticRelationshipType::Contradicts => 8,
            SemanticRelationshipType::Causes => 7,
            SemanticRelationshipType::Requires => 6,
            SemanticRelationshipType::Enables => 5,
            SemanticRelationshipType::SimilarTo => 4,
            SemanticRelationshipType::RelatedTo => 3,
            SemanticRelationshipType::Custom(_) => 1,
        }
    }

    /// Check if this relationship type can coexist with another
    pub fn can_coexist_with(&self, other: &SemanticRelationshipType) -> bool {
        // Contradicts cannot coexist with most other relationships
        if matches!(self, SemanticRelationshipType::Contradicts) 
            || matches!(other, SemanticRelationshipType::Contradicts) {
            return false;
        }

        // IsA and PartOf can coexist with most relationships
        // SimilarTo can coexist with most relationships
        // RelatedTo is very general and can coexist with most
        // Causal relationships (Causes, Enables, Requires) can generally coexist
        true
    }

    /// Get human-readable description of the relationship type
    pub fn description(&self) -> &'static str {
        match self {
            SemanticRelationshipType::IsA => "represents an inheritance or classification relationship",
            SemanticRelationshipType::PartOf => "represents a compositional or containment relationship",
            SemanticRelationshipType::RelatedTo => "represents a general association or connection",
            SemanticRelationshipType::Causes => "represents a causal relationship where source causes target",
            SemanticRelationshipType::Enables => "represents an enabling relationship where source enables target",
            SemanticRelationshipType::Requires => "represents a dependency where source requires target",
            SemanticRelationshipType::Contradicts => "represents a logical contradiction between items",
            SemanticRelationshipType::SimilarTo => "represents similarity or likeness between items",
            SemanticRelationshipType::Custom(_) => "represents a custom-defined relationship type",
        }
    }
}

/// Relationship directionality enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipDirectionality {
    /// Relationship flows in one direction only
    Unidirectional,
    /// Relationship flows in both directions
    Bidirectional,
    /// Directionality is unknown or undefined
    Unknown,
}

impl RelationshipDirectionality {
    /// Check if directionality allows reverse traversal
    pub fn allows_reverse(&self) -> bool {
        matches!(self, RelationshipDirectionality::Bidirectional)
    }
}
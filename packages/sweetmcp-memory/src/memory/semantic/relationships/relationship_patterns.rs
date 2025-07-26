//! Relationship patterns for advanced querying
//!
//! This module provides blazing-fast relationship pattern matching with zero allocation
//! optimizations and elegant ergonomic interfaces for advanced semantic relationship queries.

use super::relationship_types::{SemanticRelationshipType, RelationshipDirection};
use serde::{Deserialize, Serialize};

/// Relationship pattern for advanced querying
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipPattern {
    /// Allowed relationship types
    pub relationship_types: Vec<SemanticRelationshipType>,
    /// Required direction
    pub direction: RelationshipDirection,
    /// Maximum traversal depth
    pub max_depth: usize,
    /// Minimum confidence level
    pub min_confidence: super::super::confidence::ConfidenceLevel,
    /// Whether to include transitive relationships
    pub include_transitive: bool,
}

impl RelationshipPattern {
    /// Create new relationship pattern
    #[inline]
    pub fn new() -> Self {
        Self {
            relationship_types: Vec::new(),
            direction: RelationshipDirection::Both,
            max_depth: 1,
            min_confidence: super::super::confidence::ConfidenceLevel::Low,
            include_transitive: false,
        }
    }

    /// Add relationship type to pattern
    #[inline]
    pub fn with_type(mut self, relationship_type: SemanticRelationshipType) -> Self {
        self.relationship_types.push(relationship_type);
        self
    }

    /// Set direction filter
    #[inline]
    pub fn with_direction(mut self, direction: RelationshipDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set maximum depth
    #[inline]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set minimum confidence
    #[inline]
    pub fn with_min_confidence(mut self, confidence: super::super::confidence::ConfidenceLevel) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Include transitive relationships
    #[inline]
    pub fn with_transitive(mut self, include: bool) -> Self {
        self.include_transitive = include;
        self
    }

    /// Check if pattern matches a relationship
    #[inline]
    pub fn matches_relationship(
        &self,
        relationship_type: &SemanticRelationshipType,
        confidence: &super::super::confidence::ConfidenceLevel,
        is_outgoing: bool,
    ) -> bool {
        // Check relationship type
        if !self.relationship_types.is_empty() && !self.relationship_types.contains(relationship_type) {
            return false;
        }

        // Check confidence level using float comparison
        if confidence.to_float() < self.min_confidence.to_float() {
            return false;
        }

        // Check direction
        match self.direction {
            RelationshipDirection::Outgoing => is_outgoing,
            RelationshipDirection::Incoming => !is_outgoing,
            RelationshipDirection::Both => true,
        }
    }

    /// Check if pattern is empty (matches everything)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.relationship_types.is_empty() && 
        self.direction == RelationshipDirection::Both &&
        self.max_depth == 1 &&
        self.min_confidence == super::super::confidence::ConfidenceLevel::Low &&
        !self.include_transitive
    }

    /// Get pattern complexity score
    #[inline]
    pub fn complexity_score(&self) -> f32 {
        let mut score = 0.0;
        
        // Base complexity from number of relationship types
        score += self.relationship_types.len() as f32 * 0.1;
        
        // Depth complexity
        score += (self.max_depth as f32 - 1.0) * 0.3;
        
        // Transitive complexity
        if self.include_transitive {
            score += 0.5;
        }
        
        // Confidence filtering complexity
        match self.min_confidence {
            super::super::confidence::ConfidenceLevel::VeryHigh => score += 0.3,
            super::super::confidence::ConfidenceLevel::High => score += 0.2,
            super::super::confidence::ConfidenceLevel::Medium => score += 0.1,
            super::super::confidence::ConfidenceLevel::Low => {},
            super::super::confidence::ConfidenceLevel::VeryLow => {},
        }
        
        score
    }

    /// Create pattern for hierarchical relationships
    #[inline]
    pub fn hierarchical() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::IsA)
            .with_type(SemanticRelationshipType::HasA)
            .with_type(SemanticRelationshipType::PartOf)
            .with_direction(RelationshipDirection::Outgoing)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Medium)
    }

    /// Create pattern for associative relationships
    #[inline]
    pub fn associative() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::RelatedTo)
            .with_type(SemanticRelationshipType::Causes)
            .with_direction(RelationshipDirection::Both)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Low)
    }

    /// Create pattern for transitive closure
    #[inline]
    pub fn transitive_closure(relationship_type: SemanticRelationshipType, max_depth: usize) -> Self {
        Self::new()
            .with_type(relationship_type)
            .with_max_depth(max_depth)
            .with_transitive(true)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Medium)
    }

    /// Create pattern for strong relationships only
    #[inline]
    pub fn strong_relationships() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::IsA)
            .with_type(SemanticRelationshipType::HasA)
            .with_type(SemanticRelationshipType::PartOf)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::High)
    }

    /// Create pattern for causal chains
    #[inline]
    pub fn causal_chain(max_depth: usize) -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::Causes)
            .with_direction(RelationshipDirection::Outgoing)
            .with_max_depth(max_depth)
            .with_transitive(true)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Medium)
    }

    /// Merge with another pattern (union of constraints)
    #[inline]
    pub fn merge_with(mut self, other: &RelationshipPattern) -> Self {
        // Merge relationship types (union)
        for rel_type in &other.relationship_types {
            if !self.relationship_types.contains(rel_type) {
                self.relationship_types.push(rel_type.clone());
            }
        }

        // Use more restrictive direction
        if other.direction != RelationshipDirection::Both {
            if self.direction == RelationshipDirection::Both {
                self.direction = other.direction.clone();
            } else if self.direction != other.direction {
                // Conflicting directions - keep current
            }
        }

        // Use minimum depth
        self.max_depth = self.max_depth.min(other.max_depth);

        // Use higher confidence requirement
        if other.min_confidence > self.min_confidence {
            self.min_confidence = other.min_confidence;
        }

        // Include transitive if either pattern requires it
        self.include_transitive = self.include_transitive || other.include_transitive;

        self
    }

    /// Create intersection with another pattern (more restrictive)
    #[inline]
    pub fn intersect_with(mut self, other: &RelationshipPattern) -> Self {
        // Intersect relationship types
        if !other.relationship_types.is_empty() {
            if self.relationship_types.is_empty() {
                self.relationship_types = other.relationship_types.clone();
            } else {
                self.relationship_types.retain(|t| other.relationship_types.contains(t));
            }
        }

        // Use more restrictive direction
        if other.direction != RelationshipDirection::Both {
            if self.direction == RelationshipDirection::Both {
                self.direction = other.direction.clone();
            } else if self.direction != other.direction {
                // Conflicting directions - no matches possible
                self.relationship_types.clear();
            }
        }

        // Use minimum depth
        self.max_depth = self.max_depth.min(other.max_depth);

        // Use higher confidence requirement
        if other.min_confidence > self.min_confidence {
            self.min_confidence = other.min_confidence;
        }

        // Require transitive only if both patterns require it
        self.include_transitive = self.include_transitive && other.include_transitive;

        self
    }

    /// Check if pattern is satisfiable
    #[inline]
    pub fn is_satisfiable(&self) -> bool {
        // Empty relationship types list means match all types
        !self.relationship_types.is_empty() || self.max_depth > 0
    }

    /// Get estimated result count multiplier
    #[inline]
    pub fn result_multiplier(&self) -> f32 {
        let mut multiplier = 1.0;

        // More relationship types = more results
        if self.relationship_types.is_empty() {
            multiplier *= 5.0; // All types
        } else {
            multiplier *= self.relationship_types.len() as f32;
        }

        // Direction affects result count
        match self.direction {
            RelationshipDirection::Both => multiplier *= 2.0,
            _ => {},
        }

        // Depth increases results exponentially
        multiplier *= (self.max_depth as f32).powf(1.5);

        // Transitive relationships increase results significantly
        if self.include_transitive {
            multiplier *= 3.0;
        }

        // Lower confidence requirements increase results
        match self.min_confidence {
            super::super::confidence::ConfidenceLevel::VeryLow => multiplier *= 3.0,
            super::super::confidence::ConfidenceLevel::Low => multiplier *= 2.0,
            super::super::confidence::ConfidenceLevel::Medium => multiplier *= 1.5,
            super::super::confidence::ConfidenceLevel::High => {},
            super::super::confidence::ConfidenceLevel::VeryHigh => {},
        }

        multiplier
    }
}

impl Default for RelationshipPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern matching utilities
pub struct PatternMatcher;

impl PatternMatcher {
    /// Check if a relationship matches multiple patterns (OR logic)
    #[inline]
    pub fn matches_any_pattern(
        patterns: &[RelationshipPattern],
        relationship_type: &SemanticRelationshipType,
        confidence: &super::super::confidence::ConfidenceLevel,
        is_outgoing: bool,
    ) -> bool {
        patterns.iter().any(|pattern| {
            pattern.matches_relationship(relationship_type, confidence, is_outgoing)
        })
    }

    /// Check if a relationship matches all patterns (AND logic)
    #[inline]
    pub fn matches_all_patterns(
        patterns: &[RelationshipPattern],
        relationship_type: &SemanticRelationshipType,
        confidence: &super::super::confidence::ConfidenceLevel,
        is_outgoing: bool,
    ) -> bool {
        patterns.iter().all(|pattern| {
            pattern.matches_relationship(relationship_type, confidence, is_outgoing)
        })
    }

    /// Find the most specific pattern that matches
    #[inline]
    pub fn find_most_specific_match(
        patterns: &[RelationshipPattern],
        relationship_type: &SemanticRelationshipType,
        confidence: &super::super::confidence::ConfidenceLevel,
        is_outgoing: bool,
    ) -> Option<usize> {
        let mut best_match = None;
        let mut best_specificity = 0.0;

        for (index, pattern) in patterns.iter().enumerate() {
            if pattern.matches_relationship(relationship_type, confidence, is_outgoing) {
                let specificity = pattern.complexity_score();
                if specificity > best_specificity {
                    best_specificity = specificity;
                    best_match = Some(index);
                }
            }
        }

        best_match
    }

    /// Optimize pattern for performance
    #[inline]
    pub fn optimize_pattern(mut pattern: RelationshipPattern) -> RelationshipPattern {
        // Remove duplicate relationship types
        pattern.relationship_types.sort();
        pattern.relationship_types.dedup();

        // Optimize depth for common cases
        if pattern.max_depth > 10 {
            pattern.max_depth = 10; // Reasonable limit
        }

        // If only one relationship type and it's not directional, set direction to Both
        if pattern.relationship_types.len() == 1 {
            if let Some(rel_type) = pattern.relationship_types.first() {
                if !rel_type.is_directional() && pattern.direction != RelationshipDirection::Both {
                    pattern.direction = RelationshipDirection::Both;
                }
            }
        }

        pattern
    }
}
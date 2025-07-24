//! Relationship query builder for semantic relationships
//!
//! This module provides blazing-fast relationship query building with zero allocation
//! optimizations and elegant ergonomic interfaces for semantic relationship queries.

use super::relationship_types::{SemanticRelationshipType, RelationshipDirection};
use super::relationship_patterns::RelationshipPattern;

/// Relationship query builder for advanced relationship searches
pub struct RelationshipQueryBuilder {
    pattern: RelationshipPattern,
    source_filter: Option<String>,
    target_filter: Option<String>,
}

impl RelationshipQueryBuilder {
    /// Create new query builder
    #[inline]
    pub fn new() -> Self {
        Self {
            pattern: RelationshipPattern::new(),
            source_filter: None,
            target_filter: None,
        }
    }

    /// Filter by relationship type
    #[inline]
    pub fn with_type(mut self, relationship_type: SemanticRelationshipType) -> Self {
        self.pattern = self.pattern.with_type(relationship_type);
        self
    }

    /// Filter by direction
    #[inline]
    pub fn with_direction(mut self, direction: RelationshipDirection) -> Self {
        self.pattern = self.pattern.with_direction(direction);
        self
    }

    /// Filter by source item
    #[inline]
    pub fn with_source(mut self, source_id: String) -> Self {
        self.source_filter = Some(source_id);
        self
    }

    /// Filter by target item
    #[inline]
    pub fn with_target(mut self, target_id: String) -> Self {
        self.target_filter = Some(target_id);
        self
    }

    /// Set maximum depth
    #[inline]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.pattern = self.pattern.with_max_depth(depth);
        self
    }

    /// Set minimum confidence
    #[inline]
    pub fn with_min_confidence(mut self, confidence: super::super::confidence::ConfidenceLevel) -> Self {
        self.pattern = self.pattern.with_min_confidence(confidence);
        self
    }

    /// Include transitive relationships
    #[inline]
    pub fn with_transitive(mut self, include: bool) -> Self {
        self.pattern = self.pattern.with_transitive(include);
        self
    }

    /// Get the built pattern and filters
    #[inline]
    pub fn build(self) -> (RelationshipPattern, Option<String>, Option<String>) {
        (self.pattern, self.source_filter, self.target_filter)
    }

    /// Build query for hierarchical relationships
    #[inline]
    pub fn hierarchical() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::IsA)
            .with_type(SemanticRelationshipType::HasA)
            .with_type(SemanticRelationshipType::PartOf)
            .with_direction(RelationshipDirection::Outgoing)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Medium)
    }

    /// Build query for associative relationships
    #[inline]
    pub fn associative() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::RelatedTo)
            .with_type(SemanticRelationshipType::Causes)
            .with_direction(RelationshipDirection::Both)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Low)
    }

    /// Build query for causal chains
    #[inline]
    pub fn causal_chain(max_depth: usize) -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::Causes)
            .with_direction(RelationshipDirection::Outgoing)
            .with_max_depth(max_depth)
            .with_transitive(true)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::Medium)
    }

    /// Build query for strong relationships only
    #[inline]
    pub fn strong_only() -> Self {
        Self::new()
            .with_type(SemanticRelationshipType::IsA)
            .with_type(SemanticRelationshipType::HasA)
            .with_type(SemanticRelationshipType::PartOf)
            .with_min_confidence(super::super::confidence::ConfidenceLevel::High)
    }

    /// Build query for all relationships from a source
    #[inline]
    pub fn from_source(source_id: String) -> Self {
        Self::new()
            .with_source(source_id)
            .with_direction(RelationshipDirection::Outgoing)
    }

    /// Build query for all relationships to a target
    #[inline]
    pub fn to_target(target_id: String) -> Self {
        Self::new()
            .with_target(target_id)
            .with_direction(RelationshipDirection::Incoming)
    }

    /// Build query for bidirectional relationships
    #[inline]
    pub fn bidirectional(item_id: String) -> Self {
        Self::new()
            .with_source(item_id.clone())
            .with_target(item_id)
            .with_direction(RelationshipDirection::Both)
    }
}

impl Default for RelationshipQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
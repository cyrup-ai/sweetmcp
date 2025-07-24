//! Semantic relationships module coordination
//!
//! This module coordinates all semantic relationship submodules with blazing-fast performance
//! and zero allocation optimizations, integrating relationship types, patterns, validation, and queries.

pub mod relationship_types;
pub mod relationship_patterns;
pub mod relationship_validator;
pub mod relationship_queries;

// Re-export key types and functions for ergonomic access
pub use relationship_types::{
    SemanticRelationshipType, RelationshipDirection,
};

pub use relationship_patterns::{
    RelationshipPattern, PatternMatcher,
};

pub use relationship_validator::{
    RelationshipValidator, ValidationError, ValidationWarning, 
    ValidationOptions, ValidationReport,
};

pub use relationship_queries::{
    RelationshipQueryBuilder,
};

// Common imports for all submodules
use serde::{Deserialize, Serialize};

/// Comprehensive relationship management facade for semantic operations
pub struct RelationshipManager {
    validator: RelationshipValidator,
}

impl RelationshipManager {
    /// Create new relationship manager
    #[inline]
    pub fn new() -> Self {
        Self {
            validator: RelationshipValidator,
        }
    }

    /// Create and validate a new relationship
    #[inline]
    pub fn create_relationship(
        &self,
        source_id: &str,
        target_id: &str,
        relationship_type: SemanticRelationshipType,
        source_type: &super::item_types::SemanticItemType,
        target_type: &super::item_types::SemanticItemType,
    ) -> Result<ValidatedRelationship, ValidationError> {
        // Validate the relationship
        self.validator.validate_relationship(
            source_id,
            target_id,
            &relationship_type,
            source_type,
            target_type,
        )?;

        Ok(ValidatedRelationship {
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relationship_type,
            confidence: relationship_type.default_confidence(),
            strength: relationship_type.strength_weight(),
        })
    }

    /// Batch create and validate relationships
    #[inline]
    pub fn create_relationships(
        &self,
        relationships: &[(String, String, SemanticRelationshipType)],
        item_types: &std::collections::HashMap<String, super::item_types::SemanticItemType>,
        options: ValidationOptions,
    ) -> Result<Vec<ValidatedRelationship>, ValidationReport> {
        let report = self.validator.batch_validate(relationships, item_types, options);
        
        if !report.is_valid() {
            return Err(report);
        }

        let validated_relationships = relationships
            .iter()
            .map(|(source, target, rel_type)| ValidatedRelationship {
                source_id: source.clone(),
                target_id: target.clone(),
                relationship_type: rel_type.clone(),
                confidence: rel_type.default_confidence(),
                strength: rel_type.strength_weight(),
            })
            .collect();

        Ok(validated_relationships)
    }

    /// Create query builder for relationships
    #[inline]
    pub fn query(&self) -> RelationshipQueryBuilder {
        RelationshipQueryBuilder::new()
    }

    /// Find relationships matching a pattern
    #[inline]
    pub fn find_matching_relationships(
        &self,
        relationships: &[ValidatedRelationship],
        pattern: &RelationshipPattern,
        source_filter: Option<&str>,
        target_filter: Option<&str>,
    ) -> Vec<&ValidatedRelationship> {
        relationships
            .iter()
            .filter(|rel| {
                // Apply source filter
                if let Some(source) = source_filter {
                    if rel.source_id != source {
                        return false;
                    }
                }

                // Apply target filter
                if let Some(target) = target_filter {
                    if rel.target_id != target {
                        return false;
                    }
                }

                // Apply pattern matching
                pattern.matches_relationship(
                    &rel.relationship_type,
                    &rel.confidence,
                    true, // Assume outgoing for simplicity
                )
            })
            .collect()
    }

    /// Get relationship statistics
    #[inline]
    pub fn get_relationship_statistics(
        &self,
        relationships: &[ValidatedRelationship],
    ) -> RelationshipStatistics {
        let mut stats = RelationshipStatistics::new();

        for relationship in relationships {
            // Count by type
            *stats.type_counts.entry(relationship.relationship_type.clone()).or_insert(0) += 1;

            // Update strength statistics
            stats.total_strength += relationship.strength;
            stats.min_strength = stats.min_strength.min(relationship.strength);
            stats.max_strength = stats.max_strength.max(relationship.strength);

            // Count by confidence
            match relationship.confidence {
                super::confidence::ConfidenceLevel::High => stats.high_confidence_count += 1,
                super::confidence::ConfidenceLevel::Medium => stats.medium_confidence_count += 1,
                super::confidence::ConfidenceLevel::Low => stats.low_confidence_count += 1,
            }
        }

        stats.total_count = relationships.len();
        stats.average_strength = if stats.total_count > 0 {
            stats.total_strength / stats.total_count as f32
        } else {
            0.0
        };

        stats
    }

    /// Optimize relationship set for performance
    #[inline]
    pub fn optimize_relationships(
        &self,
        relationships: Vec<ValidatedRelationship>,
    ) -> Vec<ValidatedRelationship> {
        let mut optimized = relationships;

        // Remove duplicate relationships
        optimized.sort_by(|a, b| {
            a.source_id.cmp(&b.source_id)
                .then_with(|| a.target_id.cmp(&b.target_id))
                .then_with(|| a.relationship_type.as_str().cmp(b.relationship_type.as_str()))
        });
        optimized.dedup_by(|a, b| {
            a.source_id == b.source_id && 
            a.target_id == b.target_id && 
            a.relationship_type == b.relationship_type
        });

        // Sort by strength for better cache locality
        optimized.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));

        optimized
    }
}

impl Default for RelationshipManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Validated relationship with computed properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedRelationship {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: SemanticRelationshipType,
    pub confidence: super::confidence::ConfidenceLevel,
    pub strength: f32,
}

impl ValidatedRelationship {
    /// Check if relationship is hierarchical
    #[inline]
    pub fn is_hierarchical(&self) -> bool {
        self.relationship_type.is_hierarchical()
    }

    /// Check if relationship is associative
    #[inline]
    pub fn is_associative(&self) -> bool {
        self.relationship_type.is_associative()
    }

    /// Check if relationship is directional
    #[inline]
    pub fn is_directional(&self) -> bool {
        self.relationship_type.is_directional()
    }

    /// Get relationship inverse if available
    #[inline]
    pub fn get_inverse(&self) -> Option<ValidatedRelationship> {
        self.relationship_type.inverse().map(|inv_type| ValidatedRelationship {
            source_id: self.target_id.clone(),
            target_id: self.source_id.clone(),
            relationship_type: inv_type,
            confidence: self.confidence,
            strength: self.strength,
        })
    }

    /// Create relationship key for deduplication
    #[inline]
    pub fn key(&self) -> String {
        format!("{}->{}:{}", self.source_id, self.target_id, self.relationship_type.as_str())
    }
}

/// Relationship statistics
#[derive(Debug, Clone)]
pub struct RelationshipStatistics {
    pub total_count: usize,
    pub type_counts: std::collections::HashMap<SemanticRelationshipType, usize>,
    pub high_confidence_count: usize,
    pub medium_confidence_count: usize,
    pub low_confidence_count: usize,
    pub total_strength: f32,
    pub average_strength: f32,
    pub min_strength: f32,
    pub max_strength: f32,
}

impl RelationshipStatistics {
    /// Create new statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_count: 0,
            type_counts: std::collections::HashMap::new(),
            high_confidence_count: 0,
            medium_confidence_count: 0,
            low_confidence_count: 0,
            total_strength: 0.0,
            average_strength: 0.0,
            min_strength: f32::MAX,
            max_strength: f32::MIN,
        }
    }

    /// Get most common relationship type
    #[inline]
    pub fn most_common_type(&self) -> Option<&SemanticRelationshipType> {
        self.type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(rel_type, _)| rel_type)
    }

    /// Get confidence distribution
    #[inline]
    pub fn confidence_distribution(&self) -> (f32, f32, f32) {
        let total = self.total_count as f32;
        if total == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        (
            self.high_confidence_count as f32 / total,
            self.medium_confidence_count as f32 / total,
            self.low_confidence_count as f32 / total,
        )
    }
}

impl Default for RelationshipStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macros for common relationship operations
#[macro_export]
macro_rules! create_relationship {
    ($manager:expr, $source:expr, $target:expr, $type:expr, $source_type:expr, $target_type:expr) => {
        $manager.create_relationship($source, $target, $type, $source_type, $target_type)
    };
}

#[macro_export]
macro_rules! query_relationships {
    ($manager:expr) => {
        $manager.query()
    };
    ($manager:expr, hierarchical) => {
        $manager.query().hierarchical()
    };
    ($manager:expr, associative) => {
        $manager.query().associative()
    };
    ($manager:expr, from $source:expr) => {
        $manager.query().from_source($source.to_string())
    };
    ($manager:expr, to $target:expr) => {
        $manager.query().to_target($target.to_string())
    };
}
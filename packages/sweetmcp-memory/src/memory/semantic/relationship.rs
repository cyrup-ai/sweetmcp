//! Semantic relationship implementation for semantic memory
//!
//! This module provides the SemanticRelationship struct and related functionality
//! for managing relationships between semantic items with zero allocation,
//! blazing-fast performance, and ergonomic API design.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::types::ConfidenceLevel;
use super::relationship_types::SemanticRelationshipType;
use crate::utils::{Result, Error};

/// Semantic relationship between items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    /// Unique identifier for the relationship
    pub id: String,

    /// Source item ID
    pub source_id: String,

    /// Target item ID
    pub target_id: String,

    /// Type of relationship
    pub relationship_type: SemanticRelationshipType,

    /// Confidence level of the relationship
    pub confidence: ConfidenceLevel,

    /// Additional metadata
    pub metadata: HashMap<String, Value>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl SemanticRelationship {
    /// Create a new semantic relationship
    pub fn new(
        id: &str,
        source_id: &str,
        target_id: &str,
        relationship_type: SemanticRelationshipType,
        confidence: ConfidenceLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relationship_type,
            confidence,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new relationship with generated ID
    pub fn with_generated_id(
        source_id: &str,
        target_id: &str,
        relationship_type: SemanticRelationshipType,
        confidence: ConfidenceLevel,
    ) -> Self {
        let id = format!("rel_{}_{}", source_id, target_id);
        Self::new(&id, source_id, target_id, relationship_type, confidence)
    }

    /// Create an is-a relationship
    pub fn is_a(source_id: &str, target_id: &str, confidence: ConfidenceLevel) -> Self {
        Self::with_generated_id(source_id, target_id, SemanticRelationshipType::IsA, confidence)
    }

    /// Create a part-of relationship
    pub fn part_of(source_id: &str, target_id: &str, confidence: ConfidenceLevel) -> Self {
        Self::with_generated_id(source_id, target_id, SemanticRelationshipType::PartOf, confidence)
    }

    /// Create a related-to relationship
    pub fn related_to(source_id: &str, target_id: &str, confidence: ConfidenceLevel) -> Self {
        Self::with_generated_id(source_id, target_id, SemanticRelationshipType::RelatedTo, confidence)
    }

    /// Create a causes relationship
    pub fn causes(source_id: &str, target_id: &str, confidence: ConfidenceLevel) -> Self {
        Self::with_generated_id(source_id, target_id, SemanticRelationshipType::Causes, confidence)
    }

    /// Create a similar-to relationship
    pub fn similar_to(source_id: &str, target_id: &str, confidence: ConfidenceLevel) -> Self {
        Self::with_generated_id(source_id, target_id, SemanticRelationshipType::SimilarTo, confidence)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
        self.updated_at = Utc::now();
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Remove metadata value
    pub fn remove_metadata(&mut self, key: &str) -> Option<Value> {
        let result = self.metadata.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }

    /// Update confidence level
    pub fn update_confidence(&mut self, confidence: ConfidenceLevel) {
        self.confidence = confidence;
        self.updated_at = Utc::now();
    }

    /// Check if relationship involves a specific item
    pub fn involves_item(&self, item_id: &str) -> bool {
        self.source_id == item_id || self.target_id == item_id
    }

    /// Get the other item ID in the relationship
    pub fn get_other_item_id(&self, item_id: &str) -> Option<&str> {
        if self.source_id == item_id {
            Some(&self.target_id)
        } else if self.target_id == item_id {
            Some(&self.source_id)
        } else {
            None
        }
    }

    /// Calculate relationship strength score
    pub fn strength_score(&self) -> f64 {
        let type_weight = self.relationship_type.strength_weight();
        let confidence_weight = self.confidence.to_float() as f64;
        type_weight * confidence_weight
    }

    /// Check if relationship is bidirectional
    pub fn is_bidirectional(&self) -> bool {
        self.relationship_type.is_bidirectional()
    }

    /// Validate relationship consistency
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::ValidationError("Relationship ID cannot be empty".to_string()));
        }

        if self.source_id.is_empty() {
            return Err(Error::ValidationError("Source ID cannot be empty".to_string()));
        }

        if self.target_id.is_empty() {
            return Err(Error::ValidationError("Target ID cannot be empty".to_string()));
        }

        if self.source_id == self.target_id {
            return Err(Error::ValidationError("Self-referential relationships not allowed".to_string()));
        }

        if self.created_at > self.updated_at {
            return Err(Error::ValidationError("Created timestamp cannot be after updated timestamp".to_string()));
        }

        Ok(())
    }

    /// Convert to JSON Value
    pub fn to_value(&self) -> Result<Value> {
        serde_json::to_value(self).map_err(|e| Error::ConversionError(e.to_string()))
    }

    /// Create from JSON Value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Ok(relationship) = serde_json::from_value::<SemanticRelationship>(value.clone()) {
            relationship.validate()?;
            Ok(relationship)
        } else {
            Err(Error::ConversionError("Invalid semantic relationship value".to_string()))
        }
    }

    /// Create inverse relationship if applicable
    pub fn create_inverse(&self) -> Option<Self> {
        if let Some(inverse_type) = self.relationship_type.inverse() {
            let inverse_id = format!("inv_{}", self.id);
            Some(Self::new(
                &inverse_id,
                &self.target_id,
                &self.source_id,
                inverse_type,
                self.confidence,
            ))
        } else {
            None
        }
    }

    /// Get relationship age in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get time since last update in seconds
    pub fn update_age_seconds(&self) -> i64 {
        (Utc::now() - self.updated_at).num_seconds()
    }

    /// Check if relationship is recent (created within specified seconds)
    pub fn is_recent(&self, seconds: i64) -> bool {
        self.age_seconds() <= seconds
    }

    /// Check if relationship was recently updated (within specified seconds)
    pub fn is_recently_updated(&self, seconds: i64) -> bool {
        self.update_age_seconds() <= seconds
    }

    /// Get metadata keys sorted alphabetically
    pub fn get_sorted_metadata_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.metadata.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Clear all metadata
    pub fn clear_metadata(&mut self) {
        if !self.metadata.is_empty() {
            self.metadata.clear();
            self.updated_at = Utc::now();
        }
    }

    /// Get metadata count
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Check if metadata is empty
    pub fn has_metadata(&self) -> bool {
        !self.metadata.is_empty()
    }

    /// Batch update metadata
    pub fn update_metadata(&mut self, updates: HashMap<String, Value>) {
        if !updates.is_empty() {
            for (key, value) in updates {
                self.metadata.insert(key, value);
            }
            self.updated_at = Utc::now();
        }
    }

    /// Get relationship summary
    pub fn summary(&self) -> String {
        format!(
            "{} --[{}]--> {} (confidence: {:?}, strength: {:.2})",
            self.source_id,
            self.relationship_type.as_str(),
            self.target_id,
            self.confidence,
            self.strength_score()
        )
    }

    /// Check if relationship can coexist with another
    pub fn can_coexist_with(&self, other: &SemanticRelationship) -> bool {
        // Same source and target but different relationship types
        if self.source_id == other.source_id && self.target_id == other.target_id {
            return self.relationship_type.can_coexist_with(&other.relationship_type);
        }
        
        // Different source/target combinations can always coexist
        true
    }

    /// Get relationship priority for conflict resolution
    pub fn priority(&self) -> u8 {
        self.relationship_type.priority()
    }
}
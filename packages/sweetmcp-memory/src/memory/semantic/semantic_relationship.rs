//! Semantic relationship core types and operations
//!
//! This module provides blazing-fast semantic relationship management with zero allocation
//! optimizations and elegant ergonomic interfaces for relationship operations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{
    confidence::ConfidenceLevel,
    relationships::SemanticRelationshipType,
};

/// Semantic relationship between items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: SemanticRelationshipType,
    pub confidence: ConfidenceLevel,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub strength: f32,
}

impl SemanticRelationship {
    /// Create new semantic relationship with zero allocation optimizations
    #[inline]
    pub fn new(
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
    ) -> Self {
        let id = format!("{}:{}:{}", source_id, relationship_type.as_str(), target_id);
        let confidence = relationship_type.default_confidence();
        let strength = relationship_type.strength_weight();
        let now = std::time::SystemTime::now();

        Self {
            id,
            source_id,
            target_id,
            relationship_type,
            confidence,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            strength,
        }
    }

    /// Create relationship with custom ID
    #[inline]
    pub fn with_id(
        id: String,
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
    ) -> Self {
        let confidence = relationship_type.default_confidence();
        let strength = relationship_type.strength_weight();
        let now = std::time::SystemTime::now();

        Self {
            id,
            source_id,
            target_id,
            relationship_type,
            confidence,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            strength,
        }
    }

    /// Create relationship with full configuration
    #[inline]
    pub fn with_config(
        id: String,
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
        confidence: ConfidenceLevel,
        strength: f32,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = std::time::SystemTime::now();

        Self {
            id,
            source_id,
            target_id,
            relationship_type,
            confidence,
            metadata,
            created_at: now,
            updated_at: now,
            strength: strength.clamp(0.0, 1.0),
        }
    }

    /// Update relationship strength with blazing-fast validation
    #[inline]
    pub fn update_strength(&mut self, strength: f32) {
        let new_strength = strength.clamp(0.0, 1.0);
        if (self.strength - new_strength).abs() > f32::EPSILON {
            self.strength = new_strength;
            self.updated_at = std::time::SystemTime::now();
            debug!("Relationship {} strength updated to {:.3}", self.id, new_strength);
        }
    }

    /// Update confidence level
    #[inline]
    pub fn update_confidence(&mut self, confidence: ConfidenceLevel) {
        if self.confidence != confidence {
            self.confidence = confidence;
            self.updated_at = std::time::SystemTime::now();
            debug!("Relationship {} confidence updated to {:?}", self.id, confidence);
        }
    }

    /// Add metadata with zero allocation when possible
    #[inline]
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = std::time::SystemTime::now();
    }

    /// Get metadata value with zero allocation
    #[inline]
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Remove metadata
    #[inline]
    pub fn remove_metadata(&mut self, key: &str) -> Option<serde_json::Value> {
        let result = self.metadata.remove(key);
        if result.is_some() {
            self.updated_at = std::time::SystemTime::now();
        }
        result
    }

    /// Check if relationship has metadata key
    #[inline]
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Check if relationship involves item with zero allocation
    #[inline]
    pub fn involves_item(&self, item_id: &str) -> bool {
        self.source_id == item_id || self.target_id == item_id
    }

    /// Get other item ID in relationship with zero allocation
    #[inline]
    pub fn get_other_item_id(&self, item_id: &str) -> Option<&str> {
        if self.source_id == item_id {
            Some(&self.target_id)
        } else if self.target_id == item_id {
            Some(&self.source_id)
        } else {
            None
        }
    }

    /// Check if relationship is outgoing from item
    #[inline]
    pub fn is_outgoing_from(&self, item_id: &str) -> bool {
        self.source_id == item_id
    }

    /// Check if relationship is incoming to item
    #[inline]
    pub fn is_incoming_to(&self, item_id: &str) -> bool {
        self.target_id == item_id
    }

    /// Get relationship direction relative to item
    #[inline]
    pub fn direction_from_item(&self, item_id: &str) -> Option<RelationshipDirection> {
        if self.source_id == item_id {
            Some(RelationshipDirection::Outgoing)
        } else if self.target_id == item_id {
            Some(RelationshipDirection::Incoming)
        } else {
            None
        }
    }

    /// Check if relationship is bidirectional
    #[inline]
    pub fn is_bidirectional(&self) -> bool {
        self.relationship_type.is_bidirectional()
    }

    /// Get relationship age in days
    #[inline]
    pub fn age_days(&self) -> Option<u64> {
        self.created_at.elapsed().ok().map(|d| d.as_secs() / (24 * 3600))
    }

    /// Get days since last update
    #[inline]
    pub fn days_since_update(&self) -> Option<u64> {
        self.updated_at.elapsed().ok().map(|d| d.as_secs() / (24 * 3600))
    }

    /// Check if relationship is stale (not updated recently)
    #[inline]
    pub fn is_stale(&self, max_age_days: u64) -> bool {
        self.days_since_update()
            .map(|days| days > max_age_days)
            .unwrap_or(false)
    }

    /// Calculate relationship quality score
    #[inline]
    pub fn quality_score(&self) -> f32 {
        let strength_weight = self.strength;
        let confidence_weight = self.confidence.to_float();
        let type_weight = self.relationship_type.quality_weight();
        
        (strength_weight * 0.4 + confidence_weight * 0.3 + type_weight * 0.3).min(1.0)
    }

    /// Check if relationship is weak (low quality)
    #[inline]
    pub fn is_weak(&self, threshold: f32) -> bool {
        self.quality_score() < threshold
    }

    /// Check if relationship is strong (high quality)
    #[inline]
    pub fn is_strong(&self, threshold: f32) -> bool {
        self.quality_score() >= threshold
    }

    /// Get relationship weight for graph algorithms
    #[inline]
    pub fn graph_weight(&self) -> f32 {
        self.strength * self.confidence.to_float() * self.relationship_type.strength_weight()
    }

    /// Check if relationship should be archived
    #[inline]
    pub fn should_archive(&self, config: &RelationshipArchiveConfig) -> bool {
        let age_days = self.age_days().unwrap_or(0);
        let days_since_update = self.days_since_update().unwrap_or(0);
        let quality = self.quality_score();
        
        age_days > config.max_age_days ||
        days_since_update > config.max_inactive_days ||
        quality < config.min_quality_threshold
    }

    /// Check if relationship should be deleted
    #[inline]
    pub fn should_delete(&self, config: &RelationshipDeleteConfig) -> bool {
        let age_days = self.age_days().unwrap_or(0);
        let quality = self.quality_score();
        let confidence = self.confidence.to_float();
        
        (age_days > config.max_age_days && confidence < config.min_confidence_threshold) ||
        (quality < config.min_quality_threshold && self.strength < config.min_strength_threshold)
    }

    /// Create relationship summary for reporting
    #[inline]
    pub fn create_summary(&self) -> RelationshipSummary {
        RelationshipSummary {
            id: self.id.clone(),
            source_id: self.source_id.clone(),
            target_id: self.target_id.clone(),
            relationship_type: self.relationship_type.clone(),
            confidence: self.confidence,
            strength: self.strength,
            quality_score: self.quality_score(),
            age_days: self.age_days().unwrap_or(0),
            days_since_update: self.days_since_update().unwrap_or(0),
            metadata_count: self.metadata.len(),
        }
    }

    /// Validate relationship integrity
    #[inline]
    pub fn validate(&self) -> Result<(), RelationshipValidationError> {
        if self.id.is_empty() {
            return Err(RelationshipValidationError::EmptyId);
        }

        if self.source_id.is_empty() {
            return Err(RelationshipValidationError::EmptySourceId);
        }

        if self.target_id.is_empty() {
            return Err(RelationshipValidationError::EmptyTargetId);
        }

        if self.source_id == self.target_id {
            return Err(RelationshipValidationError::SelfReference);
        }

        if !(0.0..=1.0).contains(&self.strength) {
            return Err(RelationshipValidationError::InvalidStrength(self.strength));
        }

        if self.created_at > self.updated_at {
            return Err(RelationshipValidationError::InvalidTimestamps);
        }

        Ok(())
    }
}

/// Relationship direction relative to an item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipDirection {
    /// Relationship goes out from the item
    Outgoing,
    /// Relationship comes into the item
    Incoming,
}

/// Configuration for archiving relationships
#[derive(Debug, Clone)]
pub struct RelationshipArchiveConfig {
    pub max_age_days: u64,
    pub max_inactive_days: u64,
    pub min_quality_threshold: f32,
}

impl Default for RelationshipArchiveConfig {
    fn default() -> Self {
        Self {
            max_age_days: 365, // 1 year
            max_inactive_days: 90, // 3 months
            min_quality_threshold: 0.2,
        }
    }
}

/// Configuration for deleting relationships
#[derive(Debug, Clone)]
pub struct RelationshipDeleteConfig {
    pub max_age_days: u64,
    pub min_confidence_threshold: f32,
    pub min_quality_threshold: f32,
    pub min_strength_threshold: f32,
}

impl Default for RelationshipDeleteConfig {
    fn default() -> Self {
        Self {
            max_age_days: 730, // 2 years
            min_confidence_threshold: 0.1,
            min_quality_threshold: 0.1,
            min_strength_threshold: 0.1,
        }
    }
}

/// Relationship summary for reporting and analysis
#[derive(Debug, Clone, Serialize)]
pub struct RelationshipSummary {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: SemanticRelationshipType,
    pub confidence: ConfidenceLevel,
    pub strength: f32,
    pub quality_score: f32,
    pub age_days: u64,
    pub days_since_update: u64,
    pub metadata_count: usize,
}

/// Relationship validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum RelationshipValidationError {
    #[error("Relationship ID cannot be empty")]
    EmptyId,
    #[error("Source ID cannot be empty")]
    EmptySourceId,
    #[error("Target ID cannot be empty")]
    EmptyTargetId,
    #[error("Relationship cannot reference itself")]
    SelfReference,
    #[error("Invalid strength value: {0} (must be between 0.0 and 1.0)")]
    InvalidStrength(f32),
    #[error("Invalid timestamps: created_at > updated_at")]
    InvalidTimestamps,
}
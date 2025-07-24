//! Semantic item core types and operations
//!
//! This module provides blazing-fast semantic item management with zero allocation
//! optimizations and elegant ergonomic interfaces for semantic item operations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{
    confidence::ConfidenceLevel,
    item_types::{SemanticItemType, SemanticItemTypeClassifier},
};

/// Semantic item with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticItem {
    pub id: String,
    pub content: serde_json::Value,
    pub item_type: SemanticItemType,
    pub confidence: ConfidenceLevel,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
    pub access_count: usize,
    pub last_accessed: Option<std::time::SystemTime>,
}

impl SemanticItem {
    /// Create new semantic item with zero allocation optimizations
    #[inline]
    pub fn new(id: String, content: serde_json::Value) -> Self {
        let item_type = SemanticItemTypeClassifier::classify_content(&content);
        let confidence = item_type.default_confidence();
        let now = std::time::SystemTime::now();

        Self {
            id,
            content,
            item_type,
            confidence,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            access_count: 0,
            last_accessed: None,
        }
    }

    /// Create semantic item with explicit type
    #[inline]
    pub fn with_type(id: String, content: serde_json::Value, item_type: SemanticItemType) -> Self {
        let confidence = item_type.default_confidence();
        let now = std::time::SystemTime::now();

        Self {
            id,
            content,
            item_type,
            confidence,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            access_count: 0,
            last_accessed: None,
        }
    }

    /// Create semantic item with full configuration
    #[inline]
    pub fn with_config(
        id: String,
        content: serde_json::Value,
        item_type: SemanticItemType,
        confidence: ConfidenceLevel,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = std::time::SystemTime::now();

        Self {
            id,
            content,
            item_type,
            confidence,
            metadata,
            created_at: now,
            updated_at: now,
            access_count: 0,
            last_accessed: None,
        }
    }

    /// Update content and refresh metadata
    #[inline]
    pub fn update_content(&mut self, content: serde_json::Value) {
        self.content = content;
        self.updated_at = std::time::SystemTime::now();
        
        // Re-classify if needed
        let new_type = SemanticItemTypeClassifier::classify_content(&self.content);
        if new_type != self.item_type {
            self.item_type = new_type;
            self.confidence = self.item_type.default_confidence();
            debug!("Item {} reclassified to {:?}", self.id, self.item_type);
        }
    }

    /// Record access with blazing-fast atomic operations
    #[inline]
    pub fn record_access(&mut self) {
        self.access_count = self.access_count.saturating_add(1);
        self.last_accessed = Some(std::time::SystemTime::now());
    }

    /// Update confidence level
    #[inline]
    pub fn update_confidence(&mut self, confidence: ConfidenceLevel) {
        self.confidence = confidence;
        self.updated_at = std::time::SystemTime::now();
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

    /// Check if item has metadata key
    #[inline]
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get all metadata keys
    #[inline]
    pub fn metadata_keys(&self) -> impl Iterator<Item = &String> {
        self.metadata.keys()
    }

    /// Check if item is stale (not accessed recently)
    #[inline]
    pub fn is_stale(&self, max_age_days: u64) -> bool {
        if let Some(last_accessed) = self.last_accessed {
            if let Ok(elapsed) = last_accessed.elapsed() {
                return elapsed.as_secs() > max_age_days * 24 * 3600;
            }
        }
        
        // If never accessed, check creation time
        if let Ok(elapsed) = self.created_at.elapsed() {
            elapsed.as_secs() > max_age_days * 24 * 3600
        } else {
            false
        }
    }

    /// Calculate item priority score with optimized computation
    #[inline]
    pub fn priority_score(&self) -> f32 {
        let type_weight = self.item_type.priority_weight();
        let confidence_weight = self.confidence.to_float();
        let access_weight = (self.access_count as f32).ln_1p() / 10.0; // Logarithmic scaling
        
        (type_weight + confidence_weight + access_weight) / 3.0
    }

    /// Get age in days
    #[inline]
    pub fn age_days(&self) -> Option<u64> {
        self.created_at.elapsed().ok().map(|d| d.as_secs() / (24 * 3600))
    }

    /// Get days since last access
    #[inline]
    pub fn days_since_access(&self) -> Option<u64> {
        if let Some(last_accessed) = self.last_accessed {
            last_accessed.elapsed().ok().map(|d| d.as_secs() / (24 * 3600))
        } else {
            self.age_days()
        }
    }

    /// Check if item was recently accessed
    #[inline]
    pub fn is_recently_accessed(&self, days_threshold: u64) -> bool {
        self.days_since_access()
            .map(|days| days <= days_threshold)
            .unwrap_or(false)
    }

    /// Get content size in bytes
    #[inline]
    pub fn content_size(&self) -> usize {
        self.content.to_string().len()
    }

    /// Check if item is large (exceeds size threshold)
    #[inline]
    pub fn is_large(&self, size_threshold_kb: usize) -> bool {
        self.content_size() > size_threshold_kb * 1024
    }

    /// Get item freshness score (0.0 = very old, 1.0 = very fresh)
    #[inline]
    pub fn freshness_score(&self) -> f32 {
        const MAX_FRESH_DAYS: f32 = 30.0;
        
        if let Some(days) = self.days_since_access() {
            (1.0 - (days as f32 / MAX_FRESH_DAYS)).max(0.0)
        } else {
            0.0
        }
    }

    /// Get item relevance score based on access patterns
    #[inline]
    pub fn relevance_score(&self) -> f32 {
        let access_score = (self.access_count as f32).ln_1p() / 10.0;
        let freshness_score = self.freshness_score();
        let confidence_score = self.confidence.to_float();
        
        (access_score * 0.4 + freshness_score * 0.3 + confidence_score * 0.3).min(1.0)
    }

    /// Check if item should be archived based on criteria
    #[inline]
    pub fn should_archive(&self, config: &ArchiveConfig) -> bool {
        let age_days = self.age_days().unwrap_or(0);
        let days_since_access = self.days_since_access().unwrap_or(0);
        let relevance = self.relevance_score();
        
        age_days > config.max_age_days ||
        days_since_access > config.max_inactive_days ||
        relevance < config.min_relevance_threshold
    }

    /// Check if item should be deleted based on criteria
    #[inline]
    pub fn should_delete(&self, config: &DeleteConfig) -> bool {
        let age_days = self.age_days().unwrap_or(0);
        let days_since_access = self.days_since_access().unwrap_or(0);
        let confidence = self.confidence.to_float();
        
        (age_days > config.max_age_days && confidence < config.min_confidence_threshold) ||
        (days_since_access > config.max_inactive_days && self.access_count < config.min_access_count)
    }

    /// Create item summary for reporting
    #[inline]
    pub fn create_summary(&self) -> ItemSummary {
        ItemSummary {
            id: self.id.clone(),
            item_type: self.item_type,
            confidence: self.confidence,
            content_size: self.content_size(),
            access_count: self.access_count,
            age_days: self.age_days().unwrap_or(0),
            days_since_access: self.days_since_access().unwrap_or(0),
            priority_score: self.priority_score(),
            relevance_score: self.relevance_score(),
            metadata_count: self.metadata.len(),
        }
    }

    /// Validate item integrity
    #[inline]
    pub fn validate(&self) -> Result<(), ItemValidationError> {
        if self.id.is_empty() {
            return Err(ItemValidationError::EmptyId);
        }

        if self.content.is_null() {
            return Err(ItemValidationError::NullContent);
        }

        if self.created_at > self.updated_at {
            return Err(ItemValidationError::InvalidTimestamps);
        }

        if let Some(last_accessed) = self.last_accessed {
            if last_accessed < self.created_at {
                return Err(ItemValidationError::InvalidAccessTime);
            }
        }

        Ok(())
    }
}

/// Configuration for archiving items
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    pub max_age_days: u64,
    pub max_inactive_days: u64,
    pub min_relevance_threshold: f32,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            max_age_days: 365, // 1 year
            max_inactive_days: 90, // 3 months
            min_relevance_threshold: 0.1,
        }
    }
}

/// Configuration for deleting items
#[derive(Debug, Clone)]
pub struct DeleteConfig {
    pub max_age_days: u64,
    pub max_inactive_days: u64,
    pub min_confidence_threshold: f32,
    pub min_access_count: usize,
}

impl Default for DeleteConfig {
    fn default() -> Self {
        Self {
            max_age_days: 730, // 2 years
            max_inactive_days: 180, // 6 months
            min_confidence_threshold: 0.05,
            min_access_count: 1,
        }
    }
}

/// Item summary for reporting and analysis
#[derive(Debug, Clone, Serialize)]
pub struct ItemSummary {
    pub id: String,
    pub item_type: SemanticItemType,
    pub confidence: ConfidenceLevel,
    pub content_size: usize,
    pub access_count: usize,
    pub age_days: u64,
    pub days_since_access: u64,
    pub priority_score: f32,
    pub relevance_score: f32,
    pub metadata_count: usize,
}

/// Item validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ItemValidationError {
    #[error("Item ID cannot be empty")]
    EmptyId,
    #[error("Item content cannot be null")]
    NullContent,
    #[error("Invalid timestamps: created_at > updated_at")]
    InvalidTimestamps,
    #[error("Invalid access time: last_accessed < created_at")]
    InvalidAccessTime,
}
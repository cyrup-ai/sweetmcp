//! Core semantic item structure and basic operations
//!
//! This module provides the SemanticItem struct and its fundamental methods
//! for creation and basic manipulation with zero allocation patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::confidence::ConfidenceLevel;
use super::item_types::SemanticItemType;

/// Semantic item representing knowledge, concepts, facts, or rules
/// 
/// A semantic item is a fundamental unit of knowledge in the semantic memory system,
/// containing structured information with metadata, confidence levels, and categorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticItem {
    /// Unique identifier for the item
    pub id: String,

    /// Type of the semantic item
    pub item_type: SemanticItemType,

    /// Category classification of the item
    pub category: String,

    /// Content data of the item
    pub content: Value,

    /// Tags associated with the item for categorization
    pub tags: Vec<String>,

    /// Confidence level of the item's accuracy
    pub confidence: ConfidenceLevel,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata for the item
    pub metadata: HashMap<String, Value>,
}

impl SemanticItem {
    /// Create a new semantic item
    /// 
    /// # Arguments
    /// * `id` - Unique identifier for the item
    /// * `item_type` - Type of the semantic item
    /// * `content` - Content data for the item
    /// 
    /// # Returns
    /// New SemanticItem instance
    pub fn new(id: &str, item_type: SemanticItemType, content: Value) -> Self {
        let now = Utc::now();
        let category = item_type.to_string();
        Self {
            id: id.to_string(),
            item_type,
            category,
            content,
            tags: Vec::new(),
            confidence: ConfidenceLevel::Medium,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Add a tag to the item
    /// 
    /// # Arguments
    /// * `tag` - Tag to add
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Set confidence level for the item
    /// 
    /// # Arguments
    /// * `confidence` - Confidence level to set
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn with_confidence(mut self, confidence: ConfidenceLevel) -> Self {
        self.confidence = confidence;
        self
    }

    /// Add metadata to the item
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Update the content of the item
    /// 
    /// # Arguments
    /// * `content` - New content for the item
    pub fn update_content(&mut self, content: Value) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Get the unique identifier
    /// 
    /// # Returns
    /// Reference to the item ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the item type
    /// 
    /// # Returns
    /// Reference to the item type
    pub fn item_type(&self) -> &SemanticItemType {
        &self.item_type
    }

    /// Get the category
    /// 
    /// # Returns
    /// Reference to the category string
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Get the content
    /// 
    /// # Returns
    /// Reference to the content value
    pub fn content(&self) -> &Value {
        &self.content
    }

    /// Get the confidence level
    /// 
    /// # Returns
    /// The confidence level
    pub fn confidence(&self) -> ConfidenceLevel {
        self.confidence
    }

    /// Get the creation timestamp
    /// 
    /// # Returns
    /// Reference to the creation timestamp
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Get the last update timestamp
    /// 
    /// # Returns
    /// Reference to the last update timestamp
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Set the confidence level
    /// 
    /// # Arguments
    /// * `confidence` - New confidence level
    pub fn set_confidence(&mut self, confidence: ConfidenceLevel) {
        self.confidence = confidence;
        self.updated_at = Utc::now();
    }

    /// Set the category
    /// 
    /// # Arguments
    /// * `category` - New category string
    pub fn set_category(&mut self, category: &str) {
        self.category = category.to_string();
        self.updated_at = Utc::now();
    }

    /// Check if the item is recent (created within specified days)
    /// 
    /// # Arguments
    /// * `days` - Number of days to consider as recent
    /// 
    /// # Returns
    /// True if the item was created within the specified days
    pub fn is_recent(&self, days: i64) -> bool {
        let threshold = Utc::now() - chrono::Duration::days(days);
        self.created_at > threshold
    }

    /// Check if the item was recently updated (within specified days)
    /// 
    /// # Arguments
    /// * `days` - Number of days to consider as recently updated
    /// 
    /// # Returns
    /// True if the item was updated within the specified days
    pub fn is_recently_updated(&self, days: i64) -> bool {
        let threshold = Utc::now() - chrono::Duration::days(days);
        self.updated_at > threshold
    }

    /// Get the age of the item in days
    /// 
    /// # Returns
    /// Number of days since the item was created
    pub fn age_in_days(&self) -> i64 {
        (Utc::now() - self.created_at).num_days()
    }

    /// Get the number of days since last update
    /// 
    /// # Returns
    /// Number of days since the item was last updated
    pub fn days_since_update(&self) -> i64 {
        (Utc::now() - self.updated_at).num_days()
    }

    /// Check if the item has high confidence
    /// 
    /// # Returns
    /// True if confidence is High or VeryHigh
    pub fn has_high_confidence(&self) -> bool {
        matches!(self.confidence, ConfidenceLevel::High | ConfidenceLevel::VeryHigh)
    }

    /// Check if the item has low confidence
    /// 
    /// # Returns
    /// True if confidence is Low or VeryLow
    pub fn has_low_confidence(&self) -> bool {
        matches!(self.confidence, ConfidenceLevel::Low | ConfidenceLevel::VeryLow)
    }

    /// Get a summary of the item for display
    /// 
    /// # Returns
    /// String summary of the item
    pub fn summary(&self) -> String {
        let content_preview = match &self.content {
            Value::String(s) => {
                if s.len() > 50 {
                    format!("{}...", &s[..50])
                } else {
                    s.clone()
                }
            }
            _ => "Non-text content".to_string(),
        };
        
        format!(
            "{} ({}): {} [Confidence: {}]",
            self.id,
            self.item_type.display_name(),
            content_preview,
            self.confidence.display_name()
        )
    }

    /// Get detailed information about the item
    /// 
    /// # Returns
    /// String with detailed item information
    pub fn detailed_info(&self) -> String {
        format!(
            "SemanticItem {{\n  ID: {}\n  Type: {}\n  Category: {}\n  Confidence: {}\n  Tags: {}\n  Metadata entries: {}\n  Created: {}\n  Updated: {}\n}}",
            self.id,
            self.item_type.display_name(),
            self.category,
            self.confidence.display_name(),
            self.tags.len(),
            self.metadata.len(),
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}
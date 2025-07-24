//! Semantic item implementation
//!
//! This module provides the SemanticItem struct and its associated methods
//! for creating, managing, and querying semantic items with zero allocation patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::types::{ConfidenceLevel, SemanticItemType};

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

    /// Add multiple tags at once
    /// 
    /// # Arguments
    /// * `tags` - Vector of tags to add
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
        self.updated_at = Utc::now();
    }

    /// Remove a tag from the item
    /// 
    /// # Arguments
    /// * `tag` - Tag to remove
    /// 
    /// # Returns
    /// True if the tag was found and removed
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if the item has a specific tag
    /// 
    /// # Arguments
    /// * `tag` - Tag to check for
    /// 
    /// # Returns
    /// True if the item has the tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Get all tags as a reference
    /// 
    /// # Returns
    /// Reference to the tags vector
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Update metadata value
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - New metadata value
    pub fn update_metadata(&mut self, key: &str, value: Value) {
        self.metadata.insert(key.to_string(), value);
        self.updated_at = Utc::now();
    }

    /// Remove metadata entry
    /// 
    /// # Arguments
    /// * `key` - Metadata key to remove
    /// 
    /// # Returns
    /// The removed value if it existed
    pub fn remove_metadata(&mut self, key: &str) -> Option<Value> {
        let result = self.metadata.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }

    /// Get metadata value by key
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// 
    /// # Returns
    /// Reference to the metadata value if it exists
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Check if the item matches a search query
    /// 
    /// # Arguments
    /// * `query` - Search query string
    /// 
    /// # Returns
    /// True if the item matches the query
    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        
        // Check ID
        if self.id.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Check category
        if self.category.to_lowercase().contains(&query_lower) {
            return true;
        }
        
        // Check tags
        if self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
            return true;
        }
        
        // Check content if it's a string
        if let Value::String(content_str) = &self.content {
            if content_str.to_lowercase().contains(&query_lower) {
                return true;
            }
        }
        
        false
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

    /// Get the number of tags
    /// 
    /// # Returns
    /// Number of tags associated with the item
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// Get the number of metadata entries
    /// 
    /// # Returns
    /// Number of metadata entries
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
    }

    /// Check if the item has any tags
    /// 
    /// # Returns
    /// True if the item has at least one tag
    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    /// Check if the item has any metadata
    /// 
    /// # Returns
    /// True if the item has at least one metadata entry
    pub fn has_metadata(&self) -> bool {
        !self.metadata.is_empty()
    }
}
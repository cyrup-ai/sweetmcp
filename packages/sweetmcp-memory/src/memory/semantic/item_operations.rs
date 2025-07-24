//! Semantic item operations for tags and metadata management
//!
//! This module provides methods for managing tags, metadata, and performing
//! operations on semantic items with zero allocation patterns.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use super::item_core::SemanticItem;

impl SemanticItem {
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

    /// Clear all tags
    pub fn clear_tags(&mut self) {
        self.tags.clear();
        self.updated_at = Utc::now();
    }

    /// Replace all tags with new ones
    /// 
    /// # Arguments
    /// * `tags` - New tags to replace existing ones
    pub fn replace_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
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

    /// Get all metadata as a reference
    /// 
    /// # Returns
    /// Reference to the metadata HashMap
    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }

    /// Clear all metadata
    pub fn clear_metadata(&mut self) {
        self.metadata.clear();
        self.updated_at = Utc::now();
    }

    /// Replace all metadata with new entries
    /// 
    /// # Arguments
    /// * `metadata` - New metadata to replace existing entries
    pub fn replace_metadata(&mut self, metadata: HashMap<String, Value>) {
        self.metadata = metadata;
        self.updated_at = Utc::now();
    }

    /// Merge metadata from another HashMap
    /// 
    /// # Arguments
    /// * `other_metadata` - Metadata to merge (overwrites existing keys)
    pub fn merge_metadata(&mut self, other_metadata: &HashMap<String, Value>) {
        for (key, value) in other_metadata {
            self.metadata.insert(key.clone(), value.clone());
        }
        self.updated_at = Utc::now();
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

    /// Check if the item has a specific metadata key
    /// 
    /// # Arguments
    /// * `key` - Metadata key to check for
    /// 
    /// # Returns
    /// True if the metadata key exists
    pub fn has_metadata_key(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get all metadata keys
    /// 
    /// # Returns
    /// Vector of all metadata keys
    pub fn metadata_keys(&self) -> Vec<&String> {
        self.metadata.keys().collect()
    }

    /// Get all metadata values
    /// 
    /// # Returns
    /// Vector of all metadata values
    pub fn metadata_values(&self) -> Vec<&Value> {
        self.metadata.values().collect()
    }

    /// Filter tags by a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each tag
    /// 
    /// # Returns
    /// Vector of tags that match the predicate
    pub fn filter_tags<F>(&self, predicate: F) -> Vec<&String>
    where
        F: Fn(&String) -> bool,
    {
        self.tags.iter().filter(|tag| predicate(tag)).collect()
    }

    /// Find tags containing a substring
    /// 
    /// # Arguments
    /// * `substring` - Substring to search for
    /// 
    /// # Returns
    /// Vector of tags containing the substring
    pub fn find_tags_containing(&self, substring: &str) -> Vec<&String> {
        self.tags.iter()
            .filter(|tag| tag.contains(substring))
            .collect()
    }

    /// Find metadata entries with keys containing a substring
    /// 
    /// # Arguments
    /// * `substring` - Substring to search for in keys
    /// 
    /// # Returns
    /// HashMap of matching metadata entries
    pub fn find_metadata_by_key_substring(&self, substring: &str) -> HashMap<&String, &Value> {
        self.metadata.iter()
            .filter(|(key, _)| key.contains(substring))
            .collect()
    }

    /// Update multiple metadata entries at once
    /// 
    /// # Arguments
    /// * `updates` - HashMap of key-value pairs to update
    pub fn update_multiple_metadata(&mut self, updates: HashMap<String, Value>) {
        for (key, value) in updates {
            self.metadata.insert(key, value);
        }
        self.updated_at = Utc::now();
    }

    /// Remove multiple metadata entries at once
    /// 
    /// # Arguments
    /// * `keys` - Vector of keys to remove
    /// 
    /// # Returns
    /// HashMap of removed key-value pairs
    pub fn remove_multiple_metadata(&mut self, keys: &[String]) -> HashMap<String, Value> {
        let mut removed = HashMap::new();
        for key in keys {
            if let Some(value) = self.metadata.remove(key) {
                removed.insert(key.clone(), value);
            }
        }
        if !removed.is_empty() {
            self.updated_at = Utc::now();
        }
        removed
    }

    /// Add a tag if it doesn't already exist
    /// 
    /// # Arguments
    /// * `tag` - Tag to add
    /// 
    /// # Returns
    /// True if the tag was added (didn't exist before)
    pub fn add_tag_if_not_exists(&mut self, tag: &str) -> bool {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Toggle a tag (add if not present, remove if present)
    /// 
    /// # Arguments
    /// * `tag` - Tag to toggle
    /// 
    /// # Returns
    /// True if the tag was added, false if it was removed
    pub fn toggle_tag(&mut self, tag: &str) -> bool {
        if self.has_tag(tag) {
            self.remove_tag(tag);
            false
        } else {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
            true
        }
    }

    /// Get tags that match a pattern
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// Vector of matching tags
    pub fn get_tags_matching_pattern(&self, pattern: &str) -> Vec<&String> {
        let pattern_lower = pattern.to_lowercase();
        self.tags.iter()
            .filter(|tag| tag.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// Count tags matching a pattern
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// Number of tags matching the pattern
    pub fn count_tags_matching_pattern(&self, pattern: &str) -> usize {
        let pattern_lower = pattern.to_lowercase();
        self.tags.iter()
            .filter(|tag| tag.to_lowercase().contains(&pattern_lower))
            .count()
    }
}
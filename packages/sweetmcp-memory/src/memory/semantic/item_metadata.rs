//! Semantic item metadata management operations
//!
//! This module provides methods for managing metadata on semantic items
//! with zero allocation patterns and efficient metadata operations.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use super::item_core::SemanticItem;

impl SemanticItem {
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

    /// Get the number of metadata entries
    /// 
    /// # Returns
    /// Number of metadata entries
    pub fn metadata_count(&self) -> usize {
        self.metadata.len()
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

    /// Get metadata entries matching a pattern in keys
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// HashMap of matching metadata entries
    pub fn get_metadata_matching_pattern(&self, pattern: &str) -> HashMap<&String, &Value> {
        let pattern_lower = pattern.to_lowercase();
        self.metadata.iter()
            .filter(|(key, _)| key.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// Count metadata entries matching a pattern in keys
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// Number of metadata entries matching the pattern
    pub fn count_metadata_matching_pattern(&self, pattern: &str) -> usize {
        let pattern_lower = pattern.to_lowercase();
        self.metadata.iter()
            .filter(|(key, _)| key.to_lowercase().contains(&pattern_lower))
            .count()
    }

    /// Get metadata value as string if possible
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// 
    /// # Returns
    /// String value if the metadata exists and is a string
    pub fn get_metadata_as_string(&self, key: &str) -> Option<&str> {
        self.metadata.get(key)?.as_str()
    }

    /// Get metadata value as number if possible
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// 
    /// # Returns
    /// Number value if the metadata exists and is a number
    pub fn get_metadata_as_number(&self, key: &str) -> Option<f64> {
        self.metadata.get(key)?.as_f64()
    }

    /// Get metadata value as boolean if possible
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// 
    /// # Returns
    /// Boolean value if the metadata exists and is a boolean
    pub fn get_metadata_as_bool(&self, key: &str) -> Option<bool> {
        self.metadata.get(key)?.as_bool()
    }

    /// Set metadata from string value
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - String value to set
    pub fn set_metadata_string(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), Value::String(value.to_string()));
        self.updated_at = Utc::now();
    }

    /// Set metadata from number value
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Number value to set
    pub fn set_metadata_number(&mut self, key: &str, value: f64) {
        self.metadata.insert(key.to_string(), Value::Number(
            serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0))
        ));
        self.updated_at = Utc::now();
    }

    /// Set metadata from boolean value
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `value` - Boolean value to set
    pub fn set_metadata_bool(&mut self, key: &str, value: bool) {
        self.metadata.insert(key.to_string(), Value::Bool(value));
        self.updated_at = Utc::now();
    }

    /// Check if metadata key has a specific value
    /// 
    /// # Arguments
    /// * `key` - Metadata key
    /// * `expected_value` - Expected value to compare
    /// 
    /// # Returns
    /// True if the metadata key exists and has the expected value
    pub fn metadata_equals(&self, key: &str, expected_value: &Value) -> bool {
        self.metadata.get(key)
            .map(|value| value == expected_value)
            .unwrap_or(false)
    }

    /// Get metadata keys sorted alphabetically
    /// 
    /// # Returns
    /// Vector of metadata keys sorted alphabetically
    pub fn get_sorted_metadata_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.metadata.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Clone metadata to a new HashMap
    /// 
    /// # Returns
    /// Cloned HashMap of metadata
    pub fn clone_metadata(&self) -> HashMap<String, Value> {
        self.metadata.clone()
    }

    /// Filter metadata by value type
    /// 
    /// # Arguments
    /// * `value_type` - Type of value to filter by ("string", "number", "boolean", "array", "object", "null")
    /// 
    /// # Returns
    /// HashMap of metadata entries matching the value type
    pub fn filter_metadata_by_type(&self, value_type: &str) -> HashMap<&String, &Value> {
        self.metadata.iter()
            .filter(|(_, value)| {
                match value_type {
                    "string" => value.is_string(),
                    "number" => value.is_number(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    "null" => value.is_null(),
                    _ => false,
                }
            })
            .collect()
    }

    /// Get metadata summary for display
    /// 
    /// # Returns
    /// String summary of metadata
    pub fn metadata_summary(&self) -> String {
        if self.metadata.is_empty() {
            "No metadata".to_string()
        } else {
            let keys: Vec<&String> = self.metadata.keys().collect();
            format!("Metadata keys: {}", keys.join(", "))
        }
    }
}
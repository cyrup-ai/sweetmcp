//! Advanced semantic item metadata operations
//!
//! This module provides advanced metadata management methods including filtering,
//! pattern matching, and type-specific operations with zero allocation patterns.

use serde_json::Value;
use std::collections::HashMap;

use super::item_core::SemanticItem;

impl SemanticItem {
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

    /// Get metadata keys sorted alphabetically
    /// 
    /// # Returns
    /// Vector of metadata keys sorted alphabetically
    pub fn get_sorted_metadata_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.metadata.keys().cloned().collect();
        keys.sort();
        keys
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

    /// Get metadata entries where values match a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each value
    /// 
    /// # Returns
    /// HashMap of metadata entries where values match the predicate
    pub fn filter_metadata_by_value<F>(&self, predicate: F) -> HashMap<&String, &Value>
    where
        F: Fn(&Value) -> bool,
    {
        self.metadata.iter()
            .filter(|(_, value)| predicate(value))
            .collect()
    }

    /// Get metadata entries where keys match a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each key
    /// 
    /// # Returns
    /// HashMap of metadata entries where keys match the predicate
    pub fn filter_metadata_by_key<F>(&self, predicate: F) -> HashMap<&String, &Value>
    where
        F: Fn(&String) -> bool,
    {
        self.metadata.iter()
            .filter(|(key, _)| predicate(key))
            .collect()
    }

    /// Get all string metadata values
    /// 
    /// # Returns
    /// HashMap of metadata entries with string values
    pub fn get_string_metadata(&self) -> HashMap<&String, &str> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                value.as_str().map(|s| (key, s))
            })
            .collect()
    }

    /// Get all number metadata values
    /// 
    /// # Returns
    /// HashMap of metadata entries with number values
    pub fn get_number_metadata(&self) -> HashMap<&String, f64> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                value.as_f64().map(|n| (key, n))
            })
            .collect()
    }

    /// Get all boolean metadata values
    /// 
    /// # Returns
    /// HashMap of metadata entries with boolean values
    pub fn get_boolean_metadata(&self) -> HashMap<&String, bool> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                value.as_bool().map(|b| (key, b))
            })
            .collect()
    }

    /// Find metadata keys with values containing a substring (for string values only)
    /// 
    /// # Arguments
    /// * `substring` - Substring to search for in string values
    /// 
    /// # Returns
    /// Vector of keys whose string values contain the substring
    pub fn find_keys_with_string_value_containing(&self, substring: &str) -> Vec<&String> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                if let Some(s) = value.as_str() {
                    if s.contains(substring) {
                        Some(key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get metadata entries with numeric values within a range
    /// 
    /// # Arguments
    /// * `min` - Minimum value (inclusive)
    /// * `max` - Maximum value (inclusive)
    /// 
    /// # Returns
    /// HashMap of metadata entries with numeric values in the range
    pub fn get_metadata_in_numeric_range(&self, min: f64, max: f64) -> HashMap<&String, f64> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                if let Some(n) = value.as_f64() {
                    if n >= min && n <= max {
                        Some((key, n))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Count metadata entries by type
    /// 
    /// # Returns
    /// HashMap with counts of each value type
    pub fn count_metadata_by_type(&self) -> HashMap<&'static str, usize> {
        let mut counts = HashMap::new();
        
        for (_, value) in &self.metadata {
            let type_name = match value {
                Value::String(_) => "string",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
                Value::Null => "null",
            };
            *counts.entry(type_name).or_insert(0) += 1;
        }
        
        counts
    }

    /// Get metadata keys that start with a prefix
    /// 
    /// # Arguments
    /// * `prefix` - Prefix to match
    /// 
    /// # Returns
    /// Vector of keys that start with the prefix
    pub fn get_keys_with_prefix(&self, prefix: &str) -> Vec<&String> {
        self.metadata.keys()
            .filter(|key| key.starts_with(prefix))
            .collect()
    }

    /// Get metadata keys that end with a suffix
    /// 
    /// # Arguments
    /// * `suffix` - Suffix to match
    /// 
    /// # Returns
    /// Vector of keys that end with the suffix
    pub fn get_keys_with_suffix(&self, suffix: &str) -> Vec<&String> {
        self.metadata.keys()
            .filter(|key| key.ends_with(suffix))
            .collect()
    }

    /// Check if any metadata values match a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each value
    /// 
    /// # Returns
    /// True if any metadata value matches the predicate
    pub fn any_metadata_value_matches<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Value) -> bool,
    {
        self.metadata.values().any(predicate)
    }

    /// Check if all metadata values match a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each value
    /// 
    /// # Returns
    /// True if all metadata values match the predicate
    pub fn all_metadata_values_match<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Value) -> bool,
    {
        self.metadata.values().all(predicate)
    }

    /// Get the first metadata entry matching a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each key-value pair
    /// 
    /// # Returns
    /// First matching key-value pair if found
    pub fn find_first_metadata<F>(&self, predicate: F) -> Option<(&String, &Value)>
    where
        F: Fn(&String, &Value) -> bool,
    {
        self.metadata.iter()
            .find(|(key, value)| predicate(key, value))
    }

    /// Get metadata statistics
    /// 
    /// # Returns
    /// Tuple of (total_entries, string_count, number_count, boolean_count, array_count, object_count, null_count)
    pub fn get_metadata_statistics(&self) -> (usize, usize, usize, usize, usize, usize, usize) {
        let total = self.metadata.len();
        let mut string_count = 0;
        let mut number_count = 0;
        let mut boolean_count = 0;
        let mut array_count = 0;
        let mut object_count = 0;
        let mut null_count = 0;
        
        for value in self.metadata.values() {
            match value {
                Value::String(_) => string_count += 1,
                Value::Number(_) => number_count += 1,
                Value::Bool(_) => boolean_count += 1,
                Value::Array(_) => array_count += 1,
                Value::Object(_) => object_count += 1,
                Value::Null => null_count += 1,
            }
        }
        
        (total, string_count, number_count, boolean_count, array_count, object_count, null_count)
    }
}
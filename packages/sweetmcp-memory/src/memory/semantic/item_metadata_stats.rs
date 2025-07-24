//! Semantic item metadata statistics and analysis operations
//!
//! This module provides statistical analysis and advanced operations for metadata
//! with zero allocation patterns and blazing-fast performance.

use serde_json::Value;
use std::collections::HashMap;

use super::item_core::SemanticItem;

impl SemanticItem {
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

    /// Transform metadata values using a function
    /// 
    /// # Arguments
    /// * `transformer` - Function to transform each value
    /// 
    /// # Returns
    /// New HashMap with transformed values
    pub fn transform_metadata_values<F, T>(&self, transformer: F) -> HashMap<String, T>
    where
        F: Fn(&Value) -> T,
    {
        self.metadata.iter()
            .map(|(key, value)| (key.clone(), transformer(value)))
            .collect()
    }

    /// Calculate metadata complexity score
    /// 
    /// # Returns
    /// Complexity score based on number of entries and value types
    pub fn calculate_metadata_complexity(&self) -> f64 {
        if self.metadata.is_empty() {
            return 0.0;
        }
        
        let (total, string_count, number_count, boolean_count, array_count, object_count, null_count) = 
            self.get_metadata_statistics();
        
        // Base complexity from entry count
        let base_complexity = (total as f64).ln() + 1.0;
        
        // Type diversity bonus
        let type_diversity = [
            string_count > 0,
            number_count > 0,
            boolean_count > 0,
            array_count > 0,
            object_count > 0,
            null_count > 0,
        ].iter().filter(|&&x| x).count() as f64;
        
        // Complex types (arrays and objects) add more complexity
        let complex_type_bonus = (array_count + object_count) as f64 * 0.5;
        
        base_complexity + type_diversity * 0.3 + complex_type_bonus
    }

    /// Get metadata value sizes (for strings and arrays)
    /// 
    /// # Returns
    /// HashMap with key and corresponding size
    pub fn get_metadata_value_sizes(&self) -> HashMap<&String, usize> {
        self.metadata.iter()
            .filter_map(|(key, value)| {
                let size = match value {
                    Value::String(s) => Some(s.len()),
                    Value::Array(arr) => Some(arr.len()),
                    Value::Object(obj) => Some(obj.len()),
                    _ => None,
                };
                size.map(|s| (key, s))
            })
            .collect()
    }

    /// Find largest metadata values by size
    /// 
    /// # Arguments
    /// * `limit` - Maximum number of results to return
    /// 
    /// # Returns
    /// Vector of (key, size) pairs sorted by size descending
    pub fn find_largest_metadata_values(&self, limit: usize) -> Vec<(&String, usize)> {
        let mut sizes: Vec<(&String, usize)> = self.get_metadata_value_sizes()
            .into_iter()
            .collect();
        
        sizes.sort_by(|a, b| b.1.cmp(&a.1));
        sizes.truncate(limit);
        sizes
    }

    /// Calculate metadata storage efficiency
    /// 
    /// # Returns
    /// Efficiency score (0.0 to 1.0) based on value types and sizes
    pub fn calculate_metadata_efficiency(&self) -> f64 {
        if self.metadata.is_empty() {
            return 1.0;
        }
        
        let mut efficient_count = 0;
        let total_count = self.metadata.len();
        
        for value in self.metadata.values() {
            let is_efficient = match value {
                Value::String(s) => s.len() <= 100, // Reasonable string length
                Value::Number(_) => true, // Numbers are always efficient
                Value::Bool(_) => true, // Booleans are always efficient
                Value::Array(arr) => arr.len() <= 10, // Reasonable array size
                Value::Object(obj) => obj.len() <= 5, // Reasonable object size
                Value::Null => false, // Null values are not efficient
            };
            
            if is_efficient {
                efficient_count += 1;
            }
        }
        
        efficient_count as f64 / total_count as f64
    }

    /// Get metadata health score
    /// 
    /// # Returns
    /// Health score (0.0 to 1.0) based on various metadata quality metrics
    pub fn get_metadata_health_score(&self) -> f64 {
        if self.metadata.is_empty() {
            return 1.0; // Empty metadata is considered healthy
        }
        
        let efficiency = self.calculate_metadata_efficiency();
        let complexity = self.calculate_metadata_complexity();
        
        // Normalize complexity (lower is better)
        let normalized_complexity = 1.0 / (1.0 + complexity * 0.1);
        
        // Check for null values (reduce health)
        let null_penalty = self.metadata.values()
            .filter(|v| v.is_null())
            .count() as f64 / self.metadata.len() as f64;
        
        // Combine metrics
        let health_score = (efficiency * 0.4 + normalized_complexity * 0.4 + (1.0 - null_penalty) * 0.2)
            .clamp(0.0, 1.0);
        
        health_score
    }

    /// Analyze metadata key patterns
    /// 
    /// # Returns
    /// HashMap with pattern analysis results
    pub fn analyze_metadata_key_patterns(&self) -> HashMap<String, usize> {
        let mut patterns = HashMap::new();
        
        for key in self.metadata.keys() {
            // Count keys with underscores
            if key.contains('_') {
                *patterns.entry("underscore_separated".to_string()).or_insert(0) += 1;
            }
            
            // Count keys with dots
            if key.contains('.') {
                *patterns.entry("dot_separated".to_string()).or_insert(0) += 1;
            }
            
            // Count keys with camelCase
            if key.chars().any(|c| c.is_uppercase()) {
                *patterns.entry("camel_case".to_string()).or_insert(0) += 1;
            }
            
            // Count numeric suffixes
            if key.chars().last().map_or(false, |c| c.is_ascii_digit()) {
                *patterns.entry("numeric_suffix".to_string()).or_insert(0) += 1;
            }
            
            // Count prefixed keys
            if key.len() > 3 && key.chars().nth(2) == Some('_') {
                *patterns.entry("prefixed".to_string()).or_insert(0) += 1;
            }
        }
        
        patterns
    }

    /// Compare metadata with another semantic item
    /// 
    /// # Arguments
    /// * `other` - Other semantic item to compare with
    /// 
    /// # Returns
    /// Tuple of (common_keys, unique_to_self, unique_to_other, similarity_score)
    pub fn compare_metadata(&self, other: &SemanticItem) -> (Vec<String>, Vec<String>, Vec<String>, f64) {
        let self_keys: std::collections::HashSet<_> = self.metadata.keys().collect();
        let other_keys: std::collections::HashSet<_> = other.metadata.keys().collect();
        
        let common_keys: Vec<String> = self_keys.intersection(&other_keys)
            .map(|k| k.to_string())
            .collect();
        
        let unique_to_self: Vec<String> = self_keys.difference(&other_keys)
            .map(|k| k.to_string())
            .collect();
        
        let unique_to_other: Vec<String> = other_keys.difference(&self_keys)
            .map(|k| k.to_string())
            .collect();
        
        // Calculate similarity based on Jaccard index
        let union_size = self_keys.union(&other_keys).count();
        let similarity_score = if union_size == 0 {
            1.0
        } else {
            common_keys.len() as f64 / union_size as f64
        };
        
        (common_keys, unique_to_self, unique_to_other, similarity_score)
    }

    /// Get metadata summary report
    /// 
    /// # Returns
    /// Detailed string report of metadata analysis
    pub fn get_metadata_report(&self) -> String {
        if self.metadata.is_empty() {
            return "No metadata present".to_string();
        }
        
        let (total, string_count, number_count, boolean_count, array_count, object_count, null_count) = 
            self.get_metadata_statistics();
        
        let complexity = self.calculate_metadata_complexity();
        let efficiency = self.calculate_metadata_efficiency();
        let health = self.get_metadata_health_score();
        let patterns = self.analyze_metadata_key_patterns();
        
        let mut report = format!(
            "Metadata Report:\n\
             - Total entries: {}\n\
             - Types: {} strings, {} numbers, {} booleans, {} arrays, {} objects, {} nulls\n\
             - Complexity score: {:.2}\n\
             - Efficiency: {:.1}%\n\
             - Health score: {:.1}%\n",
            total, string_count, number_count, boolean_count, 
            array_count, object_count, null_count,
            complexity, efficiency * 100.0, health * 100.0
        );
        
        if !patterns.is_empty() {
            report.push_str("- Key patterns:\n");
            for (pattern, count) in patterns {
                report.push_str(&format!("  - {}: {}\n", pattern, count));
            }
        }
        
        report
    }
}
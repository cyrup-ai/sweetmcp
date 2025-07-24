//! Entity comparison utilities
//!
//! This module provides optimized entity comparison functions with
//! zero allocation fast paths for performance-critical operations.

use super::core::BaseEntity;
use std::collections::HashMap;
use surrealdb::sql::Value;

/// Entity comparison functions with performance optimizations
impl BaseEntity {
    /// Compare entities by ID with fast path
    pub fn same_id(&self, other: &BaseEntity) -> bool {
        self.id == other.id
    }

    /// Compare entities by type with fast path
    pub fn same_type(&self, other: &BaseEntity) -> bool {
        self.entity_type == other.entity_type
    }

    /// Compare entities by attributes with optimized comparison
    pub fn same_attributes(&self, other: &BaseEntity) -> bool {
        // Fast path: different lengths
        if self.attributes.len() != other.attributes.len() {
            return false;
        }

        // Fast path: empty attributes
        if self.attributes.is_empty() {
            return true;
        }

        // Compare each attribute with early termination
        for (key, value) in &self.attributes {
            match other.attributes.get(key) {
                Some(other_value) => {
                    // Use efficient JSON comparison for deep equality
                    let self_json = match serde_json::to_string(value) {
                        Ok(json) => json,
                        Err(_) => return false,
                    };
                    let other_json = match serde_json::to_string(other_value) {
                        Ok(json) => json,
                        Err(_) => return false,
                    };
                    if self_json != other_json {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Deep equality check with all optimizations
    pub fn deep_equals(&self, other: &BaseEntity) -> bool {
        // Fast path checks first
        if !self.same_id(other) || !self.same_type(other) {
            return false;
        }
        
        self.same_attributes(other)
    }

    /// Partial equality check (ID and type only)
    pub fn shallow_equals(&self, other: &BaseEntity) -> bool {
        self.same_id(other) && self.same_type(other)
    }

    /// Check if entity matches a pattern with optimized matching
    pub fn matches_pattern(&self, pattern: &EntityPattern) -> bool {
        // Check ID pattern with fast string operations
        if let Some(ref id_pattern) = pattern.id_pattern {
            match &pattern.id_match_type {
                IdMatchType::Contains => {
                    if !self.id.contains(id_pattern) {
                        return false;
                    }
                }
                IdMatchType::StartsWith => {
                    if !self.id.starts_with(id_pattern) {
                        return false;
                    }
                }
                IdMatchType::EndsWith => {
                    if !self.id.ends_with(id_pattern) {
                        return false;
                    }
                }
                IdMatchType::Exact => {
                    if self.id != *id_pattern {
                        return false;
                    }
                }
            }
        }

        // Check type pattern
        if let Some(ref type_pattern) = pattern.type_pattern {
            if self.entity_type != *type_pattern {
                return false;
            }
        }

        // Check required attributes with early termination
        for (attr_name, attr_value) in &pattern.required_attributes {
            match self.get_attribute(attr_name) {
                Some(value) => {
                    if !self.values_equal(value, attr_value) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check excluded attributes
        for attr_name in &pattern.excluded_attributes {
            if self.has_attribute(attr_name) {
                return false;
            }
        }

        true
    }

    /// Fast value equality check
    fn values_equal(&self, value1: &Value, value2: &Value) -> bool {
        let json1 = match serde_json::to_string(value1) {
            Ok(json) => json,
            Err(_) => return false,
        };
        let json2 = match serde_json::to_string(value2) {
            Ok(json) => json,
            Err(_) => return false,
        };
        json1 == json2
    }

    /// Create a filtered copy with only specified attributes
    pub fn filter_attributes(&self, attribute_names: &[&str]) -> Self {
        let mut filtered_attributes = HashMap::with_capacity(attribute_names.len());
        
        for &attr_name in attribute_names {
            if let Some(value) = self.attributes.get(attr_name) {
                filtered_attributes.insert(attr_name.to_string(), value.clone());
            }
        }

        Self {
            id: self.id.clone(),
            entity_type: self.entity_type.clone(),
            attributes: filtered_attributes,
        }
    }

    /// Create a copy without specified attributes
    pub fn exclude_attributes(&self, attribute_names: &[&str]) -> Self {
        let mut filtered_attributes = self.attributes.clone();
        
        for &attr_name in attribute_names {
            filtered_attributes.remove(attr_name);
        }

        Self {
            id: self.id.clone(),
            entity_type: self.entity_type.clone(),
            attributes: filtered_attributes,
        }
    }

    /// Merge with another entity (other entity takes precedence)
    pub fn merge(&self, other: &BaseEntity) -> Self {
        let mut merged_attributes = self.attributes.clone();
        merged_attributes.extend(other.attributes.clone());

        Self {
            id: other.id.clone(), // Other entity's ID takes precedence
            entity_type: other.entity_type.clone(), // Other entity's type takes precedence
            attributes: merged_attributes,
        }
    }

    /// Calculate similarity score between entities (0.0 to 1.0)
    pub fn similarity_score(&self, other: &BaseEntity) -> f64 {
        let mut total_attributes = std::collections::HashSet::new();
        total_attributes.extend(self.attributes.keys());
        total_attributes.extend(other.attributes.keys());

        if total_attributes.is_empty() {
            return if self.id == other.id && self.entity_type == other.entity_type {
                1.0
            } else {
                0.0
            };
        }

        let mut matching_attributes = 0;
        for attr_name in &total_attributes {
            match (self.get_attribute(attr_name), other.get_attribute(attr_name)) {
                (Some(val1), Some(val2)) => {
                    if self.values_equal(val1, val2) {
                        matching_attributes += 1;
                    }
                }
                (None, None) => matching_attributes += 1,
                _ => {} // One has the attribute, the other doesn't
            }
        }

        let attribute_similarity = matching_attributes as f64 / total_attributes.len() as f64;
        
        // Factor in ID and type similarity
        let id_similarity = if self.id == other.id { 1.0 } else { 0.0 };
        let type_similarity = if self.entity_type == other.entity_type { 1.0 } else { 0.0 };

        // Weighted average: attributes 70%, type 20%, id 10%
        0.7 * attribute_similarity + 0.2 * type_similarity + 0.1 * id_similarity
    }

    /// Create a copy with modified attributes
    pub fn with_modified_attributes<F>(mut self, modifier: F) -> Self
    where
        F: FnOnce(&mut HashMap<String, Value>),
    {
        modifier(&mut self.attributes);
        self
    }
}

/// Entity pattern for matching with optimized pattern types
#[derive(Debug, Clone)]
pub struct EntityPattern {
    pub id_pattern: Option<String>,
    pub id_match_type: IdMatchType,
    pub type_pattern: Option<String>,
    pub required_attributes: HashMap<String, Value>,
    pub excluded_attributes: std::collections::HashSet<String>,
}

/// ID matching types for flexible pattern matching
#[derive(Debug, Clone)]
pub enum IdMatchType {
    Contains,
    StartsWith,
    EndsWith,
    Exact,
}

impl EntityPattern {
    /// Create a new empty pattern
    pub fn new() -> Self {
        Self {
            id_pattern: None,
            id_match_type: IdMatchType::Contains,
            type_pattern: None,
            required_attributes: HashMap::new(),
            excluded_attributes: std::collections::HashSet::new(),
        }
    }

    /// Add ID pattern with match type
    pub fn with_id_pattern(mut self, pattern: &str, match_type: IdMatchType) -> Self {
        self.id_pattern = Some(pattern.to_string());
        self.id_match_type = match_type;
        self
    }

    /// Add type pattern
    pub fn with_type_pattern(mut self, pattern: &str) -> Self {
        self.type_pattern = Some(pattern.to_string());
        self
    }

    /// Add required attribute
    pub fn with_required_attribute(mut self, name: &str, value: Value) -> Self {
        self.required_attributes.insert(name.to_string(), value);
        self
    }

    /// Add excluded attribute
    pub fn with_excluded_attribute(mut self, name: &str) -> Self {
        self.excluded_attributes.insert(name.to_string());
        self
    }
}

impl Default for EntityPattern {
    fn default() -> Self {
        Self::new()
    }
}
//! Base entity implementation for the graph entity system
//!
//! This module provides the BaseEntity struct which serves as the foundational
//! implementation of the Entity trait with zero-allocation patterns and blazing-fast performance.

use super::types::{Entity, EntityParams};
use crate::graph::graph_db::{GraphError, Node, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Value;

/// Base entity implementation
/// 
/// Provides a concrete implementation of the Entity trait with efficient
/// attribute management and serialization support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    /// Entity ID - immutable after creation
    pub id: String,
    /// Entity type - immutable after creation  
    pub entity_type: String,
    /// Entity attributes - mutable for dynamic properties
    pub attributes: HashMap<String, Value>,
}

impl BaseEntity {
    /// Create a new entity with the specified ID and type
    /// 
    /// # Arguments
    /// * `id` - Unique identifier for the entity
    /// * `entity_type` - Type classification for the entity
    /// 
    /// # Returns
    /// A new BaseEntity instance with empty attributes
    pub fn new(id: &str, entity_type: &str) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::new(),
        }
    }

    /// Create a new entity from parameters
    /// 
    /// # Arguments
    /// * `params` - EntityParams containing ID, type, and initial attributes
    /// 
    /// # Returns
    /// Result containing the new BaseEntity or validation error
    pub fn from_params(params: EntityParams) -> Result<Self> {
        // Validate parameters first
        params.validate()?;

        let id = params.id.unwrap(); // Safe after validation
        let entity_type = params.entity_type.unwrap(); // Safe after validation

        let mut entity = Self {
            id,
            entity_type,
            attributes: params.attributes,
        };

        // Perform entity validation unless skipped
        if !params.skip_validation {
            entity.validate()?;
        }

        Ok(entity)
    }

    /// Add an attribute using builder pattern
    /// 
    /// # Arguments
    /// * `name` - Attribute name
    /// * `value` - Attribute value (converted to Value)
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn with_attribute<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.attributes.insert(name.to_string(), value.into());
        self
    }

    /// Add multiple attributes using builder pattern
    /// 
    /// # Arguments
    /// * `attributes` - HashMap of attribute name-value pairs
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Remove an attribute by name
    /// 
    /// # Arguments
    /// * `name` - Name of the attribute to remove
    /// 
    /// # Returns
    /// The removed value if it existed, None otherwise
    pub fn remove_attribute(&mut self, name: &str) -> Option<Value> {
        self.attributes.remove(name)
    }

    /// Check if an attribute exists
    /// 
    /// # Arguments
    /// * `name` - Name of the attribute to check
    /// 
    /// # Returns
    /// true if the attribute exists, false otherwise
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get all attribute names
    /// 
    /// # Returns
    /// Iterator over attribute names for zero-allocation iteration
    pub fn attribute_names(&self) -> impl Iterator<Item = &String> {
        self.attributes.keys()
    }

    /// Get the number of attributes
    /// 
    /// # Returns
    /// Count of attributes in the entity
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// Clear all attributes
    /// 
    /// Removes all attributes while preserving the HashMap capacity
    /// for potential reuse to avoid allocations.
    pub fn clear_attributes(&mut self) {
        self.attributes.clear();
    }

    /// Merge attributes from another entity
    /// 
    /// # Arguments
    /// * `other` - Another BaseEntity to merge attributes from
    /// * `overwrite` - Whether to overwrite existing attributes
    pub fn merge_attributes(&mut self, other: &BaseEntity, overwrite: bool) {
        for (key, value) in &other.attributes {
            if overwrite || !self.attributes.contains_key(key) {
                self.attributes.insert(key.clone(), value.clone());
            }
        }
    }

    /// Convert attributes to JSON string
    /// 
    /// # Returns
    /// Result containing JSON string representation of attributes
    pub fn attributes_to_json(&self) -> Result<String> {
        serde_json::to_string(&self.attributes).map_err(|e| {
            GraphError::SerializationError(format!("Failed to serialize attributes: {}", e))
        })
    }

    /// Load attributes from JSON string
    /// 
    /// # Arguments
    /// * `json` - JSON string containing attribute data
    /// 
    /// # Returns
    /// Result indicating success or failure
    pub fn attributes_from_json(&mut self, json: &str) -> Result<()> {
        let parsed_attributes: HashMap<String, serde_json::Value> =
            serde_json::from_str(json).map_err(|e| {
                GraphError::SerializationError(format!("Failed to parse JSON: {}", e))
            })?;

        // Convert serde_json::Value to surrealdb::sql::Value
        for (key, json_value) in parsed_attributes {
            let sql_value = super::conversion::json_to_sql_value(json_value)?;
            self.attributes.insert(key, sql_value);
        }

        Ok(())
    }
}

impl Entity for BaseEntity {
    fn id(&self) -> &str {
        &self.id
    }

    fn entity_type(&self) -> &str {
        &self.entity_type
    }

    fn get_attribute(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name)
    }

    fn set_attribute(&mut self, name: &str, value: Value) {
        self.attributes.insert(name.to_string(), value);
    }

    fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }

    fn validate(&self) -> Result<()> {
        // Basic validation - ensure ID and type are not empty
        if self.id.is_empty() {
            return Err(GraphError::ValidationError(
                "Entity ID cannot be empty".to_string(),
            ));
        }

        if self.entity_type.is_empty() {
            return Err(GraphError::ValidationError(
                "Entity type cannot be empty".to_string(),
            ));
        }

        // Validate that ID contains only valid characters
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(GraphError::ValidationError(
                "Entity ID can only contain alphanumeric characters, underscores, and hyphens".to_string(),
            ));
        }

        // Validate that entity type contains only valid characters
        if !self.entity_type.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(GraphError::ValidationError(
                "Entity type can only contain alphanumeric characters and underscores".to_string(),
            ));
        }

        Ok(())
    }

    fn to_node(&self) -> Node {
        let mut node = Node::new(self.id.clone(), &self.entity_type);

        for (key, value) in &self.attributes {
            // Convert surrealdb::sql::Value to serde_json::Value
            let json_value = super::conversion::sql_to_json_value(value);
            node = node.with_property(key, json_value);
        }

        node
    }

    fn from_node(node: Node) -> Result<Self> {
        let mut json_attributes = node.properties.clone();

        // Extract entity_type from attributes if present
        let entity_type = json_attributes
            .remove("entity_type")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| node.node_type.clone());

        // Convert serde_json::Value to surrealdb::sql::Value
        let mut sql_attributes = HashMap::new();
        for (key, json_value) in json_attributes {
            let sql_value = super::conversion::json_to_sql_value(json_value)?;
            sql_attributes.insert(key, sql_value);
        }

        let entity = Self {
            id: node.id,
            entity_type,
            attributes: sql_attributes,
        };

        // Validate the created entity
        entity.validate()?;

        Ok(entity)
    }
}
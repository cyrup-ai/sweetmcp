//! Core entity definitions and implementations
//!
//! This module provides the Entity trait, BaseEntity implementation, and
//! core functionality for graph entities with zero allocation fast paths.

use crate::graph::graph_db::{GraphError, Node, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use surrealdb::sql::Value;

/// Type alias for entity futures to simplify trait definitions
pub type EntityFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

/// Entity trait for domain objects
pub trait Entity: Send + Sync + Debug {
    /// Get the entity ID
    fn id(&self) -> &str;

    /// Get the entity type
    fn entity_type(&self) -> &str;

    /// Get an attribute value
    fn get_attribute(&self, name: &str) -> Option<&Value>;

    /// Set an attribute value
    fn set_attribute(&mut self, name: &str, value: Value);

    /// Get all attributes
    fn attributes(&self) -> &HashMap<String, Value>;

    /// Validate the entity
    fn validate(&self) -> Result<()>;

    /// Convert to a graph node
    fn to_node(&self) -> Node;

    /// Create from a graph node
    fn from_node(node: Node) -> Result<Self>
    where
        Self: Sized;
}

/// Base entity implementation with optimized attribute management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    /// Entity ID
    pub id: String,

    /// Entity type
    pub entity_type: String,

    /// Entity attributes with fast lookup
    pub attributes: HashMap<String, Value>,
}

impl BaseEntity {
    /// Create a new entity with pre-allocated attribute capacity
    pub fn new(id: &str, entity_type: &str) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::with_capacity(8), // Pre-allocate for common case
        }
    }

    /// Create with known attribute count for zero-allocation initialization
    pub fn with_capacity(id: &str, entity_type: &str, attribute_capacity: usize) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::with_capacity(attribute_capacity),
        }
    }

    /// Add an attribute with builder pattern
    pub fn with_attribute<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.attributes.insert(name.to_string(), value.into());
        self
    }

    /// Add multiple attributes efficiently
    pub fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Bulk insert attributes for performance
    pub fn bulk_insert_attributes(&mut self, attributes: Vec<(String, Value)>) {
        self.attributes.reserve(attributes.len());
        for (key, value) in attributes {
            self.attributes.insert(key, value);
        }
    }

    /// Get attribute with type conversion
    pub fn get_typed_attribute<T>(&self, name: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.attributes.get(name).and_then(|value| {
            // Convert surrealdb::sql::Value to serde_json::Value for deserialization
            let json_value = match serde_json::to_string(value) {
                Ok(json_str) => serde_json::from_str(&json_str).ok()?,
                Err(_) => return None,
            };
            serde_json::from_value(json_value).ok()
        })
    }

    /// Set attribute with type conversion
    pub fn set_typed_attribute<T>(&mut self, name: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let surreal_value = match serde_json::to_string(&json_value) {
            Ok(json_str) => serde_json::from_str::<Value>(&json_str)
                .map_err(|e| GraphError::SerializationError(e.to_string()))?,
            Err(e) => return Err(GraphError::SerializationError(e.to_string())),
        };
        
        self.attributes.insert(name.to_string(), surreal_value);
        Ok(())
    }

    /// Remove attribute and return its value
    pub fn remove_attribute(&mut self, name: &str) -> Option<Value> {
        self.attributes.remove(name)
    }

    /// Check if attribute exists
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get attribute names
    pub fn attribute_names(&self) -> Vec<&String> {
        self.attributes.keys().collect()
    }

    /// Clear all attributes
    pub fn clear_attributes(&mut self) {
        self.attributes.clear();
    }

    /// Get attribute count
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// Check if entity has any attributes
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Get string attribute with fallback
    pub fn get_string_attribute(&self, name: &str) -> Option<String> {
        self.get_typed_attribute::<String>(name)
    }

    /// Get number attribute with fallback
    pub fn get_number_attribute(&self, name: &str) -> Option<f64> {
        self.get_typed_attribute::<f64>(name)
    }

    /// Get boolean attribute with fallback
    pub fn get_bool_attribute(&self, name: &str) -> Option<bool> {
        self.get_typed_attribute::<bool>(name)
    }

    /// Set string attribute (optimized path)
    pub fn set_string_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), Value::from(value));
    }

    /// Set number attribute (optimized path)
    pub fn set_number_attribute(&mut self, name: &str, value: f64) {
        self.attributes.insert(name.to_string(), Value::from(value));
    }

    /// Set boolean attribute (optimized path)
    pub fn set_bool_attribute(&mut self, name: &str, value: bool) {
        self.attributes.insert(name.to_string(), Value::from(value));
    }

    /// Update multiple attributes at once
    pub fn update_attributes(&mut self, updates: HashMap<String, Value>) {
        self.attributes.extend(updates);
    }

    /// Clone with new ID
    pub fn clone_with_id(&self, new_id: &str) -> Self {
        Self {
            id: new_id.to_string(),
            entity_type: self.entity_type.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Clone with new type
    pub fn clone_with_type(&self, new_type: &str) -> Self {
        Self {
            id: self.id.clone(),
            entity_type: new_type.to_string(),
            attributes: self.attributes.clone(),
        }
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

        // Validate entity ID format (alphanumeric + underscore + hyphen)
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(GraphError::ValidationError(
                "Entity ID must contain only alphanumeric characters, underscores, and hyphens".to_string(),
            ));
        }

        // Validate entity type format
        if !self.entity_type.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(GraphError::ValidationError(
                "Entity type must contain only alphanumeric characters and underscores".to_string(),
            ));
        }

        // Check for reserved attribute names
        let reserved_names = ["id", "entity_type", "_metadata"];
        for reserved in &reserved_names {
            if self.attributes.contains_key(*reserved) {
                return Err(GraphError::ValidationError(
                    format!("Attribute name '{}' is reserved", reserved),
                ));
            }
        }

        Ok(())
    }

    fn to_node(&self) -> Node {
        let mut node = Node::new(self.id.clone(), &self.entity_type);

        // Add entity_type as a property for querying
        node = node.with_property("entity_type", serde_json::Value::String(self.entity_type.clone()));

        // Convert and add all attributes
        for (key, value) in &self.attributes {
            let json_value = match serde_json::to_string(value) {
                Ok(json_str) => serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null),
                Err(_) => serde_json::Value::Null,
            };
            node = node.with_property(key, json_value);
        }

        node
    }

    fn from_node(node: Node) -> Result<Self> {
        let mut json_attributes = node.properties.clone();

        // Extract entity_type from properties
        let entity_type = json_attributes
            .remove("entity_type")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        // Convert serde_json::Value to surrealdb::sql::Value for attributes
        let mut attributes = HashMap::with_capacity(json_attributes.len());
        for (key, value) in json_attributes {
            // Skip reserved names that shouldn't be in attributes
            if key == "id" || key == "_metadata" {
                continue;
            }

            let surreal_value = match serde_json::to_string(&value) {
                Ok(json_str) => match serde_json::from_str::<Value>(&json_str) {
                    Ok(val) => val,
                    Err(_) => Value::Null,
                },
                Err(_) => Value::Null,
            };
            attributes.insert(key, surreal_value);
        }

        let entity = Self {
            id: node.id,
            entity_type,
            attributes,
        };

        // Validate the constructed entity
        entity.validate()?;

        Ok(entity)
    }
}

/// Default implementations for common entity types
impl BaseEntity {
    /// Create a user entity
    pub fn user(id: &str) -> Self {
        Self::new(id, "user")
    }

    /// Create a document entity
    pub fn document(id: &str) -> Self {
        Self::new(id, "document")
    }

    /// Create a memory entity
    pub fn memory(id: &str) -> Self {
        Self::new(id, "memory")
    }

    /// Create a relationship entity
    pub fn relationship(id: &str) -> Self {
        Self::new(id, "relationship")
    }

    /// Create a session entity
    pub fn session(id: &str) -> Self {
        Self::new(id, "session")
    }

    /// Create a metadata entity
    pub fn metadata(id: &str) -> Self {
        Self::new(id, "metadata")
    }
}
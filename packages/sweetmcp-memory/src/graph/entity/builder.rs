//! Entity builder for fluent construction
//!
//! This module provides entity builder pattern with zero allocation
//! fast paths for efficient entity construction and configuration.

use super::core::{BaseEntity, Entity};
use crate::graph::graph_db::{GraphError, Result};
use serde::Serialize;
use std::collections::HashMap;
use surrealdb::sql::Value;

/// Entity builder for fluent construction with performance optimization
pub struct EntityBuilder {
    id: String,
    entity_type: String,
    attributes: HashMap<String, Value>,
}

impl EntityBuilder {
    /// Create a new entity builder with pre-allocated capacity
    pub fn new(id: &str, entity_type: &str) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::with_capacity(8), // Pre-allocate for common case
        }
    }

    /// Create with specific attribute capacity for zero-allocation building
    pub fn with_capacity(id: &str, entity_type: &str, capacity: usize) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::with_capacity(capacity),
        }
    }

    /// Add an attribute with type conversion
    pub fn attribute<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.attributes.insert(name.to_string(), value.into());
        self
    }

    /// Add typed attribute with serialization
    pub fn typed_attribute<T: Serialize>(mut self, name: &str, value: T) -> Result<Self> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let surreal_value = match serde_json::to_string(&json_value) {
            Ok(json_str) => serde_json::from_str::<Value>(&json_str)
                .map_err(|e| GraphError::SerializationError(e.to_string()))?,
            Err(e) => return Err(GraphError::SerializationError(e.to_string())),
        };
        
        self.attributes.insert(name.to_string(), surreal_value);
        Ok(self)
    }

    /// Add multiple attributes efficiently
    pub fn attributes<I>(mut self, attrs: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        self.attributes.extend(attrs);
        self
    }

    /// Add attributes from another entity
    pub fn inherit_attributes(mut self, entity: &BaseEntity) -> Self {
        self.attributes.extend(entity.attributes().clone());
        self
    }

    /// Add attributes conditionally
    pub fn maybe_attribute<T: Into<Value>>(self, name: &str, value: Option<T>) -> Self {
        match value {
            Some(v) => self.attribute(name, v),
            None => self,
        }
    }

    /// Bulk add attributes from vector for performance
    pub fn bulk_attributes(mut self, attrs: Vec<(String, Value)>) -> Self {
        self.attributes.reserve(attrs.len());
        for (key, value) in attrs {
            self.attributes.insert(key, value);
        }
        self
    }

    /// Set entity ID
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Set entity type
    pub fn entity_type(mut self, entity_type: &str) -> Self {
        self.entity_type = entity_type.to_string();
        self
    }

    /// Get current attribute count
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// Check if attribute exists
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// Remove an attribute
    pub fn remove_attribute(mut self, name: &str) -> Self {
        self.attributes.remove(name);
        self
    }

    /// Clear all attributes
    pub fn clear_attributes(mut self) -> Self {
        self.attributes.clear();
        self
    }

    /// Add multiple attributes from key-value pairs
    pub fn attribute_pairs<I, K, V>(mut self, pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<Value>,
    {
        for (key, value) in pairs {
            self.attributes.insert(key.into(), value.into());
        }
        self
    }

    /// Add string attribute (optimized for common case)
    pub fn string_attribute(mut self, name: &str, value: &str) -> Self {
        self.attributes.insert(name.to_string(), Value::from(value));
        self
    }

    /// Add number attribute (optimized for common case)
    pub fn number_attribute(mut self, name: &str, value: f64) -> Self {
        self.attributes.insert(name.to_string(), Value::from(value));
        self
    }

    /// Add boolean attribute (optimized for common case)
    pub fn bool_attribute(mut self, name: &str, value: bool) -> Self {
        self.attributes.insert(name.to_string(), Value::from(value));
        self
    }

    /// Build the entity without validation for performance
    pub fn build(self) -> BaseEntity {
        BaseEntity {
            id: self.id,
            entity_type: self.entity_type,
            attributes: self.attributes,
        }
    }

    /// Build and validate the entity
    pub fn build_validated(self) -> Result<BaseEntity> {
        let entity = self.build();
        entity.validate()?;
        Ok(entity)
    }

    /// Build with custom validation
    pub fn build_with_validation<F>(self, validator: F) -> Result<BaseEntity>
    where
        F: FnOnce(&BaseEntity) -> Result<()>,
    {
        let entity = self.build();
        entity.validate()?;
        validator(&entity)?;
        Ok(entity)
    }

    /// Preview the entity without consuming the builder
    pub fn preview(&self) -> BaseEntity {
        BaseEntity {
            id: self.id.clone(),
            entity_type: self.entity_type.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Validate the current builder state without building
    pub fn validate_current(&self) -> Result<()> {
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

        // Validate entity ID format
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

        Ok(())
    }

    /// Check if the builder is ready to build a valid entity
    pub fn is_valid(&self) -> bool {
        self.validate_current().is_ok()
    }

    /// Clone the builder for creating multiple similar entities
    pub fn clone_builder(&self) -> Self {
        Self {
            id: self.id.clone(),
            entity_type: self.entity_type.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Create a new builder based on this one with a different ID
    pub fn with_new_id(&self, new_id: &str) -> Self {
        let mut builder = self.clone_builder();
        builder.id = new_id.to_string();
        builder
    }

    /// Create a new builder based on this one with a different type
    pub fn with_new_type(&self, new_type: &str) -> Self {
        let mut builder = self.clone_builder();
        builder.entity_type = new_type.to_string();
        builder
    }

    /// Get a reference to the current attributes (for inspection)
    pub fn current_attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }

    /// Get the current ID
    pub fn current_id(&self) -> &str {
        &self.id
    }

    /// Get the current entity type
    pub fn current_entity_type(&self) -> &str {
        &self.entity_type
    }
}

impl Default for EntityBuilder {
    fn default() -> Self {
        Self::new("", "")
    }
}

impl Clone for EntityBuilder {
    fn clone(&self) -> Self {
        self.clone_builder()
    }
}

/// Convenience methods for creating commonly used entity builders
impl EntityBuilder {
    /// Create a user entity builder
    pub fn user(id: &str) -> Self {
        Self::new(id, "user")
    }

    /// Create a document entity builder
    pub fn document(id: &str) -> Self {
        Self::new(id, "document")
    }

    /// Create a memory entity builder
    pub fn memory(id: &str) -> Self {
        Self::new(id, "memory")
    }

    /// Create a relationship entity builder
    pub fn relationship(id: &str) -> Self {
        Self::new(id, "relationship")
    }

    /// Create a session entity builder
    pub fn session(id: &str) -> Self {
        Self::new(id, "session")
    }

    /// Create a metadata entity builder
    pub fn metadata(id: &str) -> Self {
        Self::new(id, "metadata")
    }
}
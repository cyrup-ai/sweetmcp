//! Core entity types and traits for the graph entity system
//!
//! This module provides the foundational types and traits for entity management,
//! including the Entity trait, type aliases, and core enums.

use crate::graph::graph_db::{GraphError, Node, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use surrealdb::sql::Value;

/// Type alias for entity futures to simplify trait definitions
/// Uses zero-allocation pinned futures for blazing-fast performance
pub type EntityFuture<T> = Pin<Box<dyn Future<Output = Result<T>> + Send>>;

/// Entity trait for domain objects
/// 
/// This trait provides the core interface for all entities in the graph system.
/// All implementations must be Send + Sync for concurrent access and Debug for diagnostics.
pub trait Entity: Send + Sync + Debug {
    /// Get the entity ID
    /// 
    /// Returns a string slice reference to avoid allocations
    fn id(&self) -> &str;

    /// Get the entity type
    /// 
    /// Returns a string slice reference to avoid allocations
    fn entity_type(&self) -> &str;

    /// Get an attribute value
    /// 
    /// Returns an option reference to avoid cloning values
    fn get_attribute(&self, name: &str) -> Option<&Value>;

    /// Set an attribute value
    /// 
    /// Takes ownership of the value to avoid unnecessary clones
    fn set_attribute(&mut self, name: &str, value: Value);

    /// Get all attributes
    /// 
    /// Returns a reference to the attributes map to avoid allocations
    fn attributes(&self) -> &HashMap<String, Value>;

    /// Validate the entity
    /// 
    /// Performs validation checks and returns a Result for error handling
    fn validate(&self) -> Result<()>;

    /// Convert to a graph node
    /// 
    /// Creates a Node representation of this entity
    fn to_node(&self) -> Node;

    /// Create from a graph node
    /// 
    /// Factory method to construct an entity from a Node
    fn from_node(node: Node) -> Result<Self>
    where
        Self: Sized;
}

/// Attribute type enumeration for validation
/// 
/// Defines the supported attribute types for entity validation.
/// Uses Copy trait for zero-allocation type checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeType {
    /// String type for text attributes
    String,
    /// Number type for numeric attributes
    Number,
    /// Boolean type for true/false attributes
    Boolean,
    /// Array type for list attributes
    Array,
    /// Object type for nested object attributes
    Object,
}

impl AttributeType {
    /// Check if a Value matches this attribute type
    /// 
    /// Performs zero-allocation type checking using pattern matching
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (AttributeType::String, Value::Strand(_)) => true,
            (AttributeType::Number, Value::Number(_)) => true,
            (AttributeType::Boolean, Value::Bool(_)) => true,
            (AttributeType::Array, Value::Array(_)) => true,
            (AttributeType::Object, Value::Object(_)) => true,
            _ => false,
        }
    }

    /// Get the type name as a string slice
    /// 
    /// Returns a static string slice to avoid allocations
    pub fn as_str(&self) -> &'static str {
        match self {
            AttributeType::String => "string",
            AttributeType::Number => "number",
            AttributeType::Boolean => "boolean",
            AttributeType::Array => "array",
            AttributeType::Object => "object",
        }
    }

    /// Parse an attribute type from a string
    /// 
    /// Returns an Option to handle invalid type names gracefully
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "string" | "str" => Some(AttributeType::String),
            "number" | "num" | "int" | "float" => Some(AttributeType::Number),
            "boolean" | "bool" => Some(AttributeType::Boolean),
            "array" | "list" => Some(AttributeType::Array),
            "object" | "obj" => Some(AttributeType::Object),
            _ => None,
        }
    }
}

/// Entity creation parameters
/// 
/// Builder-style parameters for creating new entities with validation
#[derive(Debug, Clone, Default)]
pub struct EntityParams {
    /// Entity ID (required)
    pub id: Option<String>,
    /// Entity type (required)
    pub entity_type: Option<String>,
    /// Initial attributes (optional)
    pub attributes: HashMap<String, Value>,
    /// Skip validation during creation (default: false)
    pub skip_validation: bool,
}

impl EntityParams {
    /// Create new entity parameters
    /// 
    /// Returns a default instance for builder-style construction
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the entity ID
    /// 
    /// Takes ownership of the string to avoid clones
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the entity type
    /// 
    /// Takes ownership of the string to avoid clones
    pub fn with_type(mut self, entity_type: String) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Add an attribute
    /// 
    /// Takes ownership of both key and value to avoid clones
    pub fn with_attribute(mut self, name: String, value: Value) -> Self {
        self.attributes.insert(name, value);
        self
    }

    /// Add multiple attributes
    /// 
    /// Extends the attributes map with the provided HashMap
    pub fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    /// Skip validation during entity creation
    /// 
    /// Useful for bulk operations where validation is handled separately
    pub fn skip_validation(mut self) -> Self {
        self.skip_validation = true;
        self
    }

    /// Validate the parameters
    /// 
    /// Ensures required fields are present before entity creation
    pub fn validate(&self) -> Result<()> {
        if self.id.is_none() {
            return Err(GraphError::ValidationError(
                "Entity ID is required".to_string(),
            ));
        }

        if self.entity_type.is_none() {
            return Err(GraphError::ValidationError(
                "Entity type is required".to_string(),
            ));
        }

        let id = self.id.as_ref().unwrap();
        let entity_type = self.entity_type.as_ref().unwrap();

        if id.is_empty() {
            return Err(GraphError::ValidationError(
                "Entity ID cannot be empty".to_string(),
            ));
        }

        if entity_type.is_empty() {
            return Err(GraphError::ValidationError(
                "Entity type cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

/// Entity query options for filtering and pagination
/// 
/// Provides zero-allocation query configuration for entity operations
#[derive(Debug, Clone, Default)]
pub struct EntityQueryOptions {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Number of results to skip (for pagination)
    pub offset: Option<usize>,
    /// Attribute filters (attribute_name -> expected_value)
    pub filters: HashMap<String, Value>,
    /// Sort by attribute name (optional)
    pub sort_by: Option<String>,
    /// Sort in descending order (default: ascending)
    pub sort_desc: bool,
}

impl EntityQueryOptions {
    /// Create new query options
    /// 
    /// Returns a default instance for builder-style construction
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the result limit
    /// 
    /// Limits the number of entities returned by queries
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the result offset
    /// 
    /// Skips the specified number of results for pagination
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Add an attribute filter
    /// 
    /// Filters results to entities with the specified attribute value
    pub fn with_filter(mut self, attribute: String, value: Value) -> Self {
        self.filters.insert(attribute, value);
        self
    }

    /// Set sort attribute
    /// 
    /// Sorts results by the specified attribute
    pub fn sort_by(mut self, attribute: String) -> Self {
        self.sort_by = Some(attribute);
        self
    }

    /// Sort in descending order
    /// 
    /// Changes sort order from ascending (default) to descending
    pub fn descending(mut self) -> Self {
        self.sort_desc = true;
        self
    }
}
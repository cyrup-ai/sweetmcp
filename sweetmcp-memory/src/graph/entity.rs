// src/graph/entity.rs
//! Entity model for Rust-mem0.
//!
//! This module provides a comprehensive entity model that maps domain objects
//! to graph nodes, with support for attributes, validation, and serialization.

use crate::graph::graph_db::{GraphDatabase, GraphError, GraphQueryOptions, Node, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
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

/// Base entity implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    /// Entity ID
    pub id: String,

    /// Entity type
    pub entity_type: String,

    /// Entity attributes
    pub attributes: HashMap<String, Value>,
}

impl BaseEntity {
    /// Create a new entity
    pub fn new(id: &str, entity_type: &str) -> Self {
        Self {
            id: id.to_string(),
            entity_type: entity_type.to_string(),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute
    pub fn with_attribute<T: Into<Value>>(mut self, name: &str, value: T) -> Self {
        self.attributes.insert(name.to_string(), value.into());
        self
    }

    /// Add multiple attributes
    pub fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes.extend(attributes);
        self
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

        Ok(())
    }

    fn to_node(&self) -> Node {
        let mut node = Node::new(self.id.clone(), &self.entity_type);

        for (key, value) in &self.attributes {
            // Convert surrealdb::sql::Value to serde_json::Value
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

        // Extract entity_type from attributes
        let entity_type = json_attributes
            .remove("entity_type")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        // Convert serde_json::Value to surrealdb::sql::Value
        let mut attributes = HashMap::new();
        for (key, value) in json_attributes {
            let surreal_value = match serde_json::to_string(&value) {
                Ok(json_str) => match serde_json::from_str::<surrealdb::sql::Value>(&json_str) {
                    Ok(sv) => sv,
                    Err(_) => surrealdb::sql::Value::Null,
                },
                Err(_) => surrealdb::sql::Value::Null,
            };
            attributes.insert(key, surreal_value);
        }

        Ok(Self {
            id: node.id,
            entity_type,
            attributes,
        })
    }
}

/// Entity validation rule
pub trait ValidationRule: Send + Sync {
    /// Validate an entity
    fn validate(&self, entity: &dyn Entity) -> Result<()>;

    /// Get the rule name
    fn name(&self) -> &str;

    /// Clone this validation rule
    fn clone_rule(&self) -> Box<dyn ValidationRule>;
}

/// Required attribute validation rule
pub struct RequiredAttributeRule {
    /// Rule name
    name: String,

    /// Required attribute name
    attribute: String,
}

impl RequiredAttributeRule {
    /// Create a new required attribute rule
    pub fn new(attribute: &str) -> Self {
        Self {
            name: format!("RequiredAttribute:{}", attribute),
            attribute: attribute.to_string(),
        }
    }
}

impl ValidationRule for RequiredAttributeRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if entity.get_attribute(&self.attribute).is_none() {
            return Err(GraphError::ValidationError(format!(
                "Required attribute '{}' is missing",
                self.attribute
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(RequiredAttributeRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
        })
    }
}

/// Attribute type validation rule
pub struct AttributeTypeRule {
    /// Rule name
    name: String,

    /// Attribute name
    attribute: String,

    /// Expected type
    expected_type: AttributeType,
}

impl AttributeTypeRule {
    /// Create a new attribute type rule
    pub fn new(attribute: &str, expected_type: AttributeType) -> Self {
        Self {
            name: format!("AttributeType:{}:{:?}", attribute, expected_type),
            attribute: attribute.to_string(),
            expected_type,
        }
    }
}

/// Attribute type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    /// String type
    String,
    /// Number type
    Number,
    /// Boolean type
    Boolean,
    /// Array type
    Array,
    /// Object type
    Object,
}

impl ValidationRule for AttributeTypeRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if let Some(value) = entity.get_attribute(&self.attribute) {
            let matches = match self.expected_type {
                AttributeType::String => matches!(value, Value::Strand(_)),
                AttributeType::Number => matches!(value, Value::Number(_)),
                AttributeType::Boolean => matches!(value, Value::Bool(_)),
                AttributeType::Array => matches!(value, Value::Array(_)),
                AttributeType::Object => matches!(value, Value::Object(_)),
            };

            if !matches {
                return Err(GraphError::ValidationError(format!(
                    "Attribute '{}' has incorrect type, expected {:?}",
                    self.attribute, self.expected_type
                )));
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(AttributeTypeRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
            expected_type: self.expected_type,
        })
    }
}

/// Custom validation rule
pub struct CustomValidationRule {
    /// Rule name
    name: String,

    /// Validation function
    validator: Box<dyn Fn(&dyn Entity) -> Result<()> + Send + Sync>,
}

impl CustomValidationRule {
    /// Create a new custom validation rule
    pub fn new(
        name: &str,
        validator: Box<dyn Fn(&dyn Entity) -> Result<()> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            validator,
        }
    }
}

impl ValidationRule for CustomValidationRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        (self.validator)(entity)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        // Note: Custom validation rules cannot be cloned due to closure constraints
        // This is a limitation of the current design
        panic!("CustomValidationRule cannot be cloned")
    }
}

/// Entity validator
pub struct EntityValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
}

impl EntityValidator {
    /// Create a new entity validator
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a validation rule
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Add a required attribute rule
    pub fn require_attribute(&mut self, attribute: &str) {
        self.add_rule(RequiredAttributeRule::new(attribute));
    }

    /// Add an attribute type rule
    pub fn require_attribute_type(&mut self, attribute: &str, expected_type: AttributeType) {
        self.add_rule(AttributeTypeRule::new(attribute, expected_type));
    }

    /// Add a custom validation rule
    pub fn add_custom_rule(
        &mut self,
        name: &str,
        validator: Box<dyn Fn(&dyn Entity) -> Result<()> + Send + Sync>,
    ) {
        self.add_rule(CustomValidationRule::new(name, validator));
    }

    /// Validate an entity
    pub fn validate(&self, entity: &dyn Entity) -> Result<()> {
        // First run the entity's own validation
        entity.validate()?;

        // Then run all the rules
        for rule in &self.rules {
            rule.validate(entity)?;
        }

        Ok(())
    }
}

impl Clone for EntityValidator {
    fn clone(&self) -> Self {
        let cloned_rules = self.rules.iter().map(|rule| rule.clone_rule()).collect();

        Self {
            rules: cloned_rules,
        }
    }
}

/// Entity repository trait
pub trait EntityRepository: Send + Sync {
    /// Create a new entity
    fn create(&self, entity: &dyn Entity) -> EntityFuture<Box<dyn Entity>>;

    /// Get an entity by ID
    fn get(&self, id: &str) -> EntityFuture<Option<Box<dyn Entity>>>;

    /// Update an entity
    fn update(&self, entity: &dyn Entity) -> EntityFuture<Box<dyn Entity>>;

    /// Delete an entity
    fn delete(&self, id: &str) -> EntityFuture<bool>;

    /// Find entities by type
    fn find_by_type(
        &self,
        entity_type: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> EntityFuture<Vec<Box<dyn Entity>>>;

    /// Find entities by attribute
    fn find_by_attribute(
        &self,
        attribute: &str,
        value: &Value,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> EntityFuture<Vec<Box<dyn Entity>>>;

    /// Count entities by type
    fn count_by_type(&self, entity_type: &str) -> EntityFuture<usize>;
}

/// SurrealDB entity repository
pub struct SurrealEntityRepository<E: Entity + 'static> {
    /// Graph database
    db: Arc<dyn GraphDatabase>,

    /// Entity validator
    validator: Option<EntityValidator>,

    /// Table name for this entity type
    pub table_name: String,

    /// Entity type
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Entity + 'static> SurrealEntityRepository<E> {
    /// Create a new SurrealDB entity repository
    pub fn new(db: Arc<dyn GraphDatabase>) -> Self {
        let table_name = std::any::type_name::<E>()
            .split("::")
            .last()
            .unwrap_or("entity")
            .to_lowercase();
        Self {
            db,
            validator: None,
            table_name,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new SurrealDB entity repository with validation
    pub fn with_validation(db: Arc<dyn GraphDatabase>, validator: EntityValidator) -> Self {
        let table_name = std::any::type_name::<E>()
            .split("::")
            .last()
            .unwrap_or("entity")
            .to_lowercase();
        Self {
            db,
            validator: Some(validator),
            table_name,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<E: Entity + Clone + 'static> EntityRepository for SurrealEntityRepository<E> {
    fn create(&self, entity: &dyn Entity) -> EntityFuture<Box<dyn Entity>> {
        // Validate the entity synchronously if a validator is configured
        if let Some(validator) = &self.validator {
            if let Err(e) = validator.validate(entity) {
                return Box::pin(async move { Err(e) });
            }
        }

        // Clone necessary data for the async block
        let db = self.db.clone();

        // Convert the entity to a node
        let node = entity.to_node();

        // Return a boxed future
        Box::pin(async move {
            // Create the node in the database
            let created_node_id = db.create_node(node.properties).await?;

            // Get the created node and convert back to an entity
            let created_node = db
                .get_node(&created_node_id)
                .await?
                .ok_or_else(|| GraphError::NodeNotFound(created_node_id.clone()))?;
            let created_entity = E::from_node(created_node)?;

            Ok(Box::new(created_entity) as Box<dyn Entity>)
        })
    }
    fn get(&self, id: &str) -> EntityFuture<Option<Box<dyn Entity>>> {
        // Clone necessary data for the async block
        let db = self.db.clone();
        let id_string = id.to_string();

        // Return a boxed future
        Box::pin(async move {
            // Get the node from the database
            let node_id = id_string;
            let node_option = db.get_node(&node_id).await?;

            // Convert the node to an entity if it exists
            match node_option {
                Some(node) => {
                    let entity = E::from_node(node)?;
                    Ok(Some(Box::new(entity) as Box<dyn Entity>))
                }
                None => Ok(None),
            }
        })
    }

    fn update(&self, entity: &dyn Entity) -> EntityFuture<Box<dyn Entity>> {
        // Validate the entity synchronously if a validator is configured
        if let Some(validator) = &self.validator {
            if let Err(e) = validator.validate(entity) {
                return Box::pin(async move { Err(e) });
            }
        }

        // Clone necessary data for the async block
        let db = self.db.clone();

        // Convert the entity to a node
        let node = entity.to_node();

        // Return a boxed future
        Box::pin(async move {
            // Update the node in the database
            db.update_node(&node.id, node.properties).await?;

            // Get the updated node and convert back to an entity
            let updated_node = db
                .get_node(&node.id)
                .await?
                .ok_or_else(|| GraphError::NodeNotFound(node.id.clone()))?;
            let updated_entity = E::from_node(updated_node)?;

            Ok(Box::new(updated_entity) as Box<dyn Entity>)
        })
    }

    fn delete(&self, id: &str) -> EntityFuture<bool> {
        // Clone necessary data for the async block
        let db = self.db.clone();
        let id_string = id.to_string();

        // Return a boxed future
        Box::pin(async move {
            // Delete the node from the database
            let node_id = id_string;
            match db.delete_node(&node_id).await {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        })
    }

    fn find_by_type(
        &self,
        entity_type: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> EntityFuture<Vec<Box<dyn Entity>>> {
        // Clone necessary data for the async block
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let entity_type_string = entity_type.to_string();
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        // Return a boxed future
        Box::pin(async move {
            // Build query with database-level pagination
            let query = format!(
                "SELECT * FROM {} WHERE entity_type = $entity_type LIMIT {} START {}",
                table_name, limit, offset
            );

            // Create query options with entity_type parameter
            let mut options = GraphQueryOptions::new();
            options.filters.insert(
                "entity_type".to_string(),
                serde_json::Value::String(entity_type_string),
            );

            // Execute query with pagination pushed to database
            let mut results_stream = db.query(&query, Some(options));

            // Collect results directly - no client-side filtering needed
            let mut entities: Vec<Box<dyn Entity>> = Vec::with_capacity(limit);
            use futures::StreamExt;
            while let Some(node_result) = results_stream.next().await {
                let node = node_result?;
                let entity = E::from_node(node)?;
                entities.push(Box::new(entity) as Box<dyn Entity>);
            }

            Ok(entities)
        })
    }

    fn find_by_attribute(
        &self,
        attribute: &str,
        value: &Value,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> EntityFuture<Vec<Box<dyn Entity>>> {
        // Clone necessary data for the async block
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let attribute_string = attribute.to_string();
        let value_clone = value.clone();
        let limit = limit.unwrap_or(100);

        // Return a boxed future
        Box::pin(async move {
            // Query the database directly for entities with the given attribute value
            let query = format!(
                "SELECT * FROM {} WHERE {} = $value LIMIT {} START {}",
                table_name,
                attribute_string,
                limit,
                offset.unwrap_or(0)
            );

            // Convert surrealdb::sql::Value to serde_json::Value for the filter
            let json_value = match serde_json::to_string(&value_clone) {
                Ok(json_str) => serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null),
                Err(_) => serde_json::Value::Null,
            };

            let mut options = GraphQueryOptions::new();
            options.filters.insert("value".to_string(), json_value);

            let mut results_stream = db.query(&query, Some(options));

            let mut entities: Vec<Box<dyn Entity>> = Vec::new();
            use futures::StreamExt;
            while let Some(node_result) = results_stream.next().await {
                if let Ok(node) = node_result {
                    if let Ok(entity) = E::from_node(node) {
                        entities.push(Box::new(entity) as Box<dyn Entity>);
                    }
                }
            }

            Ok(entities)
        })
    }

    fn count_by_type(&self, entity_type: &str) -> EntityFuture<usize> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let entity_type = entity_type.to_string();

        Box::pin(async move {
            let query = if entity_type.is_empty() {
                format!("SELECT count() FROM {} GROUP ALL", table_name)
            } else {
                format!(
                    "SELECT count() FROM {} WHERE entity_type = $entity_type GROUP ALL",
                    table_name
                )
            };

            let mut response_stream = if entity_type.is_empty() {
                db.query(&query, None)
            } else {
                let mut options = GraphQueryOptions::new();
                options.filters.insert(
                    "entity_type".to_string(),
                    serde_json::Value::String(entity_type),
                );
                db.query(&query, Some(options))
            };

            let mut count = 0;
            use futures::StreamExt;
            while let Some(result) = response_stream.next().await {
                if result.is_ok() {
                    count += 1;
                }
            }
            Ok(count)
        })
    }
}

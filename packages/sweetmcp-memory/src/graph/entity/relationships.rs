//! Entity relationship management
//!
//! This module provides comprehensive relationship management between entities
//! with support for typed relationships, bidirectional links, and relationship
//! queries with zero allocation fast paths and blazing-fast performance.

use crate::graph::graph_db::{GraphDatabase, GraphError, GraphQueryOptions, Node, Result};
use super::core::{Entity, EntityFuture};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use surrealdb::sql::Value;

/// Relationship between entities with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationship {
    /// Unique relationship ID
    pub id: String,
    
    /// Source entity ID
    pub from_entity_id: String,
    
    /// Target entity ID
    pub to_entity_id: String,
    
    /// Relationship type
    pub relationship_type: String,
    
    /// Relationship properties
    pub properties: HashMap<String, Value>,
    
    /// Relationship strength/weight
    pub weight: f64,
    
    /// Bidirectional flag
    pub bidirectional: bool,
    
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EntityRelationship {
    /// Create a new relationship
    pub fn new(
        from_entity_id: &str,
        to_entity_id: &str,
        relationship_type: &str,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from_entity_id: from_entity_id.to_string(),
            to_entity_id: to_entity_id.to_string(),
            relationship_type: relationship_type.to_string(),
            properties: HashMap::new(),
            weight: 1.0,
            bidirectional: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a bidirectional relationship
    pub fn bidirectional(
        entity_a_id: &str,
        entity_b_id: &str,
        relationship_type: &str,
    ) -> Self {
        let mut rel = Self::new(entity_a_id, entity_b_id, relationship_type);
        rel.bidirectional = true;
        rel
    }

    /// Add property to relationship
    pub fn with_property<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.properties.insert(key.to_string(), value.into());
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Set relationship weight
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Get property value
    #[inline]
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }

    /// Set property value
    pub fn set_property(&mut self, key: &str, value: Value) {
        self.properties.insert(key.to_string(), value);
        self.updated_at = chrono::Utc::now();
    }

    /// Remove property
    pub fn remove_property(&mut self, key: &str) -> Option<Value> {
        let result = self.properties.remove(key);
        if result.is_some() {
            self.updated_at = chrono::Utc::now();
        }
        result
    }

    /// Check if relationship involves entity
    #[inline]
    pub fn involves_entity(&self, entity_id: &str) -> bool {
        self.from_entity_id == entity_id || self.to_entity_id == entity_id
    }

    /// Get the other entity in the relationship
    pub fn get_other_entity_id(&self, entity_id: &str) -> Option<&str> {
        if self.from_entity_id == entity_id {
            Some(&self.to_entity_id)
        } else if self.to_entity_id == entity_id {
            Some(&self.from_entity_id)
        } else {
            None
        }
    }

    /// Validate relationship
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(GraphError::ValidationError("Relationship ID cannot be empty".to_string()));
        }
        if self.from_entity_id.is_empty() {
            return Err(GraphError::ValidationError("From entity ID cannot be empty".to_string()));
        }
        if self.to_entity_id.is_empty() {
            return Err(GraphError::ValidationError("To entity ID cannot be empty".to_string()));
        }
        if self.relationship_type.is_empty() {
            return Err(GraphError::ValidationError("Relationship type cannot be empty".to_string()));
        }
        if self.from_entity_id == self.to_entity_id {
            return Err(GraphError::ValidationError("Self-relationships not allowed".to_string()));
        }
        Ok(())
    }
}

/// Relationship manager for handling entity relationships
pub struct RelationshipManager {
    db: Arc<dyn GraphDatabase>,
    table_name: String,
}

impl RelationshipManager {
    /// Create a new relationship manager
    pub fn new(db: Arc<dyn GraphDatabase>, table_name: &str) -> Self {
        Self {
            db,
            table_name: table_name.to_string(),
        }
    }

    /// Create a new relationship
    pub fn create_relationship(&self, relationship: EntityRelationship) -> EntityFuture<EntityRelationship> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();

        Box::pin(async move {
            // Validate relationship
            relationship.validate()?;

            // Convert to node for storage
            let node = Node {
                id: relationship.id.clone(),
                labels: vec!["relationship".to_string(), relationship.relationship_type.clone()],
                properties: {
                    let mut props = relationship.properties.clone();
                    props.insert("from_entity_id".to_string(), relationship.from_entity_id.clone().into());
                    props.insert("to_entity_id".to_string(), relationship.to_entity_id.clone().into());
                    props.insert("weight".to_string(), relationship.weight.into());
                    props.insert("bidirectional".to_string(), relationship.bidirectional.into());
                    props.insert("created_at".to_string(), relationship.created_at.to_rfc3339().into());
                    props.insert("updated_at".to_string(), relationship.updated_at.to_rfc3339().into());
                    props
                },
            };

            // Store in database
            db.create_node(&table_name, node).await?;

            Ok(relationship)
        })
    }

    /// Get relationship by ID
    pub fn get_relationship(&self, relationship_id: &str) -> EntityFuture<Option<EntityRelationship>> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let relationship_id = relationship_id.to_string();

        Box::pin(async move {
            match db.get_node(&table_name, &relationship_id).await? {
                Some(node) => {
                    let relationship = Self::node_to_relationship(node)?;
                    Ok(Some(relationship))
                }
                None => Ok(None),
            }
        })
    }

    /// Get relationships for an entity
    pub fn get_entity_relationships(&self, entity_id: &str) -> EntityFuture<Vec<EntityRelationship>> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let entity_id = entity_id.to_string();

        Box::pin(async move {
            let query = format!(
                "SELECT * FROM {} WHERE from_entity_id = $entity_id OR to_entity_id = $entity_id",
                table_name
            );

            let mut options = GraphQueryOptions::new();
            options.filters.insert("entity_id".to_string(), entity_id.into());

            let mut results_stream = db.query(&query, Some(options));
            let mut relationships = Vec::new();

            use futures::StreamExt;
            while let Some(node_result) = results_stream.next().await {
                if let Ok(node) = node_result {
                    if let Ok(relationship) = Self::node_to_relationship(node) {
                        relationships.push(relationship);
                    }
                }
            }

            Ok(relationships)
        })
    }

    /// Get relationships by type
    pub fn get_relationships_by_type(&self, relationship_type: &str) -> EntityFuture<Vec<EntityRelationship>> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let relationship_type = relationship_type.to_string();

        Box::pin(async move {
            let query = format!(
                "SELECT * FROM {} WHERE $relationship_type IN labels",
                table_name
            );

            let mut options = GraphQueryOptions::new();
            options.filters.insert("relationship_type".to_string(), relationship_type.into());

            let mut results_stream = db.query(&query, Some(options));
            let mut relationships = Vec::new();

            use futures::StreamExt;
            while let Some(node_result) = results_stream.next().await {
                if let Ok(node) = node_result {
                    if let Ok(relationship) = Self::node_to_relationship(node) {
                        relationships.push(relationship);
                    }
                }
            }

            Ok(relationships)
        })
    }

    /// Update relationship
    pub fn update_relationship(&self, relationship: EntityRelationship) -> EntityFuture<EntityRelationship> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();

        Box::pin(async move {
            // Validate relationship
            relationship.validate()?;

            // Convert to node for update
            let node = Node {
                id: relationship.id.clone(),
                labels: vec!["relationship".to_string(), relationship.relationship_type.clone()],
                properties: {
                    let mut props = relationship.properties.clone();
                    props.insert("from_entity_id".to_string(), relationship.from_entity_id.clone().into());
                    props.insert("to_entity_id".to_string(), relationship.to_entity_id.clone().into());
                    props.insert("weight".to_string(), relationship.weight.into());
                    props.insert("bidirectional".to_string(), relationship.bidirectional.into());
                    props.insert("created_at".to_string(), relationship.created_at.to_rfc3339().into());
                    props.insert("updated_at".to_string(), relationship.updated_at.to_rfc3339().into());
                    props
                },
            };

            // Update in database
            db.update_node(&table_name, node).await?;

            Ok(relationship)
        })
    }

    /// Delete relationship
    pub fn delete_relationship(&self, relationship_id: &str) -> EntityFuture<bool> {
        let db = self.db.clone();
        let table_name = self.table_name.clone();
        let relationship_id = relationship_id.to_string();

        Box::pin(async move {
            db.delete_node(&table_name, &relationship_id).await
        })
    }

    /// Convert node to relationship
    fn node_to_relationship(node: Node) -> Result<EntityRelationship> {
        let from_entity_id = node.properties.get("from_entity_id")
            .and_then(|v| match v {
                Value::Strand(s) => Some(s.to_string()),
                _ => None,
            })
            .ok_or_else(|| GraphError::ValidationError("Missing from_entity_id".to_string()))?;

        let to_entity_id = node.properties.get("to_entity_id")
            .and_then(|v| match v {
                Value::Strand(s) => Some(s.to_string()),
                _ => None,
            })
            .ok_or_else(|| GraphError::ValidationError("Missing to_entity_id".to_string()))?;

        let relationship_type = node.labels.iter()
            .find(|label| *label != "relationship")
            .ok_or_else(|| GraphError::ValidationError("Missing relationship type".to_string()))?
            .clone();

        let weight = node.properties.get("weight")
            .and_then(|v| match v {
                Value::Number(n) => n.as_float(),
                _ => None,
            })
            .unwrap_or(1.0);

        let bidirectional = node.properties.get("bidirectional")
            .and_then(|v| match v {
                Value::Bool(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false);

        let created_at = node.properties.get("created_at")
            .and_then(|v| match v {
                Value::Strand(s) => chrono::DateTime::parse_from_rfc3339(&s.to_string()).ok(),
                _ => None,
            })
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let updated_at = node.properties.get("updated_at")
            .and_then(|v| match v {
                Value::Strand(s) => chrono::DateTime::parse_from_rfc3339(&s.to_string()).ok(),
                _ => None,
            })
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let mut properties = node.properties.clone();
        // Remove system properties
        properties.remove("from_entity_id");
        properties.remove("to_entity_id");
        properties.remove("weight");
        properties.remove("bidirectional");
        properties.remove("created_at");
        properties.remove("updated_at");

        Ok(EntityRelationship {
            id: node.id,
            from_entity_id,
            to_entity_id,
            relationship_type,
            properties,
            weight,
            bidirectional,
            created_at,
            updated_at,
        })
    }
}
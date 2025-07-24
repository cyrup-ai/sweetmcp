//! Extended entity with metadata support
//!
//! This module provides extended entity functionality with lifecycle
//! metadata tracking and versioning support.

use super::core::{BaseEntity, Entity};
use crate::graph::graph_db::{Node, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Value;

/// Entity metadata for tracking entity lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub tags: Vec<String>,
}

impl EntityMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: None,
            updated_by: None,
            tags: Vec::new(),
        }
    }

    /// Create with creator
    pub fn new_with_creator(created_by: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            version: 1,
            created_by: Some(created_by),
            updated_by: None,
            tags: Vec::new(),
        }
    }

    /// Update metadata for modification
    pub fn update(&mut self, updated_by: Option<String>) {
        self.updated_at = chrono::Utc::now();
        self.version += 1;
        self.updated_by = updated_by;
    }

    /// Add tag if not already present
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Add multiple tags
    pub fn add_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.add_tag(tag);
        }
    }

    /// Remove tag
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Check if has tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.created_at).num_seconds()
    }

    /// Get time since last update in seconds
    pub fn update_age_seconds(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.updated_at).num_seconds()
    }

    /// Check if entity was recently created (within threshold)
    pub fn is_recent(&self, threshold_seconds: i64) -> bool {
        self.age_seconds() <= threshold_seconds
    }

    /// Check if entity was recently updated (within threshold)
    pub fn is_recently_updated(&self, threshold_seconds: i64) -> bool {
        self.update_age_seconds() <= threshold_seconds
    }

    /// Clear all tags
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }

    /// Get tag count
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }
}

impl Default for EntityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended entity with metadata and lifecycle tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedEntity {
    pub entity: BaseEntity,
    pub metadata: EntityMetadata,
}

impl ExtendedEntity {
    /// Create new extended entity  
    pub fn new(entity: BaseEntity) -> Self {
        Self {
            entity,
            metadata: EntityMetadata::new(),
        }
    }

    /// Create with creator information
    pub fn new_with_creator(entity: BaseEntity, created_by: String) -> Self {
        Self {
            entity,
            metadata: EntityMetadata::new_with_creator(created_by),
        }
    }

    /// Create with existing metadata
    pub fn with_metadata(entity: BaseEntity, metadata: EntityMetadata) -> Self {
        Self { entity, metadata }
    }

    /// Update entity and metadata
    pub fn update_entity(&mut self, entity: BaseEntity, updated_by: Option<String>) {
        self.entity = entity;
        self.metadata.update(updated_by);
    }

    /// Update entity attributes and metadata
    pub fn update_attributes(&mut self, attributes: HashMap<String, Value>, updated_by: Option<String>) {
        self.entity.attributes.extend(attributes);
        self.metadata.update(updated_by);
    }

    /// Add tag to entity
    pub fn add_tag(&mut self, tag: String) {
        self.metadata.add_tag(tag);
    }

    /// Remove tag from entity
    pub fn remove_tag(&mut self, tag: &str) {
        self.metadata.remove_tag(tag);
    }

    /// Check if entity has tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.has_tag(tag)
    }

    /// Get entity age in seconds
    pub fn age_seconds(&self) -> i64 {
        self.metadata.age_seconds()
    }

    /// Check if entity is recent
    pub fn is_recent(&self, threshold_seconds: i64) -> bool {
        self.metadata.is_recent(threshold_seconds)
    }

    /// Check if entity was recently updated
    pub fn is_recently_updated(&self, threshold_seconds: i64) -> bool {
        self.metadata.is_recently_updated(threshold_seconds)
    }

    /// Get the underlying base entity
    pub fn base_entity(&self) -> &BaseEntity {
        &self.entity
    }

    /// Get mutable reference to base entity (updates metadata)
    pub fn base_entity_mut(&mut self, updated_by: Option<String>) -> &mut BaseEntity {
        self.metadata.update(updated_by);
        &mut self.entity
    }

    /// Get entity metadata
    pub fn metadata(&self) -> &EntityMetadata {
        &self.metadata
    }

    /// Get mutable reference to metadata
    pub fn metadata_mut(&mut self) -> &mut EntityMetadata {
        &mut self.metadata
    }

    /// Clone the underlying entity without metadata
    pub fn clone_base_entity(&self) -> BaseEntity {
        self.entity.clone()
    }

    /// Convert to base entity, discarding metadata
    pub fn into_base_entity(self) -> BaseEntity {
        self.entity
    }
}

impl Entity for ExtendedEntity {
    fn id(&self) -> &str {
        self.entity.id()
    }

    fn entity_type(&self) -> &str {
        self.entity.entity_type()
    }

    fn get_attribute(&self, name: &str) -> Option<&Value> {
        self.entity.get_attribute(name)
    }

    fn set_attribute(&mut self, name: &str, value: Value) {
        self.entity.set_attribute(name, value);
        self.metadata.update(None);
    }

    fn attributes(&self) -> &HashMap<String, Value> {
        self.entity.attributes()
    }

    fn validate(&self) -> Result<()> {
        self.entity.validate()
    }

    fn to_node(&self) -> Node {
        let mut node = self.entity.to_node();
        
        // Add metadata as properties with prefix to avoid conflicts
        if let Ok(metadata_json) = serde_json::to_value(&self.metadata) {
            node = node.with_property("_metadata", metadata_json);
        }
        
        node
    }

    fn from_node(node: Node) -> Result<Self> {
        let mut properties = node.properties.clone();
        
        // Extract metadata if present
        let metadata = properties
            .remove("_metadata")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        
        // Reconstruct node without metadata for base entity
        let cleaned_node = Node {
            id: node.id,
            labels: node.labels,
            properties,
        };
        
        let entity = BaseEntity::from_node(cleaned_node)?;
        
        Ok(Self::with_metadata(entity, metadata))
    }
}
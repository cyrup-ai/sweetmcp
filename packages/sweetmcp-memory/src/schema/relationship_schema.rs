// src/schema/relationship_schema.rs
//! Relationship schema definition.

use crate::utils;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use surrealdb::RecordId;

/// Type alias for backwards compatibility
pub type RelationshipSchema = Relationship;

/// Relationship schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Relationship ID
    pub id: RecordId,
    /// Source memory ID
    pub source_id: String,
    /// Target memory ID
    pub target_id: String,
    /// Relationship type
    pub relationship_type: String,
    /// Relationship metadata
    pub metadata: Value,
    /// Creation timestamp (milliseconds since epoch)
    pub created_at: u64,
    /// Last update timestamp (milliseconds since epoch)
    pub updated_at: u64,
    /// Relationship strength (0.0 to 1.0)
    #[serde(default)]
    pub strength: f32,
    /// Additional fields
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub additional_fields: HashMap<String, Value>,
}

impl Relationship {
    /// Create a new relationship
    pub fn new(
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relationship_type: impl Into<String>,
    ) -> Self {
        let now = utils::current_timestamp_ms();
        let id = utils::generate_id();

        Self {
            id: RecordId::from(("memory_relationship", id.as_str())),
            source_id: source_id.into(),
            target_id: target_id.into(),
            relationship_type: relationship_type.into(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            strength: 1.0,
            additional_fields: HashMap::new(),
        }
    }

    /// Create a new relationship with ID
    pub fn new_with_id(
        id: impl Into<String>,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relationship_type: impl Into<String>,
    ) -> Self {
        let now = utils::current_timestamp_ms();
        let id_str = id.into();

        Self {
            id: RecordId::from(("memory_relationship", id_str.as_str())),
            source_id: source_id.into(),
            target_id: target_id.into(),
            relationship_type: relationship_type.into(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            strength: 1.0,
            additional_fields: HashMap::new(),
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set strength
    pub fn with_strength(mut self, strength: f32) -> Self {
        self.strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Update metadata
    pub fn update_metadata(&mut self, metadata: Value) {
        self.metadata = metadata;
        self.updated_at = utils::current_timestamp_ms();
    }

    /// Update strength
    pub fn update_strength(&mut self, strength: f32) {
        self.strength = strength.clamp(0.0, 1.0);
        self.updated_at = utils::current_timestamp_ms();
    }

    /// Get creation timestamp as ISO 8601 string
    pub fn created_at_iso8601(&self) -> String {
        utils::timestamp_to_iso8601(self.created_at)
    }

    /// Get update timestamp as ISO 8601 string
    pub fn updated_at_iso8601(&self) -> String {
        utils::timestamp_to_iso8601(self.updated_at)
    }

    /// Get metadata value
    pub fn get_metadata_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        if let serde_json::Value::Object(ref map) = self.metadata {
            map.get(key)
                .and_then(|value| serde_json::from_value(value.clone()).ok())
        } else {
            None
        }
    }

    /// Set metadata value
    pub fn set_metadata_value<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(value)?;

        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.to_string(), value);
        } else {
            // If metadata is not an object, create a new object
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), value);
            self.metadata = serde_json::Value::Object(map);
        }

        self.updated_at = utils::current_timestamp_ms();
        Ok(())
    }

    /// Remove metadata value
    pub fn remove_metadata_value(&mut self, key: &str) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.remove(key);
        }

        self.updated_at = utils::current_timestamp_ms();
    }

    /// Get additional field value
    pub fn get_additional_field<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.additional_fields
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// Set additional field value
    pub fn set_additional_field<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(value)?;
        self.additional_fields.insert(key.to_string(), value);
        self.updated_at = utils::current_timestamp_ms();
        Ok(())
    }

    /// Remove additional field
    pub fn remove_additional_field(&mut self, key: &str) {
        self.additional_fields.remove(key);
        self.updated_at = utils::current_timestamp_ms();
    }

    /// Check if this relationship is between the specified memories
    pub fn is_between(&self, memory_id1: &str, memory_id2: &str) -> bool {
        (self.source_id == memory_id1 && self.target_id == memory_id2)
            || (self.source_id == memory_id2 && self.target_id == memory_id1)
    }

    /// Check if this relationship involves the specified memory
    pub fn involves(&self, memory_id: &str) -> bool {
        self.source_id == memory_id || self.target_id == memory_id
    }

    /// Get the other memory ID in the relationship
    pub fn get_other_memory_id(&self, memory_id: &str) -> Option<&str> {
        if self.source_id == memory_id {
            Some(&self.target_id)
        } else if self.target_id == memory_id {
            Some(&self.source_id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_creation() {
        let relationship = Relationship::new("source-id", "target-id", "related_to");

        // Check that the ID has been generated (RecordId always has a value)
        let expected_table = "memory_relationship";
        assert!(relationship.id.to_string().starts_with(expected_table));
        assert_eq!(relationship.source_id, "source-id");
        assert_eq!(relationship.target_id, "target-id");
        assert_eq!(relationship.relationship_type, "related_to");
        assert_eq!(
            relationship.metadata,
            serde_json::Value::Object(serde_json::Map::new())
        );
        assert_eq!(relationship.created_at, relationship.updated_at);
        assert_eq!(relationship.strength, 1.0);
        assert!(relationship.additional_fields.is_empty());
    }

    #[test]
    fn test_relationship_with_id() {
        let relationship =
            Relationship::new_with_id("rel-id", "source-id", "target-id", "related_to");

        assert_eq!(
            relationship.id,
            RecordId::from(("memory_relationship", "rel-id"))
        );
        assert_eq!(relationship.source_id, "source-id");
        assert_eq!(relationship.target_id, "target-id");
        assert_eq!(relationship.relationship_type, "related_to");
    }

    #[test]
    fn test_relationship_builder_pattern() {
        let relationship = Relationship::new("source-id", "target-id", "related_to")
            .with_metadata(serde_json::json!({"key": "value"}))
            .with_strength(0.75);

        if let serde_json::Value::Object(map) = &relationship.metadata {
            assert!(map.contains_key("key"));
            assert_eq!(map.get("key").unwrap(), &serde_json::json!("value"));
        } else {
            panic!("Expected metadata to be an object");
        }
        assert_eq!(relationship.strength, 0.75);
    }

    #[test]
    fn test_relationship_update_methods() {
        let mut relationship = Relationship::new("source-id", "target-id", "related_to");
        let original_created_at = relationship.created_at;
        let original_updated_at = relationship.updated_at;

        // Wait a bit to ensure timestamps are different
        std::thread::sleep(std::time::Duration::from_millis(10));

        relationship.update_metadata(serde_json::json!({"key": "updated"}));
        relationship.update_strength(0.5);

        if let serde_json::Value::Object(map) = &relationship.metadata {
            assert!(map.contains_key("key"));
            assert_eq!(map.get("key").unwrap(), &serde_json::json!("updated"));
        } else {
            panic!("Expected metadata to be an object");
        }
        assert_eq!(relationship.strength, 0.5);

        // Check timestamps
        assert_eq!(relationship.created_at, original_created_at);
        assert!(relationship.updated_at > original_updated_at);
    }

    #[test]
    fn test_relationship_metadata() {
        let mut relationship = Relationship::new("source-id", "target-id", "related_to");

        // Set metadata value
        relationship.set_metadata_value("number", 42).unwrap();
        relationship.set_metadata_value("string", "value").unwrap();

        // Get metadata value
        assert_eq!(relationship.get_metadata_value::<i32>("number"), Some(42));
        assert_eq!(
            relationship.get_metadata_value::<String>("string"),
            Some("value".to_string())
        );
        assert_eq!(relationship.get_metadata_value::<bool>("nonexistent"), None);

        // Remove metadata value
        relationship.remove_metadata_value("number");
        assert_eq!(relationship.get_metadata_value::<i32>("number"), None);
    }

    #[test]
    fn test_relationship_additional_fields() {
        let mut relationship = Relationship::new("source-id", "target-id", "related_to");

        // Set additional field
        relationship.set_additional_field("field1", 123).unwrap();
        relationship
            .set_additional_field("field2", "value")
            .unwrap();

        // Get additional field
        assert_eq!(
            relationship.get_additional_field::<i32>("field1"),
            Some(123)
        );
        assert_eq!(
            relationship.get_additional_field::<String>("field2"),
            Some("value".to_string())
        );
        assert_eq!(
            relationship.get_additional_field::<bool>("nonexistent"),
            None
        );

        // Remove additional field
        relationship.remove_additional_field("field1");
        assert_eq!(relationship.get_additional_field::<i32>("field1"), None);
    }

    #[test]
    fn test_relationship_timestamp_iso8601() {
        let relationship = Relationship::new("source-id", "target-id", "related_to");

        let created_iso = relationship.created_at_iso8601();
        let updated_iso = relationship.updated_at_iso8601();

        assert!(!created_iso.is_empty());
        assert!(!updated_iso.is_empty());
        assert_eq!(created_iso, updated_iso);
    }

    #[test]
    fn test_relationship_between_and_involves() {
        let relationship = Relationship::new("memory1", "memory2", "related_to");

        assert!(relationship.is_between("memory1", "memory2"));
        assert!(relationship.is_between("memory2", "memory1"));
        assert!(!relationship.is_between("memory1", "memory3"));

        assert!(relationship.involves("memory1"));
        assert!(relationship.involves("memory2"));
        assert!(!relationship.involves("memory3"));

        assert_eq!(relationship.get_other_memory_id("memory1"), Some("memory2"));
        assert_eq!(relationship.get_other_memory_id("memory2"), Some("memory1"));
        assert_eq!(relationship.get_other_memory_id("memory3"), None);
    }

    #[test]
    fn test_relationship_serialization() {
        let relationship = Relationship::new("source-id", "target-id", "related_to")
            .with_metadata(serde_json::json!({"key": "value"}))
            .with_strength(0.75);

        let json = serde_json::to_string(&relationship).unwrap();
        let deserialized: Relationship = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, relationship.id);
        assert_eq!(deserialized.source_id, relationship.source_id);
        assert_eq!(deserialized.target_id, relationship.target_id);
        assert_eq!(
            deserialized.relationship_type,
            relationship.relationship_type
        );
        assert_eq!(deserialized.metadata, relationship.metadata);
        assert_eq!(deserialized.strength, relationship.strength);
    }
}

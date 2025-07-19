// src/schema/memory_schema.rs
//! Defines the schema for memory nodes.

use serde::{Deserialize, Serialize};
use crate::schema::MemoryType; // Correctly refers to MemoryType from src/schema/mod.rs
use crate::utils; // For utility functions like generate_id and current_timestamp_ms

/// Represents a memory node in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub id: String,
    pub r#type: MemoryType, // Renamed to avoid keyword conflict with `type`
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at: u64, // Timestamp in milliseconds
    pub updated_at: u64, // Timestamp in milliseconds
    pub last_accessed_at: u64, // Timestamp in milliseconds
    pub score: Option<f32>,      // Optional score, e.g., from search results
    // Relationships are typically handled by a separate edge collection in SurrealDB
    // or by direct links. For simplicity here, we might not store them directly in the node,
    // or if we do, it would be a list of relationship IDs.
    // pub relationships: Vec<String>, // IDs of related MemoryRelationship objects
}

impl Memory {
    /// Creates a new memory node.
    pub fn new(content: String, memory_type: MemoryType) -> Self {
        let now = utils::current_timestamp_ms();
        Self {
            id: utils::generate_id(),
            r#type: memory_type,
            content,
            embedding: None,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            last_accessed_at: now,
            score: None,
            // relationships: Vec::new(),
        }
    }

    /// Updates the last accessed timestamp.
    pub fn touch(&mut self) {
        self.last_accessed_at = utils::current_timestamp_ms();
    }

    /// Sets an embedding for the memory node.
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
        self.updated_at = utils::current_timestamp_ms();
    }

    /// Adds or updates a metadata field.
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key, value);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key, value);
            self.metadata = serde_json::Value::Object(map);
        }
        self.updated_at = utils::current_timestamp_ms();
    }

    /// Removes a metadata field.
    pub fn remove_metadata(&mut self, key: &str) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.remove(key);
        }
        self.updated_at = utils::current_timestamp_ms();
    }
}



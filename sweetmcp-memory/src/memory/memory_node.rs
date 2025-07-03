// src/memory/memory_node.rs
//! Memory node implementation for the memory system.
//! This module defines the core data structures for memory nodes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;

use super::memory_metadata::MemoryMetadata;

/// Types of memory that can be stored in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Episodic memory - experiences and events
    Episodic,
    /// Semantic memory - facts and knowledge
    Semantic,
    /// Procedural memory - skills and procedures
    Procedural,
    /// Custom memory type
    Custom(String),
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Episodic => write!(f, "episodic"),
            MemoryType::Semantic => write!(f, "semantic"),
            MemoryType::Procedural => write!(f, "procedural"),
            MemoryType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

impl From<&str> for MemoryType {
    fn from(s: &str) -> Self {
        match s {
            "episodic" => MemoryType::Episodic,
            "semantic" => MemoryType::Semantic,
            "procedural" => MemoryType::Procedural,
            s if s.starts_with("custom:") => {
                MemoryType::Custom(s.strip_prefix("custom:").unwrap_or(s).to_string())
            }
            _ => MemoryType::Custom(s.to_string()),
        }
    }
}

/// A memory node in the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    /// Unique identifier for the memory
    pub id: String,
    /// Content of the memory
    pub content: String,
    /// Type of memory
    pub memory_type: MemoryType,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Embedding vector
    pub embedding: Option<Vec<f32>>,
    /// Metadata associated with the memory
    pub metadata: MemoryMetadata,
}

impl MemoryNode {
    /// Create a new memory node
    pub fn new(content: String, memory_type: MemoryType) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id,
            content,
            memory_type,
            created_at: now,
            updated_at: now,
            embedding: None,
            metadata: MemoryMetadata::new(),
        }
    }

    /// Create a new memory node with a specific ID
    pub fn with_id(id: String, content: String, memory_type: MemoryType) -> Self {
        let now = Utc::now();

        Self {
            id,
            content,
            memory_type,
            created_at: now,
            updated_at: now,
            embedding: None,
            metadata: MemoryMetadata::new(),
        }
    }

    /// Set the embedding for this memory
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding.clone());
        self.metadata.embedding = Some(embedding);
        self
    }

    /// Set the importance for this memory
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.metadata.importance = importance;
        self
    }

    /// Add custom metadata to this memory
    pub fn with_custom_metadata(mut self, key: String, value: String) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.metadata.custom {
            map.insert(key, serde_json::Value::String(value));
        }
        self
    }

    /// Update the last accessed time
    pub fn update_last_accessed(&mut self) {
        self.metadata.last_accessed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

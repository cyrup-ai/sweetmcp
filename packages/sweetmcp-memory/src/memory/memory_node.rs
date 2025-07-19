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

/// A memory node in the memory system - cache-line aligned for optimal performance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(align(64))] // Cache-line alignment for CPU cache efficiency
pub struct MemoryNode {
    // Hot fields (frequently accessed during search/retrieval) placed first
    /// Unique identifier for the memory
    pub id: String,
    /// Embedding vector - hot path for similarity search
    pub embedding: Option<Vec<f32>>,
    /// Type of memory - hot path for filtering
    pub memory_type: MemoryType,
    /// Content of the memory - accessed during retrieval
    pub content: String,
    
    // Cold fields (less frequently accessed) placed last
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Metadata associated with the memory
    pub metadata: MemoryMetadata,
}

// Compile-time assertion to validate MemoryNode size efficiency
const _: () = {
    const CACHE_LINE_SIZE: usize = 64;
    // Ensure MemoryNode is reasonably sized for cache efficiency
    // This is a soft check - the struct can be larger than one cache line,
    // but we want to be aware of its size
    const MEMORY_NODE_SIZE: usize = std::mem::size_of::<MemoryNode>();
    
    // Log the size at compile time for optimization awareness
    // If this becomes too large, consider using Box<str> for content
    // or other size optimizations
    const _SIZE_CHECK: () = {
        // For now, just validate it's not excessively large (8 cache lines)
        // In practice, large structs are acceptable if they're accessed infrequently
        if MEMORY_NODE_SIZE > CACHE_LINE_SIZE * 8 { // Allow up to 8 cache lines
            panic!("MemoryNode size is excessively large for any reasonable use");
        }
        // Generate a compile-time size report
        ["MemoryNode size: "; MEMORY_NODE_SIZE]; // This will show in compiler output
    };
    _SIZE_CHECK
};

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

//! Database schema for memory nodes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::memory::memory_node::MemoryType;

/// Database schema for memory nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNodeSchema {
    /// Unique identifier
    pub id: RecordId,
    /// Content of the memory
    pub content: String,
    /// Type of memory
    pub memory_type: MemoryType,
    /// Metadata associated with the memory
    pub metadata: MemoryMetadataSchema,
}

/// Database schema for memory metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadataSchema {
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last accessed time
    pub last_accessed_at: DateTime<Utc>,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Vector embedding
    pub embedding: Option<Vec<f32>>,
    /// Custom metadata
    pub custom: serde_json::Value,
}

/// Public memory type for API access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique identifier
    pub id: String,
    /// Content of the memory
    pub content: String,
    /// Type of memory
    pub memory_type: String,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last accessed time
    pub last_accessed_at: DateTime<Utc>,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Tags
    pub tags: Vec<String>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

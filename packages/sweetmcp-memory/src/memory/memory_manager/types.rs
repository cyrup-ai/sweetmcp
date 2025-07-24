//! Data structures and schemas for memory manager
//!
//! This module provides the core data structures used by the memory manager,
//! including content structures for creating/updating memory nodes and relationships.

use serde::{Deserialize, Serialize};
use crate::memory::memory_node::{MemoryNode, MemoryType};
use crate::memory::memory_relationship::MemoryRelationship;
use crate::schema::memory_schema::MemoryMetadataSchema;

/// Content structure for creating/updating memory nodes (without ID)
/// 
/// This structure represents the essential data needed to create or update
/// a memory node, excluding the ID which is managed by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNodeCreateContent {
    /// The textual content of the memory node
    pub content: String,
    /// The type classification of the memory
    pub memory_type: MemoryType,
    /// Associated metadata including embeddings and timestamps
    pub metadata: MemoryMetadataSchema,
}

impl From<&MemoryNode> for MemoryNodeCreateContent {
    /// Convert a MemoryNode reference to create content
    /// 
    /// This conversion extracts the essential data from an existing memory node
    /// for use in create/update operations, ensuring zero allocation where possible.
    fn from(memory: &MemoryNode) -> Self {
        Self {
            content: memory.content.clone(),
            memory_type: memory.memory_type.clone(),
            metadata: MemoryMetadataSchema {
                created_at: memory.metadata.created_at,
                last_accessed_at: memory
                    .metadata
                    .last_accessed_at
                    .unwrap_or(memory.metadata.created_at),
                importance: memory.metadata.importance,
                embedding: memory.metadata.embedding.clone(),
                custom: memory.metadata.custom.clone(),
            },
        }
    }
}

/// Content structure for creating relationships (without ID)
/// 
/// This structure represents the essential data needed to create a relationship
/// between memory nodes, excluding the ID which is managed by the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipCreateContent {
    /// ID of the source memory node
    pub source_id: String,
    /// ID of the target memory node
    pub target_id: String,
    /// Type of relationship (e.g., "relates_to", "depends_on", "similar_to")
    pub relationship_type: String,
    /// Additional metadata for the relationship
    pub metadata: serde_json::Value,
    /// Timestamp when the relationship was created
    pub created_at: u64,
    /// Timestamp when the relationship was last updated
    pub updated_at: u64,
    /// Strength of the relationship (0.0 to 1.0)
    pub strength: f32,
}

impl From<&MemoryRelationship> for RelationshipCreateContent {
    /// Convert a MemoryRelationship reference to create content
    /// 
    /// This conversion extracts the essential data from an existing relationship
    /// for use in create/update operations with zero allocation patterns.
    fn from(relationship: &MemoryRelationship) -> Self {
        Self {
            source_id: relationship.source_id.clone(),
            target_id: relationship.target_id.clone(),
            relationship_type: relationship.relationship_type.clone(),
            metadata: relationship.metadata.clone(),
            created_at: relationship.created_at,
            updated_at: relationship.updated_at,
            strength: relationship.strength,
        }
    }
}
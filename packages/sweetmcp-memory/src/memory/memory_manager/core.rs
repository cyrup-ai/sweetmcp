//! Core SurrealDBMemoryManager implementation
//!
//! This module provides the core SurrealDBMemoryManager struct and basic
//! implementation methods for database connection management and utility functions.

use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::memory::memory_node::MemoryNode;
use crate::memory::memory_relationship::MemoryRelationship;
use crate::memory::memory_metadata::MemoryMetadata;
use crate::schema::memory_schema::{MemoryNodeSchema, MemoryMetadataSchema};
use crate::schema::relationship_schema::RelationshipSchema;
use super::trait_def::MemoryManager;

/// SurrealDB-based implementation of the MemoryManager trait
/// 
/// This struct provides a concrete implementation of memory management
/// operations using SurrealDB as the underlying storage engine.
/// Optimized for zero allocation patterns and blazing-fast performance.
#[derive(Debug, Clone)]
pub struct SurrealDBMemoryManager {
    /// SurrealDB database connection
    pub(crate) db: Surreal<Any>,
}

impl SurrealDBMemoryManager {
    /// Create a new SurrealDBMemoryManager instance
    /// 
    /// # Arguments
    /// * `db` - SurrealDB connection instance
    /// 
    /// # Returns
    /// New SurrealDBMemoryManager instance ready for operations
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Convert MemoryMetadataSchema to MemoryMetadata
    /// 
    /// This utility function converts database metadata schema to domain metadata
    /// with zero allocation patterns where possible.
    pub(crate) fn convert_metadata_schema(schema: MemoryMetadataSchema) -> MemoryMetadata {
        // Extract custom fields from JSON if they exist
        let custom_obj = schema.custom.as_object();
        
        MemoryMetadata {
            user_id: custom_obj
                .and_then(|obj| obj.get("user_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            agent_id: custom_obj
                .and_then(|obj| obj.get("agent_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            context: custom_obj
                .and_then(|obj| obj.get("context"))
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string(),
            keywords: custom_obj
                .and_then(|obj| obj.get("keywords"))
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    /// Convert MemoryNodeSchema to MemoryNode
    /// 
    /// This utility function converts database schema objects to domain objects
    /// with zero allocation patterns where possible.
    /// 
    /// # Arguments
    /// * `schema` - MemoryNodeSchema from database query
    /// 
    /// # Returns
    /// MemoryNode domain object
    pub(crate) fn from_schema(schema: MemoryNodeSchema) -> MemoryNode {
        MemoryNode {
            id: schema.id,
            content: schema.content,
            memory_type: schema.memory_type,
            metadata: Self::convert_metadata_schema(schema.metadata),
        }
    }

    /// Convert RelationshipSchema to MemoryRelationship
    /// 
    /// This utility function converts database relationship schema objects
    /// to domain relationship objects with efficient memory usage.
    /// 
    /// # Arguments
    /// * `schema` - RelationshipSchema from database query
    /// 
    /// # Returns
    /// MemoryRelationship domain object
    pub(crate) fn relationship_from_schema(schema: RelationshipSchema) -> MemoryRelationship {
        MemoryRelationship {
            id: schema.id,
            source_id: schema.source_id,
            target_id: schema.target_id,
            relationship_type: schema.relationship_type,
            metadata: schema.metadata,
            created_at: schema.created_at,
            updated_at: schema.updated_at,
            strength: schema.strength,
        }
    }

    /// Get database connection reference
    /// 
    /// Provides access to the underlying SurrealDB connection for
    /// advanced operations while maintaining encapsulation.
    /// 
    /// # Returns
    /// Reference to the SurrealDB connection
    pub fn db(&self) -> &Surreal<Any> {
        &self.db
    }

    /// Validate memory node before database operations
    /// 
    /// Performs validation checks on memory nodes to ensure data integrity
    /// before persisting to the database.
    /// 
    /// # Arguments
    /// * `memory` - Memory node to validate
    /// 
    /// # Returns
    /// Result indicating validation success or error
    pub(crate) fn validate_memory_node(memory: &MemoryNode) -> Result<(), crate::utils::error::Error> {
        if memory.content.is_empty() {
            return Err(crate::utils::error::Error::ValidationError(
                "Memory content cannot be empty".to_string(),
            ));
        }

        if memory.metadata.importance < 0.0 || memory.metadata.importance > 1.0 {
            return Err(crate::utils::error::Error::ValidationError(
                "Memory importance must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate relationship before database operations
    /// 
    /// Performs validation checks on relationships to ensure data integrity
    /// and referential consistency.
    /// 
    /// # Arguments
    /// * `relationship` - Relationship to validate
    /// 
    /// # Returns
    /// Result indicating validation success or error
    pub(crate) fn validate_relationship(relationship: &MemoryRelationship) -> Result<(), crate::utils::error::Error> {
        if relationship.source_id.is_empty() {
            return Err(crate::utils::error::Error::ValidationError(
                "Relationship source_id cannot be empty".to_string(),
            ));
        }

        if relationship.target_id.is_empty() {
            return Err(crate::utils::error::Error::ValidationError(
                "Relationship target_id cannot be empty".to_string(),
            ));
        }

        if relationship.source_id == relationship.target_id {
            return Err(crate::utils::error::Error::ValidationError(
                "Relationship cannot connect a node to itself".to_string(),
            ));
        }

        if relationship.relationship_type.is_empty() {
            return Err(crate::utils::error::Error::ValidationError(
                "Relationship type cannot be empty".to_string(),
            ));
        }

        if relationship.strength < 0.0 || relationship.strength > 1.0 {
            return Err(crate::utils::error::Error::ValidationError(
                "Relationship strength must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }
}
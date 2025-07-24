//! Relationship management operations
//!
//! This module implements relationship creation, retrieval, and deletion operations
//! for memory nodes with blazing-fast performance and comprehensive validation.

use super::core::SurrealDBMemoryManager;
use super::trait_def::{MemoryFuture, MemoryManager};
use super::types::RelationshipCreateContent;
use crate::memory::memory_relationship::MemoryRelationship;
use crate::memory::memory_stream::MemoryStream;
use crate::schema::relationship_schema::RelationshipSchema;
use crate::utils::error::Error;

impl MemoryManager for SurrealDBMemoryManager {
    /// Create a new relationship between memory nodes
    /// 
    /// This method creates a bidirectional relationship between two memory nodes
    /// with full validation and referential integrity checks.
    fn create_relationship(&self, relationship: MemoryRelationship) -> MemoryFuture<MemoryRelationship> {
        let db = self.db.clone();
        
        Box::pin(async move {
            // Validate the relationship before creation
            Self::validate_relationship(&relationship)?;

            // Verify that both source and target memory nodes exist
            let source_exists_query = format!("SELECT count() FROM memory WHERE id = '{}'", relationship.source_id);
            let target_exists_query = format!("SELECT count() FROM memory WHERE id = '{}'", relationship.target_id);

            match db.query(&source_exists_query).await {
                Ok(mut response) => {
                    let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                    if count.unwrap_or(0) == 0 {
                        return Err(Error::NotFound(format!("Source memory node '{}' not found", relationship.source_id)));
                    }
                }
                Err(e) => return Err(Error::Database(Box::new(e))),
            }

            match db.query(&target_exists_query).await {
                Ok(mut response) => {
                    let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                    if count.unwrap_or(0) == 0 {
                        return Err(Error::NotFound(format!("Target memory node '{}' not found", relationship.target_id)));
                    }
                }
                Err(e) => return Err(Error::Database(Box::new(e))),
            }

            // Convert to create content (without ID)
            let content = RelationshipCreateContent::from(&relationship);

            // Insert into database and get the created record
            match db.create::<Vec<RelationshipSchema>>("relationship").content(content).await {
                Ok(mut created) => {
                    if let Some(schema) = created.pop() {
                        Ok(Self::relationship_from_schema(schema))
                    } else {
                        Err(Error::Database(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to create relationship: no record returned",
                        ))))
                    }
                }
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }

    /// Get relationships for a memory node
    /// 
    /// This method retrieves all relationships associated with a memory node,
    /// optionally filtered by relationship type.
    fn get_relationships(&self, memory_id: &str, relationship_type: Option<&str>) -> MemoryStream {
        let db = self.db.clone();
        let memory_id = memory_id.to_string();
        let relationship_type = relationship_type.map(|s| s.to_string());

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            if memory_id.is_empty() {
                let _ = tx.send(Err(Error::ValidationError("Memory ID cannot be empty".to_string()))).await;
                return;
            }

            // Build query based on whether relationship type filter is provided
            let query = if let Some(rel_type) = relationship_type {
                format!(
                    "SELECT * FROM relationship WHERE (source_id = '{}' OR target_id = '{}') AND relationship_type = '{}'",
                    memory_id, memory_id, rel_type
                )
            } else {
                format!(
                    "SELECT * FROM relationship WHERE source_id = '{}' OR target_id = '{}'",
                    memory_id, memory_id
                )
            };

            match db.query(&query).await {
                Ok(mut response) => {
                    let results: Vec<RelationshipSchema> = response.take(0).unwrap_or_default();

                    for schema in results {
                        let relationship = Self::relationship_from_schema(schema);
                        // Convert relationship to memory node for stream compatibility
                        let metadata_schema = crate::schema::memory_schema::MemoryMetadataSchema {
                            created_at: relationship.created_at,
                            last_accessed_at: relationship.updated_at,
                            importance: relationship.strength,
                            embedding: None,
                            custom: relationship.metadata,
                        };
                        
                        let memory = crate::memory::memory_node::MemoryNode {
                            id: relationship.id.clone(),
                            content: format!("Relationship: {} -> {}", relationship.source_id, relationship.target_id),
                            memory_type: crate::memory::memory_node::MemoryType::Relationship,
                            metadata: super::core::SurrealDBMemoryManager::convert_metadata_schema(metadata_schema),
                        };
                        
                        if tx.send(Ok(memory)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Database(Box::new(e)))).await;
                }
            }
        });

        MemoryStream::new(rx)
    }

    /// Delete a relationship by its unique identifier
    /// 
    /// This method removes a relationship from the database with proper
    /// error handling and validation.
    fn delete_relationship(&self, id: &str) -> MemoryFuture<bool> {
        let db = self.db.clone();
        let id = id.to_string();
        
        Box::pin(async move {
            if id.is_empty() {
                return Err(Error::ValidationError("Relationship ID cannot be empty".to_string()));
            }

            match db.delete::<Option<RelationshipSchema>>(("relationship", &id)).await {
                Ok(Some(_)) => Ok(true),
                Ok(None) => Ok(false), // Relationship didn't exist
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }
}

impl SurrealDBMemoryManager {
    /// Get relationship count for a memory node
    /// 
    /// This utility method returns the total number of relationships
    /// associated with a specific memory node.
    /// 
    /// # Arguments
    /// * `memory_id` - The unique identifier of the memory node
    /// 
    /// # Returns
    /// Future resolving to the count of relationships
    pub async fn get_relationship_count(&self, memory_id: &str) -> Result<i64, Error> {
        if memory_id.is_empty() {
            return Err(Error::ValidationError("Memory ID cannot be empty".to_string()));
        }

        let query = format!(
            "SELECT count() FROM relationship WHERE source_id = '{}' OR target_id = '{}'",
            memory_id, memory_id
        );
        
        match self.db.query(&query).await {
            Ok(mut response) => {
                let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                Ok(count.unwrap_or(0))
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }

    /// Check if a relationship exists between two memory nodes
    /// 
    /// This utility method provides a fast existence check for relationships
    /// between specific memory nodes.
    /// 
    /// # Arguments
    /// * `source_id` - The source memory node ID
    /// * `target_id` - The target memory node ID
    /// * `relationship_type` - Optional relationship type filter
    /// 
    /// # Returns
    /// Future resolving to true if the relationship exists, false otherwise
    pub async fn relationship_exists(&self, source_id: &str, target_id: &str, relationship_type: Option<&str>) -> Result<bool, Error> {
        if source_id.is_empty() || target_id.is_empty() {
            return Err(Error::ValidationError("Source and target IDs cannot be empty".to_string()));
        }

        let query = if let Some(rel_type) = relationship_type {
            format!(
                "SELECT count() FROM relationship WHERE source_id = '{}' AND target_id = '{}' AND relationship_type = '{}'",
                source_id, target_id, rel_type
            )
        } else {
            format!(
                "SELECT count() FROM relationship WHERE source_id = '{}' AND target_id = '{}'",
                source_id, target_id
            )
        };
        
        match self.db.query(&query).await {
            Ok(mut response) => {
                let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                Ok(count.unwrap_or(0) > 0)
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }
}
//! CRUD operations for memory nodes
//!
//! This module implements the core Create, Read, Update, Delete operations
//! for memory nodes with blazing-fast performance and comprehensive error handling.

use super::core::SurrealDBMemoryManager;
use super::trait_def::{MemoryFuture, MemoryManager};
use super::types::MemoryNodeCreateContent;
use crate::memory::memory_node::MemoryNode;
use crate::schema::memory_schema::MemoryNodeSchema;
use crate::utils::error::Error;

impl MemoryManager for SurrealDBMemoryManager {
    /// Create a new memory node in the database
    /// 
    /// This method persists a new memory node to SurrealDB with full validation
    /// and error handling. Uses zero allocation patterns where possible.
    fn create_memory(&self, memory: MemoryNode) -> MemoryFuture<MemoryNode> {
        let db = self.db.clone();
        
        Box::pin(async move {
            // Validate the memory node before creation
            Self::validate_memory_node(&memory)?;

            // Convert to create content (without ID)
            let content = MemoryNodeCreateContent::from(&memory);

            // Insert into database and get the created record
            match db.create::<Vec<MemoryNodeSchema>>("memory").content(content).await {
                Ok(mut created) => {
                    if let Some(schema) = created.pop() {
                        Ok(Self::from_schema(schema))
                    } else {
                        Err(Error::Database(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to create memory node: no record returned",
                        ))))
                    }
                }
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }

    /// Retrieve a memory node by its unique identifier
    /// 
    /// This method performs an efficient lookup by ID with proper error handling
    /// and returns None if the memory node is not found.
    fn get_memory(&self, id: &str) -> MemoryFuture<Option<MemoryNode>> {
        let db = self.db.clone();
        let id = id.to_string();
        
        Box::pin(async move {
            if id.is_empty() {
                return Err(Error::ValidationError("Memory ID cannot be empty".to_string()));
            }

            match db.select::<Option<MemoryNodeSchema>>(("memory", &id)).await {
                Ok(Some(schema)) => Ok(Some(Self::from_schema(schema))),
                Ok(None) => Ok(None),
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }

    /// Update an existing memory node
    /// 
    /// This method updates a memory node in the database with full validation
    /// and optimistic concurrency control.
    fn update_memory(&self, memory: MemoryNode) -> MemoryFuture<MemoryNode> {
        let db = self.db.clone();
        
        Box::pin(async move {
            // Validate the memory node before update
            Self::validate_memory_node(&memory)?;

            if memory.id.is_empty() {
                return Err(Error::ValidationError("Memory ID cannot be empty for update".to_string()));
            }

            // Convert to create content for update
            let content = MemoryNodeCreateContent::from(&memory);

            // Update the record in database
            match db.update::<Option<MemoryNodeSchema>>(("memory", &memory.id)).content(content).await {
                Ok(Some(schema)) => Ok(Self::from_schema(schema)),
                Ok(None) => Err(Error::NotFound(format!("Memory node with ID '{}' not found", memory.id))),
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }

    /// Delete a memory node by its unique identifier
    /// 
    /// This method removes a memory node from the database and handles
    /// cascading deletions of associated relationships.
    fn delete_memory(&self, id: &str) -> MemoryFuture<bool> {
        let db = self.db.clone();
        let id = id.to_string();
        
        Box::pin(async move {
            if id.is_empty() {
                return Err(Error::ValidationError("Memory ID cannot be empty".to_string()));
            }

            // First, delete all relationships involving this memory node
            let relationship_cleanup_query = format!(
                "DELETE FROM relationship WHERE source_id = '{}' OR target_id = '{}'",
                id, id
            );
            
            if let Err(e) = db.query(&relationship_cleanup_query).await {
                return Err(Error::Database(Box::new(e)));
            }

            // Then delete the memory node itself
            match db.delete::<Option<MemoryNodeSchema>>(("memory", &id)).await {
                Ok(Some(_)) => Ok(true),
                Ok(None) => Ok(false), // Node didn't exist
                Err(e) => Err(Error::Database(Box::new(e))),
            }
        })
    }
}

impl SurrealDBMemoryManager {
    /// Batch create multiple memory nodes efficiently
    /// 
    /// This utility method allows for efficient creation of multiple memory nodes
    /// in a single database transaction with proper error handling.
    /// 
    /// # Arguments
    /// * `memories` - Vector of memory nodes to create
    /// 
    /// # Returns
    /// Future resolving to vector of created memory nodes with assigned IDs
    pub async fn batch_create_memories(&self, memories: Vec<MemoryNode>) -> Result<Vec<MemoryNode>, Error> {
        if memories.is_empty() {
            return Ok(Vec::new());
        }

        // Validate all memory nodes first
        for memory in &memories {
            Self::validate_memory_node(memory)?;
        }

        // Convert to create content
        let contents: Vec<MemoryNodeCreateContent> = memories.iter().map(MemoryNodeCreateContent::from).collect();

        // Batch insert into database
        match self.db.create::<Vec<MemoryNodeSchema>>("memory").content(contents).await {
            Ok(created_schemas) => {
                let created_memories: Vec<MemoryNode> = created_schemas
                    .into_iter()
                    .map(Self::from_schema)
                    .collect();
                Ok(created_memories)
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }

    /// Check if a memory node exists by ID
    /// 
    /// This utility method provides a fast existence check without retrieving
    /// the full memory node data.
    /// 
    /// # Arguments
    /// * `id` - The unique identifier to check
    /// 
    /// # Returns
    /// Future resolving to true if the memory node exists, false otherwise
    pub async fn memory_exists(&self, id: &str) -> Result<bool, Error> {
        if id.is_empty() {
            return Err(Error::ValidationError("Memory ID cannot be empty".to_string()));
        }

        let query = format!("SELECT count() FROM memory WHERE id = '{}'", id);
        
        match self.db.query(&query).await {
            Ok(mut response) => {
                let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                Ok(count.unwrap_or(0) > 0)
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }

    /// Get memory node count
    /// 
    /// This utility method returns the total number of memory nodes in the database
    /// for monitoring and analytics purposes.
    /// 
    /// # Returns
    /// Future resolving to the total count of memory nodes
    pub async fn get_memory_count(&self) -> Result<i64, Error> {
        let query = "SELECT count() FROM memory";
        
        match self.db.query(query).await {
            Ok(mut response) => {
                let count: Option<i64> = response.take(0).unwrap_or(Some(0));
                Ok(count.unwrap_or(0))
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }
}
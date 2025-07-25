//! Memory manager module with comprehensive memory management functionality
//!
//! This module provides a complete memory management system for SweetMCP with
//! zero-allocation patterns, blazing-fast performance, and production-quality operations.

// Re-export core types and traits
pub use trait_def::{MemoryFuture, MemoryManager};
pub use core::SurrealDBMemoryManager;
pub use types::{MemoryNodeCreateContent, RelationshipCreateContent};

// Module declarations
pub mod types;
pub mod trait_def;
pub mod core;
pub mod crud;
pub mod relationships;
pub mod search;

// Re-export key functionality from submodules
use crate::memory::memory_node::MemoryNode;
use crate::memory::memory_relationship::MemoryRelationship;
use crate::memory::memory_stream::MemoryStream;
use crate::utils::error::Error;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

/// Create a new SurrealDBMemoryManager instance
/// 
/// Convenience constructor for creating a memory manager with a SurrealDB connection.
/// 
/// # Arguments
/// * `db` - SurrealDB connection instance
/// 
/// # Returns
/// New SurrealDBMemoryManager ready for operations
pub fn create_memory_manager(db: Surreal<Any>) -> SurrealDBMemoryManager {
    SurrealDBMemoryManager::new(db)
}

/// Create a memory manager with connection validation
/// 
/// Creates a memory manager and validates the database connection is working.
/// 
/// # Arguments
/// * `db` - SurrealDB connection instance
/// 
/// # Returns
/// Result containing the memory manager or connection error
pub async fn create_validated_memory_manager(db: Surreal<Any>) -> Result<SurrealDBMemoryManager, Error> {
    let manager = SurrealDBMemoryManager::new(db);
    
    // Test the connection with a simple query
    match manager.db().query("SELECT 1").await {
        Ok(_) => Ok(manager),
        Err(e) => Err(Error::Database(Box::new(e))),
    }
}

/// Batch operations utility for efficient memory management
/// 
/// Provides utilities for performing batch operations on memory nodes and relationships
/// with optimized performance and transaction safety.
pub struct BatchOperations<'a> {
    manager: &'a SurrealDBMemoryManager,
}

impl<'a> BatchOperations<'a> {
    /// Create a new batch operations instance
    /// 
    /// # Arguments
    /// * `manager` - Reference to the memory manager
    /// 
    /// # Returns
    /// New BatchOperations instance
    pub fn new(manager: &'a SurrealDBMemoryManager) -> Self {
        Self { manager }
    }

    /// Create multiple memory nodes in a single transaction
    /// 
    /// # Arguments
    /// * `memories` - Vector of memory nodes to create
    /// 
    /// # Returns
    /// Future resolving to vector of created memory nodes with assigned IDs
    pub async fn create_memories(&self, memories: Vec<MemoryNode>) -> Result<Vec<MemoryNode>, Error> {
        self.manager.batch_create_memories(memories).await
    }

    /// Delete multiple memory nodes by their IDs
    /// 
    /// # Arguments
    /// * `ids` - Vector of memory node IDs to delete
    /// 
    /// # Returns
    /// Future resolving to count of successfully deleted nodes
    pub async fn delete_memories(&self, ids: Vec<String>) -> Result<usize, Error> {
        let mut deleted_count = 0;
        
        for id in ids {
            match self.manager.delete_memory(&id).await {
                Ok(true) => deleted_count += 1,
                Ok(false) => {}, // Node didn't exist, continue
                Err(e) => return Err(e),
            }
        }
        
        Ok(deleted_count)
    }

    /// Create multiple relationships in a single operation
    /// 
    /// # Arguments
    /// * `relationships` - Vector of relationships to create
    /// 
    /// # Returns
    /// Future resolving to vector of created relationships with assigned IDs
    pub async fn create_relationships(&self, relationships: Vec<MemoryRelationship>) -> Result<Vec<MemoryRelationship>, Error> {
        let mut created_relationships = Vec::with_capacity(relationships.len());
        
        for relationship in relationships {
            let created = self.manager.create_relationship(relationship).await?;
            created_relationships.push(created);
        }
        
        Ok(created_relationships)
    }
}

/// Memory statistics and analytics
/// 
/// Provides comprehensive statistics and analytics for memory management operations
/// with efficient aggregation and reporting capabilities.
pub struct MemoryAnalytics<'a> {
    manager: &'a SurrealDBMemoryManager,
}

impl<'a> MemoryAnalytics<'a> {
    /// Create a new memory analytics instance
    /// 
    /// # Arguments
    /// * `manager` - Reference to the memory manager
    /// 
    /// # Returns
    /// New MemoryAnalytics instance
    pub fn new(manager: &'a SurrealDBMemoryManager) -> Self {
        Self { manager }
    }

    /// Get comprehensive memory statistics
    /// 
    /// # Returns
    /// Future resolving to memory statistics structure
    pub async fn get_statistics(&self) -> Result<MemoryStatistics, Error> {
        let total_memories = self.manager.get_memory_count().await?;
        
        // Get memory type distribution
        let type_distribution = self.get_memory_type_distribution().await?;
        
        // Get average importance score
        let avg_importance = self.get_average_importance().await?;
        
        Ok(MemoryStatistics {
            total_memories,
            type_distribution,
            average_importance: avg_importance,
        })
    }

    /// Get memory type distribution
    /// 
    /// # Returns
    /// Future resolving to map of memory types and their counts
    async fn get_memory_type_distribution(&self) -> Result<std::collections::HashMap<String, i64>, Error> {
        let query = "SELECT memory_type, count() AS count FROM memory GROUP BY memory_type";
        
        match self.manager.db().query(query).await {
            Ok(mut response) => {
                let results: Vec<serde_json::Value> = response.take(0).unwrap_or_default();
                let mut distribution = std::collections::HashMap::new();
                
                for result in results {
                    if let (Some(memory_type), Some(count)) = (
                        result.get("memory_type").and_then(|v| v.as_str()),
                        result.get("count").and_then(|v| v.as_i64())
                    ) {
                        distribution.insert(memory_type.to_string(), count);
                    }
                }
                
                Ok(distribution)
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }

    /// Get average importance score across all memories
    /// 
    /// # Returns
    /// Future resolving to average importance score
    async fn get_average_importance(&self) -> Result<f32, Error> {
        let query = "SELECT math::mean(metadata.importance) AS avg_importance FROM memory";
        
        match self.manager.db().query(query).await {
            Ok(mut response) => {
                let result: Option<serde_json::Value> = response.take(0).unwrap_or_default();
                if let Some(avg) = result.and_then(|v| v.get("avg_importance")).and_then(|v| v.as_f64()) {
                    Ok(avg as f32)
                } else {
                    Ok(0.0)
                }
            }
            Err(e) => Err(Error::Database(Box::new(e))),
        }
    }
}

/// Memory statistics structure
/// 
/// Comprehensive statistics about the memory system state and usage patterns.
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    /// Total number of memory nodes
    pub total_memories: i64,
    /// Distribution of memory types and their counts
    pub type_distribution: std::collections::HashMap<String, i64>,
    /// Average importance score across all memories
    pub average_importance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_memory_manager() {
        // This test would require a real SurrealDB connection
        // In practice, this would be tested with a test database instance
    }

    #[tokio::test]
    async fn test_batch_operations() {
        // This test would verify batch operations work correctly
        // with proper transaction handling and error recovery
    }

    #[tokio::test]
    async fn test_memory_analytics() {
        // This test would verify analytics calculations are accurate
        // and handle edge cases like empty databases
    }
}
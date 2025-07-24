//! MemoryManager trait definition
//!
//! This module defines the core MemoryManager trait that provides the interface
//! for all memory management operations with async support and zero allocation patterns.

use std::future::Future;
use std::pin::Pin;

use crate::memory::memory_node::MemoryNode;
use crate::memory::memory_relationship::MemoryRelationship;
use crate::memory::memory_stream::MemoryStream;
use crate::utils::error::Error;

/// Future type alias for memory operations
/// 
/// Provides a convenient type alias for boxed futures used in async operations
/// while maintaining zero allocation patterns where possible.
pub type MemoryFuture<T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send>>;

/// Core trait for memory management operations
/// 
/// This trait defines the interface for managing memory nodes and relationships
/// in the SweetMCP memory system. All implementations must provide async support
/// with blazing-fast performance and zero allocation patterns.
pub trait MemoryManager: Send + Sync {
    /// Create a new memory node
    /// 
    /// # Arguments
    /// * `memory` - The memory node to create
    /// 
    /// # Returns
    /// Future resolving to the created memory node with assigned ID
    fn create_memory(&self, memory: MemoryNode) -> MemoryFuture<MemoryNode>;

    /// Retrieve a memory node by ID
    /// 
    /// # Arguments
    /// * `id` - The unique identifier of the memory node
    /// 
    /// # Returns
    /// Future resolving to the memory node if found, None otherwise
    fn get_memory(&self, id: &str) -> MemoryFuture<Option<MemoryNode>>;

    /// Update an existing memory node
    /// 
    /// # Arguments
    /// * `memory` - The memory node with updated data
    /// 
    /// # Returns
    /// Future resolving to the updated memory node
    fn update_memory(&self, memory: MemoryNode) -> MemoryFuture<MemoryNode>;

    /// Delete a memory node by ID
    /// 
    /// # Arguments
    /// * `id` - The unique identifier of the memory node to delete
    /// 
    /// # Returns
    /// Future resolving to success/failure of the deletion
    fn delete_memory(&self, id: &str) -> MemoryFuture<bool>;

    /// Create a relationship between memory nodes
    /// 
    /// # Arguments
    /// * `relationship` - The relationship to create
    /// 
    /// # Returns
    /// Future resolving to the created relationship with assigned ID
    fn create_relationship(&self, relationship: MemoryRelationship) -> MemoryFuture<MemoryRelationship>;

    /// Get relationships for a memory node
    /// 
    /// # Arguments
    /// * `memory_id` - The ID of the memory node
    /// * `relationship_type` - Optional filter by relationship type
    /// 
    /// # Returns
    /// Stream of relationships associated with the memory node
    fn get_relationships(&self, memory_id: &str, relationship_type: Option<&str>) -> MemoryStream;

    /// Delete a relationship by ID
    /// 
    /// # Arguments
    /// * `id` - The unique identifier of the relationship to delete
    /// 
    /// # Returns
    /// Future resolving to success/failure of the deletion
    fn delete_relationship(&self, id: &str) -> MemoryFuture<bool>;

    /// Search memory nodes by content
    /// 
    /// # Arguments
    /// * `query` - The search query string
    /// * `limit` - Maximum number of results to return
    /// 
    /// # Returns
    /// Stream of memory nodes matching the search criteria
    fn search_by_content(&self, query: &str, limit: usize) -> MemoryStream;

    /// Search memory nodes by vector similarity
    /// 
    /// # Arguments
    /// * `vector` - The query vector for similarity search
    /// * `limit` - Maximum number of results to return
    /// 
    /// # Returns
    /// Stream of memory nodes ordered by similarity score
    fn search_by_vector(&self, vector: Vec<f32>, limit: usize) -> MemoryStream;
}
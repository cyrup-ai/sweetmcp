//! Memory repository module
//!
//! This module provides comprehensive in-memory repository functionality for
//! managing memory storage, indexing, searching, and relationships with zero
//! allocation patterns and blazing-fast performance.

pub mod core;
pub mod search;
pub mod relationships;

// Re-export core types and traits for ergonomic usage
pub use core::{
    MemoryRepository, RepositoryStats,
};

pub use search::{
    SearchStats,
};

pub use relationships::{
    RelationshipStats,
};

/// Create a new memory repository
pub fn repository() -> MemoryRepository {
    MemoryRepository::new()
}

/// Create a new memory repository (alias for consistency)
pub fn new_repository() -> MemoryRepository {
    MemoryRepository::new()
}
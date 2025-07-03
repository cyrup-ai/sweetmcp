//! Memory module that provides the core memory functionality

pub mod episodic;
pub mod evolution;
pub mod filter;
pub mod history;
pub mod manager;
pub mod memory_manager;
pub mod memory_metadata;
pub mod memory_node;
pub mod memory_relationship;
pub mod memory_type;
pub mod procedural;
pub mod query;
pub mod relationship;
pub mod repository;
pub mod retrieval;
pub mod semantic;
pub mod storage;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use episodic::*;
pub use evolution::*;
pub use history::*;
pub use manager::*;
pub use memory_manager::{MemoryManager, SurrealDBMemoryManager};
pub use memory_metadata::MemoryMetadata;
pub use memory_node::MemoryNode;
pub use memory_node::MemoryType;
pub use memory_relationship::MemoryRelationship;
pub use procedural::*;
pub use semantic::*;

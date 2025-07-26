//! Memory module that provides the core memory functionality

pub mod caching;
pub mod episodic;
pub mod evolution;
pub mod filter;
pub mod history;
pub mod lifecycle;
#[cfg(feature = "bench")]
pub mod memory_benchmarks;
pub mod memory_manager;
pub mod memory_metadata;
pub mod memory_node;
pub mod memory_relationship;
pub mod memory_schema;
pub mod memory_stream;
pub mod memory_type;
pub mod pending_types;
pub mod procedural;
pub mod query;
// relationship functionality moved to semantic module
pub mod repository;
pub mod retrieval;
pub mod semantic;
pub mod storage;
pub mod storage_coordinator;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use episodic::*;
pub use evolution::*;
pub use history::*;
// manager functionality moved to memory_manager module
pub use memory_manager::{
    MemoryManager, SurrealDBMemoryManager,
};
pub use query::MemoryQuery;
pub use memory_stream::{MemoryStream, RelationshipStream};
pub use pending_types::{PendingDeletion, PendingMemory, PendingRelationship};
pub use memory_metadata::MemoryMetadata;
pub use memory_node::MemoryNode;
pub use memory_type::MemoryTypeEnum as MemoryType;
pub use memory_relationship::MemoryRelationship;

// Alias for backward compatibility
pub use memory_node::MemoryNode as Memory;
pub use procedural::*;
pub use semantic::*;

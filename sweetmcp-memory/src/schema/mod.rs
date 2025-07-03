//! Schemas for database interaction

pub mod graph_schema;
pub mod memory_schema;
pub mod relationship_schema;

pub use crate::memory::MemoryType;
pub use graph_schema::*;
pub use memory_schema::Memory;
pub use relationship_schema::Relationship;

// Placeholder for RelationshipDirection if it doesn't exist elsewhere
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipDirection {
    Outgoing,
    Incoming,
    Both,
}

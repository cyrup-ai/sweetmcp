//! Memory type module with decomposed submodules

pub mod base_memory;
pub mod content;
pub mod enums;
pub mod metadata;
pub mod traits;

// Re-export all types for API compatibility
pub use base_memory::BaseMemory;
pub use content::MemoryContent;
pub use enums::{MemoryContentType, MemoryTypeEnum};
pub use metadata::MemoryMetadata;
pub use traits::{
    json_to_surreal_value, surreal_to_json_value, Memory, MemoryFactory, MemorySerializer,
    MemoryValidator,
};
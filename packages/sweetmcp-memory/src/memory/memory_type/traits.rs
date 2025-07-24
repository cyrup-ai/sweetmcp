//! Memory traits and utilities extracted from memory_type.rs

use serde_json::Value;
use std::fmt::Debug;

use crate::graph::entity::BaseEntity;
use crate::utils::Result;
use super::{enums::MemoryTypeEnum, metadata::MemoryMetadata, content::MemoryContent};

/// Utility function to convert serde_json::Value to surrealdb::sql::Value with optimized performance
pub fn json_to_surreal_value(json: serde_json::Value) -> surrealdb::sql::Value {
    match json {
        serde_json::Value::Null => surrealdb::sql::Value::Null,
        serde_json::Value::Bool(b) => surrealdb::sql::Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                surrealdb::sql::Value::Number(surrealdb::sql::Number::Int(i))
            } else if let Some(f) = n.as_f64() {
                surrealdb::sql::Value::Number(surrealdb::sql::Number::Float(f))
            } else {
                surrealdb::sql::Value::Null
            }
        },
        serde_json::Value::String(s) => surrealdb::sql::Value::Strand(s.into()),
        serde_json::Value::Array(arr) => {
            let values: Vec<surrealdb::sql::Value> = arr.into_iter().map(json_to_surreal_value).collect();
            surrealdb::sql::Value::Array(values.into())
        },
        serde_json::Value::Object(obj) => {
            let mut map = surrealdb::sql::Object::default();
            for (k, v) in obj {
                map.insert(k, json_to_surreal_value(v));
            }
            surrealdb::sql::Value::Object(map)
        },
    }
}

/// Convert surrealdb::sql::Value to serde_json::Value (reverse conversion)
pub fn surreal_to_json_value(surreal: surrealdb::sql::Value) -> serde_json::Value {
    match surreal {
        surrealdb::sql::Value::Null => serde_json::Value::Null,
        surrealdb::sql::Value::Bool(b) => serde_json::Value::Bool(b),
        surrealdb::sql::Value::Number(n) => match n {
            surrealdb::sql::Number::Int(i) => serde_json::Value::Number(i.into()),
            surrealdb::sql::Number::Float(f) => {
                serde_json::Number::from_f64(f).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)
            },
            _ => serde_json::Value::Null,
        },
        surrealdb::sql::Value::Strand(s) => serde_json::Value::String(s.to_string()),
        surrealdb::sql::Value::Array(arr) => {
            let values: Vec<serde_json::Value> = arr.into_iter().map(surreal_to_json_value).collect();
            serde_json::Value::Array(values)
        },
        surrealdb::sql::Value::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj
                .into_iter()
                .map(|(k, v)| (k, surreal_to_json_value(v)))
                .collect();
            serde_json::Value::Object(map)
        },
        _ => serde_json::Value::Null, // Handle other SurrealDB types
    }
}

/// Core memory trait with optimized performance characteristics
pub trait Memory: Send + Sync + Debug {
    /// Get the memory ID (zero-copy reference)
    fn id(&self) -> &str;

    /// Get the memory type
    fn memory_type(&self) -> MemoryTypeEnum;

    /// Get the memory metadata (zero-copy reference)
    fn metadata(&self) -> &MemoryMetadata;

    /// Get mutable memory metadata
    fn metadata_mut(&mut self) -> &mut MemoryMetadata;

    /// Get the memory content (zero-copy reference)
    fn content(&self) -> &MemoryContent;

    /// Get mutable memory content
    fn content_mut(&mut self) -> &mut MemoryContent;

    /// Get the memory data (zero-copy reference)
    fn data(&self) -> &Value {
        &self.content().data
    }

    /// Record access with minimal overhead
    fn record_access(&mut self) {
        self.metadata_mut().record_access();
    }

    /// Record modification with timestamp update
    fn record_modification(&mut self) {
        self.metadata_mut().record_modification();
    }

    /// Get memory strength (computed property)
    fn strength(&self) -> f32 {
        self.metadata().calculate_strength()
    }

    /// Check if memory is recently accessed (within last hour)
    fn is_recently_accessed(&self) -> bool {
        self.metadata()
            .time_since_access_seconds()
            .map(|seconds| seconds < 3600)
            .unwrap_or(false)
    }

    /// Check if memory is frequently accessed (access count > threshold)
    fn is_frequently_accessed(&self, threshold: u64) -> bool {
        self.metadata().access_count > threshold
    }

    /// Check if memory is important (importance > threshold)
    fn is_important(&self, threshold: f32) -> bool {
        self.metadata().importance > threshold
    }

    /// Get content size in bytes
    fn content_size(&self) -> usize {
        self.content().size_bytes()
    }

    /// Check if content is empty
    fn is_empty(&self) -> bool {
        self.content().is_empty()
    }

    /// Get content hash for deduplication
    fn content_hash(&self) -> u64 {
        self.content().content_hash()
    }

    /// Validate the memory with comprehensive checks
    fn validate(&self) -> Result<()>;

    /// Convert to an entity with optimized serialization
    fn to_entity(&self) -> BaseEntity;

    /// Create from an entity with robust deserialization
    fn from_entity(entity: BaseEntity) -> Result<Self>
    where
        Self: Sized;
}

/// Memory factory trait for creating different memory types
pub trait MemoryFactory: Send + Sync {
    /// The memory type this factory creates
    type MemoryType: Memory;

    /// Create a new memory instance
    fn create_memory(
        &self,
        id: &str,
        memory_type: MemoryTypeEnum,
        content: MemoryContent,
    ) -> Result<Self::MemoryType>;

    /// Create a text memory
    fn create_text_memory(
        &self,
        id: &str,
        memory_type: MemoryTypeEnum,
        text: &str,
    ) -> Result<Self::MemoryType> {
        self.create_memory(id, memory_type, MemoryContent::text(text))
    }

    /// Create a JSON memory
    fn create_json_memory(
        &self,
        id: &str,
        memory_type: MemoryTypeEnum,
        data: Value,
    ) -> Result<Self::MemoryType> {
        self.create_memory(id, memory_type, MemoryContent::json(data))
    }

    /// Create a binary memory
    fn create_binary_memory(
        &self,
        id: &str,
        memory_type: MemoryTypeEnum,
        data: Vec<u8>,
    ) -> Result<Self::MemoryType> {
        self.create_memory(id, memory_type, MemoryContent::binary(data))
    }
}

/// Memory validator trait for extensible validation
pub trait MemoryValidator: Send + Sync {
    /// Validate memory with custom rules
    fn validate_memory(&self, memory: &dyn Memory) -> Result<()>;

    /// Validate memory type compatibility
    fn validate_memory_type(&self, memory_type: MemoryTypeEnum) -> Result<()>;

    /// Validate content format
    fn validate_content(&self, content: &MemoryContent) -> Result<()>;
}

/// Memory serializer trait for different output formats
pub trait MemorySerializer: Send + Sync {
    /// Serialize memory to bytes
    fn serialize(&self, memory: &dyn Memory) -> Result<Vec<u8>>;

    /// Deserialize memory from bytes
    fn deserialize(&self, data: &[u8]) -> Result<Box<dyn Memory>>;

    /// Get the format name
    fn format_name(&self) -> &'static str;
}
//! Base memory implementation extracted from memory_type.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::graph::entity::{BaseEntity, Entity};
use crate::utils::{Result, error::Error};
use super::{
    enums::MemoryTypeEnum,
    metadata::MemoryMetadata,
    content::MemoryContent,
    traits::{Memory, json_to_surreal_value},
};

/// High-performance base memory implementation with optimized storage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMemory {
    /// Memory ID
    pub id: String,
    /// Memory name
    pub name: String,
    /// Memory description
    pub description: String,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Memory metadata
    pub metadata: MemoryMetadata,
    /// Memory content
    pub content: MemoryContent,
}

impl BaseMemory {
    /// Create a new memory with optimized initialization
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        memory_type: MemoryTypeEnum,
        content: MemoryContent,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            updated_at: now,
            metadata: MemoryMetadata::new(memory_type),
            content,
        }
    }

    /// Create a new text memory with optimal string handling
    pub fn text(id: &str, memory_type: MemoryTypeEnum, text: &str) -> Self {
        let mut memory = Self::new(
            id,
            "Text Memory",
            "Auto-generated text memory",
            memory_type,
            MemoryContent::text(text),
        );
        memory.name = format!("text_memory_{}", id);
        memory.description = if text.len() > 50 {
            format!("Text memory containing: {}...", &text[..47])
        } else {
            format!("Text memory containing: {}", text)
        };
        memory
    }

    /// Create a new JSON memory with structured data
    pub fn json(id: &str, memory_type: MemoryTypeEnum, data: Value) -> Self {
        let mut memory = Self::new(
            id,
            "JSON Memory",
            "Auto-generated JSON memory",
            memory_type,
            MemoryContent::json(data),
        );
        memory.name = format!("json_memory_{}", id);
        memory.description = "JSON memory containing structured data".to_string();
        memory
    }

    /// Create a new binary memory with efficient data handling
    pub fn binary(id: &str, memory_type: MemoryTypeEnum, data: Vec<u8>) -> Self {
        let data_len = data.len();
        let mut memory = Self::new(
            id,
            "Binary Memory",
            "Auto-generated binary memory",
            memory_type,
            MemoryContent::binary(data),
        );
        memory.name = format!("binary_memory_{}", id);
        memory.description = format!("Binary memory containing {} bytes", data_len);
        memory
    }

    /// Create a new memory with custom name and description
    pub fn with_name_description(
        id: &str,
        name: &str,
        description: &str,
        memory_type: MemoryTypeEnum,
    ) -> Self {
        let mut map = serde_json::Map::with_capacity(2);
        map.insert("name".to_string(), Value::String(name.to_string()));
        map.insert("description".to_string(), Value::String(description.to_string()));
        
        let content = MemoryContent::structured(Value::Object(map));
        let mut memory = Self::new(id, "Custom Memory", "Auto-generated custom memory", memory_type, content);
        memory.name = name.to_string();
        memory.description = description.to_string();
        memory
    }

    /// Create a memory with embedding
    pub fn with_embedding(
        id: &str,
        name: &str,
        description: &str,
        memory_type: MemoryTypeEnum,
        content: MemoryContent,
        embedding: Vec<f32>,
    ) -> Self {
        let mut memory = Self::new(id, name, description, memory_type, content.with_embedding(embedding));
        memory
    }

    /// Update the memory name
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
        self.record_modification();
    }

    /// Update the memory description
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
        self.record_modification();
    }

    /// Update both name and description atomically
    pub fn set_name_and_description(&mut self, name: &str, description: &str) {
        self.name = name.to_string();
        self.description = description.to_string();
        self.record_modification();
    }

    /// Get memory age in seconds
    #[inline]
    pub fn age_seconds(&self) -> i64 {
        self.metadata.age_seconds()
    }

    /// Check if memory was recently modified (within last hour)
    pub fn is_recently_modified(&self) -> bool {
        (Utc::now() - self.updated_at).num_seconds() < 3600
    }

    /// Get a reference to the name
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the description
    #[inline]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the last updated timestamp
    #[inline]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Memory for BaseMemory {
    #[inline]
    fn id(&self) -> &str {
        &self.id
    }

    #[inline]
    fn memory_type(&self) -> MemoryTypeEnum {
        self.metadata.memory_type
    }

    #[inline]
    fn metadata(&self) -> &MemoryMetadata {
        &self.metadata
    }

    #[inline]
    fn metadata_mut(&mut self) -> &mut MemoryMetadata {
        &mut self.metadata
    }

    #[inline]
    fn content(&self) -> &MemoryContent {
        &self.content
    }

    #[inline]
    fn content_mut(&mut self) -> &mut MemoryContent {
        &mut self.content
    }

    fn record_modification(&mut self) {
        self.updated_at = Utc::now();
        self.metadata.record_modification();
    }

    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::ValidationError("Memory ID cannot be empty".to_string()));
        }
        
        if self.name.is_empty() {
            return Err(Error::ValidationError("Memory name cannot be empty".to_string()));
        }

        // Validate content based on type
        match self.content.content_type {
            super::enums::MemoryContentType::Text => {
                if !self.content.data.is_string() {
                    return Err(Error::ValidationError("Text content must be a string".to_string()));
                }
            },
            super::enums::MemoryContentType::Binary => {
                if !self.content.data.is_string() {
                    return Err(Error::ValidationError("Binary content must be base64 encoded string".to_string()));
                }
            },
            super::enums::MemoryContentType::Json => {
                // JSON content can be any valid JSON Value
            },
        }

        Ok(())
    }

    fn to_entity(&self) -> BaseEntity {
        let mut entity = BaseEntity::new(&self.id, &format!("memory_{}", self.metadata.memory_type));

        // Add base memory fields
        entity.attributes.insert("name".to_string(), json_to_surreal_value(Value::String(self.name.clone())));
        entity.attributes.insert("description".to_string(), json_to_surreal_value(Value::String(self.description.clone())));
        entity.attributes.insert("updated_at".to_string(), json_to_surreal_value(Value::String(self.updated_at.to_rfc3339())));

        // Add metadata as attributes
        let metadata_attrs = self.metadata.to_entity();
        for (key, value) in metadata_attrs {
            entity.attributes.insert(key, json_to_surreal_value(value));
        }

        // Add content as attributes
        let content_attrs = self.content.to_entity();
        for (key, value) in content_attrs {
            entity.attributes.insert(key, json_to_surreal_value(value));
        }

        entity
    }

    fn from_entity(entity: BaseEntity) -> Result<Self>
    where
        Self: Sized,
    {
        let id = entity.id().to_string();

        // Extract memory type from entity type
        let entity_type = entity.entity_type();
        let memory_type = if entity_type.starts_with("memory_") {
            MemoryTypeEnum::from_string(&entity_type[7..])?
        } else {
            MemoryTypeEnum::LongTerm
        };

        // Create maps for metadata and content extraction
        let mut metadata_map = HashMap::new();
        let mut content_map = HashMap::new();
        let mut base_fields = HashMap::new();

        for (key, value) in entity.attributes() {
            let json_value: serde_json::Value = super::traits::surreal_to_json_value(value.clone());
            
            match key.as_str() {
                "name" | "description" | "updated_at" => {
                    base_fields.insert(key.clone(), json_value);
                },
                k if k.starts_with("content_") || k == "data" || k == "embedding" => {
                    content_map.insert(key.clone(), json_value);
                },
                _ => {
                    metadata_map.insert(key.clone(), json_value);
                },
            }
        }

        let name = base_fields.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Base Memory")
            .to_string();

        let description = base_fields.get("description")
            .and_then(|v| v.as_str()) 
            .unwrap_or("Default base memory")
            .to_string();

        let updated_at = base_fields.get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let metadata = MemoryMetadata::from_entity(&metadata_map)?;
        let content = MemoryContent::from_entity(&content_map)?;

        Ok(Self {
            id,
            name,
            description,
            updated_at,
            metadata,
            content,
        })
    }
}
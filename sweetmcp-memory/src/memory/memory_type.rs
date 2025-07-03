// src/memory/memory_type.rs
//! Memory type definitions and traits for the memory system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{self, Debug};

use crate::graph::entity::BaseEntity;
use crate::utils::Result;
use crate::utils::error::Error;
use base64::Engine;

/// Convert serde_json::Value to surrealdb::sql::Value
fn json_to_surreal_value(json: serde_json::Value) -> surrealdb::sql::Value {
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
        }
        serde_json::Value::String(s) => surrealdb::sql::Value::Strand(s.into()),
        serde_json::Value::Array(arr) => {
            let values: Vec<surrealdb::sql::Value> =
                arr.into_iter().map(json_to_surreal_value).collect();
            surrealdb::sql::Value::Array(values.into())
        }
        serde_json::Value::Object(obj) => {
            let mut map = surrealdb::sql::Object::default();
            for (k, v) in obj {
                map.insert(k, json_to_surreal_value(v));
            }
            surrealdb::sql::Value::Object(map)
        }
    }
}

/// Memory type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryTypeEnum {
    /// Semantic memory (knowledge graph)
    Semantic,
    /// Episodic memory (events and experiences)
    Episodic,
    /// Procedural memory (skills and procedures)
    Procedural,
    /// Working memory (temporary storage)
    Working,
    /// Long-term memory (permanent storage)
    LongTerm,
    /// Custom memory type
    Custom(u8),
}

impl fmt::Display for MemoryTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryTypeEnum::Semantic => write!(f, "semantic"),
            MemoryTypeEnum::Episodic => write!(f, "episodic"),
            MemoryTypeEnum::Procedural => write!(f, "procedural"),
            MemoryTypeEnum::Working => write!(f, "working"),
            MemoryTypeEnum::LongTerm => write!(f, "longterm"),
            MemoryTypeEnum::Custom(id) => write!(f, "custom_{}", id),
        }
    }
}

impl MemoryTypeEnum {
    /// Convert from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "semantic" => Ok(MemoryTypeEnum::Semantic),
            "episodic" => Ok(MemoryTypeEnum::Episodic),
            "procedural" => Ok(MemoryTypeEnum::Procedural),
            "working" => Ok(MemoryTypeEnum::Working),
            "longterm" => Ok(MemoryTypeEnum::LongTerm),
            s if s.starts_with("custom_") => {
                let id_str = s.strip_prefix("custom_").unwrap_or("");
                let id = id_str.parse::<u8>().map_err(|_| {
                    Error::ConversionError(format!("Invalid custom memory type ID: {}", id_str))
                })?;
                Ok(MemoryTypeEnum::Custom(id))
            }
            _ => Err(Error::ConversionError(format!(
                "Unknown memory type: {}",
                s
            ))),
        }
    }
}

/// Memory metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Memory type
    pub memory_type: MemoryTypeEnum,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,

    /// Last accessed timestamp
    pub accessed_at: Option<DateTime<Utc>>,

    /// Access count
    pub access_count: u64,

    /// Importance (0.0 to 1.0)
    pub importance: f32,

    /// Relevance (0.0 to 1.0)
    pub relevance: f32,

    /// Custom metadata
    pub custom: HashMap<String, Value>,
}

impl MemoryMetadata {
    /// Create new metadata
    pub fn new(memory_type: MemoryTypeEnum) -> Self {
        let now = Utc::now();
        Self {
            memory_type,
            created_at: now,
            updated_at: now,
            accessed_at: None,
            access_count: 0,
            importance: 0.5,
            relevance: 0.5,
            custom: HashMap::new(),
        }
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.accessed_at = Some(Utc::now());
        self.access_count += 1;
    }

    /// Record modification
    pub fn record_modification(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Set importance
    pub fn set_importance(&mut self, importance: f32) {
        self.importance = importance.max(0.0).min(1.0);
    }

    /// Set relevance
    pub fn set_relevance(&mut self, relevance: f32) {
        self.relevance = relevance.max(0.0).min(1.0);
    }

    /// Add custom metadata
    pub fn add_custom<T: Into<Value>>(&mut self, key: &str, value: T) -> &mut Self {
        self.custom.insert(key.to_string(), value.into());
        self
    }

    /// Get custom metadata
    pub fn get_custom(&self, key: &str) -> Option<&Value> {
        self.custom.get(key)
    }

    /// Remove custom metadata
    pub fn remove_custom(&mut self, key: &str) -> Option<Value> {
        self.custom.remove(key)
    }

    /// Convert to entity
    pub fn to_entity(&self) -> HashMap<String, Value> {
        let mut entity = HashMap::new();
        entity.insert(
            "memory_type".to_string(),
            Value::String(self.memory_type.to_string()),
        );
        entity.insert(
            "created_at".to_string(),
            Value::String(self.created_at.to_rfc3339()),
        );
        entity.insert(
            "updated_at".to_string(),
            Value::String(self.updated_at.to_rfc3339()),
        );

        if let Some(accessed_at) = self.accessed_at {
            entity.insert(
                "accessed_at".to_string(),
                Value::String(accessed_at.to_rfc3339()),
            );
        }

        entity.insert(
            "access_count".to_string(),
            Value::Number(self.access_count.into()),
        );
        entity.insert(
            "importance".to_string(),
            Value::Number(serde_json::Number::from_f64(self.importance as f64).unwrap()),
        );
        entity.insert(
            "relevance".to_string(),
            Value::Number(serde_json::Number::from_f64(self.relevance as f64).unwrap()),
        );

        if !self.custom.is_empty() {
            entity.insert(
                "custom".to_string(),
                Value::Object(serde_json::Map::from_iter(
                    self.custom.iter().map(|(k, v)| (k.clone(), v.clone())),
                )),
            );
        }

        entity
    }

    /// Create from entity
    pub fn from_entity(entity: &HashMap<String, Value>) -> Result<Self> {
        let memory_type = if let Some(Value::String(s)) = entity.get("memory_type") {
            MemoryTypeEnum::from_string(s)?
        } else {
            return Err(Error::ConversionError(
                "Missing memory_type in entity".to_string(),
            ));
        };

        let created_at = if let Some(Value::String(s)) = entity.get("created_at") {
            DateTime::parse_from_rfc3339(s)
                .map_err(|_| Error::ConversionError("Invalid created_at format".to_string()))?
                .with_timezone(&Utc)
        } else {
            return Err(Error::ConversionError(
                "Missing created_at in entity".to_string(),
            ));
        };

        let updated_at = if let Some(Value::String(s)) = entity.get("updated_at") {
            DateTime::parse_from_rfc3339(s)
                .map_err(|_| Error::ConversionError("Invalid updated_at format".to_string()))?
                .with_timezone(&Utc)
        } else {
            created_at
        };

        let accessed_at = if let Some(Value::String(s)) = entity.get("accessed_at") {
            Some(
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid accessed_at format".to_string()))?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let access_count = if let Some(Value::Number(n)) = entity.get("access_count") {
            n.as_u64().unwrap_or(0)
        } else {
            0
        };

        let importance = if let Some(Value::Number(n)) = entity.get("importance") {
            n.as_f64().unwrap_or(0.5) as f32
        } else {
            0.5
        };

        let relevance = if let Some(Value::Number(n)) = entity.get("relevance") {
            n.as_f64().unwrap_or(0.5) as f32
        } else {
            0.5
        };

        let mut custom = HashMap::new();
        if let Some(Value::Object(obj)) = entity.get("custom") {
            for (key, value) in obj.iter() {
                custom.insert(key.clone(), value.clone());
            }
        }

        Ok(Self {
            memory_type,
            created_at,
            updated_at,
            accessed_at,
            access_count,
            importance,
            relevance,
            custom,
        })
    }
}

/// Memory content type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryContentType {
    /// Text content
    Text,
    /// JSON content
    Json,
    /// Binary content
    Binary,
}

impl fmt::Display for MemoryContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryContentType::Text => write!(f, "text"),
            MemoryContentType::Json => write!(f, "json"),
            MemoryContentType::Binary => write!(f, "binary"),
        }
    }
}

impl MemoryContentType {
    /// Convert from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" => Ok(MemoryContentType::Text),
            "json" => Ok(MemoryContentType::Json),
            "binary" => Ok(MemoryContentType::Binary),
            _ => Err(Error::ConversionError(format!(
                "Unknown content type: {}",
                s
            ))),
        }
    }
}

/// Memory content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContent {
    /// Content type
    pub content_type: MemoryContentType,

    /// Content data
    pub data: Value,

    /// Content embedding
    pub embedding: Option<Vec<f32>>,
}

impl MemoryContent {
    /// Create new text content
    pub fn text(text: &str) -> Self {
        Self {
            content_type: MemoryContentType::Text,
            data: Value::String(text.to_string()),
            embedding: None,
        }
    }

    /// Create new JSON content
    pub fn json(data: Value) -> Self {
        Self {
            content_type: MemoryContentType::Json,
            data,
            embedding: None,
        }
    }

    /// Create new binary content
    pub fn binary(data: Vec<u8>) -> Self {
        use base64::Engine;
        Self {
            content_type: MemoryContentType::Binary,
            data: Value::String(base64::engine::general_purpose::STANDARD.encode(&data)),
            embedding: None,
        }
    }

    /// Create structured content from a Value
    pub fn structured(data: Value) -> Self {
        Self {
            content_type: MemoryContentType::Json,
            data,
            embedding: None,
        }
    }

    /// Set embedding
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Get text content
    pub fn get_text(&self) -> Result<String> {
        match self.content_type {
            MemoryContentType::Text => {
                if let Value::String(s) = &self.data {
                    Ok(s.clone())
                } else {
                    Err(Error::ConversionError(
                        "Text content is not a string".to_string(),
                    ))
                }
            }
            MemoryContentType::Json => Ok(serde_json::to_string(&self.data).map_err(|e| {
                Error::ConversionError(format!("Failed to convert JSON to string: {}", e))
            })?),
            MemoryContentType::Binary => Err(Error::ConversionError(
                "Cannot convert binary content to text".to_string(),
            )),
        }
    }

    /// Get binary content
    pub fn get_binary(&self) -> Result<Vec<u8>> {
        match self.content_type {
            MemoryContentType::Text => {
                if let Value::String(s) = &self.data {
                    Ok(s.as_bytes().to_vec())
                } else {
                    Err(Error::ConversionError(
                        "Text content is not a string".to_string(),
                    ))
                }
            }
            MemoryContentType::Json => {
                let json_string = serde_json::to_string(&self.data).map_err(|e| {
                    Error::ConversionError(format!("Failed to convert JSON to string: {}", e))
                })?;
                Ok(json_string.as_bytes().to_vec())
            }
            MemoryContentType::Binary => {
                if let Value::String(s) = &self.data {
                    base64::engine::general_purpose::STANDARD
                        .decode(s)
                        .map_err(|e| {
                            Error::ConversionError(format!("Failed to decode base64: {}", e))
                        })
                } else {
                    Err(Error::ConversionError(
                        "Binary content is not a string".to_string(),
                    ))
                }
            }
        }
    }

    /// Convert to entity
    pub fn to_entity(&self) -> HashMap<String, Value> {
        let mut entity = HashMap::new();
        entity.insert(
            "content_type".to_string(),
            Value::String(self.content_type.to_string()),
        );
        entity.insert("data".to_string(), self.data.clone());

        if let Some(embedding) = &self.embedding {
            entity.insert(
                "embedding".to_string(),
                serde_json::to_value(embedding).unwrap_or(Value::Null),
            );
        }

        entity
    }

    /// Create from entity
    pub fn from_entity(entity: &HashMap<String, Value>) -> Result<Self> {
        let content_type = if let Some(Value::String(s)) = entity.get("content_type") {
            MemoryContentType::from_string(s)?
        } else {
            return Err(Error::ConversionError(
                "Missing content_type in entity".to_string(),
            ));
        };

        let data = if let Some(value) = entity.get("data") {
            value.clone()
        } else {
            return Err(Error::ConversionError("Missing data in entity".to_string()));
        };

        let embedding = if let Some(Value::Array(arr)) = entity.get("embedding") {
            let mut embedding = Vec::new();
            for value in arr.iter() {
                if let Value::Number(n) = value {
                    if let Some(f) = n.as_f64() {
                        embedding.push(f as f32);
                    }
                }
            }
            if !embedding.is_empty() {
                Some(embedding)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            content_type,
            data,
            embedding,
        })
    }
}

/// Memory trait
pub trait Memory: Send + Sync + Debug {
    /// Get the memory ID
    fn id(&self) -> &str;

    /// Get the memory type
    fn memory_type(&self) -> MemoryTypeEnum;

    /// Get the memory metadata
    fn metadata(&self) -> &MemoryMetadata;

    /// Get mutable memory metadata
    fn metadata_mut(&mut self) -> &mut MemoryMetadata;

    /// Get the memory content
    fn content(&self) -> &MemoryContent;

    /// Get mutable memory content
    fn content_mut(&mut self) -> &mut MemoryContent;

    /// Get the memory data
    fn data(&self) -> &Value {
        &self.content().data
    }

    /// Record access
    fn record_access(&mut self) {
        self.metadata_mut().record_access();
    }

    /// Record modification
    fn record_modification(&mut self) {
        self.metadata_mut().record_modification();
    }

    /// Validate the memory
    fn validate(&self) -> Result<()>;

    /// Convert to an entity
    fn to_entity(&self) -> BaseEntity;

    /// Create from an entity
    fn from_entity(entity: BaseEntity) -> Result<Self>
    where
        Self: Sized;
}

/// Base memory implementation
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
    /// Create a new memory
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

    /// Create a new text memory
    pub fn text(id: &str, memory_type: MemoryTypeEnum, text: &str) -> Self {
        let mut memory = Self::new(
            id,
            "Text Memory",
            "Auto-generated text memory",
            memory_type,
            MemoryContent::text(text),
        );
        memory.name = format!("text_memory_{}", id);
        memory.description = format!(
            "Text memory containing: {}",
            text.chars().take(50).collect::<String>()
        );
        memory
    }

    /// Create a new JSON memory
    pub fn json(id: &str, memory_type: MemoryTypeEnum, data: Value) -> Self {
        let mut memory = Self::new(
            id,
            "JSON Memory",
            "Auto-generated JSON memory",
            memory_type,
            MemoryContent::json(data.clone()),
        );
        memory.name = format!("json_memory_{}", id);
        memory.description = format!("JSON memory containing structured data");
        memory
    }

    /// Create a new binary memory
    pub fn binary(id: &str, memory_type: MemoryTypeEnum, data: Vec<u8>) -> Self {
        let mut memory = Self::new(
            id,
            "Binary Memory",
            "Auto-generated binary memory",
            memory_type,
            MemoryContent::binary(data.clone()),
        );
        memory.name = format!("binary_memory_{}", id);
        memory.description = format!("Binary memory containing {} bytes", data.len());
        memory
    }

    /// Create a new memory with name, description, and type
    pub fn with_name_description(
        id: &str,
        name: &str,
        description: &str,
        memory_type: MemoryTypeEnum,
    ) -> Self {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), Value::String(name.to_string()));
        map.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
        let content = MemoryContent::structured(Value::Object(map));
        let mut memory = Self::new(
            id,
            "Custom Memory",
            "Auto-generated custom memory",
            memory_type,
            content,
        );
        memory.name = name.to_string();
        memory.description = description.to_string();
        memory
    }
}

impl Memory for BaseMemory {
    fn id(&self) -> &str {
        &self.id
    }

    fn memory_type(&self) -> MemoryTypeEnum {
        self.metadata.memory_type
    }

    fn metadata(&self) -> &MemoryMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut MemoryMetadata {
        &mut self.metadata
    }

    fn content(&self) -> &MemoryContent {
        &self.content
    }

    fn content_mut(&mut self) -> &mut MemoryContent {
        &mut self.content
    }

    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::ValidationError(
                "Memory ID cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn to_entity(&self) -> BaseEntity {
        use crate::graph::entity::BaseEntity;
        let mut entity =
            BaseEntity::new(&self.id, &format!("memory_{}", self.metadata.memory_type));

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
        use crate::graph::entity::Entity;

        let id = entity.id().to_string();

        // Extract memory type from entity type
        let entity_type = entity.entity_type();
        let _memory_type = if entity_type.starts_with("memory_") {
            MemoryTypeEnum::from_string(&entity_type[7..])?
        } else {
            MemoryTypeEnum::LongTerm
        };

        // Create metadata from attributes
        let mut metadata_map = HashMap::new();
        let mut content_map = HashMap::new();

        for (key, value) in entity.attributes() {
            let json_value: serde_json::Value = value.clone().into();
            if key.starts_with("content_") {
                content_map.insert(key.clone(), json_value);
            } else {
                metadata_map.insert(key.clone(), json_value);
            }
        }

        let metadata = MemoryMetadata::from_entity(&metadata_map)?;
        let content = MemoryContent::from_entity(&content_map)?;

        Ok(Self {
            id,
            name: "Base Memory".to_string(),
            description: "Default base memory".to_string(),
            updated_at: chrono::Utc::now(),
            metadata,
            content,
        })
    }
}

//! Memory content management extracted from memory_type.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::utils::{Result, error::Error};
use super::enums::MemoryContentType;

/// Memory content with optimized storage and access patterns
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
    /// Create new text content with optimal string handling
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

    /// Create new binary content with base64 encoding
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

    /// Set embedding with builder pattern
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set embedding mutably
    pub fn set_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
    }

    /// Check if content has embedding
    #[inline]
    pub fn has_embedding(&self) -> bool {
        self.embedding.is_some()
    }

    /// Get embedding dimensions
    #[inline]
    pub fn embedding_dimensions(&self) -> Option<usize> {
        self.embedding.as_ref().map(|e| e.len())
    }

    /// Get text content with error handling
    pub fn get_text(&self) -> Result<String> {
        match self.content_type {
            MemoryContentType::Text => {
                self.data.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| Error::ConversionError("Text content is not a string".to_string()))
            },
            MemoryContentType::Json => {
                serde_json::to_string(&self.data)
                    .map_err(|e| Error::ConversionError(format!("Failed to convert JSON to string: {}", e)))
            },
            MemoryContentType::Binary => {
                Err(Error::ConversionError("Cannot convert binary content to text".to_string()))
            },
        }
    }

    /// Get text content as string slice (zero-copy when possible)
    pub fn get_text_ref(&self) -> Result<&str> {
        match self.content_type {
            MemoryContentType::Text => {
                self.data.as_str()
                    .ok_or_else(|| Error::ConversionError("Text content is not a string".to_string()))
            },
            _ => Err(Error::ConversionError("Content is not text type".to_string())),
        }
    }

    /// Get binary content with optimized decoding
    pub fn get_binary(&self) -> Result<Vec<u8>> {
        match self.content_type {
            MemoryContentType::Text => {
                self.data.as_str()
                    .map(|s| s.as_bytes().to_vec())
                    .ok_or_else(|| Error::ConversionError("Text content is not a string".to_string()))
            },
            MemoryContentType::Json => {
                let json_string = serde_json::to_string(&self.data)
                    .map_err(|e| Error::ConversionError(format!("Failed to convert JSON to string: {}", e)))?;
                Ok(json_string.into_bytes())
            },
            MemoryContentType::Binary => {
                self.data.as_str()
                    .ok_or_else(|| Error::ConversionError("Binary content is not a string".to_string()))
                    .and_then(|s| {
                        base64::engine::general_purpose::STANDARD
                            .decode(s)
                            .map_err(|e| Error::ConversionError(format!("Failed to decode base64: {}", e)))
                    })
            },
        }
    }

    /// Get content size in bytes
    pub fn size_bytes(&self) -> usize {
        match &self.data {
            Value::String(s) => s.len(),
            Value::Array(arr) => arr.len() * std::mem::size_of::<Value>(),
            Value::Object(obj) => obj.len() * std::mem::size_of::<Value>() * 2, // Key + value pairs
            _ => std::mem::size_of::<Value>(),
        }
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match &self.data {
            Value::String(s) => s.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            Value::Null => true,
            _ => false,
        }
    }

    /// Get content hash for deduplication
    pub fn content_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.content_type.hash(&mut hasher);
        
        // Hash the serialized form for consistency
        if let Ok(serialized) = serde_json::to_string(&self.data) {
            serialized.hash(&mut hasher);
        }
        
        hasher.finish()
    }

    /// Convert to entity with optimized allocations
    pub fn to_entity(&self) -> HashMap<String, Value> {
        let mut entity = HashMap::with_capacity(3);
        
        entity.insert("content_type".to_string(), Value::String(self.content_type.to_string()));
        entity.insert("data".to_string(), self.data.clone());

        if let Some(embedding) = &self.embedding {
            // Convert to Value efficiently
            let embedding_value = Value::Array(
                embedding.iter().map(|&f| Value::Number(
                    serde_json::Number::from_f64(f as f64).unwrap_or(serde_json::Number::from(0))
                )).collect()
            );
            entity.insert("embedding".to_string(), embedding_value);
        }

        entity
    }

    /// Create from entity with robust parsing
    pub fn from_entity(entity: &HashMap<String, Value>) -> Result<Self> {
        let content_type = entity
            .get("content_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ConversionError("Missing content_type".to_string()))
            .and_then(MemoryContentType::from_string)?;

        let data = entity
            .get("data")
            .cloned()
            .ok_or_else(|| Error::ConversionError("Missing data in entity".to_string()))?;

        let embedding = entity
            .get("embedding")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                let mut embedding = Vec::with_capacity(arr.len());
                for value in arr.iter() {
                    if let Some(n) = value.as_number().and_then(|n| n.as_f64()) {
                        embedding.push(n as f32);
                    } else {
                        return None; // Invalid embedding format
                    }
                }
                if embedding.is_empty() { None } else { Some(embedding) }
            });

        Ok(Self {
            content_type,
            data,
            embedding,
        })
    }
}
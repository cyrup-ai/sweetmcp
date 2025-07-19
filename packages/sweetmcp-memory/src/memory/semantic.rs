// src/memory/semantic.rs
//! Semantic memory implementation for Rust-mem0.
//!
//! This module provides a specialized memory type for storing knowledge
//! and semantic information, with support for concepts, relationships,
//! and reasoning capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::memory::memory_type::{BaseMemory, MemoryTypeEnum};
use crate::utils::Result;
use crate::utils::error::Error;

/// Confidence level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Very low confidence
    VeryLow,
    /// Low confidence
    Low,
    /// Medium confidence
    Medium,
    /// High confidence
    High,
    /// Very high confidence
    VeryHigh,
}

impl ConfidenceLevel {
    /// Convert confidence level to float
    pub fn to_float(&self) -> f32 {
        match self {
            ConfidenceLevel::VeryLow => 0.1,
            ConfidenceLevel::Low => 0.3,
            ConfidenceLevel::Medium => 0.5,
            ConfidenceLevel::High => 0.7,
            ConfidenceLevel::VeryHigh => 0.9,
        }
    }

    /// Convert float to confidence level
    pub fn from_float(value: f32) -> Self {
        if value < 0.2 {
            ConfidenceLevel::VeryLow
        } else if value < 0.4 {
            ConfidenceLevel::Low
        } else if value < 0.6 {
            ConfidenceLevel::Medium
        } else if value < 0.8 {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::VeryHigh
        }
    }
}

/// Semantic item type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticItemType {
    /// Concept (entity, object, idea)
    Concept,
    /// Fact (statement about concepts)
    Fact,
    /// Rule (logical rule or pattern)
    Rule,
    /// Category (classification or grouping)
    Category,
}

impl SemanticItemType {
    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            SemanticItemType::Concept => "concept".to_string(),
            SemanticItemType::Fact => "fact".to_string(),
            SemanticItemType::Rule => "rule".to_string(),
            SemanticItemType::Category => "category".to_string(),
        }
    }

    /// Convert from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "concept" => Ok(SemanticItemType::Concept),
            "fact" => Ok(SemanticItemType::Fact),
            "rule" => Ok(SemanticItemType::Rule),
            "category" => Ok(SemanticItemType::Category),
            _ => Err(Error::ConversionError(format!(
                "Invalid semantic item type: {}",
                s
            ))),
        }
    }
}

/// Semantic item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticItem {
    /// Unique identifier
    pub id: String,

    /// Item type
    pub item_type: SemanticItemType,

    /// Category of the item
    pub category: String,

    /// Content of the item
    pub content: Value,

    /// Tags for the item
    pub tags: Vec<String>,

    /// Confidence level
    pub confidence: ConfidenceLevel,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl SemanticItem {
    /// Create a new semantic item
    pub fn new(id: &str, item_type: SemanticItemType, content: Value) -> Self {
        let now = Utc::now();
        let category = item_type.to_string();
        Self {
            id: id.to_string(),
            item_type,
            category,
            content,
            tags: Vec::new(),
            confidence: ConfidenceLevel::Medium,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Add a tag to the item
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: ConfidenceLevel) -> Self {
        self.confidence = confidence;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Convert to Value
    pub fn to_value(&self) -> Result<Value> {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert(
            "type".to_string(),
            Value::String(self.item_type.to_string()),
        );
        obj.insert("content".to_string(), self.content.clone());

        let tags = self.tags.iter().map(|t| Value::String(t.clone())).collect();
        obj.insert("tags".to_string(), Value::Array(tags));

        obj.insert(
            "confidence".to_string(),
            Value::String(format!("{:?}", self.confidence)),
        );
        obj.insert(
            "created_at".to_string(),
            Value::String(self.created_at.to_rfc3339()),
        );
        obj.insert(
            "updated_at".to_string(),
            Value::String(self.updated_at.to_rfc3339()),
        );

        if !self.metadata.is_empty() {
            obj.insert(
                "metadata".to_string(),
                serde_json::to_value(&self.metadata)?,
            );
        }

        Ok(Value::Object(obj))
    }

    /// Convert from Value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::String(s)) = obj.get("id") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing id in semantic item".to_string(),
                ));
            };

            let item_type = if let Some(Value::String(s)) = obj.get("type") {
                SemanticItemType::from_string(s)?
            } else {
                return Err(Error::ConversionError(
                    "Missing type in semantic item".to_string(),
                ));
            };

            let content = if let Some(content) = obj.get("content") {
                content.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing content in semantic item".to_string(),
                ));
            };

            let mut tags = Vec::new();
            if let Some(Value::Array(arr)) = obj.get("tags") {
                for value in arr.iter() {
                    if let Value::String(s) = value {
                        tags.push(s.clone());
                    }
                }
            }

            let confidence = if let Some(Value::String(s)) = obj.get("confidence") {
                match s.as_str() {
                    "VeryLow" => ConfidenceLevel::VeryLow,
                    "Low" => ConfidenceLevel::Low,
                    "Medium" => ConfidenceLevel::Medium,
                    "High" => ConfidenceLevel::High,
                    "VeryHigh" => ConfidenceLevel::VeryHigh,
                    _ => ConfidenceLevel::Medium,
                }
            } else {
                ConfidenceLevel::Medium
            };

            let created_at = if let Some(Value::String(s)) = obj.get("created_at") {
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid created_at format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            let updated_at = if let Some(Value::String(s)) = obj.get("updated_at") {
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid updated_at format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            let mut metadata = HashMap::new();
            if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
                for (key, val) in meta_obj.iter() {
                    metadata.insert(key.to_string(), val.clone());
                }
            }

            Ok(Self {
                id,
                item_type,
                content,
                category: "default".to_string(),
                tags,
                confidence,
                created_at,
                updated_at,
                metadata,
            })
        } else {
            Err(Error::ConversionError(
                "Invalid semantic item value".to_string(),
            ))
        }
    }
}

/// Semantic relationship type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRelationshipType {
    /// Is-a relationship (inheritance)
    IsA,
    /// Has-a relationship (composition)
    HasA,
    /// Part-of relationship
    PartOf,
    /// Related-to relationship
    RelatedTo,
    /// Causes relationship
    Causes,
    /// Custom relationship
    Custom(String),
}

impl SemanticRelationshipType {
    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            SemanticRelationshipType::IsA => "is_a".to_string(),
            SemanticRelationshipType::HasA => "has_a".to_string(),
            SemanticRelationshipType::PartOf => "part_of".to_string(),
            SemanticRelationshipType::RelatedTo => "related_to".to_string(),
            SemanticRelationshipType::Causes => "causes".to_string(),
            SemanticRelationshipType::Custom(s) => s.clone(),
        }
    }

    /// Convert from string
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "is_a" => SemanticRelationshipType::IsA,
            "has_a" => SemanticRelationshipType::HasA,
            "part_of" => SemanticRelationshipType::PartOf,
            "related_to" => SemanticRelationshipType::RelatedTo,
            "causes" => SemanticRelationshipType::Causes,
            _ => SemanticRelationshipType::Custom(s.to_string()),
        }
    }
}

/// Semantic relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    /// Unique identifier
    pub id: String,

    /// Source item ID
    pub from_id: String,

    /// Target item ID
    pub to_id: String,

    /// Source item ID (alias for from_id)
    pub source_id: String,

    /// Target item ID (alias for to_id)
    pub target_id: String,

    /// Relationship type
    pub relationship_type: SemanticRelationshipType,

    /// Confidence level
    pub confidence: ConfidenceLevel,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

impl SemanticRelationship {
    /// Create a new semantic relationship
    pub fn new(
        id: &str,
        from_id: &str,
        to_id: &str,
        relationship_type: SemanticRelationshipType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            source_id: from_id.to_string(),
            target_id: to_id.to_string(),
            relationship_type,
            confidence: ConfidenceLevel::Medium,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: ConfidenceLevel) -> Self {
        self.confidence = confidence;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Convert to Value
    pub fn to_value(&self) -> Result<Value> {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert("from_id".to_string(), Value::String(self.from_id.clone()));
        obj.insert("to_id".to_string(), Value::String(self.to_id.clone()));
        obj.insert(
            "relationship_type".to_string(),
            Value::String(self.relationship_type.to_string()),
        );
        obj.insert(
            "confidence".to_string(),
            Value::String(format!("{:?}", self.confidence)),
        );
        obj.insert(
            "created_at".to_string(),
            Value::String(self.created_at.to_rfc3339()),
        );
        obj.insert(
            "updated_at".to_string(),
            Value::String(self.updated_at.to_rfc3339()),
        );

        if !self.metadata.is_empty() {
            obj.insert(
                "metadata".to_string(),
                serde_json::to_value(&self.metadata)?,
            );
        }

        Ok(Value::Object(obj))
    }

    /// Convert from Value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Object(obj) = value {
            let id = if let Some(Value::String(s)) = obj.get("id") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing id in semantic relationship".to_string(),
                ));
            };

            let from_id = if let Some(Value::String(s)) = obj.get("from_id") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing from_id in semantic relationship".to_string(),
                ));
            };

            let to_id = if let Some(Value::String(s)) = obj.get("to_id") {
                s.clone()
            } else {
                return Err(Error::ConversionError(
                    "Missing to_id in semantic relationship".to_string(),
                ));
            };

            let relationship_type = if let Some(Value::String(s)) = obj.get("relationship_type") {
                SemanticRelationshipType::from_string(s)
            } else {
                return Err(Error::ConversionError(
                    "Missing relationship_type in semantic relationship".to_string(),
                ));
            };

            let confidence = if let Some(Value::String(s)) = obj.get("confidence") {
                match s.as_str() {
                    "VeryLow" => ConfidenceLevel::VeryLow,
                    "Low" => ConfidenceLevel::Low,
                    "Medium" => ConfidenceLevel::Medium,
                    "High" => ConfidenceLevel::High,
                    "VeryHigh" => ConfidenceLevel::VeryHigh,
                    _ => ConfidenceLevel::Medium,
                }
            } else {
                ConfidenceLevel::Medium
            };

            let created_at = if let Some(Value::String(s)) = obj.get("created_at") {
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid created_at format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            let updated_at = if let Some(Value::String(s)) = obj.get("updated_at") {
                DateTime::parse_from_rfc3339(s)
                    .map_err(|_| Error::ConversionError("Invalid updated_at format".to_string()))?
                    .with_timezone(&Utc)
            } else {
                Utc::now()
            };

            let mut metadata = HashMap::new();
            if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
                for (key, val) in meta_obj.iter() {
                    metadata.insert(key.to_string(), val.clone());
                }
            }

            Ok(Self {
                id,
                from_id: from_id.clone(),
                to_id: to_id.clone(),
                source_id: from_id,
                target_id: to_id,
                relationship_type,
                confidence,
                created_at,
                updated_at,
                metadata,
            })
        } else {
            Err(Error::ConversionError(
                "Invalid semantic relationship value".to_string(),
            ))
        }
    }
}

/// Semantic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// Base memory fields
    pub base: BaseMemory,

    /// Items in the semantic memory
    pub items: Vec<SemanticItem>,

    /// Relationships between items
    pub relationships: Vec<SemanticRelationship>,
}

impl SemanticMemory {
    /// Create a new semantic memory
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            base: BaseMemory::with_name_description(
                id,
                name,
                description,
                MemoryTypeEnum::Semantic,
            ),
            items: Vec::new(),
            relationships: Vec::new(),
        }
    }

    /// Add an item to the semantic memory
    pub fn add_item(&mut self, item: SemanticItem) {
        self.items.push(item);
    }

    /// Add a relationship
    pub fn add_relationship(&mut self, relationship: SemanticRelationship) {
        self.relationships.push(relationship);
    }

    /// Get items by category
    pub fn get_items_by_category(&self, category: &str) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.category == category)
            .collect()
    }

    /// Get related items
    pub fn get_related_items(&self, item_id: &str) -> Vec<&SemanticItem> {
        let related_ids: Vec<&str> = self
            .relationships
            .iter()
            .filter_map(|rel| {
                if rel.source_id == item_id {
                    Some(rel.target_id.as_str())
                } else if rel.target_id == item_id {
                    Some(rel.source_id.as_str())
                } else {
                    None
                }
            })
            .collect();

        self.items
            .iter()
            .filter(|item| related_ids.contains(&item.id.as_str()))
            .collect()
    }
}

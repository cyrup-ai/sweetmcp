//! Semantic item conversion utilities
//!
//! This module provides conversion methods for SemanticItem to/from JSON Value
//! with zero allocation patterns and comprehensive error handling.

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

use super::item::SemanticItem;
use super::types::{ConfidenceLevel, SemanticItemType};
use crate::utils::{Result, error::Error};

/// Conversion utilities for SemanticItem
pub struct SemanticItemConverter;

impl SemanticItemConverter {
    /// Convert semantic item to JSON Value
    /// 
    /// # Arguments
    /// * `item` - SemanticItem to convert
    /// 
    /// # Returns
    /// Result containing the JSON Value representation
    pub fn to_value(item: &SemanticItem) -> Result<Value> {
        let mut obj = serde_json::Map::new();
        
        obj.insert("id".to_string(), Value::String(item.id.clone()));
        obj.insert("item_type".to_string(), Value::String(item.item_type.to_string()));
        obj.insert("category".to_string(), Value::String(item.category.clone()));
        obj.insert("content".to_string(), item.content.clone());
        
        let tags_array: Vec<Value> = item.tags.iter()
            .map(|tag| Value::String(tag.clone()))
            .collect();
        obj.insert("tags".to_string(), Value::Array(tags_array));
        
        obj.insert("confidence".to_string(), Value::String(format!("{:?}", item.confidence)));
        obj.insert("created_at".to_string(), Value::String(item.created_at.to_rfc3339()));
        obj.insert("updated_at".to_string(), Value::String(item.updated_at.to_rfc3339()));
        
        let metadata_obj: serde_json::Map<String, Value> = item.metadata.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        obj.insert("metadata".to_string(), Value::Object(metadata_obj));
        
        Ok(Value::Object(obj))
    }

    /// Convert JSON Value to semantic item
    /// 
    /// # Arguments
    /// * `value` - JSON Value to convert
    /// 
    /// # Returns
    /// Result containing the SemanticItem or error
    pub fn from_value(value: &Value) -> Result<SemanticItem> {
        if let Value::Object(obj) = value {
            let id = Self::extract_id(obj)?;
            let item_type = Self::extract_item_type(obj)?;
            let category = Self::extract_category(obj, &item_type);
            let content = Self::extract_content(obj);
            let tags = Self::extract_tags(obj);
            let confidence = Self::extract_confidence(obj);
            let created_at = Self::extract_created_at(obj)?;
            let updated_at = Self::extract_updated_at(obj)?;
            let metadata = Self::extract_metadata(obj);

            Ok(SemanticItem {
                id,
                item_type,
                category,
                content,
                tags,
                confidence,
                created_at,
                updated_at,
                metadata,
            })
        } else {
            Err(Error::ConversionError("Invalid semantic item value".to_string()))
        }
    }

    /// Extract ID from JSON object
    fn extract_id(obj: &serde_json::Map<String, Value>) -> Result<String> {
        obj.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ConversionError("Missing id in semantic item".to_string()))
            .map(|s| s.to_string())
    }

    /// Extract item type from JSON object
    fn extract_item_type(obj: &serde_json::Map<String, Value>) -> Result<SemanticItemType> {
        let item_type_str = obj.get("item_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ConversionError("Missing item_type in semantic item".to_string()))?;
        SemanticItemType::from_string(item_type_str)
    }

    /// Extract category from JSON object
    fn extract_category(obj: &serde_json::Map<String, Value>, item_type: &SemanticItemType) -> String {
        obj.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or(&item_type.to_string())
            .to_string()
    }

    /// Extract content from JSON object
    fn extract_content(obj: &serde_json::Map<String, Value>) -> Value {
        obj.get("content")
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Extract tags from JSON object
    fn extract_tags(obj: &serde_json::Map<String, Value>) -> Vec<String> {
        if let Some(Value::Array(tags_array)) = obj.get("tags") {
            tags_array.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract confidence level from JSON object
    fn extract_confidence(obj: &serde_json::Map<String, Value>) -> ConfidenceLevel {
        if let Some(Value::String(s)) = obj.get("confidence") {
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
        }
    }

    /// Extract created_at timestamp from JSON object
    fn extract_created_at(obj: &serde_json::Map<String, Value>) -> Result<DateTime<Utc>> {
        if let Some(Value::String(s)) = obj.get("created_at") {
            DateTime::parse_from_rfc3339(s)
                .map_err(|_| Error::ConversionError("Invalid created_at format".to_string()))
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            Ok(Utc::now())
        }
    }

    /// Extract updated_at timestamp from JSON object
    fn extract_updated_at(obj: &serde_json::Map<String, Value>) -> Result<DateTime<Utc>> {
        if let Some(Value::String(s)) = obj.get("updated_at") {
            DateTime::parse_from_rfc3339(s)
                .map_err(|_| Error::ConversionError("Invalid updated_at format".to_string()))
                .map(|dt| dt.with_timezone(&Utc))
        } else {
            Ok(Utc::now())
        }
    }

    /// Extract metadata from JSON object
    fn extract_metadata(obj: &serde_json::Map<String, Value>) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        if let Some(Value::Object(meta_obj)) = obj.get("metadata") {
            for (key, val) in meta_obj.iter() {
                metadata.insert(key.to_string(), val.clone());
            }
        }
        metadata
    }

    /// Validate semantic item data
    /// 
    /// # Arguments
    /// * `item` - SemanticItem to validate
    /// 
    /// # Returns
    /// Result indicating validation success or error
    pub fn validate_item(item: &SemanticItem) -> Result<()> {
        if item.id.is_empty() {
            return Err(Error::ConversionError("SemanticItem ID cannot be empty".to_string()));
        }

        if item.category.is_empty() {
            return Err(Error::ConversionError("SemanticItem category cannot be empty".to_string()));
        }

        // Validate timestamps
        if item.updated_at < item.created_at {
            return Err(Error::ConversionError(
                "SemanticItem updated_at cannot be before created_at".to_string()
            ));
        }

        Ok(())
    }

    /// Create a minimal semantic item from basic data
    /// 
    /// # Arguments
    /// * `id` - Item identifier
    /// * `item_type` - Type of the item
    /// * `content_str` - Content as string
    /// 
    /// # Returns
    /// SemanticItem with minimal required fields
    pub fn create_minimal(id: &str, item_type: SemanticItemType, content_str: &str) -> SemanticItem {
        SemanticItem::new(id, item_type, Value::String(content_str.to_string()))
    }

    /// Create a semantic item with tags
    /// 
    /// # Arguments
    /// * `id` - Item identifier
    /// * `item_type` - Type of the item
    /// * `content` - Content value
    /// * `tags` - Vector of tags
    /// 
    /// # Returns
    /// SemanticItem with specified tags
    pub fn create_with_tags(
        id: &str,
        item_type: SemanticItemType,
        content: Value,
        tags: Vec<String>,
    ) -> SemanticItem {
        let mut item = SemanticItem::new(id, item_type, content);
        item.tags = tags;
        item
    }

    /// Create a semantic item with metadata
    /// 
    /// # Arguments
    /// * `id` - Item identifier
    /// * `item_type` - Type of the item
    /// * `content` - Content value
    /// * `metadata` - Metadata map
    /// 
    /// # Returns
    /// SemanticItem with specified metadata
    pub fn create_with_metadata(
        id: &str,
        item_type: SemanticItemType,
        content: Value,
        metadata: HashMap<String, Value>,
    ) -> SemanticItem {
        let mut item = SemanticItem::new(id, item_type, content);
        item.metadata = metadata;
        item
    }

    /// Clone semantic item with new ID
    /// 
    /// # Arguments
    /// * `item` - Original item to clone
    /// * `new_id` - New ID for the cloned item
    /// 
    /// # Returns
    /// Cloned SemanticItem with new ID and updated timestamps
    pub fn clone_with_new_id(item: &SemanticItem, new_id: &str) -> SemanticItem {
        let now = Utc::now();
        SemanticItem {
            id: new_id.to_string(),
            item_type: item.item_type.clone(),
            category: item.category.clone(),
            content: item.content.clone(),
            tags: item.tags.clone(),
            confidence: item.confidence,
            created_at: now,
            updated_at: now,
            metadata: item.metadata.clone(),
        }
    }

    /// Merge two semantic items
    /// 
    /// # Arguments
    /// * `base` - Base item to merge into
    /// * `other` - Other item to merge from
    /// 
    /// # Returns
    /// New SemanticItem with merged data
    pub fn merge_items(base: &SemanticItem, other: &SemanticItem) -> SemanticItem {
        let mut merged = base.clone();
        
        // Merge tags (avoid duplicates)
        for tag in &other.tags {
            if !merged.tags.contains(tag) {
                merged.tags.push(tag.clone());
            }
        }
        
        // Merge metadata (other takes precedence)
        for (key, value) in &other.metadata {
            merged.metadata.insert(key.clone(), value.clone());
        }
        
        // Use higher confidence
        if other.confidence > merged.confidence {
            merged.confidence = other.confidence;
        }
        
        merged.updated_at = Utc::now();
        merged
    }
}
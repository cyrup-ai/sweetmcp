//! Memory metadata management extracted from memory_type.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::utils::{Result, error::Error};
use super::enums::MemoryTypeEnum;

/// Memory metadata with high-performance access patterns
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
    /// Create new metadata with optimized initialization
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
            custom: HashMap::with_capacity(4), // Optimize for common case
        }
    }

    /// Record access with minimal allocation
    #[inline]
    pub fn record_access(&mut self) {
        self.accessed_at = Some(Utc::now());
        self.access_count = self.access_count.saturating_add(1);
    }

    /// Record modification with timestamp update
    #[inline]
    pub fn record_modification(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Set importance with bounds checking
    #[inline]
    pub fn set_importance(&mut self, importance: f32) {
        self.importance = importance.clamp(0.0, 1.0);
    }

    /// Set relevance with bounds checking
    #[inline]
    pub fn set_relevance(&mut self, relevance: f32) {
        self.relevance = relevance.clamp(0.0, 1.0);
    }

    /// Add custom metadata with builder pattern
    pub fn add_custom<T: Into<Value>>(&mut self, key: &str, value: T) -> &mut Self {
        self.custom.insert(key.to_string(), value.into());
        self
    }

    /// Get custom metadata without allocation
    #[inline]
    pub fn get_custom(&self, key: &str) -> Option<&Value> {
        self.custom.get(key)
    }

    /// Remove custom metadata
    pub fn remove_custom(&mut self, key: &str) -> Option<Value> {
        self.custom.remove(key)
    }

    /// Check if metadata has custom field
    #[inline]
    pub fn has_custom(&self, key: &str) -> bool {
        self.custom.contains_key(key)
    }

    /// Get age in seconds since creation
    #[inline]
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get time since last access in seconds
    #[inline]
    pub fn time_since_access_seconds(&self) -> Option<i64> {
        self.accessed_at.map(|accessed| (Utc::now() - accessed).num_seconds())
    }

    /// Calculate memory strength based on access patterns and importance
    pub fn calculate_strength(&self) -> f32 {
        let access_factor = if self.access_count == 0 {
            0.1
        } else {
            (self.access_count as f32).ln().min(5.0) / 5.0
        };

        let recency_factor = if let Some(last_access) = self.accessed_at {
            let hours_since_access = (Utc::now() - last_access).num_hours() as f32;
            // Exponential decay with 24-hour half-life
            2_f32.powf(-hours_since_access / 24.0)
        } else {
            0.1
        };

        // Weighted combination of factors
        (self.importance * 0.4) + (access_factor * 0.3) + (recency_factor * 0.3)
    }

    /// Convert to entity with minimal allocations
    pub fn to_entity(&self) -> HashMap<String, Value> {
        let mut entity = HashMap::with_capacity(8 + self.custom.len());
        
        entity.insert("memory_type".to_string(), Value::String(self.memory_type.to_string()));
        entity.insert("created_at".to_string(), Value::String(self.created_at.to_rfc3339()));
        entity.insert("updated_at".to_string(), Value::String(self.updated_at.to_rfc3339()));
        
        if let Some(accessed_at) = self.accessed_at {
            entity.insert("accessed_at".to_string(), Value::String(accessed_at.to_rfc3339()));
        }

        entity.insert("access_count".to_string(), Value::Number(self.access_count.into()));
        
        // Use safer float conversion
        if let Some(importance_num) = serde_json::Number::from_f64(self.importance as f64) {
            entity.insert("importance".to_string(), Value::Number(importance_num));
        }
        
        if let Some(relevance_num) = serde_json::Number::from_f64(self.relevance as f64) {
            entity.insert("relevance".to_string(), Value::Number(relevance_num));
        }

        if !self.custom.is_empty() {
            entity.insert("custom".to_string(), Value::Object(
                serde_json::Map::from_iter(self.custom.iter().map(|(k, v)| (k.clone(), v.clone())))
            ));
        }

        entity
    }

    /// Create from entity with comprehensive error handling
    pub fn from_entity(entity: &HashMap<String, Value>) -> Result<Self> {
        let memory_type = entity
            .get("memory_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ConversionError("Missing or invalid memory_type".to_string()))
            .and_then(MemoryTypeEnum::from_string)?;

        let created_at = entity
            .get("created_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ConversionError("Missing created_at".to_string()))
            .and_then(|s| DateTime::parse_from_rfc3339(s)
                .map_err(|_| Error::ConversionError("Invalid created_at format".to_string())))
            .map(|dt| dt.with_timezone(&Utc))?;

        let updated_at = entity
            .get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(created_at);

        let accessed_at = entity
            .get("accessed_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let access_count = entity
            .get("access_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let importance = entity
            .get("importance")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        let relevance = entity
            .get("relevance")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        let custom = entity
            .get("custom")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_else(HashMap::new);

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
// src/memory/memory_metadata.rs
//! Memory metadata implementation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata for memory nodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryMetadata {
    /// User ID associated with this memory
    pub user_id: Option<String>,

    /// Agent ID associated with this memory
    pub agent_id: Option<String>,

    /// Context or domain of the memory
    pub context: String,

    /// Keywords extracted from content
    pub keywords: Vec<String>,

    /// Classification tags
    pub tags: Vec<String>,

    /// Category classification
    pub category: String,

    /// Importance score (0.0 to 1.0)
    pub importance: f32,

    /// Source of the memory
    pub source: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last access timestamp
    pub last_accessed_at: Option<DateTime<Utc>>,

    /// Embedding vector
    pub embedding: Option<Vec<f32>>,

    /// Additional custom metadata as key-value pairs
    pub custom: serde_json::Value,
}

impl MemoryMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self {
            user_id: None,
            agent_id: None,
            context: "General".to_string(),
            keywords: Vec::new(),
            tags: Vec::new(),
            category: "Uncategorized".to_string(),
            importance: 0.5,
            source: None,
            created_at: Utc::now(),
            last_accessed_at: None,
            embedding: None,
            custom: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Create metadata with context and category
    pub fn with_context(context: &str, category: &str) -> Self {
        Self {
            user_id: None,
            agent_id: None,
            context: context.to_string(),
            keywords: Vec::new(),
            tags: Vec::new(),
            category: category.to_string(),
            importance: 0.5,
            source: None,
            created_at: Utc::now(),
            last_accessed_at: None,
            embedding: None,
            custom: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add a keyword
    pub fn add_keyword(&mut self, keyword: &str) {
        if !self.keywords.contains(&keyword.to_string()) {
            self.keywords.push(keyword.to_string());
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    /// Set custom metadata value
    pub fn set_custom<T: Serialize>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<(), serde_json::Error> {
        let json_value = serde_json::to_value(value)?;

        if let serde_json::Value::Object(ref mut map) = self.custom {
            map.insert(key.to_string(), json_value);
        } else {
            let mut map = serde_json::Map::new();
            map.insert(key.to_string(), json_value);
            self.custom = serde_json::Value::Object(map);
        }

        Ok(())
    }

    /// Get custom metadata value
    pub fn get_custom<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        if let serde_json::Value::Object(ref map) = self.custom {
            if let Some(value) = map.get(key) {
                return serde_json::from_value(value.clone()).ok();
            }
        }
        None
    }
}

/// Search filters for memory queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryFilter {
    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by session ID
    pub session_id: Option<String>,

    /// Filter by context
    pub context: Option<String>,

    /// Filter by category
    pub category: Option<String>,

    /// Filter by tags (any match)
    pub tags: Vec<String>,

    /// Filter by keywords (any match)
    pub keywords: Vec<String>,

    /// Filter by minimum importance
    pub min_importance: Option<f32>,

    /// Filter by creation time range
    pub time_range: Option<TimeRange>,

    /// Filter by custom metadata
    pub custom_filters: Vec<CustomFilter>,
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: Option<DateTime<Utc>>,

    /// End time (inclusive)
    pub end: Option<DateTime<Utc>>,
}

/// Custom filter for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFilter {
    /// Metadata key
    pub key: String,

    /// Filter operation
    pub operation: FilterOperation,

    /// Filter value
    pub value: serde_json::Value,
}

/// Filter operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperation {
    /// Equal to
    Equals,

    /// Not equal to
    NotEquals,

    /// Greater than
    GreaterThan,

    /// Less than
    LessThan,

    /// Contains substring
    Contains,

    /// Starts with substring
    StartsWith,

    /// Ends with substring
    EndsWith,

    /// In array
    In,

    /// Not in array
    NotIn,
}

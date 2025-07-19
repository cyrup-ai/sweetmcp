//! Memory filtering functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::memory::MemoryType;

/// Filter criteria for memory queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    /// Filter by memory types
    pub memory_types: Option<Vec<MemoryType>>,

    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by agent ID
    pub agent_id: Option<String>,

    /// Filter by tags
    pub tags: Option<Vec<String>>,

    /// Filter by time range
    pub time_range: Option<TimeRange>,

    /// Filter by importance score
    pub importance_range: Option<(f32, f32)>,

    /// Filter by metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Maximum number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (inclusive)
    pub start: Option<DateTime<Utc>>,

    /// End time (exclusive)
    pub end: Option<DateTime<Utc>>,
}

impl MemoryFilter {
    /// Create a new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add memory type filter
    pub fn with_memory_types(mut self, types: Vec<MemoryType>) -> Self {
        self.memory_types = Some(types);
        self
    }

    /// Add user ID filter
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add agent ID filter
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Add tags filter
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Add time range filter
    pub fn with_time_range(
        mut self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Self {
        self.time_range = Some(TimeRange { start, end });
        self
    }

    /// Add importance range filter
    pub fn with_importance_range(mut self, min: f32, max: f32) -> Self {
        self.importance_range = Some((min, max));
        self
    }

    /// Add metadata filter
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let mut metadata = self.metadata.unwrap_or_default();
        metadata.insert(key.into(), value);
        self.metadata = Some(metadata);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set result offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Builder for complex memory filters
pub struct MemoryFilterBuilder {
    filter: MemoryFilter,
}

impl MemoryFilterBuilder {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self {
            filter: MemoryFilter::new(),
        }
    }

    /// Build the filter
    pub fn build(self) -> MemoryFilter {
        self.filter
    }

    /// Add memory type filter
    pub fn memory_types(mut self, types: Vec<MemoryType>) -> Self {
        self.filter = self.filter.with_memory_types(types);
        self
    }

    /// Add user ID filter
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.filter = self.filter.with_user_id(user_id);
        self
    }

    /// Add time range filter for memories created in the last N hours
    pub fn in_last_hours(mut self, hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        self.filter = self.filter.with_time_range(Some(start), Some(end));
        self
    }

    /// Add time range filter for memories created in the last N days
    pub fn in_last_days(mut self, days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        self.filter = self.filter.with_time_range(Some(start), Some(end));
        self
    }
}

impl Default for MemoryFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

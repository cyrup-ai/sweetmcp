// src/memory/history.rs
//! History tracking implementation for Rust-mem0.
//!
//! This module provides versioning and history tracking for memory nodes,
//! with support for evolution tracking, history queries, and diff/merge operations.

use crate::graph::entity::{BaseEntity, Entity};
use crate::utils::Result;
use crate::utils::error::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use surrealdb::sql::Value;

/// Change type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Creation of a new memory
    Creation,
    /// Update of an existing memory
    Update,
    /// Deletion of a memory
    Deletion,
    /// Restoration of a deleted memory
    Restoration,
    /// Merge of multiple memories
    Merge,
    /// Split of a memory into multiple parts
    Split,
    /// Custom change type
    Custom(u8),
}

impl ChangeType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Creation => "creation",
            ChangeType::Update => "update",
            ChangeType::Deletion => "deletion",
            ChangeType::Restoration => "restoration",
            ChangeType::Merge => "merge",
            ChangeType::Split => "split",
            ChangeType::Custom(_) => "custom",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "creation" => Ok(ChangeType::Creation),
            "update" => Ok(ChangeType::Update),
            "deletion" => Ok(ChangeType::Deletion),
            "restoration" => Ok(ChangeType::Restoration),
            "merge" => Ok(ChangeType::Merge),
            "split" => Ok(ChangeType::Split),
            s if s.starts_with("custom") => {
                if let Some(code_str) = s.strip_prefix("custom") {
                    if let Ok(code) = code_str.trim().parse::<u8>() {
                        Ok(ChangeType::Custom(code))
                    } else {
                        Ok(ChangeType::Custom(0))
                    }
                } else {
                    Ok(ChangeType::Custom(0))
                }
            }
            _ => Err(Error::ValidationError(format!(
                "Invalid change type: {}",
                s
            ))),
        }
    }

    /// Convert to value
    pub fn to_value(&self) -> Value {
        match self {
            ChangeType::Creation => Value::Strand("creation".into()),
            ChangeType::Update => Value::Strand("update".into()),
            ChangeType::Deletion => Value::Strand("deletion".into()),
            ChangeType::Restoration => Value::Strand("restoration".into()),
            ChangeType::Merge => Value::Strand("merge".into()),
            ChangeType::Split => Value::Strand("split".into()),
            ChangeType::Custom(code) => Value::Strand(format!("custom{}", code).into()),
        }
    }

    /// Create from value
    pub fn from_value(value: &Value) -> Result<Self> {
        if let Value::Strand(s) = value {
            Self::from_str(&s.to_string())
        } else {
            Err(Error::ConversionError(
                "Invalid change type value".to_string(),
            ))
        }
    }
}

/// Memory version struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVersion {
    /// Version ID
    pub id: String,
    /// Memory ID
    pub memory_id: String,
    /// Version number
    pub version: u32,
    /// Change type
    pub change_type: ChangeType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// User ID
    pub user_id: Option<String>,
    /// Content
    pub content: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, Value>,
    /// Previous version ID
    pub previous_version_id: Option<String>,
    /// Related version IDs (for merge/split)
    pub related_version_ids: Vec<String>,
    /// Change summary
    pub change_summary: Option<String>,
    /// Diff from previous version
    pub diff: Option<String>,
}

impl MemoryVersion {
    /// Create a new memory version
    pub fn new(
        id: &str,
        memory_id: &str,
        version: u32,
        change_type: ChangeType,
        content: Option<&str>,
        previous_version_id: Option<&str>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: id.to_string(),
            memory_id: memory_id.to_string(),
            version,
            change_type,
            timestamp: now,
            user_id: None,
            content: content.map(|s| s.to_string()),
            metadata: HashMap::new(),
            previous_version_id: previous_version_id.map(|s| s.to_string()),
            related_version_ids: Vec::new(),
            change_summary: None,
            diff: None,
        }
    }

    /// Create a new creation version
    pub fn creation(id: &str, memory_id: &str, content: &str) -> Self {
        Self::new(id, memory_id, 1, ChangeType::Creation, Some(content), None)
    }

    /// Create a new update version
    pub fn update(
        id: &str,
        memory_id: &str,
        version: u32,
        content: &str,
        previous_version_id: &str,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            version,
            ChangeType::Update,
            Some(content),
            Some(previous_version_id),
        )
    }

    /// Create a new deletion version
    pub fn deletion(id: &str, memory_id: &str, version: u32, previous_version_id: &str) -> Self {
        Self::new(
            id,
            memory_id,
            version,
            ChangeType::Deletion,
            None,
            Some(previous_version_id),
        )
    }

    /// Create a new restoration version
    pub fn restoration(
        id: &str,
        memory_id: &str,
        version: u32,
        content: &str,
        previous_version_id: &str,
    ) -> Self {
        Self::new(
            id,
            memory_id,
            version,
            ChangeType::Restoration,
            Some(content),
            Some(previous_version_id),
        )
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Add metadata
    pub fn with_metadata<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.metadata.insert(key.to_string(), value.into());
        self
    }

    /// Add related version ID
    pub fn with_related_version_id(mut self, related_version_id: &str) -> Self {
        self.related_version_ids
            .push(related_version_id.to_string());
        self
    }

    /// Set change summary
    pub fn with_change_summary(mut self, change_summary: &str) -> Self {
        self.change_summary = Some(change_summary.to_string());
        self
    }

    /// Set diff
    pub fn with_diff(mut self, diff: &str) -> Self {
        self.diff = Some(diff.to_string());
        self
    }

    /// Convert to entity
    pub fn to_entity(&self) -> BaseEntity {
        let mut entity = BaseEntity::new(&self.id, "memory_version");

        // Add basic attributes
        entity = entity.with_attribute("memory_id", Value::Strand(self.memory_id.clone().into()));
        entity = entity.with_attribute("version", Value::Number(self.version.into()));
        entity = entity.with_attribute("change_type", self.change_type.to_value());
        entity = entity.with_attribute("timestamp", Value::Datetime(self.timestamp.into()));

        // Add optional attributes
        if let Some(ref user_id) = self.user_id {
            entity = entity.with_attribute("user_id", Value::Strand(user_id.clone().into()));
        }

        if let Some(ref content) = self.content {
            entity = entity.with_attribute("content", Value::Strand(content.clone().into()));
        }

        if let Some(ref previous_version_id) = self.previous_version_id {
            entity = entity.with_attribute(
                "previous_version_id",
                Value::Strand(previous_version_id.clone().into()),
            );
        }

        if !self.related_version_ids.is_empty() {
            let related_ids = self
                .related_version_ids
                .iter()
                .map(|id| Value::Strand(id.clone().into()))
                .collect::<Vec<_>>();
            entity = entity.with_attribute("related_version_ids", Value::Array(related_ids.into()));
        }

        if let Some(ref change_summary) = self.change_summary {
            entity = entity.with_attribute(
                "change_summary",
                Value::Strand(change_summary.clone().into()),
            );
        }

        if let Some(ref diff) = self.diff {
            entity = entity.with_attribute("diff", Value::Strand(diff.clone().into()));
        }

        // Add metadata
        for (key, value) in &self.metadata {
            entity = entity.with_attribute(&format!("metadata_{}", key), value.clone());
        }

        entity
    }

    /// Create from entity
    pub fn from_entity(entity: &dyn Entity) -> Result<Self> {
        let id = entity.id().to_string();

        let memory_id = if let Some(Value::Strand(s)) = entity.get_attribute("memory_id") {
            s.to_string()
        } else {
            return Err(Error::ConversionError(
                "Missing memory_id attribute".to_string(),
            ));
        };

        let version = if let Some(Value::Number(n)) = entity.get_attribute("version") {
            n.as_int() as u32
        } else {
            return Err(Error::ConversionError(
                "Missing version attribute".to_string(),
            ));
        };

        let change_type = if let Some(value) = entity.get_attribute("change_type") {
            ChangeType::from_value(value)?
        } else {
            return Err(Error::ConversionError(
                "Missing change_type attribute".to_string(),
            ));
        };

        let timestamp = if let Some(Value::Datetime(dt)) = entity.get_attribute("timestamp") {
            DateTime::<Utc>::from(dt.clone())
        } else {
            return Err(Error::ConversionError(
                "Missing timestamp attribute".to_string(),
            ));
        };

        let user_id = if let Some(Value::Strand(s)) = entity.get_attribute("user_id") {
            Some(s.to_string())
        } else {
            None
        };

        let content = if let Some(Value::Strand(s)) = entity.get_attribute("content") {
            Some(s.to_string())
        } else {
            None
        };

        let previous_version_id =
            if let Some(Value::Strand(s)) = entity.get_attribute("previous_version_id") {
                Some(s.to_string())
            } else {
                None
            };

        let related_version_ids =
            if let Some(Value::Array(arr)) = entity.get_attribute("related_version_ids") {
                arr.iter()
                    .filter_map(|value| {
                        if let Value::Strand(s) = value {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };

        let change_summary = if let Some(Value::Strand(s)) = entity.get_attribute("change_summary")
        {
            Some(s.to_string())
        } else {
            None
        };

        let diff = if let Some(Value::Strand(s)) = entity.get_attribute("diff") {
            Some(s.to_string())
        } else {
            None
        };

        // Extract metadata
        let mut metadata = HashMap::new();
        for (key, value) in entity.attributes() {
            if key.starts_with("metadata_") {
                let metadata_key = key.strip_prefix("metadata_").unwrap().to_string();
                metadata.insert(metadata_key, value.clone());
            }
        }

        Ok(Self {
            id,
            memory_id,
            version,
            change_type,
            timestamp,
            user_id,
            content,
            metadata,
            previous_version_id,
            related_version_ids,
            change_summary,
            diff,
        })
    }
}

/// Memory history struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHistory {
    /// Memory ID
    pub memory_id: String,
    /// Current version
    pub current_version: u32,
    /// Versions
    pub versions: Vec<MemoryVersion>,
}

impl MemoryHistory {
    /// Create a new memory history
    pub fn new(memory_id: &str) -> Self {
        Self {
            memory_id: memory_id.to_string(),
            current_version: 0,
            versions: Vec::new(),
        }
    }

    /// Add version
    pub fn add_version(&mut self, version: MemoryVersion) -> Result<()> {
        // Validate version
        if version.memory_id != self.memory_id {
            return Err(Error::ValidationError(format!(
                "Version memory ID {} does not match history memory ID {}",
                version.memory_id, self.memory_id
            )));
        }

        // Update current version if needed
        if version.version > self.current_version {
            self.current_version = version.version;
        }

        // Add version
        self.versions.push(version);

        // Sort versions by version number
        self.versions.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(())
    }

    /// Get version by number
    pub fn get_version(&self, version: u32) -> Option<&MemoryVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Get version by ID
    pub fn get_version_by_id(&self, id: &str) -> Option<&MemoryVersion> {
        self.versions.iter().find(|v| v.id == id)
    }

    /// Get current version
    pub fn get_current_version(&self) -> Option<&MemoryVersion> {
        self.get_version(self.current_version)
    }

    /// Get versions by change type
    pub fn get_versions_by_change_type(&self, change_type: ChangeType) -> Vec<&MemoryVersion> {
        self.versions
            .iter()
            .filter(|v| v.change_type == change_type)
            .collect()
    }

    /// Get versions by time range
    pub fn get_versions_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&MemoryVersion> {
        self.versions
            .iter()
            .filter(|v| v.timestamp >= start && v.timestamp <= end)
            .collect()
    }

    /// Get version history
    pub fn get_version_history(&self) -> Vec<(u32, ChangeType, DateTime<Utc>)> {
        self.versions
            .iter()
            .map(|v| (v.version, v.change_type, v.timestamp))
            .collect()
    }

    /// Get version path
    pub fn get_version_path(
        &self,
        from_version: u32,
        to_version: u32,
    ) -> Result<Vec<&MemoryVersion>> {
        // Validate versions
        if from_version > to_version {
            return Err(Error::ValidationError(format!(
                "From version {} is greater than to version {}",
                from_version, to_version
            )));
        }

        // Get versions in range
        let mut path = Vec::new();
        for version in from_version..=to_version {
            if let Some(v) = self.get_version(version) {
                path.push(v);
            } else {
                return Err(Error::NotFound(format!("Version {} not found", version)));
            }
        }

        Ok(path)
    }

    /// Calculate diff between versions
    pub fn diff_versions(&self, from_version: u32, to_version: u32) -> Result<String> {
        // Get versions
        let from = self
            .get_version(from_version)
            .ok_or_else(|| Error::NotFound(format!("Version {} not found", from_version)))?;

        let to = self
            .get_version(to_version)
            .ok_or_else(|| Error::NotFound(format!("Version {} not found", to_version)))?;

        // Get content
        let from_content = from.content.as_deref().unwrap_or("");
        let to_content = to.content.as_deref().unwrap_or("");

        // Calculate diff
        let diff = self.calculate_diff(from_content, to_content);

        Ok(diff)
    }

    /// Calculate text diff between two strings
    fn calculate_diff(&self, old: &str, new: &str) -> String {
        use std::fmt::Write;

        let mut output = String::new();
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        // Simple line-based diff
        writeln!(&mut output, "--- Version A").unwrap();
        writeln!(&mut output, "+++ Version B").unwrap();

        let max_lines = old_lines.len().max(new_lines.len());
        for i in 0..max_lines {
            match (old_lines.get(i), new_lines.get(i)) {
                (Some(old_line), Some(new_line)) if old_line != new_line => {
                    writeln!(&mut output, "-{}", old_line).unwrap();
                    writeln!(&mut output, "+{}", new_line).unwrap();
                }
                (Some(old_line), None) => {
                    writeln!(&mut output, "-{}", old_line).unwrap();
                }
                (None, Some(new_line)) => {
                    writeln!(&mut output, "+{}", new_line).unwrap();
                }
                (Some(line), Some(_)) => {
                    writeln!(&mut output, " {}", line).unwrap();
                }
                _ => {}
            }
        }

        output
    }
}

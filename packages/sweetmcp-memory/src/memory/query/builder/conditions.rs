//! Query condition definitions and implementations
//!
//! This module provides blazing-fast query condition types with zero-allocation
//! patterns and comprehensive condition matching capabilities.

use crate::memory::MemoryType;
use serde_json::Value;

/// Query condition for complex memory queries
#[derive(Debug, Clone)]
pub enum QueryCondition {
    /// Text similarity condition
    TextSimilarity { text: String, min_score: f32 },
    /// Memory type condition
    MemoryType(Vec<MemoryType>),
    /// Time range condition
    TimeRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    /// Metadata condition
    Metadata {
        key: String,
        value: Value,
    },
    /// Relationship condition
    HasRelationship {
        relationship_type: String,
        target_id: Option<String>,
    },
    /// Importance score condition
    ImportanceRange {
        min_score: Option<f32>,
        max_score: Option<f32>,
    },
    /// User condition
    User { user_id: String },
    /// Project condition
    Project { project_id: String },
    /// Content length condition
    ContentLength {
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    /// Tag condition
    HasTag { tag: String },
    /// Nested condition (for complex logic)
    Nested {
        conditions: Vec<QueryCondition>,
        operator: LogicalOperator,
    },
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

impl QueryCondition {
    /// Create text similarity condition
    #[inline]
    pub fn text_similarity(text: impl Into<String>, min_score: f32) -> Self {
        Self::TextSimilarity {
            text: text.into(),
            min_score: min_score.max(0.0).min(1.0),
        }
    }
    
    /// Create memory type condition
    #[inline]
    pub fn memory_types(types: Vec<MemoryType>) -> Self {
        Self::MemoryType(types)
    }
    
    /// Create single memory type condition
    #[inline]
    pub fn memory_type(memory_type: MemoryType) -> Self {
        Self::MemoryType(vec![memory_type])
    }
    
    /// Create time range condition
    #[inline]
    pub fn time_range(
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self::TimeRange { start, end }
    }
    
    /// Create last N days condition
    #[inline]
    pub fn last_days(days: u32) -> Self {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::days(days as i64);
        Self::TimeRange { start, end }
    }
    
    /// Create last N hours condition
    #[inline]
    pub fn last_hours(hours: u32) -> Self {
        let end = chrono::Utc::now();
        let start = end - chrono::Duration::hours(hours as i64);
        Self::TimeRange { start, end }
    }
    
    /// Create metadata condition
    #[inline]
    pub fn metadata(key: impl Into<String>, value: Value) -> Self {
        Self::Metadata {
            key: key.into(),
            value,
        }
    }
    
    /// Create string metadata condition
    #[inline]
    pub fn metadata_string(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Metadata {
            key: key.into(),
            value: Value::String(value.into()),
        }
    }
    
    /// Create boolean metadata condition
    #[inline]
    pub fn metadata_bool(key: impl Into<String>, value: bool) -> Self {
        Self::Metadata {
            key: key.into(),
            value: Value::Bool(value),
        }
    }
    
    /// Create numeric metadata condition
    #[inline]
    pub fn metadata_number(key: impl Into<String>, value: f64) -> Self {
        Self::Metadata {
            key: key.into(),
            value: Value::Number(serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0))),
        }
    }
    
    /// Create relationship condition
    #[inline]
    pub fn has_relationship(
        relationship_type: impl Into<String>,
        target_id: Option<String>,
    ) -> Self {
        Self::HasRelationship {
            relationship_type: relationship_type.into(),
            target_id,
        }
    }
    
    /// Create importance range condition
    #[inline]
    pub fn importance_range(min_score: Option<f32>, max_score: Option<f32>) -> Self {
        Self::ImportanceRange { min_score, max_score }
    }
    
    /// Create minimum importance condition
    #[inline]
    pub fn min_importance(min_score: f32) -> Self {
        Self::ImportanceRange {
            min_score: Some(min_score),
            max_score: None,
        }
    }
    
    /// Create maximum importance condition
    #[inline]
    pub fn max_importance(max_score: f32) -> Self {
        Self::ImportanceRange {
            min_score: None,
            max_score: Some(max_score),
        }
    }
    
    /// Create user condition
    #[inline]
    pub fn user(user_id: impl Into<String>) -> Self {
        Self::User {
            user_id: user_id.into(),
        }
    }
    
    /// Create project condition
    #[inline]
    pub fn project(project_id: impl Into<String>) -> Self {
        Self::Project {
            project_id: project_id.into(),
        }
    }
    
    /// Create content length range condition
    #[inline]
    pub fn content_length_range(min_length: Option<usize>, max_length: Option<usize>) -> Self {
        Self::ContentLength { min_length, max_length }
    }
    
    /// Create minimum content length condition
    #[inline]
    pub fn min_content_length(min_length: usize) -> Self {
        Self::ContentLength {
            min_length: Some(min_length),
            max_length: None,
        }
    }
    
    /// Create maximum content length condition
    #[inline]
    pub fn max_content_length(max_length: usize) -> Self {
        Self::ContentLength {
            min_length: None,
            max_length: Some(max_length),
        }
    }
    
    /// Create tag condition
    #[inline]
    pub fn has_tag(tag: impl Into<String>) -> Self {
        Self::HasTag {
            tag: tag.into(),
        }
    }
    
    /// Create nested condition
    #[inline]
    pub fn nested(conditions: Vec<QueryCondition>, operator: LogicalOperator) -> Self {
        Self::Nested { conditions, operator }
    }
    
    /// Check if condition is empty (no meaningful constraints)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::MemoryType(types) => types.is_empty(),
            Self::ImportanceRange { min_score: None, max_score: None } => true,
            Self::ContentLength { min_length: None, max_length: None } => true,
            Self::Nested { conditions, .. } => conditions.is_empty(),
            _ => false,
        }
    }
    
    /// Get condition complexity score (for optimization)
    pub fn complexity_score(&self) -> u32 {
        match self {
            Self::TextSimilarity { .. } => 10, // High complexity due to similarity computation
            Self::MemoryType(types) => types.len() as u32,
            Self::TimeRange { .. } => 2,
            Self::Metadata { .. } => 3,
            Self::HasRelationship { .. } => 5,
            Self::ImportanceRange { .. } => 2,
            Self::User { .. } => 1,
            Self::Project { .. } => 1,
            Self::ContentLength { .. } => 1,
            Self::HasTag { .. } => 2,
            Self::Nested { conditions, .. } => {
                conditions.iter().map(|c| c.complexity_score()).sum::<u32>() + 1
            }
        }
    }
    
    /// Get human-readable description of the condition
    pub fn description(&self) -> String {
        match self {
            Self::TextSimilarity { text, min_score } => {
                format!("Text similarity: '{}' (min score: {:.2})", text, min_score)
            }
            Self::MemoryType(types) => {
                format!("Memory types: {:?}", types)
            }
            Self::TimeRange { start, end } => {
                format!("Time range: {} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d"))
            }
            Self::Metadata { key, value } => {
                format!("Metadata: {} = {}", key, value)
            }
            Self::HasRelationship { relationship_type, target_id } => {
                match target_id {
                    Some(id) => format!("Has relationship: {} to {}", relationship_type, id),
                    None => format!("Has relationship: {}", relationship_type),
                }
            }
            Self::ImportanceRange { min_score, max_score } => {
                match (min_score, max_score) {
                    (Some(min), Some(max)) => format!("Importance: {} to {}", min, max),
                    (Some(min), None) => format!("Importance: >= {}", min),
                    (None, Some(max)) => format!("Importance: <= {}", max),
                    (None, None) => "Importance: any".to_string(),
                }
            }
            Self::User { user_id } => format!("User: {}", user_id),
            Self::Project { project_id } => format!("Project: {}", project_id),
            Self::ContentLength { min_length, max_length } => {
                match (min_length, max_length) {
                    (Some(min), Some(max)) => format!("Content length: {} to {}", min, max),
                    (Some(min), None) => format!("Content length: >= {}", min),
                    (None, Some(max)) => format!("Content length: <= {}", max),
                    (None, None) => "Content length: any".to_string(),
                }
            }
            Self::HasTag { tag } => format!("Has tag: {}", tag),
            Self::Nested { conditions, operator } => {
                format!("Nested ({:?}): {} conditions", operator, conditions.len())
            }
        }
    }
}

impl LogicalOperator {
    /// Check if operator is AND
    #[inline]
    pub fn is_and(&self) -> bool {
        matches!(self, LogicalOperator::And)
    }

    /// Check if operator is OR
    #[inline]
    pub fn is_or(&self) -> bool {
        matches!(self, LogicalOperator::Or)
    }

    /// Check if operator is NOT
    #[inline]
    pub fn is_not(&self) -> bool {
        matches!(self, LogicalOperator::Not)
    }

    /// Get operator precedence (higher number = higher precedence)
    #[inline]
    pub fn precedence(&self) -> u8 {
        match self {
            LogicalOperator::Not => 3,
            LogicalOperator::And => 2,
            LogicalOperator::Or => 1,
        }
    }

    /// Get operator symbol
    #[inline]
    pub fn symbol(&self) -> &'static str {
        match self {
            LogicalOperator::And => "AND",
            LogicalOperator::Or => "OR",
            LogicalOperator::Not => "NOT",
        }
    }
    
    /// Apply operator to boolean values
    #[inline]
    pub fn apply(&self, left: bool, right: bool) -> bool {
        match self {
            LogicalOperator::And => left && right,
            LogicalOperator::Or => left || right,
            LogicalOperator::Not => !left, // Note: right is ignored for NOT
        }
    }
    
    /// Check if operator is commutative
    #[inline]
    pub fn is_commutative(&self) -> bool {
        matches!(self, LogicalOperator::And | LogicalOperator::Or)
    }
    
    /// Get inverse operator
    #[inline]
    pub fn inverse(&self) -> Self {
        match self {
            LogicalOperator::And => LogicalOperator::Or,
            LogicalOperator::Or => LogicalOperator::And,
            LogicalOperator::Not => LogicalOperator::Not, // NOT is its own inverse
        }
    }
}
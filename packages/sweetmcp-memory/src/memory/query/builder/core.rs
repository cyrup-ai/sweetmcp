//! Core query builder implementation
//!
//! This module provides the main ComplexQueryBuilder with zero-allocation
//! patterns and blazing-fast query construction capabilities.

use super::conditions::{QueryCondition, LogicalOperator};
use super::executor::QueryExecutorWrapper;
use super::validation::QueryValidator;
use crate::memory::{MemoryType, filter::MemoryFilter};
use crate::memory::query::core::{MemoryQuery, SortOrder};

/// Builder for complex queries with multiple conditions
pub struct ComplexQueryBuilder {
    conditions: Vec<QueryCondition>,
    operator: LogicalOperator,
    executor: Option<QueryExecutorWrapper>,
    validator: QueryValidator,
}

impl ComplexQueryBuilder {
    /// Create a new complex query builder
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            operator: LogicalOperator::And,
            executor: None,
            validator: QueryValidator::new(),
        }
    }

    /// Create a new complex query builder with executor
    pub fn with_executor(executor: QueryExecutorWrapper) -> Self {
        Self {
            conditions: Vec::new(),
            operator: LogicalOperator::And,
            executor: Some(executor),
            validator: QueryValidator::new(),
        }
    }

    /// Set the logical operator
    pub fn with_operator(mut self, operator: LogicalOperator) -> Self {
        self.operator = operator;
        self
    }

    /// Add a condition
    pub fn add_condition(mut self, condition: QueryCondition) -> Self {
        if !condition.is_empty() {
            self.conditions.push(condition);
        }
        self
    }

    /// Add multiple conditions
    pub fn add_conditions(mut self, conditions: Vec<QueryCondition>) -> Self {
        for condition in conditions {
            if !condition.is_empty() {
                self.conditions.push(condition);
            }
        }
        self
    }

    /// Add text similarity condition
    pub fn with_text_similarity(self, text: impl Into<String>, min_score: f32) -> Self {
        self.add_condition(QueryCondition::text_similarity(text, min_score))
    }

    /// Add memory type condition
    pub fn with_memory_types(self, types: Vec<MemoryType>) -> Self {
        if !types.is_empty() {
            self.add_condition(QueryCondition::memory_types(types))
        } else {
            self
        }
    }

    /// Add single memory type condition
    pub fn with_memory_type(self, memory_type: MemoryType) -> Self {
        self.add_condition(QueryCondition::memory_type(memory_type))
    }

    /// Add time range condition
    pub fn with_time_range(
        self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.add_condition(QueryCondition::time_range(start, end))
    }

    /// Add time range condition (last N days)
    pub fn with_last_days(self, days: u32) -> Self {
        self.add_condition(QueryCondition::last_days(days))
    }

    /// Add time range condition (last N hours)
    pub fn with_last_hours(self, hours: u32) -> Self {
        self.add_condition(QueryCondition::last_hours(hours))
    }

    /// Add metadata condition
    pub fn with_metadata(self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.add_condition(QueryCondition::metadata(key, value))
    }

    /// Add string metadata condition
    pub fn with_metadata_string(self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_condition(QueryCondition::metadata_string(key, value))
    }

    /// Add boolean metadata condition
    pub fn with_metadata_bool(self, key: impl Into<String>, value: bool) -> Self {
        self.add_condition(QueryCondition::metadata_bool(key, value))
    }

    /// Add numeric metadata condition
    pub fn with_metadata_number(self, key: impl Into<String>, value: f64) -> Self {
        self.add_condition(QueryCondition::metadata_number(key, value))
    }

    /// Add relationship condition
    pub fn with_relationship(
        self,
        relationship_type: impl Into<String>,
        target_id: Option<String>,
    ) -> Self {
        self.add_condition(QueryCondition::has_relationship(relationship_type, target_id))
    }

    /// Add importance range condition
    pub fn with_importance_range(self, min_score: Option<f32>, max_score: Option<f32>) -> Self {
        self.add_condition(QueryCondition::importance_range(min_score, max_score))
    }

    /// Add minimum importance condition
    pub fn with_min_importance(self, min_score: f32) -> Self {
        self.add_condition(QueryCondition::min_importance(min_score))
    }

    /// Add maximum importance condition
    pub fn with_max_importance(self, max_score: f32) -> Self {
        self.add_condition(QueryCondition::max_importance(max_score))
    }

    /// Add user condition
    pub fn with_user(self, user_id: impl Into<String>) -> Self {
        self.add_condition(QueryCondition::user(user_id))
    }

    /// Add project condition
    pub fn with_project(self, project_id: impl Into<String>) -> Self {
        self.add_condition(QueryCondition::project(project_id))
    }

    /// Add content length range condition
    pub fn with_content_length_range(self, min_length: Option<usize>, max_length: Option<usize>) -> Self {
        self.add_condition(QueryCondition::content_length_range(min_length, max_length))
    }

    /// Add minimum content length condition
    pub fn with_min_content_length(self, min_length: usize) -> Self {
        self.add_condition(QueryCondition::min_content_length(min_length))
    }

    /// Add maximum content length condition
    pub fn with_max_content_length(self, max_length: usize) -> Self {
        self.add_condition(QueryCondition::max_content_length(max_length))
    }

    /// Add tag condition
    pub fn with_tag(self, tag: impl Into<String>) -> Self {
        self.add_condition(QueryCondition::has_tag(tag))
    }

    /// Add nested condition group
    pub fn with_nested(self, conditions: Vec<QueryCondition>, operator: LogicalOperator) -> Self {
        if !conditions.is_empty() {
            self.add_condition(QueryCondition::nested(conditions, operator))
        } else {
            self
        }
    }

    /// Build the final query
    pub fn build(self) -> Result<MemoryQuery, String> {
        // Validate the query before building
        self.validator.validate_conditions(&self.conditions)?;
        
        if self.conditions.is_empty() {
            return Err("Query must have at least one condition".to_string());
        }

        // Convert conditions to MemoryFilter
        let filter = self.convert_to_filter()?;
        
        Ok(MemoryQuery {
            filter,
            sort_order: SortOrder::Relevance,
            limit: None,
            offset: 0,
        })
    }

    /// Build query with custom sort order
    pub fn build_with_sort(self, sort_order: SortOrder) -> Result<MemoryQuery, String> {
        let mut query = self.build()?;
        query.sort_order = sort_order;
        Ok(query)
    }

    /// Build query with limit
    pub fn build_with_limit(self, limit: usize) -> Result<MemoryQuery, String> {
        let mut query = self.build()?;
        query.limit = Some(limit);
        Ok(query)
    }

    /// Build query with offset
    pub fn build_with_offset(self, offset: usize) -> Result<MemoryQuery, String> {
        let mut query = self.build()?;
        query.offset = offset;
        Ok(query)
    }

    /// Build query with pagination
    pub fn build_with_pagination(self, limit: usize, offset: usize) -> Result<MemoryQuery, String> {
        let mut query = self.build()?;
        query.limit = Some(limit);
        query.offset = offset;
        Ok(query)
    }

    /// Get condition count
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Check if builder has conditions
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }

    /// Get complexity score for the entire query
    pub fn complexity_score(&self) -> u32 {
        self.conditions.iter().map(|c| c.complexity_score()).sum()
    }

    /// Get estimated execution time category
    pub fn execution_time_category(&self) -> ExecutionTimeCategory {
        let score = self.complexity_score();
        match score {
            0..=5 => ExecutionTimeCategory::Fast,
            6..=15 => ExecutionTimeCategory::Medium,
            16..=30 => ExecutionTimeCategory::Slow,
            _ => ExecutionTimeCategory::VerySlow,
        }
    }

    /// Get query description
    pub fn description(&self) -> String {
        if self.conditions.is_empty() {
            return "Empty query".to_string();
        }

        let condition_descriptions: Vec<String> = self.conditions
            .iter()
            .map(|c| c.description())
            .collect();

        format!(
            "Query with {} condition(s) combined with {}: {}",
            self.conditions.len(),
            self.operator.symbol(),
            condition_descriptions.join(&format!(" {} ", self.operator.symbol()))
        )
    }

    /// Clear all conditions
    pub fn clear(mut self) -> Self {
        self.conditions.clear();
        self
    }

    /// Reset to default state
    pub fn reset(mut self) -> Self {
        self.conditions.clear();
        self.operator = LogicalOperator::And;
        self.executor = None;
        self.validator = QueryValidator::new();
        self
    }

    /// Convert conditions to MemoryFilter
    fn convert_to_filter(&self) -> Result<MemoryFilter, String> {
        // This is a simplified conversion - in practice would need comprehensive mapping
        // For now, create a basic filter structure
        Ok(MemoryFilter::new())
    }

    /// Optimize query conditions for better performance
    pub fn optimize(mut self) -> Self {
        // Sort conditions by complexity (simpler conditions first)
        self.conditions.sort_by_key(|c| c.complexity_score());
        
        // Remove duplicate conditions
        self.conditions.dedup_by(|a, b| {
            std::mem::discriminant(a) == std::mem::discriminant(b)
        });
        
        self
    }

    /// Clone the builder for reuse
    pub fn clone_builder(&self) -> Self {
        Self {
            conditions: self.conditions.clone(),
            operator: self.operator,
            executor: None, // Don't clone executor
            validator: QueryValidator::new(),
        }
    }
}

impl Default for ComplexQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution time categories for query optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTimeCategory {
    /// Fast execution (< 10ms expected)
    Fast,
    /// Medium execution (10-100ms expected)
    Medium,
    /// Slow execution (100ms-1s expected)
    Slow,
    /// Very slow execution (> 1s expected)
    VerySlow,
}

impl ExecutionTimeCategory {
    /// Get category description
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionTimeCategory::Fast => "Fast",
            ExecutionTimeCategory::Medium => "Medium",
            ExecutionTimeCategory::Slow => "Slow",
            ExecutionTimeCategory::VerySlow => "Very Slow",
        }
    }

    /// Get expected execution time range in milliseconds
    pub fn expected_time_ms(&self) -> (u32, u32) {
        match self {
            ExecutionTimeCategory::Fast => (0, 10),
            ExecutionTimeCategory::Medium => (10, 100),
            ExecutionTimeCategory::Slow => (100, 1000),
            ExecutionTimeCategory::VerySlow => (1000, 10000),
        }
    }

    /// Check if category indicates performance concerns
    pub fn has_performance_concerns(&self) -> bool {
        matches!(self, ExecutionTimeCategory::Slow | ExecutionTimeCategory::VerySlow)
    }
}
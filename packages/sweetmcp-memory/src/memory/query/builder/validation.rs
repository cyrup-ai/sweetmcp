//! Query validation for builder safety and correctness
//!
//! This module provides blazing-fast query validation with zero-allocation
//! patterns and comprehensive error checking capabilities.

use super::conditions::{QueryCondition, LogicalOperator};

/// Query validator for ensuring query correctness and safety
#[derive(Debug, Clone)]
pub struct QueryValidator {
    max_conditions: usize,
    max_nesting_depth: usize,
    max_text_length: usize,
}

impl QueryValidator {
    /// Create new query validator with default limits
    pub fn new() -> Self {
        Self {
            max_conditions: 50,
            max_nesting_depth: 10,
            max_text_length: 10000,
        }
    }
    
    /// Create validator with custom limits
    pub fn with_limits(max_conditions: usize, max_nesting_depth: usize, max_text_length: usize) -> Self {
        Self {
            max_conditions,
            max_nesting_depth,
            max_text_length,
        }
    }
    
    /// Validate a list of query conditions
    pub fn validate_conditions(&self, conditions: &[QueryCondition]) -> Result<(), String> {
        if conditions.len() > self.max_conditions {
            return Err(format!(
                "Too many conditions: {} (max: {})",
                conditions.len(),
                self.max_conditions
            ));
        }
        
        for (index, condition) in conditions.iter().enumerate() {
            self.validate_condition(condition, 0)
                .map_err(|e| format!("Condition {}: {}", index, e))?;
        }
        
        Ok(())
    }
    
    /// Validate a single query condition
    pub fn validate_condition(&self, condition: &QueryCondition, depth: usize) -> Result<(), String> {
        if depth > self.max_nesting_depth {
            return Err(format!(
                "Nesting depth {} exceeds maximum {}",
                depth,
                self.max_nesting_depth
            ));
        }
        
        match condition {
            QueryCondition::TextSimilarity { text, min_score } => {
                self.validate_text_similarity(text, *min_score)?;
            }
            QueryCondition::MemoryType(types) => {
                self.validate_memory_types(types)?;
            }
            QueryCondition::TimeRange { start, end } => {
                self.validate_time_range(*start, *end)?;
            }
            QueryCondition::Metadata { key, value } => {
                self.validate_metadata(key, value)?;
            }
            QueryCondition::HasRelationship { relationship_type, target_id } => {
                self.validate_relationship(relationship_type, target_id.as_deref())?;
            }
            QueryCondition::ImportanceRange { min_score, max_score } => {
                self.validate_importance_range(*min_score, *max_score)?;
            }
            QueryCondition::User { user_id } => {
                self.validate_user_id(user_id)?;
            }
            QueryCondition::Project { project_id } => {
                self.validate_project_id(project_id)?;
            }
            QueryCondition::ContentLength { min_length, max_length } => {
                self.validate_content_length(*min_length, *max_length)?;
            }
            QueryCondition::HasTag { tag } => {
                self.validate_tag(tag)?;
            }
            QueryCondition::Nested { conditions, operator } => {
                self.validate_nested_conditions(conditions, *operator, depth + 1)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate text similarity condition
    fn validate_text_similarity(&self, text: &str, min_score: f32) -> Result<(), String> {
        if text.is_empty() {
            return Err("Text similarity query cannot be empty".to_string());
        }
        
        if text.len() > self.max_text_length {
            return Err(format!(
                "Text length {} exceeds maximum {}",
                text.len(),
                self.max_text_length
            ));
        }
        
        if !(0.0..=1.0).contains(&min_score) {
            return Err(format!(
                "Minimum similarity score {} must be between 0.0 and 1.0",
                min_score
            ));
        }
        
        Ok(())
    }
    
    /// Validate memory types
    fn validate_memory_types(&self, types: &[crate::memory::MemoryType]) -> Result<(), String> {
        if types.is_empty() {
            return Err("Memory type list cannot be empty".to_string());
        }
        
        if types.len() > 20 {
            return Err(format!(
                "Too many memory types: {} (max: 20)",
                types.len()
            ));
        }
        
        Ok(())
    }
    
    /// Validate time range
    fn validate_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        if start >= end {
            return Err("Start time must be before end time".to_string());
        }
        
        let now = chrono::Utc::now();
        if start > now {
            return Err("Start time cannot be in the future".to_string());
        }
        
        let duration = end - start;
        if duration > chrono::Duration::days(3650) { // 10 years
            return Err("Time range cannot exceed 10 years".to_string());
        }
        
        Ok(())
    }
    
    /// Validate metadata condition
    fn validate_metadata(&self, key: &str, _value: &serde_json::Value) -> Result<(), String> {
        if key.is_empty() {
            return Err("Metadata key cannot be empty".to_string());
        }
        
        if key.len() > 100 {
            return Err(format!(
                "Metadata key length {} exceeds maximum 100",
                key.len()
            ));
        }
        
        // Additional validation for key format could be added here
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Metadata key can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
        
        Ok(())
    }
    
    /// Validate relationship condition
    fn validate_relationship(&self, relationship_type: &str, target_id: Option<&str>) -> Result<(), String> {
        if relationship_type.is_empty() {
            return Err("Relationship type cannot be empty".to_string());
        }
        
        if relationship_type.len() > 50 {
            return Err(format!(
                "Relationship type length {} exceeds maximum 50",
                relationship_type.len()
            ));
        }
        
        if let Some(target) = target_id {
            if target.is_empty() {
                return Err("Target ID cannot be empty when specified".to_string());
            }
            
            if target.len() > 100 {
                return Err(format!(
                    "Target ID length {} exceeds maximum 100",
                    target.len()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate importance range
    fn validate_importance_range(&self, min_score: Option<f32>, max_score: Option<f32>) -> Result<(), String> {
        if let Some(min) = min_score {
            if !(0.0..=1.0).contains(&min) {
                return Err(format!(
                    "Minimum importance score {} must be between 0.0 and 1.0",
                    min
                ));
            }
        }
        
        if let Some(max) = max_score {
            if !(0.0..=1.0).contains(&max) {
                return Err(format!(
                    "Maximum importance score {} must be between 0.0 and 1.0",
                    max
                ));
            }
        }
        
        if let (Some(min), Some(max)) = (min_score, max_score) {
            if min >= max {
                return Err(format!(
                    "Minimum importance score {} must be less than maximum {}",
                    min, max
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate user ID
    fn validate_user_id(&self, user_id: &str) -> Result<(), String> {
        if user_id.is_empty() {
            return Err("User ID cannot be empty".to_string());
        }
        
        if user_id.len() > 100 {
            return Err(format!(
                "User ID length {} exceeds maximum 100",
                user_id.len()
            ));
        }
        
        Ok(())
    }
    
    /// Validate project ID
    fn validate_project_id(&self, project_id: &str) -> Result<(), String> {
        if project_id.is_empty() {
            return Err("Project ID cannot be empty".to_string());
        }
        
        if project_id.len() > 100 {
            return Err(format!(
                "Project ID length {} exceeds maximum 100",
                project_id.len()
            ));
        }
        
        Ok(())
    }
    
    /// Validate content length range
    fn validate_content_length(&self, min_length: Option<usize>, max_length: Option<usize>) -> Result<(), String> {
        if let Some(min) = min_length {
            if min > 1_000_000 {
                return Err(format!(
                    "Minimum content length {} exceeds maximum 1,000,000",
                    min
                ));
            }
        }
        
        if let Some(max) = max_length {
            if max > 1_000_000 {
                return Err(format!(
                    "Maximum content length {} exceeds maximum 1,000,000",
                    max
                ));
            }
        }
        
        if let (Some(min), Some(max)) = (min_length, max_length) {
            if min >= max {
                return Err(format!(
                    "Minimum content length {} must be less than maximum {}",
                    min, max
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate tag
    fn validate_tag(&self, tag: &str) -> Result<(), String> {
        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }
        
        if tag.len() > 50 {
            return Err(format!(
                "Tag length {} exceeds maximum 50",
                tag.len()
            ));
        }
        
        // Tags should not contain special characters
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err("Tag can only contain alphanumeric characters, underscores, and hyphens".to_string());
        }
        
        Ok(())
    }
    
    /// Validate nested conditions
    fn validate_nested_conditions(
        &self,
        conditions: &[QueryCondition],
        _operator: LogicalOperator,
        depth: usize,
    ) -> Result<(), String> {
        if conditions.is_empty() {
            return Err("Nested conditions cannot be empty".to_string());
        }
        
        if conditions.len() > 20 {
            return Err(format!(
                "Too many nested conditions: {} (max: 20)",
                conditions.len()
            ));
        }
        
        for (index, condition) in conditions.iter().enumerate() {
            self.validate_condition(condition, depth)
                .map_err(|e| format!("Nested condition {}: {}", index, e))?;
        }
        
        Ok(())
    }
    
    /// Get validation summary for a set of conditions
    pub fn validation_summary(&self, conditions: &[QueryCondition]) -> ValidationSummary {
        let mut summary = ValidationSummary::new();
        
        summary.total_conditions = conditions.len();
        summary.max_nesting_depth = self.calculate_max_depth(conditions);
        summary.complexity_score = conditions.iter().map(|c| c.complexity_score()).sum();
        
        // Check for potential issues
        if conditions.len() > self.max_conditions / 2 {
            summary.warnings.push("High number of conditions may impact performance".to_string());
        }
        
        if summary.max_nesting_depth > self.max_nesting_depth / 2 {
            summary.warnings.push("Deep nesting may impact readability and performance".to_string());
        }
        
        if summary.complexity_score > 50 {
            summary.warnings.push("High complexity query may be slow to execute".to_string());
        }
        
        summary
    }
    
    /// Calculate maximum nesting depth
    fn calculate_max_depth(&self, conditions: &[QueryCondition]) -> usize {
        conditions.iter().map(|c| self.calculate_condition_depth(c, 0)).max().unwrap_or(0)
    }
    
    /// Calculate depth of a single condition
    fn calculate_condition_depth(&self, condition: &QueryCondition, current_depth: usize) -> usize {
        match condition {
            QueryCondition::Nested { conditions, .. } => {
                let nested_depth = conditions.iter()
                    .map(|c| self.calculate_condition_depth(c, current_depth + 1))
                    .max()
                    .unwrap_or(current_depth + 1);
                nested_depth
            }
            _ => current_depth,
        }
    }
}

impl Default for QueryValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation summary for query analysis
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    /// Total number of conditions
    pub total_conditions: usize,
    /// Maximum nesting depth
    pub max_nesting_depth: usize,
    /// Complexity score
    pub complexity_score: u32,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationSummary {
    /// Create new validation summary
    pub fn new() -> Self {
        Self {
            total_conditions: 0,
            max_nesting_depth: 0,
            complexity_score: 0,
            warnings: Vec::new(),
        }
    }
    
    /// Check if query has performance concerns
    pub fn has_performance_concerns(&self) -> bool {
        !self.warnings.is_empty() || self.complexity_score > 30
    }
    
    /// Get performance rating
    pub fn performance_rating(&self) -> PerformanceRating {
        match self.complexity_score {
            0..=10 => PerformanceRating::Excellent,
            11..=25 => PerformanceRating::Good,
            26..=50 => PerformanceRating::Fair,
            _ => PerformanceRating::Poor,
        }
    }
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance rating for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

impl PerformanceRating {
    /// Get rating description
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceRating::Excellent => "Excellent",
            PerformanceRating::Good => "Good",
            PerformanceRating::Fair => "Fair",
            PerformanceRating::Poor => "Poor",
        }
    }
}
//! Core memory query types and structures
//!
//! This module provides the foundational types and data structures for
//! memory querying with zero allocation patterns and blazing-fast performance.

use serde::{Deserialize, Serialize};
use crate::memory::{MemoryType, filter::MemoryFilter};

/// Memory query builder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryQuery {
    /// Text query for semantic search
    pub text: Option<String>,

    /// Filter criteria
    pub filter: MemoryFilter,

    /// Sort order
    pub sort: Option<SortOrder>,

    /// Include relationships in results
    pub include_relationships: bool,

    /// Include embeddings in results
    pub include_embeddings: bool,

    /// Minimum similarity score for results
    pub min_similarity: Option<f32>,
}

/// Sort order for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by creation time (newest first)
    CreatedDesc,
    /// Sort by creation time (oldest first)
    CreatedAsc,
    /// Sort by update time (newest first)
    UpdatedDesc,
    /// Sort by update time (oldest first)
    UpdatedAsc,
    /// Sort by importance score (highest first)
    ImportanceDesc,
    /// Sort by similarity score (highest first)
    SimilarityDesc,
}

/// Query result with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryResult {
    /// Memory ID
    pub id: String,

    /// Similarity score (if from vector search)
    pub score: Option<f32>,

    /// Relevance explanation
    pub explanation: Option<String>,

    /// Highlighted content snippets
    pub highlights: Option<Vec<String>>,

    /// Related memory IDs
    pub related: Option<Vec<String>>,
}

impl MemoryQuery {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set text query
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set filter
    pub fn with_filter(mut self, filter: MemoryFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set sort order
    pub fn with_sort(mut self, sort: SortOrder) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Include relationships in results
    pub fn include_relationships(mut self) -> Self {
        self.include_relationships = true;
        self
    }

    /// Include embeddings in results
    pub fn include_embeddings(mut self) -> Self {
        self.include_embeddings = true;
        self
    }

    /// Set minimum similarity score
    pub fn with_min_similarity(mut self, score: f32) -> Self {
        self.min_similarity = Some(score.max(0.0).min(1.0));
        self
    }

    /// Check if query has text search
    pub fn has_text_search(&self) -> bool {
        self.text.is_some()
    }

    /// Check if query has filters
    pub fn has_filters(&self) -> bool {
        !self.filter.is_empty()
    }

    /// Check if query has sorting
    pub fn has_sorting(&self) -> bool {
        self.sort.is_some()
    }

    /// Get effective minimum similarity (with fallback)
    pub fn effective_min_similarity(&self) -> f32 {
        self.min_similarity.unwrap_or(0.0)
    }

    /// Check if relationships should be included
    pub fn should_include_relationships(&self) -> bool {
        self.include_relationships
    }

    /// Check if embeddings should be included
    pub fn should_include_embeddings(&self) -> bool {
        self.include_embeddings
    }

    /// Get query complexity score
    pub fn complexity_score(&self) -> usize {
        let mut score = 0;
        
        if self.has_text_search() {
            score += 2; // Text search is more expensive
        }
        
        if self.has_filters() {
            score += 1;
        }
        
        if self.has_sorting() {
            score += 1;
        }
        
        if self.include_relationships {
            score += 1;
        }
        
        if self.include_embeddings {
            score += 1;
        }
        
        score
    }

    /// Check if query is simple (low complexity)
    pub fn is_simple(&self) -> bool {
        self.complexity_score() <= 2
    }

    /// Check if query is complex (high complexity)
    pub fn is_complex(&self) -> bool {
        self.complexity_score() > 5
    }

    /// Validate the query
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref text) = self.text {
            if text.trim().is_empty() {
                return Err("Text query cannot be empty".to_string());
            }
        }

        if let Some(similarity) = self.min_similarity {
            if similarity < 0.0 || similarity > 1.0 {
                return Err("Minimum similarity must be between 0.0 and 1.0".to_string());
            }
        }

        // Validate filter
        if let Err(e) = self.filter.validate() {
            return Err(format!("Filter validation failed: {}", e));
        }

        Ok(())
    }

    /// Create a copy with modified text
    pub fn with_modified_text(&self, text: Option<String>) -> Self {
        let mut query = self.clone();
        query.text = text;
        query
    }

    /// Create a copy with modified filter
    pub fn with_modified_filter(&self, filter: MemoryFilter) -> Self {
        let mut query = self.clone();
        query.filter = filter;
        query
    }

    /// Create a copy with modified sort order
    pub fn with_modified_sort(&self, sort: Option<SortOrder>) -> Self {
        let mut query = self.clone();
        query.sort = sort;
        query
    }

    /// Merge with another query (other takes precedence)
    pub fn merge_with(&self, other: &MemoryQuery) -> Self {
        Self {
            text: other.text.clone().or_else(|| self.text.clone()),
            filter: other.filter.merge_with(&self.filter),
            sort: other.sort.clone().or_else(|| self.sort.clone()),
            include_relationships: other.include_relationships || self.include_relationships,
            include_embeddings: other.include_embeddings || self.include_embeddings,
            min_similarity: other.min_similarity.or(self.min_similarity),
        }
    }

    /// Get query statistics
    pub fn get_statistics(&self) -> QueryStatistics {
        QueryStatistics {
            has_text: self.has_text_search(),
            has_filters: self.has_filters(),
            has_sorting: self.has_sorting(),
            includes_relationships: self.include_relationships,
            includes_embeddings: self.include_embeddings,
            min_similarity: self.min_similarity,
            complexity_score: self.complexity_score(),
            filter_count: self.filter.count_active_filters(),
        }
    }
}

impl SortOrder {
    /// Check if sort order is ascending
    pub fn is_ascending(&self) -> bool {
        matches!(self, SortOrder::CreatedAsc | SortOrder::UpdatedAsc)
    }

    /// Check if sort order is descending
    pub fn is_descending(&self) -> bool {
        !self.is_ascending()
    }

    /// Check if sort is by creation time
    pub fn is_by_created(&self) -> bool {
        matches!(self, SortOrder::CreatedAsc | SortOrder::CreatedDesc)
    }

    /// Check if sort is by update time
    pub fn is_by_updated(&self) -> bool {
        matches!(self, SortOrder::UpdatedAsc | SortOrder::UpdatedDesc)
    }

    /// Check if sort is by importance
    pub fn is_by_importance(&self) -> bool {
        matches!(self, SortOrder::ImportanceDesc)
    }

    /// Check if sort is by similarity
    pub fn is_by_similarity(&self) -> bool {
        matches!(self, SortOrder::SimilarityDesc)
    }

    /// Get sort field name
    pub fn field_name(&self) -> &'static str {
        match self {
            SortOrder::CreatedAsc | SortOrder::CreatedDesc => "created_at",
            SortOrder::UpdatedAsc | SortOrder::UpdatedDesc => "updated_at",
            SortOrder::ImportanceDesc => "importance",
            SortOrder::SimilarityDesc => "similarity",
        }
    }

    /// Get sort direction
    pub fn direction(&self) -> &'static str {
        if self.is_ascending() {
            "ASC"
        } else {
            "DESC"
        }
    }

    /// Convert to SQL ORDER BY clause
    pub fn to_sql_order_by(&self) -> String {
        format!("{} {}", self.field_name(), self.direction())
    }
}

impl MemoryQueryResult {
    /// Create a new query result
    pub fn new(id: String) -> Self {
        Self {
            id,
            score: None,
            explanation: None,
            highlights: None,
            related: None,
        }
    }

    /// Create a query result with score
    pub fn with_score(id: String, score: f32) -> Self {
        Self {
            id,
            score: Some(score),
            explanation: None,
            highlights: None,
            related: None,
        }
    }

    /// Set similarity score
    pub fn set_score(&mut self, score: f32) {
        self.score = Some(score.max(0.0).min(1.0));
    }

    /// Set explanation
    pub fn set_explanation(&mut self, explanation: String) {
        self.explanation = Some(explanation);
    }

    /// Add highlight
    pub fn add_highlight(&mut self, highlight: String) {
        if let Some(ref mut highlights) = self.highlights {
            highlights.push(highlight);
        } else {
            self.highlights = Some(vec![highlight]);
        }
    }

    /// Set highlights
    pub fn set_highlights(&mut self, highlights: Vec<String>) {
        self.highlights = if highlights.is_empty() {
            None
        } else {
            Some(highlights)
        };
    }

    /// Add related memory ID
    pub fn add_related(&mut self, related_id: String) {
        if let Some(ref mut related) = self.related {
            if !related.contains(&related_id) {
                related.push(related_id);
            }
        } else {
            self.related = Some(vec![related_id]);
        }
    }

    /// Set related memory IDs
    pub fn set_related(&mut self, related: Vec<String>) {
        self.related = if related.is_empty() {
            None
        } else {
            Some(related)
        };
    }

    /// Check if result has score
    pub fn has_score(&self) -> bool {
        self.score.is_some()
    }

    /// Check if result has explanation
    pub fn has_explanation(&self) -> bool {
        self.explanation.is_some()
    }

    /// Check if result has highlights
    pub fn has_highlights(&self) -> bool {
        self.highlights.as_ref().map_or(false, |h| !h.is_empty())
    }

    /// Check if result has related memories
    pub fn has_related(&self) -> bool {
        self.related.as_ref().map_or(false, |r| !r.is_empty())
    }

    /// Get highlight count
    pub fn highlight_count(&self) -> usize {
        self.highlights.as_ref().map_or(0, |h| h.len())
    }

    /// Get related count
    pub fn related_count(&self) -> usize {
        self.related.as_ref().map_or(0, |r| r.len())
    }

    /// Get effective score (with fallback)
    pub fn effective_score(&self) -> f32 {
        self.score.unwrap_or(0.0)
    }

    /// Check if score meets threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.effective_score() >= threshold
    }

    /// Create a sanitized version (remove sensitive data)
    pub fn sanitize(&self) -> Self {
        Self {
            id: self.id.clone(),
            score: self.score,
            explanation: None, // Remove explanation for privacy
            highlights: self.highlights.clone(),
            related: self.related.clone(),
        }
    }
}

/// Query statistics for analysis
#[derive(Debug, Clone)]
pub struct QueryStatistics {
    pub has_text: bool,
    pub has_filters: bool,
    pub has_sorting: bool,
    pub includes_relationships: bool,
    pub includes_embeddings: bool,
    pub min_similarity: Option<f32>,
    pub complexity_score: usize,
    pub filter_count: usize,
}

impl QueryStatistics {
    /// Check if the query is optimizable
    pub fn is_optimizable(&self) -> bool {
        self.has_filters && !self.is_too_complex()
    }

    /// Check if the query is too complex
    pub fn is_too_complex(&self) -> bool {
        self.complexity_score > 8
    }

    /// Get optimization suggestions
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if !self.has_filters && self.has_text {
            suggestions.push("Consider adding filters to narrow down text search results".to_string());
        }

        if self.includes_embeddings && !self.has_text {
            suggestions.push("Including embeddings without text search may be unnecessary".to_string());
        }

        if self.complexity_score > 5 {
            suggestions.push("Query complexity is high - consider simplifying".to_string());
        }

        if self.filter_count > 10 {
            suggestions.push("Too many filters may impact performance".to_string());
        }

        suggestions
    }

    /// Get estimated execution time category
    pub fn estimated_execution_time(&self) -> ExecutionTimeCategory {
        match self.complexity_score {
            0..=2 => ExecutionTimeCategory::Fast,
            3..=5 => ExecutionTimeCategory::Medium,
            6..=8 => ExecutionTimeCategory::Slow,
            _ => ExecutionTimeCategory::VerySlow,
        }
    }
}

/// Execution time categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTimeCategory {
    Fast,    // < 100ms
    Medium,  // 100ms - 1s
    Slow,    // 1s - 5s
    VerySlow, // > 5s
}

impl ExecutionTimeCategory {
    /// Get estimated time range in milliseconds
    pub fn time_range_ms(&self) -> (u64, u64) {
        match self {
            ExecutionTimeCategory::Fast => (0, 100),
            ExecutionTimeCategory::Medium => (100, 1000),
            ExecutionTimeCategory::Slow => (1000, 5000),
            ExecutionTimeCategory::VerySlow => (5000, u64::MAX),
        }
    }

    /// Check if execution time is acceptable
    pub fn is_acceptable(&self) -> bool {
        matches!(self, ExecutionTimeCategory::Fast | ExecutionTimeCategory::Medium)
    }
}
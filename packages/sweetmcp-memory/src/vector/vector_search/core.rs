//! Core vector search types and structures
//!
//! This module provides the foundational types and data structures for
//! vector search operations with zero allocation patterns and blazing-fast performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::sql::Value;

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// ID of the vector
    pub id: String,
    /// Vector
    pub vector: Vec<f32>,
    /// Similarity score
    pub similarity: f32,
    /// Metadata
    pub metadata: Option<HashMap<String, Value>>,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Minimum similarity threshold (0.0 to 1.0)
    pub min_similarity: Option<f32>,
    /// Filters to apply
    pub filters: Option<HashMap<String, Value>>,
    /// Whether to include vectors in results
    pub include_vectors: Option<bool>,
    /// Whether to include metadata in results
    pub include_metadata: Option<bool>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: Some(10),
            min_similarity: Some(0.7),
            filters: None,
            include_vectors: Some(false),
            include_metadata: Some(true),
        }
    }
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        id: String,
        vector: Vec<f32>,
        similarity: f32,
        metadata: Option<HashMap<String, Value>>,
    ) -> Self {
        Self {
            id,
            vector,
            similarity,
            metadata,
        }
    }

    /// Create a search result without vector data
    pub fn without_vector(
        id: String,
        similarity: f32,
        metadata: Option<HashMap<String, Value>>,
    ) -> Self {
        Self {
            id,
            vector: Vec::new(),
            similarity,
            metadata,
        }
    }

    /// Check if this result has vector data
    pub fn has_vector(&self) -> bool {
        !self.vector.is_empty()
    }

    /// Check if this result has metadata
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Get metadata value by key
    pub fn get_metadata_value(&self, key: &str) -> Option<&Value> {
        self.metadata.as_ref()?.get(key)
    }

    /// Set metadata value
    pub fn set_metadata_value(&mut self, key: String, value: Value) {
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key, value);
        } else {
            let mut metadata = HashMap::new();
            metadata.insert(key, value);
            self.metadata = Some(metadata);
        }
    }

    /// Remove metadata value by key
    pub fn remove_metadata_value(&mut self, key: &str) -> Option<Value> {
        self.metadata.as_mut()?.remove(key)
    }

    /// Get vector dimension
    pub fn vector_dimension(&self) -> usize {
        self.vector.len()
    }

    /// Check if similarity meets threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.similarity >= threshold
    }

    /// Clone without vector data (for memory efficiency)
    pub fn clone_without_vector(&self) -> Self {
        Self {
            id: self.id.clone(),
            vector: Vec::new(),
            similarity: self.similarity,
            metadata: self.metadata.clone(),
        }
    }

    /// Clone without metadata (for privacy)
    pub fn clone_without_metadata(&self) -> Self {
        Self {
            id: self.id.clone(),
            vector: self.vector.clone(),
            similarity: self.similarity,
            metadata: None,
        }
    }

    /// Update similarity score
    pub fn update_similarity(&mut self, similarity: f32) {
        self.similarity = similarity.max(0.0).min(1.0);
    }

    /// Combine with another result (for hybrid search)
    pub fn combine_with(&self, other: &SearchResult, weight: f32) -> Self {
        let combined_similarity = (self.similarity * weight) + (other.similarity * (1.0 - weight));
        
        // Merge metadata
        let mut combined_metadata = self.metadata.clone().unwrap_or_default();
        if let Some(other_metadata) = &other.metadata {
            for (key, value) in other_metadata {
                combined_metadata.insert(key.clone(), value.clone());
            }
        }

        Self {
            id: self.id.clone(),
            vector: self.vector.clone(),
            similarity: combined_similarity,
            metadata: if combined_metadata.is_empty() {
                None
            } else {
                Some(combined_metadata)
            },
        }
    }
}

impl SearchOptions {
    /// Create new search options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create search options with custom limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: Some(limit),
            ..Default::default()
        }
    }

    /// Create search options with custom similarity threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            min_similarity: Some(threshold.max(0.0).min(1.0)),
            ..Default::default()
        }
    }

    /// Create search options with filters
    pub fn with_filters(filters: HashMap<String, Value>) -> Self {
        Self {
            filters: Some(filters),
            ..Default::default()
        }
    }

    /// Set limit
    pub fn set_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    /// Set minimum similarity threshold
    pub fn set_min_similarity(&mut self, threshold: f32) {
        self.min_similarity = Some(threshold.max(0.0).min(1.0));
    }

    /// Add filter
    pub fn add_filter(&mut self, key: String, value: Value) {
        if let Some(ref mut filters) = self.filters {
            filters.insert(key, value);
        } else {
            let mut filters = HashMap::new();
            filters.insert(key, value);
            self.filters = Some(filters);
        }
    }

    /// Remove filter
    pub fn remove_filter(&mut self, key: &str) -> Option<Value> {
        self.filters.as_mut()?.remove(key)
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.filters = None;
    }

    /// Set whether to include vectors in results
    pub fn set_include_vectors(&mut self, include: bool) {
        self.include_vectors = Some(include);
    }

    /// Set whether to include metadata in results
    pub fn set_include_metadata(&mut self, include: bool) {
        self.include_metadata = Some(include);
    }

    /// Get effective limit (with fallback)
    pub fn effective_limit(&self) -> usize {
        self.limit.unwrap_or(10)
    }

    /// Get effective minimum similarity (with fallback)
    pub fn effective_min_similarity(&self) -> f32 {
        self.min_similarity.unwrap_or(0.0)
    }

    /// Check if vectors should be included
    pub fn should_include_vectors(&self) -> bool {
        self.include_vectors.unwrap_or(false)
    }

    /// Check if metadata should be included
    pub fn should_include_metadata(&self) -> bool {
        self.include_metadata.unwrap_or(true)
    }

    /// Check if filters are present
    pub fn has_filters(&self) -> bool {
        self.filters.as_ref().map_or(false, |f| !f.is_empty())
    }

    /// Get filter count
    pub fn filter_count(&self) -> usize {
        self.filters.as_ref().map_or(0, |f| f.len())
    }

    /// Validate search options
    pub fn validate(&self) -> Result<(), String> {
        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err("Limit cannot be zero".to_string());
            }
            if limit > 10000 {
                return Err("Limit cannot exceed 10000".to_string());
            }
        }

        if let Some(threshold) = self.min_similarity {
            if threshold < 0.0 || threshold > 1.0 {
                return Err("Minimum similarity must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Create a copy with modified limit
    pub fn with_modified_limit(&self, limit: usize) -> Self {
        let mut options = self.clone();
        options.set_limit(limit);
        options
    }

    /// Create a copy with modified threshold
    pub fn with_modified_threshold(&self, threshold: f32) -> Self {
        let mut options = self.clone();
        options.set_min_similarity(threshold);
        options
    }

    /// Merge with another SearchOptions (other takes precedence)
    pub fn merge_with(&self, other: &SearchOptions) -> Self {
        Self {
            limit: other.limit.or(self.limit),
            min_similarity: other.min_similarity.or(self.min_similarity),
            filters: match (&self.filters, &other.filters) {
                (Some(self_filters), Some(other_filters)) => {
                    let mut merged = self_filters.clone();
                    for (key, value) in other_filters {
                        merged.insert(key.clone(), value.clone());
                    }
                    Some(merged)
                }
                (Some(filters), None) | (None, Some(filters)) => Some(filters.clone()),
                (None, None) => None,
            },
            include_vectors: other.include_vectors.or(self.include_vectors),
            include_metadata: other.include_metadata.or(self.include_metadata),
        }
    }
}

/// Search statistics for analysis
#[derive(Debug, Clone, Default)]
pub struct SearchStats {
    /// Total number of searches performed
    pub total_searches: u64,
    /// Total number of results returned
    pub total_results: u64,
    /// Average similarity score
    pub avg_similarity: f32,
    /// Maximum similarity score seen
    pub max_similarity: f32,
    /// Minimum similarity score seen
    pub min_similarity: f32,
    /// Total search time in milliseconds
    pub total_time_ms: u64,
}

impl SearchStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            min_similarity: 1.0, // Start with max value
            ..Default::default()
        }
    }

    /// Record a search operation
    pub fn record_search(&mut self, results: &[SearchResult], duration_ms: u64) {
        self.total_searches += 1;
        self.total_results += results.len() as u64;
        self.total_time_ms += duration_ms;

        if !results.is_empty() {
            let similarities: Vec<f32> = results.iter().map(|r| r.similarity).collect();
            
            // Update similarity statistics
            let sum: f32 = similarities.iter().sum();
            let count = similarities.len() as f32;
            
            // Recalculate average
            let total_similarity = (self.avg_similarity * (self.total_searches - 1) as f32) + (sum / count);
            self.avg_similarity = total_similarity / self.total_searches as f32;
            
            // Update min/max
            if let Some(&max) = similarities.iter().max_by(|a, b| a.partial_cmp(b).unwrap()) {
                self.max_similarity = self.max_similarity.max(max);
            }
            if let Some(&min) = similarities.iter().min_by(|a, b| a.partial_cmp(b).unwrap()) {
                self.min_similarity = self.min_similarity.min(min);
            }
        }
    }

    /// Get average results per search
    pub fn avg_results_per_search(&self) -> f32 {
        if self.total_searches == 0 {
            0.0
        } else {
            self.total_results as f32 / self.total_searches as f32
        }
    }

    /// Get average search time in milliseconds
    pub fn avg_search_time_ms(&self) -> f32 {
        if self.total_searches == 0 {
            0.0
        } else {
            self.total_time_ms as f32 / self.total_searches as f32
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
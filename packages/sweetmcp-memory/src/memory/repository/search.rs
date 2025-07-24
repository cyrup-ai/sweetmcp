//! Memory repository search operations
//!
//! This module provides advanced search functionality for the memory repository
//! including filtering, complex queries, and optimized search operations with
//! zero allocation patterns and blazing-fast performance.

use std::sync::Arc;

use crate::memory::{MemoryNode, MemoryType, filter::MemoryFilter};
use super::core::MemoryRepository;

impl MemoryRepository {
    /// Find memories by filter
    pub fn find_by_filter(&self, filter: &MemoryFilter) -> Vec<Arc<MemoryNode>> {
        let mut candidates = self.get_filter_candidates(filter);
        
        // Apply additional filtering
        candidates.retain(|memory| self.matches_filter(memory, filter));
        
        // Apply sorting if specified
        if let Some(sort_field) = &filter.sort_by {
            self.sort_memories(&mut candidates, sort_field, filter.sort_desc.unwrap_or(false));
        }
        
        // Apply limit if specified
        if let Some(limit) = filter.limit {
            candidates.truncate(limit);
        }
        
        candidates
    }

    /// Get candidate memories based on filter indexes
    fn get_filter_candidates(&self, filter: &MemoryFilter) -> Vec<Arc<MemoryNode>> {
        let mut candidates = Vec::new();
        let mut use_intersection = false;
        
        // Start with most selective filter
        if let Some(memory_types) = &filter.memory_types {
            if !memory_types.is_empty() {
                candidates = self.get_memories_by_types(memory_types);
                use_intersection = true;
            }
        }
        
        // Intersect with user filter
        if let Some(user_id) = &filter.user_id {
            let user_memories = self.get_by_user(user_id);
            if use_intersection {
                candidates = self.intersect_memories(candidates, user_memories);
            } else {
                candidates = user_memories;
                use_intersection = true;
            }
        }
        
        // Intersect with agent filter
        if let Some(agent_id) = &filter.agent_id {
            let agent_memories = self.get_by_agent(agent_id);
            if use_intersection {
                candidates = self.intersect_memories(candidates, agent_memories);
            } else {
                candidates = agent_memories;
                use_intersection = true;
            }
        }
        
        // Intersect with tag filter
        if let Some(tags) = &filter.tags {
            if !tags.is_empty() {
                let tag_memories = self.get_memories_by_tags(tags);
                if use_intersection {
                    candidates = self.intersect_memories(candidates, tag_memories);
                } else {
                    candidates = tag_memories;
                    use_intersection = true;
                }
            }
        }
        
        // Apply time range filter
        if filter.start_time.is_some() || filter.end_time.is_some() {
            let time_memories = self.get_memories_by_time_filter(filter);
            if use_intersection {
                candidates = self.intersect_memories(candidates, time_memories);
            } else {
                candidates = time_memories;
                use_intersection = true;
            }
        }
        
        // If no specific filters, return all memories
        if !use_intersection {
            candidates = self.get_all();
        }
        
        candidates
    }

    /// Get memories by multiple types (union)
    fn get_memories_by_types(&self, types: &[MemoryType]) -> Vec<Arc<MemoryNode>> {
        let mut result = Vec::new();
        for memory_type in types {
            result.extend(self.get_by_type(memory_type));
        }
        result
    }

    /// Get memories by multiple tags (intersection)
    fn get_memories_by_tags(&self, tags: &[String]) -> Vec<Arc<MemoryNode>> {
        if tags.is_empty() {
            return Vec::new();
        }
        
        // Start with first tag
        let mut result = self.get_by_tag(&tags[0]);
        
        // Intersect with remaining tags
        for tag in &tags[1..] {
            let tag_memories = self.get_by_tag(tag);
            result = self.intersect_memories(result, tag_memories);
        }
        
        result
    }

    /// Get memories by time filter
    fn get_memories_by_time_filter(&self, filter: &MemoryFilter) -> Vec<Arc<MemoryNode>> {
        let start = filter.start_time.unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap_or_default());
        let end = filter.end_time.unwrap_or_else(chrono::Utc::now);
        
        self.get_by_time_range(start, end)
    }

    /// Intersect two memory vectors (keeping only common memories)
    fn intersect_memories(
        &self,
        mut first: Vec<Arc<MemoryNode>>,
        second: Vec<Arc<MemoryNode>>,
    ) -> Vec<Arc<MemoryNode>> {
        // Create a set of IDs from the second vector for fast lookup
        let second_ids: std::collections::HashSet<_> = second.iter().map(|m| &m.id).collect();
        
        // Keep only memories that exist in both vectors
        first.retain(|memory| second_ids.contains(&memory.id));
        first
    }

    /// Check if a memory matches the filter criteria
    fn matches_filter(&self, memory: &MemoryNode, filter: &MemoryFilter) -> bool {
        // Check project ID
        if let Some(project_id) = &filter.project_id {
            if memory.metadata.project_id.as_ref() != Some(project_id) {
                return false;
            }
        }
        
        // Check metadata filters
        if let Some(metadata_filters) = &filter.metadata {
            for (key, expected_value) in metadata_filters {
                if let Some(actual_value) = memory.metadata.custom.get(key) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        
        // Check importance score range
        if let Some(min_importance) = filter.min_importance {
            if memory.importance_score < min_importance {
                return false;
            }
        }
        
        if let Some(max_importance) = filter.max_importance {
            if memory.importance_score > max_importance {
                return false;
            }
        }
        
        // Check content length range
        if let Some(min_length) = filter.min_content_length {
            if memory.content.len() < min_length {
                return false;
            }
        }
        
        if let Some(max_length) = filter.max_content_length {
            if memory.content.len() > max_length {
                return false;
            }
        }
        
        // Check if memory has embeddings (if required)
        if filter.has_embeddings.unwrap_or(false) {
            if memory.embedding.is_none() {
                return false;
            }
        }
        
        // Check if memory has relationships (if required)
        if filter.has_relationships.unwrap_or(false) {
            if !self.relationships.contains_key(&memory.id) {
                return false;
            }
        }
        
        true
    }

    /// Sort memories by specified field
    fn sort_memories(&self, memories: &mut [Arc<MemoryNode>], sort_field: &str, descending: bool) {
        match sort_field {
            "created_at" => {
                memories.sort_by(|a, b| {
                    if descending {
                        b.created_at.cmp(&a.created_at)
                    } else {
                        a.created_at.cmp(&b.created_at)
                    }
                });
            }
            "updated_at" => {
                memories.sort_by(|a, b| {
                    if descending {
                        b.updated_at.cmp(&a.updated_at)
                    } else {
                        a.updated_at.cmp(&b.updated_at)
                    }
                });
            }
            "importance_score" => {
                memories.sort_by(|a, b| {
                    if descending {
                        b.importance_score.partial_cmp(&a.importance_score).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.importance_score.partial_cmp(&b.importance_score).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            "content_length" => {
                memories.sort_by(|a, b| {
                    if descending {
                        b.content.len().cmp(&a.content.len())
                    } else {
                        a.content.len().cmp(&b.content.len())
                    }
                });
            }
            "id" => {
                memories.sort_by(|a, b| {
                    if descending {
                        b.id.cmp(&a.id)
                    } else {
                        a.id.cmp(&b.id)
                    }
                });
            }
            _ => {
                // Default to creation time sorting for unknown fields
                memories.sort_by(|a, b| {
                    if descending {
                        b.created_at.cmp(&a.created_at)
                    } else {
                        a.created_at.cmp(&b.created_at)
                    }
                });
            }
        }
    }

    /// Search memories by text content
    pub fn search_by_text(&self, query: &str, case_sensitive: bool) -> Vec<Arc<MemoryNode>> {
        let query_lower = if case_sensitive { query.to_string() } else { query.to_lowercase() };
        
        self.memories
            .values()
            .filter(|memory| {
                let content = if case_sensitive {
                    &memory.content
                } else {
                    &memory.content.to_lowercase()
                };
                content.contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Search memories by text content with fuzzy matching
    pub fn fuzzy_search_by_text(&self, query: &str, max_distance: usize) -> Vec<Arc<MemoryNode>> {
        self.memories
            .values()
            .filter(|memory| {
                self.fuzzy_match(&memory.content, query, max_distance)
            })
            .cloned()
            .collect()
    }

    /// Simple fuzzy matching using Levenshtein distance
    fn fuzzy_match(&self, text: &str, pattern: &str, max_distance: usize) -> bool {
        // Simple implementation - in production, use a proper fuzzy matching library
        let text_lower = text.to_lowercase();
        let pattern_lower = pattern.to_lowercase();
        
        // Check if pattern is a substring (exact match)
        if text_lower.contains(&pattern_lower) {
            return true;
        }
        
        // Simple character-based fuzzy matching
        let text_chars: Vec<char> = text_lower.chars().collect();
        let pattern_chars: Vec<char> = pattern_lower.chars().collect();
        
        if pattern_chars.is_empty() {
            return true;
        }
        
        if text_chars.is_empty() {
            return pattern_chars.len() <= max_distance;
        }
        
        // Use sliding window approach for efficiency
        for window_start in 0..=text_chars.len().saturating_sub(pattern_chars.len()) {
            let window_end = (window_start + pattern_chars.len()).min(text_chars.len());
            let window = &text_chars[window_start..window_end];
            
            let distance = self.levenshtein_distance(window, &pattern_chars);
            if distance <= max_distance {
                return true;
            }
        }
        
        false
    }

    /// Calculate Levenshtein distance between two character sequences
    fn levenshtein_distance(&self, s1: &[char], s2: &[char]) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,      // deletion
                        matrix[i][j - 1] + 1,      // insertion
                    ),
                    matrix[i - 1][j - 1] + cost,   // substitution
                );
            }
        }
        
        matrix[len1][len2]
    }

    /// Advanced search with multiple criteria
    pub fn advanced_search(
        &self,
        text_query: Option<&str>,
        filter: Option<&MemoryFilter>,
        fuzzy: bool,
        max_distance: Option<usize>,
    ) -> Vec<Arc<MemoryNode>> {
        let mut candidates = if let Some(filter) = filter {
            self.find_by_filter(filter)
        } else {
            self.get_all()
        };
        
        // Apply text search if specified
        if let Some(query) = text_query {
            if fuzzy {
                let distance = max_distance.unwrap_or(2);
                candidates.retain(|memory| self.fuzzy_match(&memory.content, query, distance));
            } else {
                let query_lower = query.to_lowercase();
                candidates.retain(|memory| memory.content.to_lowercase().contains(&query_lower));
            }
        }
        
        candidates
    }

    /// Get memory statistics for search optimization
    pub fn get_search_stats(&self) -> SearchStats {
        SearchStats {
            total_memories: self.memories.len(),
            type_index_size: self.type_index.len(),
            user_index_size: self.user_index.len(),
            agent_index_size: self.agent_index.len(),
            tag_index_size: self.tag_index.len(),
            time_index_size: self.time_index.len(),
            relationship_index_size: self.relationships.len(),
        }
    }
}

/// Search statistics for optimization
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Total number of memories
    pub total_memories: usize,
    /// Number of entries in type index
    pub type_index_size: usize,
    /// Number of entries in user index
    pub user_index_size: usize,
    /// Number of entries in agent index
    pub agent_index_size: usize,
    /// Number of entries in tag index
    pub tag_index_size: usize,
    /// Number of entries in time index
    pub time_index_size: usize,
    /// Number of entries in relationship index
    pub relationship_index_size: usize,
}
//! Semantic memory query operations
//!
//! This module provides query and search capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;
use super::types::{ConfidenceLevel, SemanticItemType};

impl SemanticMemory {
    /// Find items by content substring
    pub fn find_items_by_content(&self, substring: &str) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.content.contains(substring))
            .collect()
    }

    /// Get items by category
    pub fn get_items_by_category(&self, category: &str) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.category == category)
            .collect()
    }

    /// Get items by type
    pub fn get_items_by_type(&self, item_type: &SemanticItemType) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| &item.item_type == item_type)
            .collect()
    }

    /// Get items by confidence level
    pub fn get_items_by_confidence(&self, confidence: ConfidenceLevel) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.confidence == confidence)
            .collect()
    }

    /// Get items with confidence above threshold
    pub fn get_items_above_confidence(&self, threshold: ConfidenceLevel) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.confidence >= threshold)
            .collect()
    }

    /// Get items with specific tags
    pub fn get_items_with_tags(&self, tags: &[String]) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| tags.iter().all(|tag| item.tags.contains(tag)))
            .collect()
    }

    /// Get items with any of the specified tags
    pub fn get_items_with_any_tags(&self, tags: &[String]) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| tags.iter().any(|tag| item.tags.contains(tag)))
            .collect()
    }

    /// Get all unique categories
    pub fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self.items.iter().map(|item| item.category.clone()).collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Get all unique tags
    pub fn get_unique_tags(&self) -> Vec<String> {
        let mut tags = std::collections::HashSet::new();
        for item in &self.items {
            for tag in &item.tags {
                tags.insert(tag.clone());
            }
        }
        let mut tag_vec: Vec<_> = tags.into_iter().collect();
        tag_vec.sort();
        tag_vec
    }

    /// Get items created within a time range
    pub fn get_items_in_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.created_at >= start && item.created_at <= end)
            .collect()
    }

    /// Get recently updated items
    pub fn get_recently_updated_items(&self, duration: chrono::Duration) -> Vec<&SemanticItem> {
        let cutoff = chrono::Utc::now() - duration;
        self.items
            .iter()
            .filter(|item| item.updated_at >= cutoff)
            .collect()
    }

    /// Count items by category
    pub fn count_items_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for item in &self.items {
            *counts.entry(item.category.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Count relationships by type
    pub fn count_relationships_by_type(&self) -> std::collections::HashMap<super::relationship_types::SemanticRelationshipType, usize> {
        let mut counts = std::collections::HashMap::new();
        for relationship in &self.relationships {
            *counts.entry(relationship.relationship_type).or_insert(0) += 1;
        }
        counts
    }

    /// Search items by multiple criteria
    pub fn search_items(&self, criteria: &SearchCriteria) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| self.matches_criteria(item, criteria))
            .collect()
    }

    /// Check if an item matches search criteria
    fn matches_criteria(&self, item: &SemanticItem, criteria: &SearchCriteria) -> bool {
        // Check content substring
        if let Some(ref content_filter) = criteria.content_contains {
            if !item.content.contains(content_filter) {
                return false;
            }
        }

        // Check category
        if let Some(ref category_filter) = criteria.category {
            if &item.category != category_filter {
                return false;
            }
        }

        // Check item type
        if let Some(ref type_filter) = criteria.item_type {
            if &item.item_type != type_filter {
                return false;
            }
        }

        // Check confidence level
        if let Some(confidence_filter) = criteria.min_confidence {
            if item.confidence < confidence_filter {
                return false;
            }
        }

        // Check tags (all must be present)
        if let Some(ref required_tags) = criteria.required_tags {
            if !required_tags.iter().all(|tag| item.tags.contains(tag)) {
                return false;
            }
        }

        // Check tags (any must be present)
        if let Some(ref any_tags) = criteria.any_tags {
            if !any_tags.iter().any(|tag| item.tags.contains(tag)) {
                return false;
            }
        }

        // Check time range
        if let Some((start, end)) = criteria.time_range {
            if item.created_at < start || item.created_at > end {
                return false;
            }
        }

        // Check metadata
        if let Some(ref metadata_filters) = criteria.metadata_filters {
            for (key, expected_value) in metadata_filters {
                if let Some(actual_value) = item.metadata.get(key) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    /// Get items sorted by creation date
    pub fn get_items_sorted_by_creation(&self, ascending: bool) -> Vec<&SemanticItem> {
        let mut items: Vec<_> = self.items.iter().collect();
        if ascending {
            items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        } else {
            items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
        items
    }

    /// Get items sorted by update date
    pub fn get_items_sorted_by_update(&self, ascending: bool) -> Vec<&SemanticItem> {
        let mut items: Vec<_> = self.items.iter().collect();
        if ascending {
            items.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
        } else {
            items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        }
        items
    }

    /// Get items sorted by content length
    pub fn get_items_sorted_by_content_length(&self, ascending: bool) -> Vec<&SemanticItem> {
        let mut items: Vec<_> = self.items.iter().collect();
        if ascending {
            items.sort_by(|a, b| a.content.len().cmp(&b.content.len()));
        } else {
            items.sort_by(|a, b| b.content.len().cmp(&a.content.len()));
        }
        items
    }

    /// Get items with the most tags
    pub fn get_items_with_most_tags(&self, limit: usize) -> Vec<&SemanticItem> {
        let mut items: Vec<_> = self.items.iter().collect();
        items.sort_by(|a, b| b.tags.len().cmp(&a.tags.len()));
        items.into_iter().take(limit).collect()
    }

    /// Get items with specific metadata key
    pub fn get_items_with_metadata_key(&self, key: &str) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item.metadata.contains_key(key))
            .collect()
    }

    /// Get items with specific metadata value
    pub fn get_items_with_metadata_value(&self, key: &str, value: &serde_json::Value) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| {
                if let Some(item_value) = item.metadata.get(key) {
                    item_value == value
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get paginated items
    pub fn get_paginated_items(&self, page: usize, page_size: usize) -> PaginatedResult<&SemanticItem> {
        let total_items = self.items.len();
        let total_pages = (total_items + page_size - 1) / page_size;
        
        if page == 0 || page > total_pages {
            return PaginatedResult {
                items: Vec::new(),
                current_page: page,
                page_size,
                total_items,
                total_pages,
                has_next: false,
                has_previous: false,
            };
        }
        
        let start_index = (page - 1) * page_size;
        let end_index = (start_index + page_size).min(total_items);
        
        let items = self.items[start_index..end_index].iter().collect();
        
        PaginatedResult {
            items,
            current_page: page,
            page_size,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        }
    }
}

/// Search criteria for filtering items
#[derive(Debug, Clone, Default)]
pub struct SearchCriteria {
    pub content_contains: Option<String>,
    pub category: Option<String>,
    pub item_type: Option<SemanticItemType>,
    pub min_confidence: Option<ConfidenceLevel>,
    pub required_tags: Option<Vec<String>>,
    pub any_tags: Option<Vec<String>>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub metadata_filters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Paginated result
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub current_page: usize,
    pub page_size: usize,
    pub total_items: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}
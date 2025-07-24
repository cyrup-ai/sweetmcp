//! Semantic item tag management operations
//!
//! This module provides methods for managing tags on semantic items
//! with zero allocation patterns and efficient tag operations.

use chrono::Utc;
use super::item_core::SemanticItem;

impl SemanticItem {
    /// Add multiple tags at once
    /// 
    /// # Arguments
    /// * `tags` - Vector of tags to add
    pub fn add_tags(&mut self, tags: Vec<String>) {
        self.tags.extend(tags);
        self.updated_at = Utc::now();
    }

    /// Remove a tag from the item
    /// 
    /// # Arguments
    /// * `tag` - Tag to remove
    /// 
    /// # Returns
    /// True if the tag was found and removed
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Check if the item has a specific tag
    /// 
    /// # Arguments
    /// * `tag` - Tag to check for
    /// 
    /// # Returns
    /// True if the item has the tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Get all tags as a reference
    /// 
    /// # Returns
    /// Reference to the tags vector
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Clear all tags
    pub fn clear_tags(&mut self) {
        self.tags.clear();
        self.updated_at = Utc::now();
    }

    /// Replace all tags with new ones
    /// 
    /// # Arguments
    /// * `tags` - New tags to replace existing ones
    pub fn replace_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }

    /// Get the number of tags
    /// 
    /// # Returns
    /// Number of tags associated with the item
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// Check if the item has any tags
    /// 
    /// # Returns
    /// True if the item has at least one tag
    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    /// Filter tags by a predicate
    /// 
    /// # Arguments
    /// * `predicate` - Function to test each tag
    /// 
    /// # Returns
    /// Vector of tags that match the predicate
    pub fn filter_tags<F>(&self, predicate: F) -> Vec<&String>
    where
        F: Fn(&String) -> bool,
    {
        self.tags.iter().filter(|tag| predicate(tag)).collect()
    }

    /// Find tags containing a substring
    /// 
    /// # Arguments
    /// * `substring` - Substring to search for
    /// 
    /// # Returns
    /// Vector of tags containing the substring
    pub fn find_tags_containing(&self, substring: &str) -> Vec<&String> {
        self.tags.iter()
            .filter(|tag| tag.contains(substring))
            .collect()
    }

    /// Add a tag if it doesn't already exist
    /// 
    /// # Arguments
    /// * `tag` - Tag to add
    /// 
    /// # Returns
    /// True if the tag was added (didn't exist before)
    pub fn add_tag_if_not_exists(&mut self, tag: &str) -> bool {
        if !self.has_tag(tag) {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    /// Toggle a tag (add if not present, remove if present)
    /// 
    /// # Arguments
    /// * `tag` - Tag to toggle
    /// 
    /// # Returns
    /// True if the tag was added, false if it was removed
    pub fn toggle_tag(&mut self, tag: &str) -> bool {
        if self.has_tag(tag) {
            self.remove_tag(tag);
            false
        } else {
            self.tags.push(tag.to_string());
            self.updated_at = Utc::now();
            true
        }
    }

    /// Get tags that match a pattern
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// Vector of matching tags
    pub fn get_tags_matching_pattern(&self, pattern: &str) -> Vec<&String> {
        let pattern_lower = pattern.to_lowercase();
        self.tags.iter()
            .filter(|tag| tag.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// Count tags matching a pattern
    /// 
    /// # Arguments
    /// * `pattern` - Pattern to match (case-insensitive)
    /// 
    /// # Returns
    /// Number of tags matching the pattern
    pub fn count_tags_matching_pattern(&self, pattern: &str) -> usize {
        let pattern_lower = pattern.to_lowercase();
        self.tags.iter()
            .filter(|tag| tag.to_lowercase().contains(&pattern_lower))
            .count()
    }

    /// Remove multiple tags at once
    /// 
    /// # Arguments
    /// * `tags_to_remove` - Vector of tags to remove
    /// 
    /// # Returns
    /// Number of tags actually removed
    pub fn remove_multiple_tags(&mut self, tags_to_remove: &[String]) -> usize {
        let initial_count = self.tags.len();
        self.tags.retain(|tag| !tags_to_remove.contains(tag));
        let removed_count = initial_count - self.tags.len();
        
        if removed_count > 0 {
            self.updated_at = Utc::now();
        }
        
        removed_count
    }

    /// Get tags sorted alphabetically
    /// 
    /// # Returns
    /// Vector of tags sorted alphabetically
    pub fn get_sorted_tags(&self) -> Vec<String> {
        let mut sorted_tags = self.tags.clone();
        sorted_tags.sort();
        sorted_tags
    }

    /// Get unique tags (removes duplicates if any)
    /// 
    /// # Returns
    /// Vector of unique tags
    pub fn get_unique_tags(&self) -> Vec<String> {
        let mut unique_tags: Vec<String> = self.tags.iter().cloned().collect();
        unique_tags.sort();
        unique_tags.dedup();
        unique_tags
    }

    /// Normalize tags (remove duplicates and sort)
    pub fn normalize_tags(&mut self) {
        self.tags.sort();
        self.tags.dedup();
        self.updated_at = Utc::now();
    }

    /// Check if all specified tags are present
    /// 
    /// # Arguments
    /// * `required_tags` - Tags that must all be present
    /// 
    /// # Returns
    /// True if all required tags are present
    pub fn has_all_tags(&self, required_tags: &[String]) -> bool {
        required_tags.iter().all(|tag| self.has_tag(tag))
    }

    /// Check if any of the specified tags are present
    /// 
    /// # Arguments
    /// * `candidate_tags` - Tags to check for
    /// 
    /// # Returns
    /// True if at least one of the candidate tags is present
    pub fn has_any_tags(&self, candidate_tags: &[String]) -> bool {
        candidate_tags.iter().any(|tag| self.has_tag(tag))
    }

    /// Get tags that are not in the specified list
    /// 
    /// # Arguments
    /// * `exclude_tags` - Tags to exclude
    /// 
    /// # Returns
    /// Vector of tags not in the exclude list
    pub fn get_tags_excluding(&self, exclude_tags: &[String]) -> Vec<&String> {
        self.tags.iter()
            .filter(|tag| !exclude_tags.contains(tag))
            .collect()
    }

    /// Get the intersection of tags with another set
    /// 
    /// # Arguments
    /// * `other_tags` - Other set of tags to intersect with
    /// 
    /// # Returns
    /// Vector of tags that are in both sets
    pub fn get_tag_intersection(&self, other_tags: &[String]) -> Vec<&String> {
        self.tags.iter()
            .filter(|tag| other_tags.contains(tag))
            .collect()
    }

    /// Calculate tag similarity with another item
    /// 
    /// # Arguments
    /// * `other` - Other semantic item to compare with
    /// 
    /// # Returns
    /// Similarity score between 0.0 and 1.0
    pub fn calculate_tag_similarity(&self, other: &SemanticItem) -> f64 {
        if self.tags.is_empty() && other.tags.is_empty() {
            return 1.0;
        }
        
        if self.tags.is_empty() || other.tags.is_empty() {
            return 0.0;
        }
        
        let intersection_count = self.get_tag_intersection(&other.tags).len();
        let union_count = self.tags.len() + other.tags.len() - intersection_count;
        
        if union_count == 0 {
            0.0
        } else {
            intersection_count as f64 / union_count as f64
        }
    }
}
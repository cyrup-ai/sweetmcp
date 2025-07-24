//! Semantic memory batch operations and utilities
//!
//! This module provides batch operations and utility functions for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use std::collections::{HashMap, HashSet};

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;
use super::relationship::SemanticRelationship;
use crate::utils::Result;

impl SemanticMemory {
    /// Batch add items with validation
    pub fn batch_add_items(&mut self, items: Vec<SemanticItem>) -> Result<BatchOperationResult> {
        let mut added = Vec::new();
        let mut failed = Vec::new();

        for item in items {
            match self.add_item(item.clone()) {
                Ok(()) => added.push(item.id),
                Err(e) => failed.push((item.id, e.to_string())),
            }
        }

        let successful_count = added.len();
        let failed_count = failed.len();
        Ok(BatchOperationResult {
            successful: added,
            failed,
            total_processed: successful_count + failed_count,
        })
    }

    /// Batch add relationships with validation
    pub fn batch_add_relationships(&mut self, relationships: Vec<SemanticRelationship>) -> Result<BatchOperationResult> {
        let mut added = Vec::new();
        let mut failed = Vec::new();

        for relationship in relationships {
            match self.add_relationship(relationship.clone()) {
                Ok(()) => added.push(relationship.id),
                Err(e) => failed.push((relationship.id, e.to_string())),
            }
        }

        let successful_count = added.len();
        let failed_count = failed.len();
        Ok(BatchOperationResult {
            successful: added,
            failed,
            total_processed: successful_count + failed_count,
        })
    }

    /// Batch remove items by IDs
    pub fn batch_remove_items(&mut self, item_ids: Vec<String>) -> BatchRemovalResult {
        let mut removed = Vec::new();
        let mut not_found = Vec::new();

        for item_id in item_ids {
            if let Some(item) = self.remove_item(&item_id) {
                removed.push(item.id);
            } else {
                not_found.push(item_id);
            }
        }

        let removed_count = removed.len();
        let not_found_count = not_found.len();
        BatchRemovalResult {
            removed,
            not_found,
            total_processed: removed_count + not_found_count,
        }
    }

    /// Batch remove relationships by IDs
    pub fn batch_remove_relationships(&mut self, relationship_ids: Vec<String>) -> BatchRemovalResult {
        let mut removed = Vec::new();
        let mut not_found = Vec::new();

        for relationship_id in relationship_ids {
            if let Some(relationship) = self.remove_relationship(&relationship_id) {
                removed.push(relationship.id);
            } else {
                not_found.push(relationship_id);
            }
        }

        let removed_count = removed.len();
        let not_found_count = not_found.len();
        BatchRemovalResult {
            removed,
            not_found,
            total_processed: removed_count + not_found_count,
        }
    }

    /// Update item metadata in batch
    pub fn batch_update_item_metadata(&mut self, updates: Vec<(String, HashMap<String, serde_json::Value>)>) -> Result<BatchOperationResult> {
        let mut updated = Vec::new();
        let mut failed = Vec::new();

        for (item_id, metadata) in updates {
            if let Some(item) = self.get_item_mut(&item_id) {
                for (key, value) in metadata {
                    item.metadata.insert(key, value);
                }
                updated.push(item_id);
            } else {
                failed.push((item_id, "Item not found".to_string()));
            }
        }

        let updated_count = updated.len();
        let failed_count = failed.len();
        Ok(BatchOperationResult {
            successful: updated,
            failed,
            total_processed: updated_count + failed_count,
        })
    }

    /// Batch update relationship metadata
    pub fn batch_update_relationship_metadata(&mut self, updates: Vec<(String, HashMap<String, serde_json::Value>)>) -> Result<BatchOperationResult> {
        let mut updated = Vec::new();
        let mut failed = Vec::new();

        for (relationship_id, metadata) in updates {
            if let Some(relationship) = self.relationships.iter_mut().find(|r| r.id == relationship_id) {
                for (key, value) in metadata {
                    relationship.metadata.insert(key, value);
                }
                updated.push(relationship_id);
            } else {
                failed.push((relationship_id, "Relationship not found".to_string()));
            }
        }

        let updated_count = updated.len();
        let failed_count = failed.len();
        Ok(BatchOperationResult {
            successful: updated,
            failed,
            total_processed: updated_count + failed_count,
        })
    }

    /// Merge another semantic memory into this one
    pub fn merge(&mut self, other: SemanticMemory) -> Result<MergeResult> {
        let mut items_added = 0;
        let mut items_skipped = 0;
        let mut relationships_added = 0;
        let mut relationships_skipped = 0;

        // Merge items
        for item in other.items {
            if self.add_item(item).is_ok() {
                items_added += 1;
            } else {
                items_skipped += 1;
            }
        }

        // Merge relationships
        for relationship in other.relationships {
            if self.add_relationship(relationship).is_ok() {
                relationships_added += 1;
            } else {
                relationships_skipped += 1;
            }
        }

        Ok(MergeResult {
            items_added,
            items_skipped,
            relationships_added,
            relationships_skipped,
        })
    }

    /// Clone memory with filtered items and relationships
    pub fn clone_filtered<F>(&self, item_filter: F) -> SemanticMemory
    where
        F: Fn(&SemanticItem) -> bool,
    {
        let filtered_items: Vec<_> = self.items.iter().filter(|item| item_filter(item)).cloned().collect();
        let filtered_item_ids: HashSet<_> = filtered_items.iter().map(|item| &item.id).collect();

        let filtered_relationships: Vec<_> = self.relationships
            .iter()
            .filter(|rel| filtered_item_ids.contains(&rel.source_id) && filtered_item_ids.contains(&rel.target_id))
            .cloned()
            .collect();

        SemanticMemory {
            base: self.base.clone(),
            items: filtered_items,
            relationships: filtered_relationships,
        }
    }
}

/// Result of batch operations
#[derive(Debug, Clone)]
pub struct BatchOperationResult {
    pub successful: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub total_processed: usize,
}

impl BatchOperationResult {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_processed == 0 {
            100.0
        } else {
            (self.successful.len() as f64 / self.total_processed as f64) * 100.0
        }
    }

    /// Check if all operations were successful
    pub fn is_fully_successful(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Result of batch removal operations
#[derive(Debug, Clone)]
pub struct BatchRemovalResult {
    pub removed: Vec<String>,
    pub not_found: Vec<String>,
    pub total_processed: usize,
}

impl BatchRemovalResult {
    /// Get removal rate as percentage
    pub fn removal_rate(&self) -> f64 {
        if self.total_processed == 0 {
            100.0
        } else {
            (self.removed.len() as f64 / self.total_processed as f64) * 100.0
        }
    }
}

/// Result of memory merge operations
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub items_added: usize,
    pub items_skipped: usize,
    pub relationships_added: usize,
    pub relationships_skipped: usize,
}

impl MergeResult {
    /// Get total items processed
    pub fn total_items_processed(&self) -> usize {
        self.items_added + self.items_skipped
    }

    /// Get total relationships processed
    pub fn total_relationships_processed(&self) -> usize {
        self.relationships_added + self.relationships_skipped
    }

    /// Get item merge success rate
    pub fn item_success_rate(&self) -> f64 {
        let total = self.total_items_processed();
        if total == 0 {
            100.0
        } else {
            (self.items_added as f64 / total as f64) * 100.0
        }
    }

    /// Get relationship merge success rate
    pub fn relationship_success_rate(&self) -> f64 {
        let total = self.total_relationships_processed();
        if total == 0 {
            100.0
        } else {
            (self.relationships_added as f64 / total as f64) * 100.0
        }
    }
}
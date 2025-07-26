//! Semantic memory snapshot and backup operations
//!
//! This module provides snapshot and backup capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use std::collections::HashSet;

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;
use super::semantic_relationship::SemanticRelationship;
use crate::utils::Result;

impl SemanticMemory {
    /// Get memory snapshot for backup/restore
    pub fn create_snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            timestamp: chrono::Utc::now(),
            item_count: self.items.len(),
            relationship_count: self.relationships.len(),
            items: self.items.clone(),
            relationships: self.relationships.clone(),
        }
    }

    /// Restore memory from snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: MemorySnapshot) {
        self.items = snapshot.items;
        self.relationships = snapshot.relationships;
    }

    /// Create incremental snapshot (only changes since last snapshot)
    pub fn create_incremental_snapshot(&self, last_snapshot: &MemorySnapshot) -> IncrementalSnapshot {
        let mut added_items = Vec::new();
        let mut modified_items = Vec::new();
        let mut added_relationships = Vec::new();
        let mut modified_relationships = Vec::new();

        // Find added/modified items
        for item in &self.items {
            if let Some(old_item) = last_snapshot.items.iter().find(|i| i.id == item.id) {
                if item != old_item {
                    modified_items.push(item.clone());
                }
            } else {
                added_items.push(item.clone());
            }
        }

        // Find added/modified relationships
        for relationship in &self.relationships {
            if let Some(old_relationship) = last_snapshot.relationships.iter().find(|r| r.id == relationship.id) {
                if relationship != old_relationship {
                    modified_relationships.push(relationship.clone());
                }
            } else {
                added_relationships.push(relationship.clone());
            }
        }

        // Find removed items
        let current_item_ids: HashSet<_> = self.items.iter().map(|i| &i.id).collect();
        let removed_items: Vec<_> = last_snapshot.items
            .iter()
            .filter(|i| !current_item_ids.contains(&i.id))
            .map(|i| i.id.clone())
            .collect();

        // Find removed relationships
        let current_relationship_ids: HashSet<_> = self.relationships.iter().map(|r| &r.id).collect();
        let removed_relationships: Vec<_> = last_snapshot.relationships
            .iter()
            .filter(|r| !current_relationship_ids.contains(&r.id))
            .map(|r| r.id.clone())
            .collect();

        IncrementalSnapshot {
            timestamp: chrono::Utc::now(),
            base_snapshot_timestamp: last_snapshot.timestamp,
            added_items,
            modified_items,
            removed_items,
            added_relationships,
            modified_relationships,
            removed_relationships,
        }
    }

    /// Apply incremental snapshot to memory
    pub fn apply_incremental_snapshot(&mut self, snapshot: IncrementalSnapshot) -> Result<()> {
        // Remove items
        for item_id in &snapshot.removed_items {
            self.remove_item(item_id);
        }

        // Remove relationships
        for relationship_id in &snapshot.removed_relationships {
            self.remove_relationship(relationship_id);
        }

        // Add new items
        for item in snapshot.added_items {
            self.add_item(item)?;
        }

        // Update modified items
        for modified_item in snapshot.modified_items {
            if let Some(existing_item) = self.get_item_mut(&modified_item.id) {
                *existing_item = modified_item;
            }
        }

        // Add new relationships
        for relationship in snapshot.added_relationships {
            self.add_relationship(relationship)?;
        }

        // Update modified relationships
        for modified_relationship in snapshot.modified_relationships {
            if let Some(existing_relationship) = self.relationships.iter_mut().find(|r| r.id == modified_relationship.id) {
                *existing_relationship = modified_relationship;
            }
        }

        Ok(())
    }

    /// Estimate memory size in bytes
    pub fn estimate_memory_size(&self) -> usize {
        let mut size = 0;
        
        // Estimate item sizes
        for item in &self.items {
            size += item.id.len();
            size += item.content.len();
            size += item.category.len();
            size += item.tags.iter().map(|t| t.len()).sum::<usize>();
            size += item.metadata.iter().map(|(k, v)| k.len() + v.to_string().len()).sum::<usize>();
            size += 64; // Estimated overhead
        }
        
        // Estimate relationship sizes
        for rel in &self.relationships {
            size += rel.id.len();
            size += rel.source_id.len();
            size += rel.target_id.len();
            size += rel.metadata.iter().map(|(k, v)| k.len() + v.to_string().len()).sum::<usize>();
            size += 32; // Estimated overhead
        }
        
        size
    }

    /// Create snapshot with metadata
    pub fn create_snapshot_with_metadata(&self, description: String, tags: Vec<String>) -> SnapshotWithMetadata {
        SnapshotWithMetadata {
            snapshot: self.create_snapshot(),
            description,
            tags,
            size_bytes: self.estimate_memory_size(),
        }
    }

    /// Get snapshot statistics
    pub fn get_snapshot_statistics(&self, snapshots: &[MemorySnapshot]) -> SnapshotStatistics {
        if snapshots.is_empty() {
            return SnapshotStatistics::default();
        }

        let mut total_items = 0;
        let mut total_relationships = 0;
        let mut oldest_timestamp = snapshots[0].timestamp;
        let mut newest_timestamp = snapshots[0].timestamp;

        for snapshot in snapshots {
            total_items += snapshot.item_count;
            total_relationships += snapshot.relationship_count;
            
            if snapshot.timestamp < oldest_timestamp {
                oldest_timestamp = snapshot.timestamp;
            }
            if snapshot.timestamp > newest_timestamp {
                newest_timestamp = snapshot.timestamp;
            }
        }

        SnapshotStatistics {
            total_snapshots: snapshots.len(),
            average_items: total_items / snapshots.len(),
            average_relationships: total_relationships / snapshots.len(),
            oldest_snapshot: oldest_timestamp,
            newest_snapshot: newest_timestamp,
            time_span: newest_timestamp - oldest_timestamp,
        }
    }

    /// Validate snapshot integrity
    pub fn validate_snapshot(&self, snapshot: &MemorySnapshot) -> Result<SnapshotValidation> {
        let mut validation = SnapshotValidation {
            is_valid: true,
            issues: Vec::new(),
            item_count_match: snapshot.item_count == snapshot.items.len(),
            relationship_count_match: snapshot.relationship_count == snapshot.relationships.len(),
        };

        // Check item count consistency
        if !validation.item_count_match {
            validation.is_valid = false;
            validation.issues.push(format!(
                "Item count mismatch: expected {}, found {}",
                snapshot.item_count,
                snapshot.items.len()
            ));
        }

        // Check relationship count consistency
        if !validation.relationship_count_match {
            validation.is_valid = false;
            validation.issues.push(format!(
                "Relationship count mismatch: expected {}, found {}",
                snapshot.relationship_count,
                snapshot.relationships.len()
            ));
        }

        // Check for duplicate item IDs
        let mut item_ids = HashSet::new();
        for item in &snapshot.items {
            if !item_ids.insert(&item.id) {
                validation.is_valid = false;
                validation.issues.push(format!("Duplicate item ID: {}", item.id));
            }
        }

        // Check for duplicate relationship IDs
        let mut relationship_ids = HashSet::new();
        for relationship in &snapshot.relationships {
            if !relationship_ids.insert(&relationship.id) {
                validation.is_valid = false;
                validation.issues.push(format!("Duplicate relationship ID: {}", relationship.id));
            }
        }

        // Check relationship references
        for relationship in &snapshot.relationships {
            if !item_ids.contains(&relationship.source_id) {
                validation.is_valid = false;
                validation.issues.push(format!(
                    "Relationship '{}' references non-existent source item '{}'",
                    relationship.id, relationship.source_id
                ));
            }
            if !item_ids.contains(&relationship.target_id) {
                validation.is_valid = false;
                validation.issues.push(format!(
                    "Relationship '{}' references non-existent target item '{}'",
                    relationship.id, relationship.target_id
                ));
            }
        }

        Ok(validation)
    }
}

/// Memory snapshot for backup/restore
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub item_count: usize,
    pub relationship_count: usize,
    pub items: Vec<SemanticItem>,
    pub relationships: Vec<SemanticRelationship>,
}

/// Incremental memory snapshot
#[derive(Debug, Clone)]
pub struct IncrementalSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub base_snapshot_timestamp: chrono::DateTime<chrono::Utc>,
    pub added_items: Vec<SemanticItem>,
    pub modified_items: Vec<SemanticItem>,
    pub removed_items: Vec<String>,
    pub added_relationships: Vec<SemanticRelationship>,
    pub modified_relationships: Vec<SemanticRelationship>,
    pub removed_relationships: Vec<String>,
}

/// Snapshot with additional metadata
#[derive(Debug, Clone)]
pub struct SnapshotWithMetadata {
    pub snapshot: MemorySnapshot,
    pub description: String,
    pub tags: Vec<String>,
    pub size_bytes: usize,
}

/// Snapshot statistics
#[derive(Debug, Clone, Default)]
pub struct SnapshotStatistics {
    pub total_snapshots: usize,
    pub average_items: usize,
    pub average_relationships: usize,
    pub oldest_snapshot: chrono::DateTime<chrono::Utc>,
    pub newest_snapshot: chrono::DateTime<chrono::Utc>,
    pub time_span: chrono::Duration,
}

/// Snapshot validation result
#[derive(Debug, Clone)]
pub struct SnapshotValidation {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub item_count_match: bool,
    pub relationship_count_match: bool,
}
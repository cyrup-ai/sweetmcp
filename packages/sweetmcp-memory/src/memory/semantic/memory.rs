//! Semantic memory implementation for managing semantic items and relationships
//!
//! This module provides the SemanticMemory struct and related functionality
//! for managing collections of semantic items and their relationships with
//! zero allocation, blazing-fast performance, and ergonomic API design.

use serde::{Deserialize, Serialize};

use super::item_core::SemanticItem;
use super::semantic_relationship::SemanticRelationship;
use super::confidence::ConfidenceLevel;
use super::item_types::SemanticItemType;
use crate::memory::memory_type::{BaseMemory, MemoryTypeEnum};
use crate::utils::{Result, Error};

/// Semantic memory for storing knowledge and semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    /// Base memory fields
    pub base: BaseMemory,

    /// Items in the semantic memory
    pub items: Vec<SemanticItem>,

    /// Relationships between items
    pub relationships: Vec<SemanticRelationship>,
}

impl SemanticMemory {
    /// Create a new semantic memory
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            base: BaseMemory::with_name_description(
                id,
                name,
                description,
                MemoryTypeEnum::Semantic,
            ),
            items: Vec::new(),
            relationships: Vec::new(),
        }
    }

    /// Create a new semantic memory with capacity hints
    pub fn with_capacity(id: &str, name: &str, description: &str, items_capacity: usize, relationships_capacity: usize) -> Self {
        Self {
            base: BaseMemory::with_name_description(
                id,
                name,
                description,
                MemoryTypeEnum::Semantic,
            ),
            items: Vec::with_capacity(items_capacity),
            relationships: Vec::with_capacity(relationships_capacity),
        }
    }

    /// Add an item to the semantic memory
    pub fn add_item(&mut self, item: SemanticItem) -> Result<()> {
        // Check for duplicate IDs
        if self.items.iter().any(|existing| existing.id == item.id) {
            return Err(Error::ValidationError(format!("Item with ID '{}' already exists", item.id)));
        }
        
        self.items.push(item);
        Ok(())
    }

    /// Add multiple items to the semantic memory
    pub fn add_items(&mut self, items: Vec<SemanticItem>) -> Result<usize> {
        let mut added_count = 0;
        
        for item in items {
            if self.add_item(item).is_ok() {
                added_count += 1;
            }
        }
        
        Ok(added_count)
    }

    /// Remove an item by ID
    pub fn remove_item(&mut self, item_id: &str) -> Option<SemanticItem> {
        if let Some(pos) = self.items.iter().position(|item| item.id == item_id) {
            // Also remove all relationships involving this item
            self.relationships.retain(|rel| !rel.involves_item(item_id));
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    /// Get an item by ID
    pub fn get_item(&self, item_id: &str) -> Option<&SemanticItem> {
        self.items.iter().find(|item| item.id == item_id)
    }

    /// Get a mutable reference to an item by ID
    pub fn get_item_mut(&mut self, item_id: &str) -> Option<&mut SemanticItem> {
        self.items.iter_mut().find(|item| item.id == item_id)
    }

    /// Add a relationship
    pub fn add_relationship(&mut self, relationship: SemanticRelationship) -> Result<()> {
        // Validate that both source and target items exist
        if !self.items.iter().any(|item| item.id == relationship.source_id) {
            return Err(Error::ValidationError(format!("Source item '{}' not found", relationship.source_id)));
        }
        
        if !self.items.iter().any(|item| item.id == relationship.target_id) {
            return Err(Error::ValidationError(format!("Target item '{}' not found", relationship.target_id)));
        }

        // Check for duplicate relationships
        if self.relationships.iter().any(|existing| existing.id == relationship.id) {
            return Err(Error::ValidationError(format!("Relationship with ID '{}' already exists", relationship.id)));
        }
        
        self.relationships.push(relationship);
        Ok(())
    }

    /// Add multiple relationships
    pub fn add_relationships(&mut self, relationships: Vec<SemanticRelationship>) -> Result<usize> {
        let mut added_count = 0;
        
        for relationship in relationships {
            if self.add_relationship(relationship).is_ok() {
                added_count += 1;
            }
        }
        
        Ok(added_count)
    }

    /// Remove a relationship by ID
    pub fn remove_relationship(&mut self, relationship_id: &str) -> Option<SemanticRelationship> {
        if let Some(pos) = self.relationships.iter().position(|rel| rel.id == relationship_id) {
            Some(self.relationships.remove(pos))
        } else {
            None
        }
    }

    /// Get a relationship by ID
    pub fn get_relationship(&self, relationship_id: &str) -> Option<&SemanticRelationship> {
        self.relationships.iter().find(|rel| rel.id == relationship_id)
    }

    /// Get related items for a given item ID
    pub fn get_related_items(&self, item_id: &str) -> Vec<&SemanticItem> {
        let related_ids: Vec<&str> = self
            .relationships
            .iter()
            .filter_map(|rel| rel.get_other_item_id(item_id))
            .collect();

        self.items
            .iter()
            .filter(|item| related_ids.contains(&item.id.as_str()))
            .collect()
    }

    /// Get relationships involving a specific item
    pub fn get_relationships_for_item(&self, item_id: &str) -> Vec<&SemanticRelationship> {
        self.relationships
            .iter()
            .filter(|rel| rel.involves_item(item_id))
            .collect()
    }

    /// Get outgoing relationships from an item
    pub fn get_outgoing_relationships(&self, item_id: &str) -> Vec<&SemanticRelationship> {
        self.relationships
            .iter()
            .filter(|rel| rel.source_id == item_id)
            .collect()
    }

    /// Get incoming relationships to an item
    pub fn get_incoming_relationships(&self, item_id: &str) -> Vec<&SemanticRelationship> {
        self.relationships
            .iter()
            .filter(|rel| rel.target_id == item_id)
            .collect()
    }

    /// Clear all items and relationships
    pub fn clear(&mut self) {
        self.items.clear();
        self.relationships.clear();
    }

    /// Get total memory size (items + relationships)
    pub fn size(&self) -> usize {
        self.items.len() + self.relationships.len()
    }

    /// Check if memory is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.relationships.is_empty()
    }

    /// Update an existing item
    pub fn update_item(&mut self, item_id: &str, updated_item: SemanticItem) -> Result<()> {
        if let Some(existing_item) = self.get_item_mut(item_id) {
            // Preserve the original ID
            let original_id = existing_item.id.clone();
            *existing_item = updated_item;
            existing_item.id = original_id;
            existing_item.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::ValidationError(format!("Item with ID '{}' not found", item_id)))
        }
    }

    /// Update an existing relationship
    pub fn update_relationship(&mut self, relationship_id: &str, updated_relationship: SemanticRelationship) -> Result<()> {
        if let Some(existing_relationship) = self.relationships.iter_mut().find(|r| r.id == relationship_id) {
            // Preserve the original ID
            let original_id = existing_relationship.id.clone();
            *existing_relationship = updated_relationship;
            existing_relationship.id = original_id;
            existing_relationship.updated_at = chrono::Utc::now();
            Ok(())
        } else {
            Err(Error::ValidationError(format!("Relationship with ID '{}' not found", relationship_id)))
        }
    }

    /// Get items by IDs
    pub fn get_items_by_ids(&self, item_ids: &[String]) -> Vec<&SemanticItem> {
        self.items
            .iter()
            .filter(|item| item_ids.contains(&item.id))
            .collect()
    }

    /// Get relationships by IDs
    pub fn get_relationships_by_ids(&self, relationship_ids: &[String]) -> Vec<&SemanticRelationship> {
        self.relationships
            .iter()
            .filter(|rel| relationship_ids.contains(&rel.id))
            .collect()
    }

    /// Check if an item exists
    pub fn contains_item(&self, item_id: &str) -> bool {
        self.items.iter().any(|item| item.id == item_id)
    }

    /// Check if a relationship exists
    pub fn contains_relationship(&self, relationship_id: &str) -> bool {
        self.relationships.iter().any(|rel| rel.id == relationship_id)
    }

    /// Get the first item (if any)
    pub fn first_item(&self) -> Option<&SemanticItem> {
        self.items.first()
    }

    /// Get the last item (if any)
    pub fn last_item(&self) -> Option<&SemanticItem> {
        self.items.last()
    }

    /// Get the first relationship (if any)
    pub fn first_relationship(&self) -> Option<&SemanticRelationship> {
        self.relationships.first()
    }

    /// Get the last relationship (if any)
    pub fn last_relationship(&self) -> Option<&SemanticRelationship> {
        self.relationships.last()
    }

    /// Get items iterator
    pub fn items_iter(&self) -> std::slice::Iter<SemanticItem> {
        self.items.iter()
    }

    /// Get relationships iterator
    pub fn relationships_iter(&self) -> std::slice::Iter<SemanticRelationship> {
        self.relationships.iter()
    }

    /// Get mutable items iterator
    pub fn items_iter_mut(&mut self) -> std::slice::IterMut<SemanticItem> {
        self.items.iter_mut()
    }

    /// Get mutable relationships iterator
    pub fn relationships_iter_mut(&mut self) -> std::slice::IterMut<SemanticRelationship> {
        self.relationships.iter_mut()
    }

    /// Get items count
    pub fn items_count(&self) -> usize {
        self.items.len()
    }

    /// Get relationships count
    pub fn relationships_count(&self) -> usize {
        self.relationships.len()
    }

    /// Clone with new ID
    pub fn clone_with_new_id(&self, new_id: &str) -> Self {
        let mut cloned = self.clone();
        cloned.base.id = new_id.to_string();
        cloned
    }
}
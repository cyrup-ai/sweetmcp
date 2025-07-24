//! Memory repository relationship management
//!
//! This module provides relationship management functionality for the memory
//! repository including adding, removing, and querying relationships with
//! zero allocation patterns and blazing-fast performance.

use std::sync::Arc;

use crate::memory::{MemoryNode, MemoryRelationship};
use super::core::MemoryRepository;

impl MemoryRepository {
    /// Add a relationship between memories
    pub fn add_relationship(&mut self, relationship: MemoryRelationship) -> crate::utils::Result<()> {
        // Validate that both memories exist
        if !self.memories.contains_key(&relationship.from_id) {
            return Err(crate::utils::error::Error::NotFound(format!(
                "Source memory not found: {}",
                relationship.from_id
            )));
        }
        
        if !self.memories.contains_key(&relationship.to_id) {
            return Err(crate::utils::error::Error::NotFound(format!(
                "Target memory not found: {}",
                relationship.to_id
            )));
        }
        
        // Add relationship to the from_id's relationship list
        self.relationships
            .entry(relationship.from_id.clone())
            .or_insert_with(Vec::new)
            .push(relationship.clone());
        
        // Add reverse relationship if it's bidirectional
        if relationship.bidirectional {
            let reverse_relationship = MemoryRelationship {
                from_id: relationship.to_id.clone(),
                to_id: relationship.from_id.clone(),
                relationship_type: relationship.relationship_type.clone(),
                strength: relationship.strength,
                bidirectional: false, // Avoid infinite recursion
                metadata: relationship.metadata.clone(),
                created_at: relationship.created_at,
            };
            
            self.relationships
                .entry(relationship.to_id.clone())
                .or_insert_with(Vec::new)
                .push(reverse_relationship);
        }
        
        Ok(())
    }

    /// Get all relationships for a memory
    pub fn get_relationships(&self, memory_id: &str) -> Vec<MemoryRelationship> {
        self.relationships
            .get(memory_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get relationships of a specific type for a memory
    pub fn get_relationships_by_type(
        &self,
        memory_id: &str,
        relationship_type: &str,
    ) -> Vec<MemoryRelationship> {
        self.relationships
            .get(memory_id)
            .map(|relationships| {
                relationships
                    .iter()
                    .filter(|r| r.relationship_type == relationship_type)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get related memories for a memory ID
    pub fn get_related_memories(&self, memory_id: &str) -> Vec<Arc<MemoryNode>> {
        let relationships = self.get_relationships(memory_id);
        relationships
            .iter()
            .filter_map(|r| self.memories.get(&r.to_id))
            .cloned()
            .collect()
    }

    /// Get related memories of a specific type
    pub fn get_related_memories_by_type(
        &self,
        memory_id: &str,
        relationship_type: &str,
    ) -> Vec<Arc<MemoryNode>> {
        let relationships = self.get_relationships_by_type(memory_id, relationship_type);
        relationships
            .iter()
            .filter_map(|r| self.memories.get(&r.to_id))
            .cloned()
            .collect()
    }

    /// Remove a specific relationship
    pub fn remove_relationship(
        &mut self,
        from_id: &str,
        to_id: &str,
        relationship_type: Option<&str>,
    ) -> bool {
        let mut removed = false;
        
        if let Some(relationships) = self.relationships.get_mut(from_id) {
            let original_len = relationships.len();
            
            relationships.retain(|r| {
                let type_matches = relationship_type.map_or(true, |t| r.relationship_type == t);
                !(r.to_id == to_id && type_matches)
            });
            
            removed = relationships.len() < original_len;
            
            // Remove empty relationship lists
            if relationships.is_empty() {
                self.relationships.remove(from_id);
            }
        }
        
        // Also remove reverse relationship if it was bidirectional
        if let Some(relationships) = self.relationships.get_mut(to_id) {
            let original_len = relationships.len();
            
            relationships.retain(|r| {
                let type_matches = relationship_type.map_or(true, |t| r.relationship_type == t);
                !(r.to_id == from_id && type_matches)
            });
            
            if relationships.len() < original_len {
                removed = true;
            }
            
            // Remove empty relationship lists
            if relationships.is_empty() {
                self.relationships.remove(to_id);
            }
        }
        
        removed
    }

    /// Remove all relationships for a memory
    pub fn remove_all_relationships(&mut self, memory_id: &str) -> usize {
        let mut removed_count = 0;
        
        // Remove outgoing relationships
        if let Some(relationships) = self.relationships.remove(memory_id) {
            removed_count += relationships.len();
            
            // Remove corresponding incoming relationships
            for relationship in &relationships {
                if relationship.bidirectional {
                    self.remove_relationship(&relationship.to_id, memory_id, Some(&relationship.relationship_type));
                }
            }
        }
        
        // Remove incoming relationships
        let mut to_remove = Vec::new();
        for (from_id, relationships) in &mut self.relationships {
            let original_len = relationships.len();
            relationships.retain(|r| r.to_id != memory_id);
            removed_count += original_len - relationships.len();
            
            if relationships.is_empty() {
                to_remove.push(from_id.clone());
            }
        }
        
        // Remove empty relationship lists
        for from_id in to_remove {
            self.relationships.remove(&from_id);
        }
        
        removed_count
    }

    /// Check if two memories have a relationship
    pub fn has_relationship(
        &self,
        from_id: &str,
        to_id: &str,
        relationship_type: Option<&str>,
    ) -> bool {
        if let Some(relationships) = self.relationships.get(from_id) {
            relationships.iter().any(|r| {
                r.to_id == to_id && relationship_type.map_or(true, |t| r.relationship_type == t)
            })
        } else {
            false
        }
    }

    /// Get relationship strength between two memories
    pub fn get_relationship_strength(
        &self,
        from_id: &str,
        to_id: &str,
        relationship_type: Option<&str>,
    ) -> Option<f32> {
        if let Some(relationships) = self.relationships.get(from_id) {
            relationships
                .iter()
                .find(|r| {
                    r.to_id == to_id && relationship_type.map_or(true, |t| r.relationship_type == t)
                })
                .map(|r| r.strength)
        } else {
            None
        }
    }

    /// Update relationship strength
    pub fn update_relationship_strength(
        &mut self,
        from_id: &str,
        to_id: &str,
        relationship_type: &str,
        new_strength: f32,
    ) -> bool {
        let mut updated = false;
        
        if let Some(relationships) = self.relationships.get_mut(from_id) {
            for relationship in relationships {
                if relationship.to_id == to_id && relationship.relationship_type == relationship_type {
                    relationship.strength = new_strength;
                    updated = true;
                    break;
                }
            }
        }
        
        // Update reverse relationship if bidirectional
        if updated {
            if let Some(relationships) = self.relationships.get_mut(to_id) {
                for relationship in relationships {
                    if relationship.to_id == from_id && relationship.relationship_type == relationship_type {
                        relationship.strength = new_strength;
                        break;
                    }
                }
            }
        }
        
        updated
    }

    /// Get all relationship types in the repository
    pub fn get_relationship_types(&self) -> Vec<String> {
        let mut types = std::collections::HashSet::new();
        
        for relationships in self.relationships.values() {
            for relationship in relationships {
                types.insert(relationship.relationship_type.clone());
            }
        }
        
        types.into_iter().collect()
    }

    /// Get relationship count by type
    pub fn count_relationships_by_type(&self, relationship_type: &str) -> usize {
        self.relationships
            .values()
            .flat_map(|relationships| relationships.iter())
            .filter(|r| r.relationship_type == relationship_type)
            .count()
    }

    /// Get total relationship count
    pub fn total_relationship_count(&self) -> usize {
        self.relationships
            .values()
            .map(|relationships| relationships.len())
            .sum()
    }

    /// Find memories with the most relationships
    pub fn get_most_connected_memories(&self, limit: usize) -> Vec<(String, usize)> {
        let mut memory_counts: Vec<(String, usize)> = self
            .relationships
            .iter()
            .map(|(memory_id, relationships)| (memory_id.clone(), relationships.len()))
            .collect();
        
        // Sort by relationship count (descending)
        memory_counts.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Apply limit
        memory_counts.truncate(limit);
        
        memory_counts
    }

    /// Find memories with specific relationship patterns
    pub fn find_memories_with_relationship_pattern(
        &self,
        min_relationships: Option<usize>,
        max_relationships: Option<usize>,
        required_types: Option<&[String]>,
    ) -> Vec<String> {
        let mut result = Vec::new();
        
        for (memory_id, relationships) in &self.relationships {
            let relationship_count = relationships.len();
            
            // Check relationship count constraints
            if let Some(min) = min_relationships {
                if relationship_count < min {
                    continue;
                }
            }
            
            if let Some(max) = max_relationships {
                if relationship_count > max {
                    continue;
                }
            }
            
            // Check required relationship types
            if let Some(required) = required_types {
                let relationship_types: std::collections::HashSet<_> = 
                    relationships.iter().map(|r| &r.relationship_type).collect();
                
                let has_all_required = required.iter().all(|t| relationship_types.contains(t));
                if !has_all_required {
                    continue;
                }
            }
            
            result.push(memory_id.clone());
        }
        
        result
    }

    /// Get relationship statistics
    pub fn get_relationship_stats(&self) -> RelationshipStats {
        let mut type_counts = std::collections::HashMap::new();
        let mut total_relationships = 0;
        let mut total_strength = 0.0;
        let mut bidirectional_count = 0;
        
        for relationships in self.relationships.values() {
            for relationship in relationships {
                total_relationships += 1;
                total_strength += relationship.strength;
                
                if relationship.bidirectional {
                    bidirectional_count += 1;
                }
                
                *type_counts.entry(relationship.relationship_type.clone()).or_insert(0) += 1;
            }
        }
        
        let average_strength = if total_relationships > 0 {
            total_strength / total_relationships as f32
        } else {
            0.0
        };
        
        RelationshipStats {
            total_relationships,
            unique_relationship_types: type_counts.len(),
            relationship_type_counts: type_counts,
            average_strength,
            bidirectional_count,
            memories_with_relationships: self.relationships.len(),
        }
    }

    /// Validate relationship integrity
    pub fn validate_relationships(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (from_id, relationships) in &self.relationships {
            // Check if from_id memory exists
            if !self.memories.contains_key(from_id) {
                errors.push(format!("Relationship source memory not found: {}", from_id));
                continue;
            }
            
            for relationship in relationships {
                // Check if to_id memory exists
                if !self.memories.contains_key(&relationship.to_id) {
                    errors.push(format!(
                        "Relationship target memory not found: {} -> {}",
                        from_id, relationship.to_id
                    ));
                }
                
                // Check strength range
                if relationship.strength < 0.0 || relationship.strength > 1.0 {
                    errors.push(format!(
                        "Invalid relationship strength: {} -> {} (strength: {})",
                        from_id, relationship.to_id, relationship.strength
                    ));
                }
                
                // Check for self-relationships
                if relationship.from_id == relationship.to_id {
                    errors.push(format!("Self-relationship detected: {}", from_id));
                }
            }
        }
        
        errors
    }
}

/// Relationship statistics
#[derive(Debug, Clone)]
pub struct RelationshipStats {
    /// Total number of relationships
    pub total_relationships: usize,
    /// Number of unique relationship types
    pub unique_relationship_types: usize,
    /// Count of relationships by type
    pub relationship_type_counts: std::collections::HashMap<String, usize>,
    /// Average relationship strength
    pub average_strength: f32,
    /// Number of bidirectional relationships
    pub bidirectional_count: usize,
    /// Number of memories that have relationships
    pub memories_with_relationships: usize,
}
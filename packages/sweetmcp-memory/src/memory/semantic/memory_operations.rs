//! Semantic memory operations for advanced functionality
//!
//! This module provides advanced operations for semantic memory management
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use std::collections::HashSet;

use super::memory::SemanticMemory;
use super::item_core::SemanticItem;
use super::memory_statistics::MemoryStatistics;
use crate::utils::{Result, Error};

impl SemanticMemory {
    /// Find items connected by a path of relationships
    pub fn find_connected_items(&self, start_item_id: &str, max_depth: usize) -> Vec<&SemanticItem> {
        let mut visited = HashSet::new();
        let mut to_visit = vec![(start_item_id, 0)];
        let mut connected_items = Vec::new();

        while let Some((current_id, depth)) = to_visit.pop() {
            if visited.contains(current_id) || depth > max_depth {
                continue;
            }

            visited.insert(current_id);

            if let Some(item) = self.get_item(current_id) {
                if current_id != start_item_id {
                    connected_items.push(item);
                }
            }

            if depth < max_depth {
                for rel in self.get_relationships_for_item(current_id) {
                    if let Some(other_id) = rel.get_other_item_id(current_id) {
                        if !visited.contains(other_id) {
                            to_visit.push((other_id, depth + 1));
                        }
                    }
                }
            }
        }

        connected_items
    }

    /// Get memory statistics
    pub fn get_statistics(&self) -> MemoryStatistics {
        MemoryStatistics::calculate(&self.items, &self.relationships)
    }

    /// Validate memory consistency
    pub fn validate(&self) -> Result<()> {
        // Check that all relationships reference existing items
        for rel in &self.relationships {
            if !self.items.iter().any(|item| item.id == rel.source_id) {
                return Err(Error::ValidationError(format!("Relationship '{}' references non-existent source item '{}'", rel.id, rel.source_id)));
            }
            
            if !self.items.iter().any(|item| item.id == rel.target_id) {
                return Err(Error::ValidationError(format!("Relationship '{}' references non-existent target item '{}'", rel.id, rel.target_id)));
            }
            
            rel.validate()?;
        }

        // Validate all items
        for item in &self.items {
            item.validate()?;
        }

        Ok(())
    }

    /// Compact memory by removing orphaned relationships
    pub fn compact(&mut self) -> usize {
        let item_ids: HashSet<_> = self.items.iter().map(|item| &item.id).collect();
        let initial_count = self.relationships.len();
        
        self.relationships.retain(|rel| {
            item_ids.contains(&rel.source_id) && item_ids.contains(&rel.target_id)
        });
        
        initial_count - self.relationships.len()
    }

    /// Get memory capacity utilization
    pub fn capacity_utilization(&self) -> (f64, f64) {
        let items_utilization = if self.items.capacity() > 0 {
            self.items.len() as f64 / self.items.capacity() as f64
        } else {
            0.0
        };
        
        let relationships_utilization = if self.relationships.capacity() > 0 {
            self.relationships.len() as f64 / self.relationships.capacity() as f64
        } else {
            0.0
        };
        
        (items_utilization, relationships_utilization)
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional_items: usize, additional_relationships: usize) {
        self.items.reserve(additional_items);
        self.relationships.reserve(additional_relationships);
    }

    /// Shrink memory to fit current contents
    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
        self.relationships.shrink_to_fit();
    }

    /// Rebuild relationships index for faster lookups
    pub fn rebuild_relationships_index(&self) -> RelationshipIndex {
        let mut source_index = std::collections::HashMap::new();
        let mut target_index = std::collections::HashMap::new();
        let mut bidirectional_index = std::collections::HashMap::new();

        for (idx, relationship) in self.relationships.iter().enumerate() {
            // Source index
            source_index
                .entry(relationship.source_id.clone())
                .or_insert_with(Vec::new)
                .push(idx);

            // Target index
            target_index
                .entry(relationship.target_id.clone())
                .or_insert_with(Vec::new)
                .push(idx);

            // Bidirectional index
            bidirectional_index
                .entry(relationship.source_id.clone())
                .or_insert_with(Vec::new)
                .push(idx);
            bidirectional_index
                .entry(relationship.target_id.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        RelationshipIndex {
            source_index,
            target_index,
            bidirectional_index,
        }
    }

    /// Get relationship statistics
    pub fn get_relationship_statistics(&self) -> RelationshipStatistics {
        let mut type_counts = std::collections::HashMap::new();
        let mut confidence_counts = std::collections::HashMap::new();
        let mut source_degree_distribution = std::collections::HashMap::new();
        let mut target_degree_distribution = std::collections::HashMap::new();

        // Count relationship types
        for relationship in &self.relationships {
            *type_counts.entry(relationship.relationship_type).or_insert(0) += 1;
            *confidence_counts.entry(relationship.confidence).or_insert(0) += 1;
        }

        // Calculate degree distributions
        for item in &self.items {
            let outgoing_count = self.get_outgoing_relationships(&item.id).len();
            let incoming_count = self.get_incoming_relationships(&item.id).len();
            
            *source_degree_distribution.entry(outgoing_count).or_insert(0) += 1;
            *target_degree_distribution.entry(incoming_count).or_insert(0) += 1;
        }

        RelationshipStatistics {
            total_relationships: self.relationships.len(),
            type_distribution: type_counts,
            confidence_distribution: confidence_counts,
            source_degree_distribution,
            target_degree_distribution,
            average_relationships_per_item: if self.items.is_empty() {
                0.0
            } else {
                (self.relationships.len() * 2) as f64 / self.items.len() as f64
            },
        }
    }

    /// Get item statistics
    pub fn get_item_statistics(&self) -> ItemStatistics {
        let mut category_counts = std::collections::HashMap::new();
        let mut type_counts = std::collections::HashMap::new();
        let mut confidence_counts = std::collections::HashMap::new();
        let mut tag_counts = std::collections::HashMap::new();
        let mut content_length_distribution = std::collections::HashMap::new();

        for item in &self.items {
            *category_counts.entry(item.category.clone()).or_insert(0) += 1;
            *type_counts.entry(item.item_type.clone()).or_insert(0) += 1;
            *confidence_counts.entry(item.confidence).or_insert(0) += 1;

            for tag in &item.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }

            let content_length_bucket = (item.content.len() / 100) * 100; // Group by 100-char buckets
            *content_length_distribution.entry(content_length_bucket).or_insert(0) += 1;
        }

        let total_content_length: usize = self.items.iter().map(|item| item.content.len()).sum();
        let average_content_length = if self.items.is_empty() {
            0.0
        } else {
            total_content_length as f64 / self.items.len() as f64
        };

        let total_tags: usize = self.items.iter().map(|item| item.tags.len()).sum();
        let average_tags_per_item = if self.items.is_empty() {
            0.0
        } else {
            total_tags as f64 / self.items.len() as f64
        };

        ItemStatistics {
            total_items: self.items.len(),
            category_distribution: category_counts,
            type_distribution: type_counts,
            confidence_distribution: confidence_counts,
            tag_distribution: tag_counts,
            content_length_distribution,
            average_content_length,
            average_tags_per_item,
            total_unique_tags: tag_counts.len(),
        }
    }

    /// Get comprehensive memory statistics
    pub fn get_comprehensive_statistics(&self) -> ComprehensiveStatistics {
        ComprehensiveStatistics {
            memory_statistics: self.get_statistics(),
            item_statistics: self.get_item_statistics(),
            relationship_statistics: self.get_relationship_statistics(),
            graph_metrics: GraphMetrics {
                diameter: self.graph_diameter(),
                radius: self.graph_radius(),
                clustering_coefficient: self.clustering_coefficient(),
                is_connected: self.is_connected(),
                component_count: self.get_connectivity_components().len(),
                isolated_items: self.get_isolated_items().len(),
                hub_items: self.get_hub_items(5).len(),
            },
        }
    }
}

/// Relationship index for fast lookups
#[derive(Debug, Clone)]
pub struct RelationshipIndex {
    pub source_index: std::collections::HashMap<String, Vec<usize>>,
    pub target_index: std::collections::HashMap<String, Vec<usize>>,
    pub bidirectional_index: std::collections::HashMap<String, Vec<usize>>,
}

/// Relationship statistics
#[derive(Debug, Clone)]
pub struct RelationshipStatistics {
    pub total_relationships: usize,
    pub type_distribution: std::collections::HashMap<super::relationships::relationship_types::SemanticRelationshipType, usize>,
    pub confidence_distribution: std::collections::HashMap<super::confidence::ConfidenceLevel, usize>,
    pub source_degree_distribution: std::collections::HashMap<usize, usize>,
    pub target_degree_distribution: std::collections::HashMap<usize, usize>,
    pub average_relationships_per_item: f64,
}

/// Item statistics
#[derive(Debug, Clone)]
pub struct ItemStatistics {
    pub total_items: usize,
    pub category_distribution: std::collections::HashMap<String, usize>,
    pub type_distribution: std::collections::HashMap<super::item_types::SemanticItemType, usize>,
    pub confidence_distribution: std::collections::HashMap<super::confidence::ConfidenceLevel, usize>,
    pub tag_distribution: std::collections::HashMap<String, usize>,
    pub content_length_distribution: std::collections::HashMap<usize, usize>,
    pub average_content_length: f64,
    pub average_tags_per_item: f64,
    pub total_unique_tags: usize,
}

/// Graph metrics
#[derive(Debug, Clone)]
pub struct GraphMetrics {
    pub diameter: Option<usize>,
    pub radius: Option<usize>,
    pub clustering_coefficient: f64,
    pub is_connected: bool,
    pub component_count: usize,
    pub isolated_items: usize,
    pub hub_items: usize,
}

/// Comprehensive statistics
#[derive(Debug, Clone)]
pub struct ComprehensiveStatistics {
    pub memory_statistics: MemoryStatistics,
    pub item_statistics: ItemStatistics,
    pub relationship_statistics: RelationshipStatistics,
    pub graph_metrics: GraphMetrics,
}
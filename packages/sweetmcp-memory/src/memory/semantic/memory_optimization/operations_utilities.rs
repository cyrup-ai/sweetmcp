//! Optimization utility functions and calculations
//!
//! This module provides blazing-fast utility functions for optimization
//! calculations with zero allocation patterns and elegant ergonomic interfaces.

use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn};

use super::super::{
    semantic_item::SemanticItem,
    semantic_relationship::SemanticRelationship,
};
use super::operations_core::OptimizationExecutor;

impl OptimizationExecutor {
    /// Calculate fragmentation level with zero allocation patterns
    #[inline]
    pub fn calculate_fragmentation_level(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        if items.is_empty() && relationships.is_empty() {
            return 0.0;
        }

        // Calculate fragmentation based on memory layout efficiency
        let total_items = items.len() + relationships.len();
        let capacity_items = items.capacity() + relationships.capacity();
        
        if capacity_items == 0 {
            return 0.0;
        }

        // Fragmentation is the ratio of unused to total capacity
        let unused_capacity = capacity_items.saturating_sub(total_items);
        unused_capacity as f64 / capacity_items as f64
    }

    /// Calculate total memory usage for items
    #[inline]
    pub fn calculate_total_memory_usage(&self, items: &HashMap<String, SemanticItem>) -> usize {
        let base_size = std::mem::size_of::<HashMap<String, SemanticItem>>();
        let entries_size = items.len() * (
            std::mem::size_of::<String>() + 
            std::mem::size_of::<SemanticItem>()
        );
        
        // Estimate string content sizes
        let string_content_size: usize = items.keys()
            .map(|k| k.len())
            .sum();
        
        base_size + entries_size + string_content_size
    }

    /// Calculate memory usage for relationships
    #[inline]
    pub fn calculate_relationship_memory_usage(&self, relationships: &HashMap<String, SemanticRelationship>) -> usize {
        let base_size = std::mem::size_of::<HashMap<String, SemanticRelationship>>();
        let entries_size = relationships.len() * (
            std::mem::size_of::<String>() + 
            std::mem::size_of::<SemanticRelationship>()
        );
        
        // Estimate string content sizes
        let string_content_size: usize = relationships.keys()
            .map(|k| k.len())
            .sum();
        
        base_size + entries_size + string_content_size
    }

    /// Check if item should be compressed
    #[inline]
    pub fn should_compress_item(&self, item: &SemanticItem) -> bool {
        // Compress items with large content or low access frequency
        self.estimate_item_size(item) > 1024 || // Items larger than 1KB
        self.get_item_access_frequency(item) < 0.1 // Rarely accessed items
    }

    /// Estimate item size in bytes
    #[inline]
    fn estimate_item_size(&self, item: &SemanticItem) -> usize {
        // Base size of SemanticItem struct
        let base_size = std::mem::size_of::<SemanticItem>();
        
        // Estimate content size (simplified calculation)
        let content_size = match item.content.as_ref() {
            Some(content) => content.len(),
            None => 0,
        };
        
        // Estimate metadata size
        let metadata_size = item.metadata.as_ref()
            .map(|m| serde_json::to_string(m).map_or(0, |s| s.len()))
            .unwrap_or(0);
        
        base_size + content_size + metadata_size
    }

    /// Get item access frequency (simplified heuristic)
    #[inline]
    /// Get item access frequency (simplified heuristic)
    pub fn get_item_access_frequency(&self, item: &SemanticItem) -> f64 {
        // Use last_accessed timestamp to estimate frequency
        match &item.last_accessed {
            Some(timestamp) => {
                let elapsed = timestamp.elapsed().unwrap_or(Duration::from_secs(u64::MAX));
                let hours_since_access = elapsed.as_secs() as f64 / 3600.0;
                
                // Frequency decreases exponentially with time
                (-hours_since_access / 24.0).exp() // Decay over 24 hours
            }
            None => 0.0, // Never accessed
        }
    }

    /// Check if item is frequently accessed
    #[inline]
    pub(crate) fn is_frequently_accessed(&self, item: &SemanticItem) -> bool {
        self.get_item_access_frequency(item) > 0.5
    }

    /// Calculate cache efficiency
    #[inline]
    pub(crate) fn calculate_cache_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        if items.is_empty() {
            return 1.0;
        }

        // Calculate efficiency based on access patterns and locality
        let frequently_accessed_count = items.values()
            .filter(|item| self.is_frequently_accessed(item))
            .count();
        
        let total_items = items.len();
        let cache_hit_ratio = frequently_accessed_count as f64 / total_items as f64;
        
        // Factor in relationship locality
        let relationship_locality = self.calculate_relationship_locality(items, relationships);
        
        (cache_hit_ratio + relationship_locality) / 2.0
    }

    /// Calculate relationship locality
    #[inline]
    /// Calculate relationship locality
    pub(crate) fn calculate_relationship_locality(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        if relationships.is_empty() {
            return 1.0;
        }

        let mut local_relationships = 0;
        let total_relationships = relationships.len();

        for relationship in relationships.values() {
            // Check if both source and target items exist and are frequently accessed
            let source_frequent = relationship.source_id.as_ref()
                .and_then(|id| items.get(id))
                .map(|item| self.is_frequently_accessed(item))
                .unwrap_or(false);
            
            let target_frequent = relationship.target_id.as_ref()
                .and_then(|id| items.get(id))
                .map(|item| self.is_frequently_accessed(item))
                .unwrap_or(false);
            
            if source_frequent && target_frequent {
                local_relationships += 1;
            }
        }

        local_relationships as f64 / total_relationships as f64
    }

    /// Calculate index efficiency
    #[inline]
    pub(crate) fn calculate_index_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        // Efficiency based on load factor and distribution
        let items_load_factor = if items.capacity() > 0 {
            items.len() as f64 / items.capacity() as f64
        } else {
            0.0
        };
        
        let relationships_load_factor = if relationships.capacity() > 0 {
            relationships.len() as f64 / relationships.capacity() as f64
        } else {
            0.0
        };
        
        // Optimal load factor is around 0.75
        let optimal_load_factor = 0.75;
        let items_efficiency = 1.0 - (items_load_factor - optimal_load_factor).abs();
        let relationships_efficiency = 1.0 - (relationships_load_factor - optimal_load_factor).abs();
        
        (items_efficiency + relationships_efficiency) / 2.0
    }

    /// Calculate memory efficiency
    #[inline]
    pub(crate) fn calculate_memory_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        let total_used = items.len() + relationships.len();
        let total_capacity = items.capacity() + relationships.capacity();
        
        if total_capacity == 0 {
            return 1.0;
        }
        
        total_used as f64 / total_capacity as f64
    }

    /// Calculate access pattern efficiency
    #[inline]
    pub(crate) fn calculate_access_pattern_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        if items.is_empty() {
            return 1.0;
        }

        // Measure how well access patterns align with data organization
        let sequential_access_score = self.calculate_sequential_access_score(items);
        let temporal_locality_score = self.calculate_temporal_locality_score(items);
        let spatial_locality_score = self.calculate_spatial_locality_score(items, relationships);
        
        (sequential_access_score + temporal_locality_score + spatial_locality_score) / 3.0
    }

    /// Calculate sequential access score
    #[inline]
    /// Calculate sequential access score
    pub(crate) fn calculate_sequential_access_score(&self, items: &HashMap<String, SemanticItem>) -> f64 {
        // Simplified heuristic: items with similar IDs accessed together
        let mut sequential_pairs = 0;
        let mut total_pairs = 0;
        
        let item_ids: Vec<_> = items.keys().collect();
        for i in 0..item_ids.len().saturating_sub(1) {
            for j in (i + 1)..item_ids.len() {
                total_pairs += 1;
                
                // Check if IDs are similar (simplified check)
                if self.are_ids_similar(item_ids[i], item_ids[j]) {
                    sequential_pairs += 1;
                }
            }
        }
        
        if total_pairs == 0 {
            1.0
        } else {
            sequential_pairs as f64 / total_pairs as f64
        }
    }

    /// Check if two IDs are similar (for sequential access)
    #[inline]
    /// Check if two IDs are similar (for sequential access)
    pub(crate) fn are_ids_similar(&self, id1: &str, id2: &str) -> bool {
        // Simple similarity check based on common prefixes
        let common_prefix_len = id1.chars()
            .zip(id2.chars())
            .take_while(|(a, b)| a == b)
            .count();
        
        let min_len = id1.len().min(id2.len());
        if min_len == 0 {
            return false;
        }
        
        common_prefix_len as f64 / min_len as f64 > 0.5
    }

    /// Calculate temporal locality score
    #[inline]
    /// Calculate temporal locality score
    pub(crate) fn calculate_temporal_locality_score(&self, items: &HashMap<String, SemanticItem>) -> f64 {
        // Items accessed around the same time have good temporal locality
        let mut temporal_groups = 0;
        let mut total_items = 0;
        
        for item in items.values() {
            total_items += 1;
            
            if let Some(last_accessed) = &item.last_accessed {
                // Check if accessed recently (within last hour)
                if last_accessed.elapsed().unwrap_or(Duration::MAX) < Duration::from_secs(3600) {
                    temporal_groups += 1;
                }
            }
        }
        
        if total_items == 0 {
            1.0
        } else {
            temporal_groups as f64 / total_items as f64
        }
    }

    /// Calculate spatial locality score
    #[inline]
    /// Calculate spatial locality score
    pub(crate) fn calculate_spatial_locality_score(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        // Related items stored close together have good spatial locality
        if relationships.is_empty() {
            return 1.0;
        }

        let mut local_relationships = 0;
        let total_relationships = relationships.len();

        for relationship in relationships.values() {
            // Check if related items exist
            let source_exists = relationship.source_id.as_ref()
                .map(|id| items.contains_key(id))
                .unwrap_or(false);
            
            let target_exists = relationship.target_id.as_ref()
                .map(|id| items.contains_key(id))
                .unwrap_or(false);
            
            if source_exists && target_exists {
                local_relationships += 1;
            }
        }

        local_relationships as f64 / total_relationships as f64
    }

    /// Check if relationship should be pruned
    #[inline]
    pub(crate) fn should_prune_relationship(
        &self,
        relationship: &SemanticRelationship,
        items: &HashMap<String, SemanticItem>,
    ) -> bool {
        // Prune relationships with weak strength or missing endpoints
        let weak_strength = relationship.strength.map(|s| s < 0.1).unwrap_or(true);
        
        let missing_source = relationship.source_id.as_ref()
            .map(|id| !items.contains_key(id))
            .unwrap_or(true);
        
        let missing_target = relationship.target_id.as_ref()
            .map(|id| !items.contains_key(id))
            .unwrap_or(true);
        
        weak_strength || missing_source || missing_target
    }

    /// Calculate structure efficiency
    #[inline]
    pub(crate) fn calculate_structure_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        // Efficiency based on data structure optimization
        let items_efficiency = self.calculate_hashmap_efficiency(items.len(), items.capacity());
        let relationships_efficiency = self.calculate_hashmap_efficiency(relationships.len(), relationships.capacity());
        
        (items_efficiency + relationships_efficiency) / 2.0
    }

    /// Calculate HashMap efficiency
    #[inline]
    /// Calculate HashMap efficiency
    pub(crate) fn calculate_hashmap_efficiency(&self, len: usize, capacity: usize) -> f64 {
        if capacity == 0 {
            return 1.0;
        }
        
        let load_factor = len as f64 / capacity as f64;
        
        // Optimal load factor for HashMap is around 0.75
        let optimal_load_factor = 0.75;
        let deviation = (load_factor - optimal_load_factor).abs();
        
        (1.0 - deviation).max(0.0)
    }

    /// Check if item structure can be optimized
    #[inline]
    pub(crate) fn can_optimize_item_structure(&self, item: &SemanticItem) -> bool {
        // Check for optimization opportunities
        let has_large_metadata = item.metadata.as_ref()
            .map(|m| serde_json::to_string(m).map_or(false, |s| s.len() > 1024))
            .unwrap_or(false);
        
        let has_unused_fields = item.content.is_none() || item.embedding.is_none();
        
        has_large_metadata || has_unused_fields
    }

    /// Check if relationship structure can be optimized
    #[inline]
    pub(crate) fn can_optimize_relationship_structure(&self, relationship: &SemanticRelationship) -> bool {
        // Check for optimization opportunities
        let has_weak_strength = relationship.strength.map(|s| s < 0.1).unwrap_or(true);
        let has_missing_metadata = relationship.metadata.is_none();
        
        has_weak_strength || has_missing_metadata
    }

    /// Check if item is orphaned
    #[inline]
    pub(crate) fn is_orphaned_item(
        &self,
        item: &SemanticItem,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> bool {
        let item_id = &item.id;
        
        // Check if item is referenced in any relationship
        let is_referenced = relationships.values().any(|rel| {
            rel.source_id.as_ref() == Some(item_id) || 
            rel.target_id.as_ref() == Some(item_id)
        });
        
        // Item is orphaned if not referenced and not accessed recently
        !is_referenced && !self.is_frequently_accessed(item)
    }

    /// Calculate pool efficiency
    #[inline]
    pub(crate) fn calculate_pool_efficiency(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        // Efficiency based on memory pool utilization
        let items_pool_efficiency = self.calculate_pool_utilization(items.len(), items.capacity());
        let relationships_pool_efficiency = self.calculate_pool_utilization(relationships.len(), relationships.capacity());
        
        (items_pool_efficiency + relationships_pool_efficiency) / 2.0
    }

    /// Calculate pool utilization
    #[inline]
    /// Calculate pool utilization
    pub(crate) fn calculate_pool_utilization(&self, used: usize, capacity: usize) -> f64 {
        if capacity == 0 {
            return 1.0;
        }
        
        let utilization = used as f64 / capacity as f64;
        
        // Good utilization is between 0.5 and 0.9
        if utilization >= 0.5 && utilization <= 0.9 {
            1.0
        } else if utilization < 0.5 {
            utilization / 0.5 // Scale from 0 to 1
        } else {
            (1.0 - utilization) / 0.1 // Scale from 1 to 0 for over-utilization
        }
    }

    /// Get optimization priority score
    #[inline]
    pub(crate) fn get_optimization_priority_score(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        let fragmentation_score = 1.0 - self.calculate_fragmentation_level(items, relationships);
        let cache_efficiency_score = self.calculate_cache_efficiency(items, relationships);
        let memory_efficiency_score = self.calculate_memory_efficiency(items, relationships);
        let structure_efficiency_score = self.calculate_structure_efficiency(items, relationships);
        
        // Weighted average of efficiency scores
        let weights = [0.3, 0.25, 0.25, 0.2]; // Fragmentation, cache, memory, structure
        let scores = [fragmentation_score, cache_efficiency_score, memory_efficiency_score, structure_efficiency_score];
        
        scores.iter()
            .zip(weights.iter())
            .map(|(score, weight)| score * weight)
            .sum()
    }

    /// Validate optimization safety
    #[inline]
    pub(crate) fn validate_optimization_safety(
        &self,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> bool {
        // Check if optimization is safe to perform
        let memory_usage = self.calculate_total_memory_usage(items) + 
                          self.calculate_relationship_memory_usage(relationships);
        
        let within_memory_limits = memory_usage <= self.safety_constraints.max_memory_usage;
        let reasonable_size = items.len() <= self.safety_constraints.max_items_per_operation;
        
        within_memory_limits && reasonable_size
    }

    /// Get optimization recommendations priority
    #[inline]
    pub(crate) fn get_recommendations_priority(&self) -> Vec<super::optimization_recommendations::RecommendationType> {
        self.strategy.priority_order.clone()
    }

    /// Calculate optimization impact estimate
    #[inline]
    pub(crate) fn calculate_optimization_impact(
        &self,
        recommendation_type: &super::optimization_recommendations::RecommendationType,
        items: &HashMap<String, SemanticItem>,
        relationships: &HashMap<String, SemanticRelationship>,
    ) -> f64 {
        use super::optimization_recommendations::RecommendationType;
        
        match recommendation_type {
            RecommendationType::Defragmentation => {
                self.calculate_fragmentation_level(items, relationships) * 100.0
            }
            RecommendationType::Compression => {
                let compressible_items = items.values()
                    .filter(|item| self.should_compress_item(item))
                    .count();
                (compressible_items as f64 / items.len().max(1) as f64) * 50.0
            }
            RecommendationType::CacheOptimization => {
                (1.0 - self.calculate_cache_efficiency(items, relationships)) * 75.0
            }
            RecommendationType::IndexOptimization => {
                (1.0 - self.calculate_index_efficiency(items, relationships)) * 60.0
            }
            RecommendationType::MemoryReallocation => {
                (1.0 - self.calculate_memory_efficiency(items, relationships)) * 40.0
            }
            RecommendationType::AccessPatternOptimization => {
                (1.0 - self.calculate_access_pattern_efficiency(items, relationships)) * 30.0
            }
            RecommendationType::RelationshipPruning => {
                let prunable_relationships = relationships.values()
                    .filter(|rel| self.should_prune_relationship(rel, items))
                    .count();
                (prunable_relationships as f64 / relationships.len().max(1) as f64) * 25.0
            }
            RecommendationType::DataStructureOptimization => {
                (1.0 - self.calculate_structure_efficiency(items, relationships)) * 35.0
            }
            RecommendationType::GarbageCollectionOptimization => {
                let orphaned_items = items.values()
                    .filter(|item| self.is_orphaned_item(item, relationships))
                    .count();
                (orphaned_items as f64 / items.len().max(1) as f64) * 20.0
            }
            RecommendationType::MemoryPoolOptimization => {
                (1.0 - self.calculate_pool_efficiency(items, relationships)) * 15.0
            }
        }
    }
}
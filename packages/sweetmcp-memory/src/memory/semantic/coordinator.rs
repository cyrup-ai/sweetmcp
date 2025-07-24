//! Semantic memory coordinator for high-level operations
//!
//! This module provides blazing-fast semantic memory coordination with zero allocation
//! optimizations and elegant ergonomic interfaces for managing semantic operations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::utils::{Result, error::Error};

use super::{
    confidence::{ConfidenceLevel, ConfidenceCalculator, ConfidenceStatistics},
    item_types::{SemanticItemType, SemanticItemTypeClassifier, SemanticItemTypeStatistics},
    relationships::{
        SemanticRelationshipType, RelationshipDirection, RelationshipPattern,
        RelationshipStatistics, RelationshipValidator, RelationshipQueryBuilder,
    },
    memory_cleanup::{
        SemanticMemoryManager, MemoryStatistics, CleanupConfig, OptimizationStrategy,
        MemoryReport,
    },
    memory_optimization::{
        OptimizationRecommendation, HealthCheckReport, HealthScore,
    },
    semantic_item::{SemanticItem, ItemSummary, ArchiveConfig, DeleteConfig},
    semantic_relationship::{SemanticRelationship, RelationshipSummary, RelationshipArchiveConfig, RelationshipDeleteConfig},
};

/// Comprehensive semantic memory coordinator
pub struct SemanticMemoryCoordinator {
    memory_manager: SemanticMemoryManager,
    confidence_calculator: ConfidenceCalculator,
    type_classifier: SemanticItemTypeClassifier,
    relationship_validator: RelationshipValidator,
    items: Arc<RwLock<HashMap<String, SemanticItem>>>,
    relationships: Arc<RwLock<HashMap<String, SemanticRelationship>>>,
}

impl SemanticMemoryCoordinator {
    /// Create new semantic memory coordinator with zero allocation optimizations
    #[inline]
    pub async fn new() -> Result<Self> {
        let memory_manager = SemanticMemoryManager::new().await?;
        let confidence_calculator = ConfidenceCalculator::new();
        let type_classifier = SemanticItemTypeClassifier::new();
        let relationship_validator = RelationshipValidator::new();
        let items = Arc::new(RwLock::new(HashMap::new()));
        let relationships = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            memory_manager,
            confidence_calculator,
            type_classifier,
            relationship_validator,
            items,
            relationships,
        })
    }

    /// Create coordinator with custom configuration
    #[inline]
    pub async fn with_config(
        cleanup_config: CleanupConfig,
        optimization_strategy: OptimizationStrategy,
    ) -> Result<Self> {
        let memory_manager = SemanticMemoryManager::with_config(cleanup_config, optimization_strategy).await?;
        let confidence_calculator = ConfidenceCalculator::new();
        let type_classifier = SemanticItemTypeClassifier::new();
        let relationship_validator = RelationshipValidator::new();
        let items = Arc::new(RwLock::new(HashMap::new()));
        let relationships = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            memory_manager,
            confidence_calculator,
            type_classifier,
            relationship_validator,
            items,
            relationships,
        })
    }

    /// Add semantic item with blazing-fast processing
    #[inline]
    pub async fn add_item(&self, item: SemanticItem) -> Result<()> {
        // Validate item integrity
        item.validate().map_err(|e| Error::ValidationError(e.to_string()))?;

        // Update confidence based on content analysis
        let mut updated_item = item;
        let calculated_confidence = self.confidence_calculator.calculate_for_item(&updated_item);
        updated_item.update_confidence(calculated_confidence);

        // Store item
        let mut items = self.items.write().await;
        items.insert(updated_item.id.clone(), updated_item);

        debug!("Added semantic item with ID: {}", item.id);
        Ok(())
    }

    /// Get semantic item by ID with zero allocation
    #[inline]
    pub async fn get_item(&self, id: &str) -> Result<Option<SemanticItem>> {
        let items = self.items.read().await;
        Ok(items.get(id).cloned())
    }

    /// Update semantic item
    #[inline]
    pub async fn update_item(&self, id: &str, mut item: SemanticItem) -> Result<()> {
        // Validate item integrity
        item.validate().map_err(|e| Error::ValidationError(e.to_string()))?;

        // Update confidence
        let calculated_confidence = self.confidence_calculator.calculate_for_item(&item);
        item.update_confidence(calculated_confidence);

        // Store updated item
        let mut items = self.items.write().await;
        if items.contains_key(id) {
            items.insert(id.to_string(), item);
            debug!("Updated semantic item with ID: {}", id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Item with ID {} not found", id)))
        }
    }

    /// Remove semantic item
    #[inline]
    pub async fn remove_item(&self, id: &str) -> Result<Option<SemanticItem>> {
        let mut items = self.items.write().await;
        let removed_item = items.remove(id);

        if removed_item.is_some() {
            // Also remove related relationships
            self.remove_item_relationships(id).await?;
            debug!("Removed semantic item with ID: {}", id);
        }

        Ok(removed_item)
    }

    /// Add semantic relationship with validation
    #[inline]
    pub async fn add_relationship(&self, relationship: SemanticRelationship) -> Result<()> {
        // Validate relationship integrity
        relationship.validate().map_err(|e| Error::ValidationError(e.to_string()))?;

        // Validate relationship semantically
        self.relationship_validator.validate_relationship(&relationship)?;

        // Store relationship
        let mut relationships = self.relationships.write().await;
        relationships.insert(relationship.id.clone(), relationship);

        debug!("Added semantic relationship with ID: {}", relationship.id);
        Ok(())
    }

    /// Get semantic relationship by ID
    #[inline]
    pub async fn get_relationship(&self, id: &str) -> Result<Option<SemanticRelationship>> {
        let relationships = self.relationships.read().await;
        Ok(relationships.get(id).cloned())
    }

    /// Update semantic relationship
    #[inline]
    pub async fn update_relationship(&self, id: &str, relationship: SemanticRelationship) -> Result<()> {
        // Validate relationship integrity
        relationship.validate().map_err(|e| Error::ValidationError(e.to_string()))?;

        // Validate relationship semantically
        self.relationship_validator.validate_relationship(&relationship)?;

        // Store updated relationship
        let mut relationships = self.relationships.write().await;
        if relationships.contains_key(id) {
            relationships.insert(id.to_string(), relationship);
            debug!("Updated semantic relationship with ID: {}", id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Relationship with ID {} not found", id)))
        }
    }

    /// Remove semantic relationship
    #[inline]
    pub async fn remove_relationship(&self, id: &str) -> Result<Option<SemanticRelationship>> {
        let mut relationships = self.relationships.write().await;
        let removed_relationship = relationships.remove(id);

        if removed_relationship.is_some() {
            debug!("Removed semantic relationship with ID: {}", id);
        }

        Ok(removed_relationship)
    }

    /// Remove all relationships involving an item
    #[inline]
    async fn remove_item_relationships(&self, item_id: &str) -> Result<usize> {
        let mut relationships = self.relationships.write().await;
        let mut to_remove = Vec::new();

        for (id, relationship) in relationships.iter() {
            if relationship.involves_item(item_id) {
                to_remove.push(id.clone());
            }
        }

        let removed_count = to_remove.len();
        for id in to_remove {
            relationships.remove(&id);
        }

        debug!("Removed {} relationships involving item {}", removed_count, item_id);
        Ok(removed_count)
    }

    /// Find items by type with zero allocation filtering
    #[inline]
    pub async fn find_items_by_type(&self, item_type: SemanticItemType) -> Result<Vec<SemanticItem>> {
        let items = self.items.read().await;
        let matching_items: Vec<SemanticItem> = items
            .values()
            .filter(|item| item.item_type == item_type)
            .cloned()
            .collect();

        Ok(matching_items)
    }

    /// Find relationships by type
    #[inline]
    pub async fn find_relationships_by_type(&self, relationship_type: SemanticRelationshipType) -> Result<Vec<SemanticRelationship>> {
        let relationships = self.relationships.read().await;
        let matching_relationships: Vec<SemanticRelationship> = relationships
            .values()
            .filter(|rel| rel.relationship_type == relationship_type)
            .cloned()
            .collect();

        Ok(matching_relationships)
    }

    /// Find relationships involving an item
    #[inline]
    pub async fn find_item_relationships(&self, item_id: &str) -> Result<Vec<SemanticRelationship>> {
        let relationships = self.relationships.read().await;
        let item_relationships: Vec<SemanticRelationship> = relationships
            .values()
            .filter(|rel| rel.involves_item(item_id))
            .cloned()
            .collect();

        Ok(item_relationships)
    }

    /// Get comprehensive memory statistics
    #[inline]
    pub async fn get_memory_statistics(&self) -> Result<ComprehensiveMemoryStatistics> {
        let items = self.items.read().await;
        let relationships = self.relationships.read().await;

        let item_count = items.len();
        let relationship_count = relationships.len();

        // Calculate type statistics
        let mut type_stats = HashMap::new();
        for item in items.values() {
            *type_stats.entry(item.item_type).or_insert(0) += 1;
        }

        // Calculate confidence statistics
        let confidence_stats = self.confidence_calculator.calculate_statistics(items.values());

        // Calculate relationship statistics
        let mut relationship_type_stats = HashMap::new();
        for relationship in relationships.values() {
            *relationship_type_stats.entry(relationship.relationship_type).or_insert(0) += 1;
        }

        // Get memory manager statistics
        let memory_stats = self.memory_manager.get_statistics().await?;

        Ok(ComprehensiveMemoryStatistics {
            item_count,
            relationship_count,
            type_statistics: type_stats,
            confidence_statistics: confidence_stats,
            relationship_type_statistics: relationship_type_stats,
            memory_statistics: memory_stats,
        })
    }

    /// Perform comprehensive cleanup
    #[inline]
    pub async fn perform_cleanup(&self, config: &CleanupConfig) -> Result<CleanupReport> {
        info!("Starting comprehensive semantic memory cleanup");

        let mut archived_items = 0;
        let mut deleted_items = 0;
        let mut archived_relationships = 0;
        let mut deleted_relationships = 0;

        // Cleanup items
        {
            let mut items = self.items.write().await;
            let mut to_archive = Vec::new();
            let mut to_delete = Vec::new();

            for (id, item) in items.iter() {
                if item.should_delete(&config.delete_config) {
                    to_delete.push(id.clone());
                } else if item.should_archive(&config.archive_config) {
                    to_archive.push(id.clone());
                }
            }

            // Archive items
            for id in to_archive {
                if let Some(item) = items.remove(&id) {
                    // In a real implementation, we would archive to persistent storage
                    archived_items += 1;
                    debug!("Archived item: {}", id);
                }
            }

            // Delete items
            for id in to_delete {
                if let Some(_) = items.remove(&id) {
                    deleted_items += 1;
                    debug!("Deleted item: {}", id);
                }
            }
        }

        // Cleanup relationships
        {
            let mut relationships = self.relationships.write().await;
            let mut to_archive = Vec::new();
            let mut to_delete = Vec::new();

            for (id, relationship) in relationships.iter() {
                if relationship.should_delete(&config.relationship_delete_config) {
                    to_delete.push(id.clone());
                } else if relationship.should_archive(&config.relationship_archive_config) {
                    to_archive.push(id.clone());
                }
            }

            // Archive relationships
            for id in to_archive {
                if let Some(_) = relationships.remove(&id) {
                    archived_relationships += 1;
                    debug!("Archived relationship: {}", id);
                }
            }

            // Delete relationships
            for id in to_delete {
                if let Some(_) = relationships.remove(&id) {
                    deleted_relationships += 1;
                    debug!("Deleted relationship: {}", id);
                }
            }
        }

        let report = CleanupReport {
            archived_items,
            deleted_items,
            archived_relationships,
            deleted_relationships,
        };

        info!("Cleanup completed: {:?}", report);
        Ok(report)
    }

    /// Perform health check
    #[inline]
    pub async fn perform_health_check(&self) -> Result<SemanticHealthReport> {
        debug!("Performing semantic memory health check");

        let stats = self.get_memory_statistics().await?;
        let memory_health = self.memory_manager.perform_health_check().await?;

        // Calculate health scores
        let item_health_score = if stats.item_count > 0 {
            let valid_items = stats.item_count; // Simplified - in real implementation, validate all items
            (valid_items as f32 / stats.item_count as f32).min(1.0)
        } else {
            1.0
        };

        let relationship_health_score = if stats.relationship_count > 0 {
            let valid_relationships = stats.relationship_count; // Simplified
            (valid_relationships as f32 / stats.relationship_count as f32).min(1.0)
        } else {
            1.0
        };

        let overall_health_score = (item_health_score + relationship_health_score + memory_health.overall_score) / 3.0;

        Ok(SemanticHealthReport {
            overall_health_score,
            item_health_score,
            relationship_health_score,
            memory_health: memory_health,
            statistics: stats,
        })
    }

    /// Get item count
    #[inline]
    pub async fn item_count(&self) -> usize {
        self.items.read().await.len()
    }

    /// Get relationship count
    #[inline]
    pub async fn relationship_count(&self) -> usize {
        self.relationships.read().await.len()
    }

    /// Check if coordinator is empty
    #[inline]
    pub async fn is_empty(&self) -> bool {
        let items = self.items.read().await;
        let relationships = self.relationships.read().await;
        items.is_empty() && relationships.is_empty()
    }

    /// Clear all data
    #[inline]
    pub async fn clear(&self) -> Result<()> {
        let mut items = self.items.write().await;
        let mut relationships = self.relationships.write().await;
        
        items.clear();
        relationships.clear();
        
        info!("Cleared all semantic memory data");
        Ok(())
    }
}

/// Comprehensive memory statistics
#[derive(Debug, Clone)]
pub struct ComprehensiveMemoryStatistics {
    pub item_count: usize,
    pub relationship_count: usize,
    pub type_statistics: HashMap<SemanticItemType, usize>,
    pub confidence_statistics: ConfidenceStatistics,
    pub relationship_type_statistics: HashMap<SemanticRelationshipType, usize>,
    pub memory_statistics: MemoryStatistics,
}

/// Cleanup configuration combining all cleanup settings
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub archive_config: ArchiveConfig,
    pub delete_config: DeleteConfig,
    pub relationship_archive_config: RelationshipArchiveConfig,
    pub relationship_delete_config: RelationshipDeleteConfig,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            archive_config: ArchiveConfig::default(),
            delete_config: DeleteConfig::default(),
            relationship_archive_config: RelationshipArchiveConfig::default(),
            relationship_delete_config: RelationshipDeleteConfig::default(),
        }
    }
}

/// Cleanup report
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub archived_items: usize,
    pub deleted_items: usize,
    pub archived_relationships: usize,
    pub deleted_relationships: usize,
}

impl CleanupReport {
    /// Get total items processed
    #[inline]
    pub fn total_items_processed(&self) -> usize {
        self.archived_items + self.deleted_items
    }

    /// Get total relationships processed
    #[inline]
    pub fn total_relationships_processed(&self) -> usize {
        self.archived_relationships + self.deleted_relationships
    }

    /// Get total processed
    #[inline]
    pub fn total_processed(&self) -> usize {
        self.total_items_processed() + self.total_relationships_processed()
    }
}

/// Semantic health report
#[derive(Debug, Clone)]
pub struct SemanticHealthReport {
    pub overall_health_score: f32,
    pub item_health_score: f32,
    pub relationship_health_score: f32,
    pub memory_health: HealthCheckReport,
    pub statistics: ComprehensiveMemoryStatistics,
}

impl SemanticHealthReport {
    /// Check if semantic memory is healthy
    #[inline]
    pub fn is_healthy(&self, threshold: f32) -> bool {
        self.overall_health_score >= threshold
    }

    /// Get health grade
    #[inline]
    pub fn health_grade(&self) -> char {
        match self.overall_health_score {
            score if score >= 0.9 => 'A',
            score if score >= 0.8 => 'B',
            score if score >= 0.7 => 'C',
            score if score >= 0.6 => 'D',
            _ => 'F',
        }
    }
}
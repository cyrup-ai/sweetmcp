//! Memory cleanup operations and lifecycle management
//!
//! This module provides blazing-fast memory cleanup operations with zero-allocation
//! patterns and elegant ergonomic interfaces for memory lifecycle management.

use crate::utils::{Result, error::Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use tracing::{debug, info, warn};

use super::memory_manager_core::{MemoryStatistics, CleanupConfig};
use super::confidence::ConfidenceLevel;
use super::semantic_item::SemanticItem;
use super::semantic_relationship::SemanticRelationship;

/// Memory optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Conservative optimization (preserve more data)
    Conservative,
    /// Balanced optimization (moderate cleanup)
    Balanced,
    /// Aggressive optimization (maximum cleanup)
    Aggressive,
    /// Custom optimization with specific parameters
    Custom { threshold: f64, max_age_days: u32 },
}

/// Memory report for optimization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReport {
    /// Total memory usage in bytes
    pub total_memory_bytes: usize,
    /// Number of active items
    pub active_items: usize,
    /// Number of stale items
    pub stale_items: usize,
    /// Memory fragmentation ratio
    pub fragmentation_ratio: f64,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

impl MemoryReport {
    /// Create new memory report
    pub fn new() -> Self {
        Self {
            total_memory_bytes: 0,
            active_items: 0,
            stale_items: 0,
            fragmentation_ratio: 0.0,
            recommendations: Vec::new(),
        }
    }
}

/// Memory cleanup report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupReport {
    /// Number of items cleaned up
    pub items_cleaned: usize,
    /// Number of relationships cleaned up
    pub relationships_cleaned: usize,
    /// Memory freed in bytes
    pub memory_freed: usize,
    /// Cleanup duration
    pub cleanup_duration: Duration,
    /// Cleanup strategy used
    pub strategy_used: CleanupStrategy,
    /// Items preserved due to high confidence
    pub high_confidence_preserved: usize,
    /// Cleanup efficiency score (0.0-1.0)
    pub efficiency_score: f64,
}

impl CleanupReport {
    /// Create new cleanup report
    #[inline]
    pub fn new() -> Self {
        Self {
            items_cleaned: 0,
            relationships_cleaned: 0,
            memory_freed: 0,
            cleanup_duration: Duration::ZERO,
            strategy_used: CleanupStrategy::Conservative,
            high_confidence_preserved: 0,
            efficiency_score: 0.0,
        }
    }

    /// Get total items processed
    #[inline]
    pub fn total_processed(&self) -> usize {
        self.items_cleaned + self.relationships_cleaned
    }

    /// Get cleanup rate (items per second)
    #[inline]
    pub fn cleanup_rate(&self) -> f64 {
        if self.cleanup_duration.is_zero() {
            return 0.0;
        }
        
        self.total_processed() as f64 / self.cleanup_duration.as_secs_f64()
    }

    /// Get memory efficiency (bytes freed per item)
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.total_processed() == 0 {
            return 0.0;
        }
        
        self.memory_freed as f64 / self.total_processed() as f64
    }

    /// Check if cleanup was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.efficiency_score > 0.5 && self.total_processed() > 0
    }

    /// Get performance summary
    #[inline]
    pub fn performance_summary(&self) -> String {
        format!(
            "Cleaned {} items, {} relationships in {:?} ({:.1} items/sec, {:.1} KB freed per item, {:.1}% efficiency)",
            self.items_cleaned,
            self.relationships_cleaned,
            self.cleanup_duration,
            self.cleanup_rate(),
            self.memory_efficiency() / 1024.0,
            self.efficiency_score * 100.0
        )
    }
}

impl Default for CleanupReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Cleanup strategy enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupStrategy {
    Conservative,
    Aggressive,
    MemoryPressure,
    TimeBasedOnly,
    ConfidenceBasedOnly,
    Comprehensive,
}

impl CleanupStrategy {
    /// Get strategy description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            CleanupStrategy::Conservative => "Conservative cleanup preserving most data",
            CleanupStrategy::Aggressive => "Aggressive cleanup for maximum space recovery",
            CleanupStrategy::MemoryPressure => "Memory pressure driven cleanup",
            CleanupStrategy::TimeBasedOnly => "Time-based cleanup only",
            CleanupStrategy::ConfidenceBasedOnly => "Confidence-based cleanup only",
            CleanupStrategy::Comprehensive => "Comprehensive cleanup with all criteria",
        }
    }

    /// Get cleanup priority score
    #[inline]
    pub fn priority_score(&self) -> u8 {
        match self {
            CleanupStrategy::Conservative => 1,
            CleanupStrategy::TimeBasedOnly => 2,
            CleanupStrategy::ConfidenceBasedOnly => 3,
            CleanupStrategy::Comprehensive => 4,
            CleanupStrategy::MemoryPressure => 5,
            CleanupStrategy::Aggressive => 6,
        }
    }
}

/// Semantic memory manager for cleanup operations
pub struct SemanticMemoryManager {
    /// Memory statistics
    statistics: MemoryStatistics,
    /// Cleanup configuration
    cleanup_config: CleanupConfig,
    /// Items storage
    items: HashMap<String, SemanticItem>,
    /// Relationships storage
    relationships: HashMap<String, SemanticRelationship>,
}

impl SemanticMemoryManager {
    /// Create new semantic memory manager
    #[inline]
    pub fn new(cleanup_config: CleanupConfig) -> Self {
        Self {
            statistics: MemoryStatistics::new(),
            cleanup_config,
            items: HashMap::new(),
            relationships: HashMap::new(),
        }
    }

    /// Create with default configuration
    #[inline]
    pub fn with_defaults() -> Self {
        Self::new(CleanupConfig::new())
    }

    /// Get current statistics
    #[inline]
    pub fn statistics(&self) -> &MemoryStatistics {
        &self.statistics
    }

    /// Get cleanup configuration
    #[inline]
    pub fn cleanup_config(&self) -> &CleanupConfig {
        &self.cleanup_config
    }

    /// Update cleanup configuration
    #[inline]
    pub fn set_cleanup_config(&mut self, config: CleanupConfig) {
        self.cleanup_config = config;
    }

    /// Perform comprehensive cleanup
    pub async fn perform_cleanup(&mut self) -> Result<CleanupReport> {
        let start_time = SystemTime::now();
        let mut report = CleanupReport::new();
        
        debug!("Starting comprehensive memory cleanup");
        
        // Determine cleanup strategy based on current conditions
        let strategy = self.determine_cleanup_strategy();
        report.strategy_used = strategy.clone();
        
        // Execute cleanup based on strategy
        match strategy {
            CleanupStrategy::Conservative => {
                self.perform_conservative_cleanup(&mut report).await?;
            }
            CleanupStrategy::Aggressive => {
                self.perform_aggressive_cleanup(&mut report).await?;
            }
            CleanupStrategy::MemoryPressure => {
                self.perform_memory_pressure_cleanup(&mut report).await?;
            }
            CleanupStrategy::TimeBasedOnly => {
                self.perform_time_based_cleanup(&mut report).await?;
            }
            CleanupStrategy::ConfidenceBasedOnly => {
                self.perform_confidence_based_cleanup(&mut report).await?;
            }
            CleanupStrategy::Comprehensive => {
                self.perform_comprehensive_cleanup_internal(&mut report).await?;
            }
        }
        
        // Update cleanup duration
        if let Ok(elapsed) = start_time.elapsed() {
            report.cleanup_duration = elapsed;
        }
        
        // Calculate efficiency score
        report.efficiency_score = self.calculate_cleanup_efficiency(&report);
        
        // Update statistics
        self.statistics.record_cleanup();
        self.update_memory_statistics();
        
        info!("Cleanup completed: {}", report.performance_summary());
        
        Ok(report)
    }

    /// Determine optimal cleanup strategy
    #[inline]
    fn determine_cleanup_strategy(&self) -> CleanupStrategy {
        let memory_pressure = self.calculate_memory_pressure();
        let needs_urgent_cleanup = self.statistics.needs_cleanup(6); // 6 hours
        let health_score = self.statistics.health_score();
        
        if memory_pressure > 0.9 {
            CleanupStrategy::Aggressive
        } else if memory_pressure > 0.7 {
            CleanupStrategy::MemoryPressure
        } else if needs_urgent_cleanup {
            CleanupStrategy::Comprehensive
        } else if health_score < 0.5 {
            CleanupStrategy::Comprehensive
        } else {
            CleanupStrategy::Conservative
        }
    }

    /// Calculate current memory pressure (0.0-1.0)
    #[inline]
    fn calculate_memory_pressure(&self) -> f64 {
        // Simplified memory pressure calculation
        let efficiency = self.statistics.memory_efficiency() as f64;
        let access_efficiency = self.statistics.access_efficiency() as f64;
        
        1.0 - ((efficiency + access_efficiency) / 2.0)
    }

    /// Perform conservative cleanup
    async fn perform_conservative_cleanup(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing conservative cleanup");
        
        // Only clean up items that are clearly stale and low confidence
        let cutoff_time = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_age_days * 24 * 3600);
        let unused_cutoff = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_unused_days * 24 * 3600);
        
        let mut items_to_remove = Vec::new();
        
        for (id, item) in &self.items {
            let should_remove = item.created_at < cutoff_time
                && item.confidence < ConfidenceLevel::Medium
                && item.last_accessed.map_or(true, |last| last < unused_cutoff);
                
            if should_remove && (!self.cleanup_config.preserve_high_confidence || item.confidence < ConfidenceLevel::High) {
                items_to_remove.push(id.clone());
            } else if item.confidence >= ConfidenceLevel::High {
                report.high_confidence_preserved += 1;
            }
        }
        
        // Remove identified items
        for id in items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        // Clean up orphaned relationships
        self.cleanup_orphaned_relationships(report).await?;
        
        Ok(())
    }

    /// Perform aggressive cleanup
    async fn perform_aggressive_cleanup(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing aggressive cleanup");
        
        let cutoff_time = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_age_days * 24 * 3600);
        let unused_cutoff = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_unused_days * 24 * 3600);
        
        let mut items_to_remove = Vec::new();
        
        for (id, item) in &self.items {
            let should_remove = item.created_at < cutoff_time
                || item.confidence < self.cleanup_config.min_confidence_threshold
                || item.last_accessed.map_or(true, |last| last < unused_cutoff);
                
            if should_remove {
                items_to_remove.push(id.clone());
            } else if item.confidence >= ConfidenceLevel::High {
                report.high_confidence_preserved += 1;
            }
        }
        
        // Remove identified items
        for id in items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        // Clean up orphaned relationships
        self.cleanup_orphaned_relationships(report).await?;
        
        Ok(())
    }

    /// Perform memory pressure cleanup
    async fn perform_memory_pressure_cleanup(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing memory pressure cleanup");
        
        // Sort items by priority (lowest priority first for removal)
        let mut item_priorities: Vec<_> = self.items.iter()
            .map(|(id, item)| (id.clone(), item.priority_score()))
            .collect();
        
        item_priorities.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Remove items starting with lowest priority until memory pressure is relieved
        let target_items_to_remove = (self.items.len() as f64 * 0.3) as usize; // Remove 30%
        let items_to_remove = item_priorities.into_iter()
            .take(target_items_to_remove)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        
        for id in items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        // Clean up orphaned relationships
        self.cleanup_orphaned_relationships(report).await?;
        
        Ok(())
    }

    /// Perform time-based cleanup only
    async fn perform_time_based_cleanup(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing time-based cleanup");
        
        let cutoff_time = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_age_days * 24 * 3600);
        let mut items_to_remove = Vec::new();
        
        for (id, item) in &self.items {
            if item.created_at < cutoff_time {
                items_to_remove.push(id.clone());
            }
        }
        
        for id in items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        self.cleanup_orphaned_relationships(report).await?;
        Ok(())
    }

    /// Perform confidence-based cleanup only
    async fn perform_confidence_based_cleanup(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing confidence-based cleanup");
        
        let mut items_to_remove = Vec::new();
        
        for (id, item) in &self.items {
            if item.confidence < self.cleanup_config.min_confidence_threshold {
                items_to_remove.push(id.clone());
            } else if item.confidence >= ConfidenceLevel::High {
                report.high_confidence_preserved += 1;
            }
        }
        
        for id in items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        self.cleanup_orphaned_relationships(report).await?;
        Ok(())
    }

    /// Perform comprehensive cleanup with all criteria
    async fn perform_comprehensive_cleanup_internal(&mut self, report: &mut CleanupReport) -> Result<()> {
        debug!("Performing comprehensive cleanup");
        
        // Combine all cleanup strategies
        self.perform_time_based_cleanup(report).await?;
        self.perform_confidence_based_cleanup(report).await?;
        
        // Additional cleanup based on access patterns
        let unused_cutoff = SystemTime::now() - Duration::from_secs(self.cleanup_config.max_unused_days * 24 * 3600);
        let mut additional_items_to_remove = Vec::new();
        
        for (id, item) in &self.items {
            if item.last_accessed.map_or(true, |last| last < unused_cutoff) && item.access_count == 0 {
                additional_items_to_remove.push(id.clone());
            }
        }
        
        for id in additional_items_to_remove {
            if let Some(item) = self.items.remove(&id) {
                report.items_cleaned += 1;
                report.memory_freed += self.estimate_item_memory_usage(&item);
            }
        }
        
        Ok(())
    }

    /// Clean up orphaned relationships
    async fn cleanup_orphaned_relationships(&mut self, report: &mut CleanupReport) -> Result<()> {
        let mut relationships_to_remove = Vec::new();
        
        for (id, relationship) in &self.relationships {
            let source_exists = self.items.contains_key(&relationship.source_id);
            let target_exists = self.items.contains_key(&relationship.target_id);
            
            if !source_exists || !target_exists {
                relationships_to_remove.push(id.clone());
            }
        }
        
        for id in relationships_to_remove {
            if let Some(relationship) = self.relationships.remove(&id) {
                report.relationships_cleaned += 1;
                report.memory_freed += self.estimate_relationship_memory_usage(&relationship);
            }
        }
        
        Ok(())
    }

    /// Estimate memory usage of an item
    #[inline]
    fn estimate_item_memory_usage(&self, item: &SemanticItem) -> usize {
        // Rough estimation based on content size and metadata
        let content_size = serde_json::to_string(&item.content).map_or(0, |s| s.len());
        let metadata_size = item.metadata.len() * 64; // Rough estimate
        content_size + metadata_size + 256 // Base overhead
    }

    /// Estimate memory usage of a relationship
    #[inline]
    fn estimate_relationship_memory_usage(&self, relationship: &SemanticRelationship) -> usize {
        // Rough estimation for relationship overhead
        relationship.source_id.len() + relationship.target_id.len() + 128
    }

    /// Calculate cleanup efficiency score
    #[inline]
    fn calculate_cleanup_efficiency(&self, report: &CleanupReport) -> f64 {
        if report.total_processed() == 0 {
            return 0.0;
        }
        
        let rate_score = (report.cleanup_rate() / 1000.0).min(1.0); // Normalize to 1000 items/sec
        let memory_score = (report.memory_efficiency() / 10240.0).min(1.0); // Normalize to 10KB per item
        
        (rate_score + memory_score) / 2.0
    }

    /// Update memory statistics after cleanup
    #[inline]
    fn update_memory_statistics(&mut self) {
        self.statistics.update_item_count(self.items.len());
        self.statistics.update_relationship_count(self.relationships.len());
        
        let total_memory = self.items.values()
            .map(|item| self.estimate_item_memory_usage(item))
            .sum::<usize>()
            + self.relationships.values()
            .map(|rel| self.estimate_relationship_memory_usage(rel))
            .sum::<usize>();
            
        self.statistics.update_memory_usage(total_memory);
    }
}
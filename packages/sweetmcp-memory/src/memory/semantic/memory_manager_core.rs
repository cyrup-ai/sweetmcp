//! Core memory management structures and statistics
//!
//! This module provides the core memory management functionality with zero-allocation
//! patterns and blazing-fast performance for semantic memory lifecycle operations.

use crate::utils::{Result, error::Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Memory management statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_items: usize,
    pub total_relationships: usize,
    pub memory_usage_bytes: usize,
    pub last_cleanup_timestamp: Option<SystemTime>,
    pub cleanup_count: usize,
    pub access_patterns: HashMap<String, usize>,
    pub optimization_count: usize,
    pub defragmentation_count: usize,
    pub cache_hit_ratio: f64,
    pub average_access_time_ms: f64,
}

impl MemoryStatistics {
    /// Create new statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_items: 0,
            total_relationships: 0,
            memory_usage_bytes: 0,
            last_cleanup_timestamp: None,
            cleanup_count: 0,
            access_patterns: HashMap::new(),
            optimization_count: 0,
            defragmentation_count: 0,
            cache_hit_ratio: 0.0,
            average_access_time_ms: 0.0,
        }
    }

    /// Update item count
    #[inline]
    pub fn update_item_count(&mut self, count: usize) {
        self.total_items = count;
    }

    /// Update relationship count
    #[inline]
    pub fn update_relationship_count(&mut self, count: usize) {
        self.total_relationships = count;
    }

    /// Update memory usage
    #[inline]
    pub fn update_memory_usage(&mut self, bytes: usize) {
        self.memory_usage_bytes = bytes;
    }

    /// Record cleanup operation
    #[inline]
    pub fn record_cleanup(&mut self) {
        self.last_cleanup_timestamp = Some(SystemTime::now());
        self.cleanup_count += 1;
    }

    /// Record optimization operation
    #[inline]
    pub fn record_optimization(&mut self) {
        self.optimization_count += 1;
    }

    /// Record defragmentation operation
    #[inline]
    pub fn record_defragmentation(&mut self) {
        self.defragmentation_count += 1;
    }

    /// Record access pattern
    #[inline]
    pub fn record_access(&mut self, pattern: String) {
        *self.access_patterns.entry(pattern).or_insert(0) += 1;
    }

    /// Update cache hit ratio
    #[inline]
    pub fn update_cache_hit_ratio(&mut self, ratio: f64) {
        self.cache_hit_ratio = ratio.clamp(0.0, 1.0);
    }

    /// Update average access time
    #[inline]
    pub fn update_average_access_time(&mut self, time_ms: f64) {
        self.average_access_time_ms = time_ms.max(0.0);
    }

    /// Get memory efficiency ratio
    #[inline]
    pub fn memory_efficiency(&self) -> f32 {
        if self.memory_usage_bytes == 0 {
            return 1.0;
        }
        
        let total_items = self.total_items + self.total_relationships;
        if total_items == 0 {
            return 0.0;
        }

        // Calculate bytes per item (lower is better)
        let bytes_per_item = self.memory_usage_bytes as f32 / total_items as f32;
        
        // Normalize to 0-1 scale (assuming 1KB per item is baseline)
        (1024.0 / (bytes_per_item + 1.0)).min(1.0)
    }

    /// Get access efficiency score
    #[inline]
    pub fn access_efficiency(&self) -> f32 {
        // Combine cache hit ratio and access time for efficiency score
        let cache_score = self.cache_hit_ratio as f32;
        let time_score = if self.average_access_time_ms > 0.0 {
            (100.0 / (self.average_access_time_ms as f32 + 1.0)).min(1.0)
        } else {
            1.0
        };
        
        (cache_score + time_score) / 2.0
    }

    /// Get overall health score
    #[inline]
    pub fn health_score(&self) -> f32 {
        let memory_score = self.memory_efficiency();
        let access_score = self.access_efficiency();
        let maintenance_score = if self.cleanup_count > 0 { 1.0 } else { 0.5 };
        
        (memory_score + access_score + maintenance_score) / 3.0
    }

    /// Check if cleanup is needed
    #[inline]
    pub fn needs_cleanup(&self, cleanup_threshold_hours: u64) -> bool {
        if let Some(last_cleanup) = self.last_cleanup_timestamp {
            if let Ok(elapsed) = last_cleanup.elapsed() {
                return elapsed.as_secs() > cleanup_threshold_hours * 3600;
            }
        }
        true // No previous cleanup recorded
    }

    /// Get cache hit rate (method interface for existing cache_hit_ratio field)
    #[inline]
    pub fn cache_hit_rate(&self) -> Option<f64> {
        Some(self.cache_hit_ratio)
    }

    /// Get memory usage percentage (calculated from existing memory_usage_bytes)
    #[inline]
    pub fn memory_usage_percent(&self) -> Option<f64> {
        // Calculate percentage based on total items and average memory per item
        let total_items = self.total_items + self.total_relationships;
        if total_items == 0 {
            return Some(0.0);
        }
        
        // Estimate percentage based on bytes per item (assuming 1KB baseline per item)
        let bytes_per_item = self.memory_usage_bytes as f64 / total_items as f64;
        let usage_percent = (bytes_per_item / 1024.0 * 100.0).min(100.0);
        Some(usage_percent)
    }

    /// Check if optimization is needed
    #[inline]
    pub fn needs_optimization(&self) -> bool {
        self.memory_efficiency() < 0.7 || self.access_efficiency() < 0.6
    }

    /// Get most frequent access patterns
    #[inline]
    pub fn top_access_patterns(&self, limit: usize) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self.access_patterns.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns.into_iter().take(limit).collect()
    }

    /// Get total operations count
    #[inline]
    pub fn total_operations(&self) -> usize {
        self.cleanup_count + self.optimization_count + self.defragmentation_count
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory cleanup configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub max_age_days: u64,
    pub min_confidence_threshold: super::confidence::ConfidenceLevel,
    pub max_unused_days: u64,
    pub preserve_high_confidence: bool,
    pub batch_size: usize,
    pub enable_parallel_cleanup: bool,
    pub memory_pressure_threshold: f64,
    pub cleanup_interval_hours: u64,
}

impl CleanupConfig {
    /// Create new cleanup configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            max_age_days: 365, // 1 year
            min_confidence_threshold: super::confidence::ConfidenceLevel::Low,
            max_unused_days: 90, // 3 months
            preserve_high_confidence: true,
            batch_size: 1000,
            enable_parallel_cleanup: true,
            memory_pressure_threshold: 0.8,
            cleanup_interval_hours: 24,
        }
    }

    /// Create aggressive cleanup configuration
    #[inline]
    pub fn aggressive() -> Self {
        Self {
            max_age_days: 30,
            min_confidence_threshold: super::confidence::ConfidenceLevel::Medium,
            max_unused_days: 7,
            preserve_high_confidence: false,
            batch_size: 500,
            enable_parallel_cleanup: true,
            memory_pressure_threshold: 0.6,
            cleanup_interval_hours: 6,
        }
    }

    /// Create conservative cleanup configuration
    #[inline]
    pub fn conservative() -> Self {
        Self {
            max_age_days: 730, // 2 years
            min_confidence_threshold: super::confidence::ConfidenceLevel::VeryLow,
            max_unused_days: 180, // 6 months
            preserve_high_confidence: true,
            batch_size: 2000,
            enable_parallel_cleanup: false,
            memory_pressure_threshold: 0.9,
            cleanup_interval_hours: 72,
        }
    }

    /// Create memory-pressure-driven configuration
    #[inline]
    pub fn memory_pressure() -> Self {
        Self {
            max_age_days: 60,
            min_confidence_threshold: super::confidence::ConfidenceLevel::Medium,
            max_unused_days: 14,
            preserve_high_confidence: true,
            batch_size: 1500,
            enable_parallel_cleanup: true,
            memory_pressure_threshold: 0.5,
            cleanup_interval_hours: 2,
        }
    }

    /// Check if configuration is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max_age_days > 0
            && self.max_unused_days > 0
            && self.batch_size > 0
            && self.memory_pressure_threshold >= 0.0
            && self.memory_pressure_threshold <= 1.0
            && self.cleanup_interval_hours > 0
    }

    /// Get effective batch size based on memory pressure
    #[inline]
    pub fn effective_batch_size(&self, current_memory_pressure: f64) -> usize {
        if current_memory_pressure > self.memory_pressure_threshold {
            // Increase batch size under memory pressure
            (self.batch_size as f64 * (1.0 + current_memory_pressure)).min(self.batch_size as f64 * 2.0) as usize
        } else {
            self.batch_size
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization strategy configuration
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    pub enable_defragmentation: bool,
    pub enable_compression: bool,
    pub enable_caching: bool,
    pub cache_size_limit: usize,
    pub defrag_threshold: f64,
    pub compression_threshold: usize,
    pub optimization_interval_hours: u64,
}

impl OptimizationStrategy {
    /// Create new optimization strategy
    #[inline]
    pub fn new() -> Self {
        Self {
            enable_defragmentation: true,
            enable_compression: true,
            enable_caching: true,
            cache_size_limit: 100 * 1024 * 1024, // 100MB
            defrag_threshold: 0.3,
            compression_threshold: 1024, // 1KB
            optimization_interval_hours: 12,
        }
    }

    /// Create performance-focused strategy
    #[inline]
    pub fn performance_focused() -> Self {
        Self {
            enable_defragmentation: true,
            enable_compression: false,
            enable_caching: true,
            cache_size_limit: 500 * 1024 * 1024, // 500MB
            defrag_threshold: 0.2,
            compression_threshold: usize::MAX,
            optimization_interval_hours: 6,
        }
    }

    /// Create memory-focused strategy
    #[inline]
    pub fn memory_focused() -> Self {
        Self {
            enable_defragmentation: true,
            enable_compression: true,
            enable_caching: false,
            cache_size_limit: 10 * 1024 * 1024, // 10MB
            defrag_threshold: 0.1,
            compression_threshold: 512, // 512B
            optimization_interval_hours: 4,
        }
    }

    /// Check if defragmentation should be performed
    #[inline]
    pub fn should_defragment(&self, fragmentation_ratio: f64) -> bool {
        self.enable_defragmentation && fragmentation_ratio > self.defrag_threshold
    }

    /// Check if compression should be applied
    #[inline]
    pub fn should_compress(&self, item_size: usize) -> bool {
        self.enable_compression && item_size >= self.compression_threshold
    }

    /// Check if caching should be used
    #[inline]
    pub fn should_cache(&self, current_cache_size: usize) -> bool {
        self.enable_caching && current_cache_size < self.cache_size_limit
    }
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        Self::new()
    }
}
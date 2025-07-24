//! Memory statistics tracking with zero-allocation atomic operations
//!
//! This module provides blazing-fast memory statistics tracking using completely
//! lock-free atomic operations and zero-allocation patterns for optimal performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::SystemTime;

/// Completely lock-free memory statistics with atomic operations
#[derive(Debug)]
pub struct AtomicMemoryStatistics {
    total_items: AtomicUsize,
    total_relationships: AtomicUsize,
    memory_usage_bytes: AtomicUsize,
    cleanup_count: AtomicUsize,
    last_cleanup_timestamp: AtomicU64,
    total_access_count: AtomicUsize,
    unique_pattern_count: AtomicUsize,
}

impl AtomicMemoryStatistics {
    /// Create new atomic statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_items: AtomicUsize::new(0),
            total_relationships: AtomicUsize::new(0),
            memory_usage_bytes: AtomicUsize::new(0),
            cleanup_count: AtomicUsize::new(0),
            last_cleanup_timestamp: AtomicU64::new(0),
            total_access_count: AtomicUsize::new(0),
            unique_pattern_count: AtomicUsize::new(0),
        }
    }

    /// Update item count atomically
    #[inline]
    pub fn update_item_count(&self, count: usize) {
        self.total_items.store(count, Ordering::Relaxed);
    }

    /// Update relationship count atomically
    #[inline]
    pub fn update_relationship_count(&self, count: usize) {
        self.total_relationships.store(count, Ordering::Relaxed);
    }

    /// Update memory usage atomically
    #[inline]
    pub fn update_memory_usage(&self, bytes: usize) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    /// Record cleanup operation atomically
    #[inline]
    pub fn record_cleanup(&self) {
        self.cleanup_count.fetch_add(1, Ordering::Relaxed);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_cleanup_timestamp.store(now, Ordering::Relaxed);
    }

    /// Record access pattern (simplified to avoid complex synchronization)
    #[inline]
    pub fn record_access(&self) {
        self.total_access_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record unique access pattern
    #[inline]
    pub fn record_unique_pattern(&self) {
        self.unique_pattern_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current item count
    #[inline]
    pub fn get_item_count(&self) -> usize {
        self.total_items.load(Ordering::Relaxed)
    }

    /// Get current relationship count
    #[inline]
    pub fn get_relationship_count(&self) -> usize {
        self.total_relationships.load(Ordering::Relaxed)
    }

    /// Get current memory usage
    #[inline]
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage_bytes.load(Ordering::Relaxed)
    }

    /// Get cleanup count
    #[inline]
    pub fn get_cleanup_count(&self) -> usize {
        self.cleanup_count.load(Ordering::Relaxed)
    }

    /// Get total access count
    #[inline]
    pub fn get_total_access_count(&self) -> usize {
        self.total_access_count.load(Ordering::Relaxed)
    }

    /// Get unique pattern count
    #[inline]
    pub fn get_unique_pattern_count(&self) -> usize {
        self.unique_pattern_count.load(Ordering::Relaxed)
    }

    /// Get last cleanup timestamp
    #[inline]
    pub fn get_last_cleanup_timestamp(&self) -> Option<SystemTime> {
        let timestamp = self.last_cleanup_timestamp.load(Ordering::Relaxed);
        if timestamp == 0 {
            None
        } else {
            SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp))
        }
    }

    /// Get memory efficiency ratio with zero allocation
    #[inline]
    pub fn memory_efficiency(&self) -> f32 {
        let memory_usage = self.memory_usage_bytes.load(Ordering::Relaxed);
        if memory_usage == 0 {
            return 1.0;
        }
        
        let total_items = self.total_items.load(Ordering::Relaxed) + 
                         self.total_relationships.load(Ordering::Relaxed);
        if total_items == 0 {
            return 0.0;
        }

        // Calculate bytes per item (lower is better)
        let bytes_per_item = memory_usage as f32 / total_items as f32;
        
        // Normalize to 0-1 scale (assuming 1KB per item is baseline)
        (1024.0 / (bytes_per_item + 1.0)).min(1.0)
    }

    /// Check if cleanup is needed based on time threshold
    #[inline]
    pub fn needs_cleanup(&self, cleanup_threshold_hours: u64) -> bool {
        let last_cleanup = self.last_cleanup_timestamp.load(Ordering::Relaxed);
        if last_cleanup == 0 {
            return true; // No previous cleanup recorded
        }

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let elapsed_hours = (now.saturating_sub(last_cleanup)) / 3600;
        elapsed_hours > cleanup_threshold_hours
    }

    /// Create snapshot of current statistics
    #[inline]
    pub fn create_snapshot(&self) -> MemoryStatistics {
        MemoryStatistics {
            total_items: self.total_items.load(Ordering::Relaxed),
            total_relationships: self.total_relationships.load(Ordering::Relaxed),
            memory_usage_bytes: self.memory_usage_bytes.load(Ordering::Relaxed),
            last_cleanup_timestamp: self.get_last_cleanup_timestamp(),
            cleanup_count: self.cleanup_count.load(Ordering::Relaxed),
            total_access_count: self.total_access_count.load(Ordering::Relaxed),
            unique_pattern_count: self.unique_pattern_count.load(Ordering::Relaxed),
        }
    }

    /// Increment item count atomically
    #[inline]
    pub fn increment_items(&self, count: usize) -> usize {
        self.total_items.fetch_add(count, Ordering::Relaxed) + count
    }

    /// Decrement item count atomically
    #[inline]
    pub fn decrement_items(&self, count: usize) -> usize {
        self.total_items.fetch_sub(count, Ordering::Relaxed).saturating_sub(count)
    }

    /// Increment relationship count atomically
    #[inline]
    pub fn increment_relationships(&self, count: usize) -> usize {
        self.total_relationships.fetch_add(count, Ordering::Relaxed) + count
    }

    /// Decrement relationship count atomically
    #[inline]
    pub fn decrement_relationships(&self, count: usize) -> usize {
        self.total_relationships.fetch_sub(count, Ordering::Relaxed).saturating_sub(count)
    }

    /// Add to memory usage atomically
    #[inline]
    pub fn add_memory_usage(&self, bytes: usize) -> usize {
        self.memory_usage_bytes.fetch_add(bytes, Ordering::Relaxed) + bytes
    }

    /// Subtract from memory usage atomically
    #[inline]
    pub fn subtract_memory_usage(&self, bytes: usize) -> usize {
        self.memory_usage_bytes.fetch_sub(bytes, Ordering::Relaxed).saturating_sub(bytes)
    }

    /// Reset all statistics
    #[inline]
    pub fn reset(&self) {
        self.total_items.store(0, Ordering::Relaxed);
        self.total_relationships.store(0, Ordering::Relaxed);
        self.memory_usage_bytes.store(0, Ordering::Relaxed);
        self.cleanup_count.store(0, Ordering::Relaxed);
        self.last_cleanup_timestamp.store(0, Ordering::Relaxed);
        self.total_access_count.store(0, Ordering::Relaxed);
        self.unique_pattern_count.store(0, Ordering::Relaxed);
    }

    /// Get access efficiency ratio
    #[inline]
    pub fn access_efficiency(&self) -> f32 {
        let total_accesses = self.total_access_count.load(Ordering::Relaxed);
        let unique_patterns = self.unique_pattern_count.load(Ordering::Relaxed);
        
        if unique_patterns == 0 {
            return 1.0;
        }
        
        let ratio = total_accesses as f32 / unique_patterns as f32;
        (ratio / 10.0).min(1.0) // Normalize assuming 10 accesses per pattern is good
    }
}

impl Default for AtomicMemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable memory statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_items: usize,
    pub total_relationships: usize,
    pub memory_usage_bytes: usize,
    pub last_cleanup_timestamp: Option<SystemTime>,
    pub cleanup_count: usize,
    pub total_access_count: usize,
    pub unique_pattern_count: usize,
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
            total_access_count: 0,
            unique_pattern_count: 0,
        }
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

    /// Get access efficiency ratio
    #[inline]
    pub fn access_efficiency(&self) -> f32 {
        if self.unique_pattern_count == 0 {
            return 1.0;
        }
        
        let ratio = self.total_access_count as f32 / self.unique_pattern_count as f32;
        (ratio / 10.0).min(1.0) // Normalize assuming 10 accesses per pattern is good
    }

    /// Calculate total entity count
    #[inline]
    pub fn total_entity_count(&self) -> usize {
        self.total_items + self.total_relationships
    }

    /// Calculate memory per entity
    #[inline]
    pub fn memory_per_entity(&self) -> f32 {
        let total_entities = self.total_entity_count();
        if total_entities == 0 {
            0.0
        } else {
            self.memory_usage_bytes as f32 / total_entities as f32
        }
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// High-performance statistics aggregator
pub struct StatisticsAggregator;

impl StatisticsAggregator {
    /// Calculate statistics summary with zero allocation
    #[inline]
    pub fn calculate_summary(stats: &AtomicMemoryStatistics) -> StatisticsSummary {
        let item_count = stats.get_item_count();
        let relationship_count = stats.get_relationship_count();
        let memory_usage = stats.get_memory_usage();
        let cleanup_count = stats.get_cleanup_count();
        
        let total_entities = item_count + relationship_count;
        let memory_per_entity = if total_entities > 0 {
            memory_usage as f32 / total_entities as f32
        } else {
            0.0
        };

        StatisticsSummary {
            total_entities,
            memory_per_entity,
            efficiency_score: stats.memory_efficiency(),
            cleanup_frequency: cleanup_count,
            access_efficiency: stats.access_efficiency(),
            needs_optimization: memory_per_entity > 2048.0 || stats.memory_efficiency() < 0.5,
        }
    }

    /// Compare two statistics snapshots
    #[inline]
    pub fn compare_snapshots(before: &MemoryStatistics, after: &MemoryStatistics) -> StatisticsComparison {
        let item_delta = after.total_items as i64 - before.total_items as i64;
        let relationship_delta = after.total_relationships as i64 - before.total_relationships as i64;
        let memory_delta = after.memory_usage_bytes as i64 - before.memory_usage_bytes as i64;
        let efficiency_delta = after.memory_efficiency() - before.memory_efficiency();
        let access_delta = after.total_access_count as i64 - before.total_access_count as i64;

        StatisticsComparison {
            item_delta,
            relationship_delta,
            memory_delta,
            efficiency_delta,
            access_delta,
            improvement_detected: efficiency_delta > 0.01 && memory_delta <= 0,
        }
    }
}

/// Statistics summary for quick analysis
#[derive(Debug, Clone)]
pub struct StatisticsSummary {
    pub total_entities: usize,
    pub memory_per_entity: f32,
    pub efficiency_score: f32,
    pub cleanup_frequency: usize,
    pub access_efficiency: f32,
    pub needs_optimization: bool,
}

/// Statistics comparison result
#[derive(Debug, Clone)]
pub struct StatisticsComparison {
    pub item_delta: i64,
    pub relationship_delta: i64,
    pub memory_delta: i64,
    pub efficiency_delta: f32,
    pub access_delta: i64,
    pub improvement_detected: bool,
}
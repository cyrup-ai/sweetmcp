//! Atomic statistics operations with zero-allocation patterns
//!
//! This module provides blazing-fast atomic statistics tracking using completely
//! lock-free operations for optimal performance.

use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::SystemTime;
use super::memory_stats::MemoryStatistics;

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

    /// Get total entity count
    #[inline]
    pub fn get_total_entity_count(&self) -> usize {
        self.total_items.load(Ordering::Relaxed) + self.total_relationships.load(Ordering::Relaxed)
    }

    /// Get memory per entity ratio
    #[inline]
    pub fn get_memory_per_entity(&self) -> f32 {
        let total_entities = self.get_total_entity_count();
        let memory_usage = self.memory_usage_bytes.load(Ordering::Relaxed);
        
        if total_entities == 0 {
            0.0
        } else {
            memory_usage as f32 / total_entities as f32
        }
    }

    /// Check if optimization is needed
    #[inline]
    pub fn needs_optimization(&self) -> bool {
        self.get_memory_per_entity() > 2048.0 || self.memory_efficiency() < 0.5
    }
}

impl Default for AtomicMemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}
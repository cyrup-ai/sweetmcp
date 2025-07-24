//! Memory statistics and performance metrics
//!
//! This module provides structures and functions for tracking and analyzing
//! memory usage patterns and performance metrics with zero-allocation optimizations.

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
    pub fn record_cleanup(&mut self) {
        self.cleanup_count += 1;
        self.last_cleanup_timestamp = Some(SystemTime::now());
    }

    /// Record optimization operation
    pub fn record_optimization(&mut self) {
        self.optimization_count += 1;
    }

    /// Record defragmentation operation
    pub fn record_defragmentation(&mut self) {
        self.defragmentation_count += 1;
    }

    /// Record access pattern
    pub fn record_access(&mut self, pattern: String) {
        *self.access_patterns.entry(pattern).or_insert(0) += 1;
    }

    /// Update cache hit ratio
    pub fn update_cache_hit_ratio(&mut self, ratio: f64) {
        self.cache_hit_ratio = ratio.clamp(0.0, 1.0);
    }

    /// Update average access time
    pub fn update_average_access_time(&mut self, time_ms: f64) {
        self.average_access_time_ms = time_ms.max(0.0);
    }

    /// Get memory efficiency ratio
    pub fn memory_efficiency(&self) -> f32 {
        // Calculate a score based on memory usage patterns
        let mut score = 0.0f32;
        
        // Higher is better (more efficient)
        if self.memory_usage_bytes > 0 {
            // Favor lower memory usage relative to items
            let bytes_per_item = self.memory_usage_bytes as f32 / (self.total_items.max(1) as f32);
            score += 100.0 / (1.0 + bytes_per_item.ln());
        }
        
        // Normalize to 0-100 range
        (score * 0.5).min(100.0).max(0.0)
    }

    /// Get access efficiency score
    pub fn access_efficiency(&self) -> f32 {
        // Calculate a score based on access patterns and cache performance
        let mut score = 0.0f32;
        
        // Cache hit ratio (higher is better)
        score += (self.cache_hit_ratio as f32) * 40.0;
        
        // Lower average access time is better
        if self.average_access_time_ms > 0.0 {
            score += 30.0 / (1.0 + (self.average_access_time_ms as f32).ln());
        }
        
        // Consider access pattern distribution
        let total_accesses: usize = self.access_patterns.values().sum();
        if total_accesses > 0 {
            let unique_patterns = self.access_patterns.len() as f32;
            // Favor having some access patterns but not too many (balance between
            // too fragmented and too uniform)
            let pattern_score = 1.0 - (1.0 / (1.0 + unique_patterns.ln()));
            score += pattern_score * 30.0;
        }
        
        score.min(100.0).max(0.0)
    }

    /// Get overall health score (0-100)
    pub fn health_score(&self) -> f32 {
        let memory_score = self.memory_efficiency();
        let access_score = self.access_efficiency();
        
        // Weighted average favoring memory efficiency slightly more
        (memory_score * 0.6) + (access_score * 0.4)
    }

    /// Check if cleanup is needed
    pub fn needs_cleanup(&self, cleanup_threshold_hours: u64) -> bool {
        match self.last_cleanup_timestamp {
            Some(last_cleanup) => {
                let since_cleanup = last_cleanup.elapsed()
                    .map(|d| d.as_secs() / 3600) // Convert to hours
                    .unwrap_or(0);
                since_cleanup >= cleanup_threshold_hours
            }
            None => true, // Never cleaned up before
        }
    }

    /// Get cache hit rate (method interface for existing cache_hit_ratio field)
    pub fn cache_hit_rate(&self) -> Option<f64> {
        if self.cache_hit_ratio > 0.0 {
            Some(self.cache_hit_ratio)
        } else {
            None
        }
    }

    /// Get memory usage percentage (based on a theoretical max or system memory)
    pub fn memory_usage_percent(&self) -> Option<f64> {
        // This is a simplified version - in a real implementation, you'd get the system's total memory
        const THEORETICAL_MAX_MB: usize = 16 * 1024; // 16GB
        const BYTES_PER_MB: usize = 1024 * 1024;
        
        if self.memory_usage_bytes == 0 {
            return None;
        }
        
        let usage_mb = self.memory_usage_bytes as f64 / BYTES_PER_MB as f64;
        let percent = (usage_mb / THEORETICAL_MAX_MB as f64) * 100.0;
        
        Some(percent.min(100.0).max(0.0))
    }

    /// Check if optimization is needed
    pub fn needs_optimization(&self) -> bool {
        // Simple heuristic: optimize if memory efficiency is low or access patterns are poor
        self.memory_efficiency() < 70.0 || self.access_efficiency() < 60.0
    }

    /// Get most frequent access patterns
    pub fn top_access_patterns(&self, limit: usize) -> Vec<(String, usize)> {
        let mut patterns: Vec<_> = self.access_patterns.iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
            
        patterns.sort_by(|a, b| b.1.cmp(&a.1));
        patterns.into_iter().take(limit).collect()
    }

    /// Get total operations count
    pub fn total_operations(&self) -> usize {
        self.cleanup_count + self.optimization_count + self.defragmentation_count
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

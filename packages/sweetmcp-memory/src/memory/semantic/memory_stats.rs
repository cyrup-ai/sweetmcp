//! Memory statistics data structures
//!
//! This module provides serializable memory statistics with blazing-fast
//! computation patterns and zero-allocation operations.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

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

    /// Calculate relationship ratio
    #[inline]
    pub fn relationship_ratio(&self) -> f32 {
        if self.total_items == 0 {
            return 0.0;
        }
        self.total_relationships as f32 / self.total_items as f32
    }

    /// Check if statistics indicate healthy state
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.memory_efficiency() > 0.5 && 
        self.memory_per_entity() < 2048.0 &&
        self.relationship_ratio() < 10.0
    }

    /// Calculate cleanup urgency (0.0 = not urgent, 1.0 = very urgent)
    #[inline]
    pub fn cleanup_urgency(&self, threshold_hours: u64) -> f32 {
        if let Some(last_cleanup) = self.last_cleanup_timestamp {
            if let Ok(elapsed) = last_cleanup.elapsed() {
                let elapsed_hours = elapsed.as_secs() / 3600;
                let ratio = elapsed_hours as f32 / threshold_hours as f32;
                return ratio.min(1.0);
            }
        }
        1.0 // Maximum urgency if no cleanup recorded
    }

    /// Get storage efficiency score
    #[inline]
    pub fn storage_efficiency(&self) -> f32 {
        let memory_eff = self.memory_efficiency();
        let access_eff = self.access_efficiency();
        (memory_eff + access_eff) / 2.0
    }

    /// Update statistics with new values
    #[inline]
    pub fn update(&mut self, 
                  items: usize, 
                  relationships: usize, 
                  memory_bytes: usize) {
        self.total_items = items;
        self.total_relationships = relationships;
        self.memory_usage_bytes = memory_bytes;
    }

    /// Record cleanup operation
    #[inline]
    pub fn record_cleanup(&mut self) {
        self.last_cleanup_timestamp = Some(SystemTime::now());
        self.cleanup_count += 1;
    }

    /// Add access count
    #[inline]
    pub fn add_access_count(&mut self, count: usize) {
        self.total_access_count += count;
    }

    /// Add unique pattern count
    #[inline]
    pub fn add_unique_pattern_count(&mut self, count: usize) {
        self.unique_pattern_count += count;
    }

    /// Get utilization score (0.0 to 1.0)
    #[inline]
    pub fn utilization_score(&self) -> f32 {
        let entity_score = if self.total_entity_count() > 0 { 0.3 } else { 0.0 };
        let memory_score = self.memory_efficiency() * 0.4;
        let access_score = self.access_efficiency() * 0.3;
        
        entity_score + memory_score + access_score
    }

    /// Check if optimization is urgently needed
    #[inline]
    pub fn needs_urgent_optimization(&self) -> bool {
        self.memory_efficiency() < 0.3 || 
        self.memory_per_entity() > 4096.0 ||
        self.relationship_ratio() > 20.0
    }

    /// Get performance grade (A-F)
    #[inline]
    pub fn performance_grade(&self) -> char {
        let score = self.storage_efficiency();
        if score >= 0.9 {
            'A'
        } else if score >= 0.8 {
            'B'
        } else if score >= 0.7 {
            'C'
        } else if score >= 0.6 {
            'D'
        } else {
            'F'
        }
    }

    /// Calculate data density (entities per MB)
    #[inline]
    pub fn data_density(&self) -> f32 {
        if self.memory_usage_bytes == 0 {
            return 0.0;
        }
        let mb = self.memory_usage_bytes as f32 / (1024.0 * 1024.0);
        self.total_entity_count() as f32 / mb
    }

    /// Get memory usage in human readable format
    #[inline]
    pub fn memory_usage_formatted(&self) -> String {
        let bytes = self.memory_usage_bytes as f64;
        if bytes >= 1073741824.0 {
            format!("{:.2} GB", bytes / 1073741824.0)
        } else if bytes >= 1048576.0 {
            format!("{:.2} MB", bytes / 1048576.0)
        } else if bytes >= 1024.0 {
            format!("{:.2} KB", bytes / 1024.0)
        } else {
            format!("{} bytes", bytes as usize)
        }
    }

    /// Get summary string
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "{} items, {} relationships, {}, {}% efficient",
            self.total_items,
            self.total_relationships,
            self.memory_usage_formatted(),
            (self.memory_efficiency() * 100.0) as u32
        )
    }

    /// Create delta from another statistics instance
    #[inline]
    pub fn delta_from(&self, other: &MemoryStatistics) -> StatisticsDelta {
        StatisticsDelta {
            item_delta: self.total_items as i64 - other.total_items as i64,
            relationship_delta: self.total_relationships as i64 - other.total_relationships as i64,
            memory_delta: self.memory_usage_bytes as i64 - other.memory_usage_bytes as i64,
            efficiency_delta: self.memory_efficiency() - other.memory_efficiency(),
            access_delta: self.total_access_count as i64 - other.total_access_count as i64,
        }
    }

    /// Check if statistics are valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        !(self.total_relationships > 0 && self.total_items == 0) &&
        !(self.memory_usage_bytes > 0 && self.total_entity_count() == 0) &&
        !(self.total_access_count > 0 && self.unique_pattern_count == 0)
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta between two memory statistics
#[derive(Debug, Clone)]
pub struct StatisticsDelta {
    pub item_delta: i64,
    pub relationship_delta: i64,
    pub memory_delta: i64,
    pub efficiency_delta: f32,
    pub access_delta: i64,
}

impl StatisticsDelta {
    /// Check if delta indicates improvement
    #[inline]
    pub fn is_improvement(&self) -> bool {
        self.efficiency_delta > 0.0 && self.memory_delta <= 0
    }

    /// Check if delta indicates degradation
    #[inline]
    pub fn is_degradation(&self) -> bool {
        self.efficiency_delta < 0.0 || (self.memory_delta > 0 && self.efficiency_delta <= 0.0)
    }

    /// Get change magnitude
    #[inline]
    pub fn magnitude(&self) -> f32 {
        let memory_change = (self.memory_delta.abs() as f32) / 1024.0; // KB
        let efficiency_change = self.efficiency_delta.abs();
        (memory_change + efficiency_change) / 2.0
    }
}
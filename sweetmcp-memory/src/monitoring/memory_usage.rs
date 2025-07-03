//! Memory usage monitoring

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    /// Total memory count
    pub total_memories: u64,
    
    /// Memory count by type
    pub memories_by_type: std::collections::HashMap<String, u64>,
    
    /// Total relationships
    pub total_relationships: u64,
    
    /// Average memory size in bytes
    pub avg_memory_size: u64,
    
    /// Total storage size in bytes
    pub total_storage_size: u64,
    
    /// Cache statistics
    pub cache_stats: CacheStats,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Cache hit rate
    pub hit_rate: f64,
    
    /// Total hits
    pub hits: u64,
    
    /// Total misses
    pub misses: u64,
    
    /// Cache size in bytes
    pub size_bytes: u64,
    
    /// Number of entries
    pub entry_count: u64,
}

/// Memory usage monitor
pub struct MemoryUsageMonitor {
    start_time: Instant,
    cache_hits: u64,
    cache_misses: u64,
}

impl MemoryUsageMonitor {
    /// Create a new monitor
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
    
    /// Get cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

impl Default for MemoryUsageMonitor {
    fn default() -> Self {
        Self::new()
    }
}
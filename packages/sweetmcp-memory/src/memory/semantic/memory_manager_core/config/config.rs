//! Memory management configuration
//!
//! This module provides configuration structures for memory management operations
//! including cleanup and optimization strategies.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Memory cleanup configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// Maximum number of items to process in a single cleanup batch
    pub batch_size: usize,
    /// Maximum time to spend on cleanup in milliseconds
    pub max_cleanup_time_ms: u64,
    /// Whether to enable aggressive cleanup
    pub aggressive: bool,
    /// Whether to enable memory pressure-based cleanup
    pub memory_pressure_based: bool,
    /// Minimum memory pressure (0-1) to trigger cleanup
    pub memory_pressure_threshold: f64,
    /// Maximum memory pressure (0-1) for aggressive cleanup
    pub max_memory_pressure: f64,
    /// Whether to enable background cleanup
    pub background_cleanup: bool,
    /// Interval between background cleanups in seconds
    pub background_interval_secs: u64,
}

impl CleanupConfig {
    /// Create new cleanup configuration
    pub fn new() -> Self {
        Self {
            batch_size: 1000,
            max_cleanup_time_ms: 100,
            aggressive: false,
            memory_pressure_based: true,
            memory_pressure_threshold: 0.7,
            max_memory_pressure: 0.9,
            background_cleanup: true,
            background_interval_secs: 3600, // 1 hour
        }
    }

    /// Create aggressive cleanup configuration
    pub fn aggressive() -> Self {
        Self {
            batch_size: 5000,
            max_cleanup_time_ms: 500,
            aggressive: true,
            memory_pressure_based: true,
            memory_pressure_threshold: 0.5,
            max_memory_pressure: 0.8,
            background_cleanup: true,
            background_interval_secs: 1800, // 30 minutes
        }
    }

    /// Create conservative cleanup configuration
    pub fn conservative() -> Self {
        Self {
            batch_size: 500,
            max_cleanup_time_ms: 50,
            aggressive: false,
            memory_pressure_based: false,
            memory_pressure_threshold: 0.9,
            max_memory_pressure: 0.95,
            background_cleanup: true,
            background_interval_secs: 7200, // 2 hours
        }
    }

    /// Create memory-pressure-driven configuration
    pub fn memory_pressure() -> Self {
        Self {
            batch_size: 1000,
            max_cleanup_time_ms: 200,
            aggressive: false,
            memory_pressure_based: true,
            memory_pressure_threshold: 0.6,
            max_memory_pressure: 0.85,
            background_cleanup: true,
            background_interval_secs: 3600, // 1 hour
        }
    }

    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        self.batch_size > 0
            && self.max_cleanup_time_ms > 0
            && self.memory_pressure_threshold > 0.0
            && self.memory_pressure_threshold < 1.0
            && self.max_memory_pressure > self.memory_pressure_threshold
            && self.max_memory_pressure <= 1.0
    }

    /// Get effective batch size based on memory pressure
    pub fn effective_batch_size(&self, current_memory_pressure: f64) -> usize {
        if !self.memory_pressure_based || current_memory_pressure < self.memory_pressure_threshold {
            return self.batch_size;
        }

        // Scale batch size based on memory pressure
        let pressure_ratio = (current_memory_pressure - self.memory_pressure_threshold)
            / (self.max_memory_pressure - self.memory_pressure_threshold);
        let scale_factor = if self.aggressive {
            pressure_ratio * 2.0
        } else {
            pressure_ratio
        };

        (self.batch_size as f64 * (1.0 + scale_factor)) as usize
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
    /// Whether to enable automatic defragmentation
    pub enable_defragmentation: bool,
    /// Fragmentation ratio threshold to trigger defragmentation
    pub fragmentation_threshold: f64,
    /// Whether to enable compression for large items
    pub enable_compression: bool,
    /// Minimum item size (in bytes) to consider for compression
    pub compression_threshold: usize,
    /// Whether to enable caching of frequently accessed items
    pub enable_caching: bool,
    /// Maximum cache size in bytes
    pub max_cache_size: usize,
    /// Whether to enable adaptive optimization based on access patterns
    pub adaptive_optimization: bool,
}

impl OptimizationStrategy {
    /// Create new optimization strategy
    pub fn new() -> Self {
        Self {
            enable_defragmentation: true,
            fragmentation_threshold: 0.5,
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            enable_caching: true,
            max_cache_size: 100 * 1024 * 1024, // 100MB
            adaptive_optimization: true,
        }
    }

    /// Create performance-focused strategy
    pub fn performance_focused() -> Self {
        Self {
            enable_defragmentation: true,
            fragmentation_threshold: 0.3, // More aggressive defragmentation
            enable_compression: false,    // Skip compression for performance
            compression_threshold: usize::MAX,
            enable_caching: true,
            max_cache_size: 500 * 1024 * 1024, // 500MB
            adaptive_optimization: true,
        }
    }

    /// Create memory-focused strategy
    pub fn memory_focused() -> Self {
        Self {
            enable_defragmentation: true,
            fragmentation_threshold: 0.7, // Less aggressive defragmentation
            enable_compression: true,
            compression_threshold: 512,  // Compress smaller items
            enable_caching: false,       // Disable caching to save memory
            max_cache_size: 0,
            adaptive_optimization: true,
        }
    }

    /// Check if defragmentation should be performed
    pub fn should_defragment(&self, fragmentation_ratio: f64) -> bool {
        self.enable_defragmentation && fragmentation_ratio > self.fragmentation_threshold
    }

    /// Check if compression should be applied
    pub fn should_compress(&self, item_size: usize) -> bool {
        self.enable_compression && item_size >= self.compression_threshold
    }

    /// Check if caching should be used
    pub fn should_cache(&self, current_cache_size: usize) -> bool {
        self.enable_caching && current_cache_size < self.max_cache_size
    }
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

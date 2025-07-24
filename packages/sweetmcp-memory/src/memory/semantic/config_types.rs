//! Basic memory management configuration types
//!
//! This module provides core configuration structures with zero-allocation
//! operations for memory optimization settings.

/// Memory cleanup configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub max_age_days: u64,
    pub min_confidence_threshold: super::super::confidence::ConfidenceLevel,
    pub max_unused_days: u64,
    pub preserve_high_confidence: bool,
    pub batch_size: usize,
}

impl CleanupConfig {
    /// Create new cleanup configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            max_age_days: 365, // 1 year
            min_confidence_threshold: super::super::confidence::ConfidenceLevel::Low,
            max_unused_days: 90, // 3 months
            preserve_high_confidence: true,
            batch_size: 1000,
        }
    }

    /// Create aggressive cleanup configuration
    #[inline]
    pub fn aggressive() -> Self {
        Self {
            max_age_days: 30,
            min_confidence_threshold: super::super::confidence::ConfidenceLevel::Medium,
            max_unused_days: 7,
            preserve_high_confidence: false,
            batch_size: 500,
        }
    }

    /// Create conservative cleanup configuration
    #[inline]
    pub fn conservative() -> Self {
        Self {
            max_age_days: 730, // 2 years
            min_confidence_threshold: super::super::confidence::ConfidenceLevel::VeryLow,
            max_unused_days: 180, // 6 months
            preserve_high_confidence: true,
            batch_size: 2000,
        }
    }

    /// Check if configuration is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.max_age_days > 0 && 
        self.max_unused_days > 0 && 
        self.batch_size > 0 &&
        self.max_unused_days <= self.max_age_days
    }

    /// Get estimated cleanup frequency in hours
    #[inline]
    pub fn estimated_cleanup_frequency(&self) -> u64 {
        if self.max_unused_days <= 7 {
            24 // Daily for aggressive
        } else if self.max_unused_days <= 30 {
            72 // Every 3 days for moderate
        } else {
            168 // Weekly for conservative
        }
    }

    /// Calculate expected cleanup volume ratio
    #[inline]
    pub fn expected_cleanup_ratio(&self) -> f32 {
        // Estimate percentage of data that will be cleaned up
        if self.preserve_high_confidence {
            0.1 // Conservative estimate
        } else {
            0.3 // More aggressive cleanup
        }
    }
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory optimization strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationStrategy {
    /// Optimize for speed (more memory usage)
    Speed,
    /// Optimize for memory (slower access)
    Memory,
    /// Balanced optimization
    Balanced,
}

impl OptimizationStrategy {
    /// Get cache size multiplier
    #[inline]
    pub fn cache_size_multiplier(&self) -> f32 {
        match self {
            OptimizationStrategy::Speed => 2.0,
            OptimizationStrategy::Memory => 0.5,
            OptimizationStrategy::Balanced => 1.0,
        }
    }

    /// Get compression threshold
    #[inline]
    pub fn compression_threshold(&self) -> usize {
        match self {
            OptimizationStrategy::Speed => 10000, // Compress less frequently
            OptimizationStrategy::Memory => 100,  // Compress more aggressively
            OptimizationStrategy::Balanced => 1000,
        }
    }

    /// Get cleanup frequency hours
    #[inline]
    pub fn cleanup_frequency_hours(&self) -> u64 {
        match self {
            OptimizationStrategy::Speed => 168, // Weekly
            OptimizationStrategy::Memory => 24,  // Daily
            OptimizationStrategy::Balanced => 72, // Every 3 days
        }
    }

    /// Get memory allocation tolerance
    #[inline]
    pub fn memory_allocation_tolerance(&self) -> f32 {
        match self {
            OptimizationStrategy::Speed => 0.9,    // Allow high memory usage
            OptimizationStrategy::Memory => 0.5,   // Strict memory limits
            OptimizationStrategy::Balanced => 0.7, // Moderate limits
        }
    }

    /// Get access pattern tracking level
    #[inline]
    pub fn access_tracking_level(&self) -> AccessTrackingLevel {
        match self {
            OptimizationStrategy::Speed => AccessTrackingLevel::Minimal,
            OptimizationStrategy::Memory => AccessTrackingLevel::Detailed,
            OptimizationStrategy::Balanced => AccessTrackingLevel::Moderate,
        }
    }

    /// Check if strategy prioritizes speed
    #[inline]
    pub fn prioritizes_speed(&self) -> bool {
        matches!(self, OptimizationStrategy::Speed)
    }

    /// Check if strategy prioritizes memory
    #[inline]
    pub fn prioritizes_memory(&self) -> bool {
        matches!(self, OptimizationStrategy::Memory)
    }

    /// Get description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationStrategy::Speed => "Speed-optimized with higher memory usage",
            OptimizationStrategy::Memory => "Memory-optimized with potential speed trade-offs",
            OptimizationStrategy::Balanced => "Balanced optimization for both speed and memory",
        }
    }
}

impl Default for OptimizationStrategy {
    fn default() -> Self {
        OptimizationStrategy::Balanced
    }
}

impl std::fmt::Display for OptimizationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationStrategy::Speed => write!(f, "Speed"),
            OptimizationStrategy::Memory => write!(f, "Memory"),
            OptimizationStrategy::Balanced => write!(f, "Balanced"),
        }
    }
}

/// Access pattern tracking level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessTrackingLevel {
    /// Minimal tracking for maximum speed
    Minimal,
    /// Moderate tracking for balance
    Moderate,
    /// Detailed tracking for memory optimization
    Detailed,
}

impl AccessTrackingLevel {
    /// Get maximum tracked patterns
    #[inline]
    pub fn max_tracked_patterns(&self) -> usize {
        match self {
            AccessTrackingLevel::Minimal => 100,
            AccessTrackingLevel::Moderate => 1000,
            AccessTrackingLevel::Detailed => 10000,
        }
    }

    /// Get tracking update frequency
    #[inline]
    pub fn update_frequency(&self) -> usize {
        match self {
            AccessTrackingLevel::Minimal => 10,  // Update every 10 accesses
            AccessTrackingLevel::Moderate => 5,  // Update every 5 accesses
            AccessTrackingLevel::Detailed => 1,  // Update every access
        }
    }

    /// Check if detailed tracking is enabled
    #[inline]
    pub fn is_detailed(&self) -> bool {
        matches!(self, AccessTrackingLevel::Detailed)
    }

    /// Check if minimal tracking is enabled
    #[inline]
    pub fn is_minimal(&self) -> bool {
        matches!(self, AccessTrackingLevel::Minimal)
    }
}
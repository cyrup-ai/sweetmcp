//! Atomic counter management for quantum entanglement metrics
//!
//! This module provides blazing-fast atomic counters with zero-allocation
//! patterns and lock-free performance tracking for optimal concurrency.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Core atomic counters for entanglement metrics
#[derive(Debug)]
pub struct EntanglementCounters {
    /// Number of entanglements created
    pub(super) entanglements_created: AtomicU64,
    /// Number of entanglements removed
    pub(super) entanglements_removed: AtomicU64,
    /// Number of entanglements pruned for performance
    pub(super) entanglements_pruned: AtomicUsize,
    /// Number of entanglement operations performed
    pub(super) entanglement_operations: AtomicU64,
    /// Number of entanglement creation failures
    pub(super) entanglement_failures: AtomicU64,
    /// Cache hits for entanglement decisions
    pub(super) cache_hits: AtomicU64,
    /// Cache misses for entanglement decisions
    pub(super) cache_misses: AtomicU64,
    /// Total time spent in entanglement operations (microseconds)
    pub(super) total_operation_time_us: AtomicU64,
    /// Number of network topology analyses performed
    pub(super) topology_analyses: AtomicU64,
    /// Number of influence calculations performed
    pub(super) influence_calculations: AtomicU64,
    /// Total processing time for influence calculations (microseconds)
    pub(super) influence_time_us: AtomicU64,
    /// Timestamp of counters creation
    pub(super) creation_time: Instant,
}

impl EntanglementCounters {
    /// Create new counters with atomic initialization
    pub fn new() -> Self {
        Self {
            entanglements_created: AtomicU64::new(0),
            entanglements_removed: AtomicU64::new(0),
            entanglements_pruned: AtomicUsize::new(0),
            entanglement_operations: AtomicU64::new(0),
            entanglement_failures: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            total_operation_time_us: AtomicU64::new(0),
            topology_analyses: AtomicU64::new(0),
            influence_calculations: AtomicU64::new(0),
            influence_time_us: AtomicU64::new(0),
            creation_time: Instant::now(),
        }
    }
    
    /// Record entanglement creation with atomic increment
    #[inline]
    pub fn record_entanglement_created(&self) {
        self.entanglements_created.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple entanglements created with atomic increment
    #[inline]
    pub fn record_entanglements_created(&self, count: u64) {
        if count > 0 {
            self.entanglements_created.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record entanglement removal with atomic increment
    #[inline]
    pub fn record_entanglement_removed(&self) {
        self.entanglements_removed.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple entanglements removed with atomic increment
    #[inline]
    pub fn record_entanglements_removed(&self, count: u64) {
        if count > 0 {
            self.entanglements_removed.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record entanglements pruned with atomic increment
    #[inline]
    pub fn record_entanglements_pruned(&self, count: usize) {
        if count > 0 {
            self.entanglements_pruned.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record entanglement operation with timing
    #[inline]
    pub fn record_entanglement_operation(&self, duration: Duration) {
        self.entanglement_operations.fetch_add(1, Ordering::Relaxed);
        self.total_operation_time_us.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }
    
    /// Record multiple entanglement operations with total timing
    #[inline]
    pub fn record_entanglement_operations(&self, count: u64, total_duration: Duration) {
        if count > 0 {
            self.entanglement_operations.fetch_add(count, Ordering::Relaxed);
            self.total_operation_time_us.fetch_add(total_duration.as_micros() as u64, Ordering::Relaxed);
        }
    }
    
    /// Record entanglement creation failure
    #[inline]
    pub fn record_entanglement_failure(&self) {
        self.entanglement_failures.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple entanglement failures
    #[inline]
    pub fn record_entanglement_failures(&self, count: u64) {
        if count > 0 {
            self.entanglement_failures.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record cache hit for entanglement decision
    #[inline]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple cache hits
    #[inline]
    pub fn record_cache_hits(&self, count: u64) {
        if count > 0 {
            self.cache_hits.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record cache miss for entanglement decision
    #[inline]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple cache misses
    #[inline]
    pub fn record_cache_misses(&self, count: u64) {
        if count > 0 {
            self.cache_misses.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record topology analysis
    #[inline]
    pub fn record_topology_analysis(&self) {
        self.topology_analyses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record multiple topology analyses
    #[inline]
    pub fn record_topology_analyses(&self, count: u64) {
        if count > 0 {
            self.topology_analyses.fetch_add(count, Ordering::Relaxed);
        }
    }
    
    /// Record influence calculation with timing
    #[inline]
    pub fn record_influence_calculation(&self, duration: Duration) {
        self.influence_calculations.fetch_add(1, Ordering::Relaxed);
        self.influence_time_us.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }
    
    /// Record multiple influence calculations with total timing
    #[inline]
    pub fn record_influence_calculations(&self, count: u64, total_duration: Duration) {
        if count > 0 {
            self.influence_calculations.fetch_add(count, Ordering::Relaxed);
            self.influence_time_us.fetch_add(total_duration.as_micros() as u64, Ordering::Relaxed);
        }
    }
    
    /// Get number of entanglements created
    #[inline]
    pub fn entanglements_created(&self) -> u64 {
        self.entanglements_created.load(Ordering::Relaxed)
    }
    
    /// Get number of entanglements removed
    #[inline]
    pub fn entanglements_removed(&self) -> u64 {
        self.entanglements_removed.load(Ordering::Relaxed)
    }
    
    /// Get number of entanglements pruned
    #[inline]
    pub fn entanglements_pruned(&self) -> usize {
        self.entanglements_pruned.load(Ordering::Relaxed)
    }
    
    /// Get number of entanglement operations
    #[inline]
    pub fn entanglement_operations(&self) -> u64 {
        self.entanglement_operations.load(Ordering::Relaxed)
    }
    
    /// Get number of entanglement failures
    #[inline]
    pub fn entanglement_failures(&self) -> u64 {
        self.entanglement_failures.load(Ordering::Relaxed)
    }
    
    /// Get number of cache hits
    #[inline]
    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }
    
    /// Get number of cache misses
    #[inline]
    pub fn cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }
    
    /// Get total operation time in microseconds
    #[inline]
    pub fn total_operation_time_us(&self) -> u64 {
        self.total_operation_time_us.load(Ordering::Relaxed)
    }
    
    /// Get number of topology analyses
    #[inline]
    pub fn topology_analyses(&self) -> u64 {
        self.topology_analyses.load(Ordering::Relaxed)
    }
    
    /// Get number of influence calculations
    #[inline]
    pub fn influence_calculations(&self) -> u64 {
        self.influence_calculations.load(Ordering::Relaxed)
    }
    
    /// Get total influence calculation time in microseconds
    #[inline]
    pub fn influence_time_us(&self) -> u64 {
        self.influence_time_us.load(Ordering::Relaxed)
    }
    
    /// Get creation timestamp
    #[inline]
    pub fn creation_time(&self) -> Instant {
        self.creation_time
    }
    
    /// Reset all counters to zero
    pub fn reset(&self) {
        self.entanglements_created.store(0, Ordering::Relaxed);
        self.entanglements_removed.store(0, Ordering::Relaxed);
        self.entanglements_pruned.store(0, Ordering::Relaxed);
        self.entanglement_operations.store(0, Ordering::Relaxed);
        self.entanglement_failures.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.total_operation_time_us.store(0, Ordering::Relaxed);
        self.topology_analyses.store(0, Ordering::Relaxed);
        self.influence_calculations.store(0, Ordering::Relaxed);
        self.influence_time_us.store(0, Ordering::Relaxed);
    }
    
    /// Get snapshot of all counter values
    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            entanglements_created: self.entanglements_created(),
            entanglements_removed: self.entanglements_removed(),
            entanglements_pruned: self.entanglements_pruned(),
            entanglement_operations: self.entanglement_operations(),
            entanglement_failures: self.entanglement_failures(),
            cache_hits: self.cache_hits(),
            cache_misses: self.cache_misses(),
            total_operation_time_us: self.total_operation_time_us(),
            topology_analyses: self.topology_analyses(),
            influence_calculations: self.influence_calculations(),
            influence_time_us: self.influence_time_us(),
            timestamp: Instant::now(),
            uptime_duration: self.creation_time.elapsed(),
        }
    }
    
    /// Compare and swap entanglements created count
    pub fn compare_and_swap_created(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.entanglements_created
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Relaxed)
    }
    
    /// Add to entanglements created and return previous value
    pub fn fetch_add_created(&self, count: u64) -> u64 {
        self.entanglements_created.fetch_add(count, Ordering::Relaxed)
    }
    
    /// Subtract from entanglements created and return previous value
    pub fn fetch_sub_created(&self, count: u64) -> u64 {
        self.entanglements_created.fetch_sub(count, Ordering::Relaxed)
    }
    
    /// Get maximum value observed for any counter
    pub fn max_counter_value(&self) -> u64 {
        [
            self.entanglements_created(),
            self.entanglements_removed(),
            self.entanglements_pruned() as u64,
            self.entanglement_operations(),
            self.entanglement_failures(),
            self.cache_hits(),
            self.cache_misses(),
            self.topology_analyses(),
            self.influence_calculations(),
        ].iter().max().copied().unwrap_or(0)
    }
    
    /// Get total number of all operations across all counters
    pub fn total_operations(&self) -> u64 {
        self.entanglement_operations() + 
        self.topology_analyses() + 
        self.influence_calculations()
    }
    
    /// Check if any operations have been recorded
    pub fn has_activity(&self) -> bool {
        self.total_operations() > 0
    }
    
    /// Get uptime since creation
    pub fn uptime(&self) -> Duration {
        self.creation_time.elapsed()
    }
    
    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.uptime().as_secs()
    }
}

impl Default for EntanglementCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EntanglementCounters {
    fn clone(&self) -> Self {
        let snapshot = self.snapshot();
        let mut new_counters = Self::new();
        
        // Set all values from snapshot
        new_counters.entanglements_created.store(snapshot.entanglements_created, Ordering::Relaxed);
        new_counters.entanglements_removed.store(snapshot.entanglements_removed, Ordering::Relaxed);
        new_counters.entanglements_pruned.store(snapshot.entanglements_pruned, Ordering::Relaxed);
        new_counters.entanglement_operations.store(snapshot.entanglement_operations, Ordering::Relaxed);
        new_counters.entanglement_failures.store(snapshot.entanglement_failures, Ordering::Relaxed);
        new_counters.cache_hits.store(snapshot.cache_hits, Ordering::Relaxed);
        new_counters.cache_misses.store(snapshot.cache_misses, Ordering::Relaxed);
        new_counters.total_operation_time_us.store(snapshot.total_operation_time_us, Ordering::Relaxed);
        new_counters.topology_analyses.store(snapshot.topology_analyses, Ordering::Relaxed);
        new_counters.influence_calculations.store(snapshot.influence_calculations, Ordering::Relaxed);
        new_counters.influence_time_us.store(snapshot.influence_time_us, Ordering::Relaxed);
        
        new_counters
    }
}

/// Immutable snapshot of counter values at a point in time
#[derive(Debug, Clone)]
pub struct CounterSnapshot {
    /// Number of entanglements created
    pub entanglements_created: u64,
    /// Number of entanglements removed
    pub entanglements_removed: u64,
    /// Number of entanglements pruned
    pub entanglements_pruned: usize,
    /// Number of entanglement operations
    pub entanglement_operations: u64,
    /// Number of entanglement failures
    pub entanglement_failures: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Total operation time in microseconds
    pub total_operation_time_us: u64,
    /// Number of topology analyses
    pub topology_analyses: u64,
    /// Number of influence calculations
    pub influence_calculations: u64,
    /// Total influence calculation time in microseconds
    pub influence_time_us: u64,
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
    /// Uptime duration when snapshot was taken
    pub uptime_duration: Duration,
}

impl CounterSnapshot {
    /// Calculate net entanglement growth
    pub fn net_entanglements(&self) -> i64 {
        self.entanglements_created as i64 - 
        self.entanglements_removed as i64 - 
        self.entanglements_pruned as i64
    }
    
    /// Calculate total cache operations
    pub fn total_cache_operations(&self) -> u64 {
        self.cache_hits + self.cache_misses
    }
    
    /// Check if snapshot indicates active system
    pub fn is_active(&self) -> bool {
        self.entanglement_operations > 0 || 
        self.topology_analyses > 0 || 
        self.influence_calculations > 0
    }
    
    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.entanglement_operations + 
        self.topology_analyses + 
        self.influence_calculations
    }
    
    /// Compare with another snapshot to calculate deltas
    pub fn delta(&self, other: &CounterSnapshot) -> CounterDelta {
        CounterDelta {
            entanglements_created: self.entanglements_created.saturating_sub(other.entanglements_created),
            entanglements_removed: self.entanglements_removed.saturating_sub(other.entanglements_removed),
            entanglements_pruned: self.entanglements_pruned.saturating_sub(other.entanglements_pruned),
            entanglement_operations: self.entanglement_operations.saturating_sub(other.entanglement_operations),
            entanglement_failures: self.entanglement_failures.saturating_sub(other.entanglement_failures),
            cache_hits: self.cache_hits.saturating_sub(other.cache_hits),
            cache_misses: self.cache_misses.saturating_sub(other.cache_misses),
            total_operation_time_us: self.total_operation_time_us.saturating_sub(other.total_operation_time_us),
            topology_analyses: self.topology_analyses.saturating_sub(other.topology_analyses),
            influence_calculations: self.influence_calculations.saturating_sub(other.influence_calculations),
            influence_time_us: self.influence_time_us.saturating_sub(other.influence_time_us),
            time_elapsed: self.timestamp.duration_since(other.timestamp),
        }
    }
}

/// Delta between two counter snapshots
#[derive(Debug, Clone)]
pub struct CounterDelta {
    /// Change in entanglements created
    pub entanglements_created: u64,
    /// Change in entanglements removed
    pub entanglements_removed: u64,
    /// Change in entanglements pruned
    pub entanglements_pruned: usize,
    /// Change in entanglement operations
    pub entanglement_operations: u64,
    /// Change in entanglement failures
    pub entanglement_failures: u64,
    /// Change in cache hits
    pub cache_hits: u64,
    /// Change in cache misses
    pub cache_misses: u64,
    /// Change in total operation time
    pub total_operation_time_us: u64,
    /// Change in topology analyses
    pub topology_analyses: u64,
    /// Change in influence calculations
    pub influence_calculations: u64,
    /// Change in influence calculation time
    pub influence_time_us: u64,
    /// Time elapsed between snapshots
    pub time_elapsed: Duration,
}

impl CounterDelta {
    /// Calculate operations per second for the delta period
    pub fn operations_per_second(&self) -> f64 {
        if self.time_elapsed.as_secs_f64() > 0.0 {
            self.entanglement_operations as f64 / self.time_elapsed.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Calculate net entanglement change
    pub fn net_entanglement_change(&self) -> i64 {
        self.entanglements_created as i64 - 
        self.entanglements_removed as i64 - 
        self.entanglements_pruned as i64
    }
    
    /// Check if delta indicates significant activity
    pub fn has_significant_activity(&self) -> bool {
        self.entanglement_operations > 10 || 
        self.topology_analyses > 5 || 
        self.influence_calculations > 20
    }
}
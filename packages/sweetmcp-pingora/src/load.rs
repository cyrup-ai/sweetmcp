//! 1-minute load-average + inflight counter overload check.
//! Lock-free implementation using atomic operations for blazing-fast performance.

use atomic_counter::{AtomicCounter, RelaxedCounter};
use std::sync::atomic::{AtomicU64, Ordering};
use sysinfo::System;

/// Lock-free load tracking with atomic counters for high-performance edge routing.
/// All operations are lock-free and safe for concurrent access from multiple threads.
/// Cache-line aligned and padded to prevent false sharing between atomic counters.
#[repr(align(64))] // Cache-line alignment for CPU cache efficiency
pub struct Load {
    /// Inflight request counter using atomic operations - hot path field
    inflight: AtomicU64,

    // Cache-line padding to prevent false sharing between atomic counters
    _pad1: [u8; 64 - 8], // 64 bytes (cache line) - 8 bytes (AtomicU64)

    /// Total request counter for statistics - separate cache line
    total_requests: RelaxedCounter,

    // Additional padding to ensure CPU count doesn't share cache line with hot counters
    _pad2: [u8; 64 - 8], // 64 bytes - 8 bytes (RelaxedCounter size)

    /// CPU count cached at initialization for load average comparison - cold field
    cpus: usize,
}

impl Load {
    /// Create a new Load tracker with system CPU detection.
    /// This operation is lock-free and safe for concurrent initialization.
    pub fn new() -> Self {
        let mut s = System::new();
        s.refresh_cpu_all();
        let cpus = s.cpus().len().max(1); // Ensure at least 1 CPU for calculations

        Self {
            inflight: AtomicU64::new(0),
            _pad1: [0; 64 - 8], // Initialize padding with zeros
            total_requests: RelaxedCounter::new(0),
            _pad2: [0; 64 - 8], // Initialize padding with zeros
            cpus,
        }
    }

    /// Increment inflight request counter atomically.
    /// Lock-free operation with relaxed ordering for maximum performance.
    #[inline]
    pub fn inc(&self) {
        self.inflight.fetch_add(1, Ordering::Relaxed);
        self.total_requests.inc();
    }

    /// Decrement inflight request counter atomically.
    /// Lock-free operation with relaxed ordering for maximum performance.
    #[inline]
    pub fn dec(&self) {
        self.inflight.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get current inflight request count atomically.
    /// Lock-free read operation for monitoring and diagnostics.
    #[inline]
    pub fn inflight_count(&self) -> u64 {
        self.inflight.load(Ordering::Relaxed)
    }

    /// Get total request count since initialization.
    /// Lock-free read operation for statistics collection.
    #[inline]
    pub fn total_count(&self) -> usize {
        self.total_requests.get()
    }

    /// Check if system is overloaded based on CPU load average and inflight requests.
    /// Now lock-free - no longer requires mutable reference!
    /// Uses static System::load_average() call for zero-allocation operation.
    #[inline]
    pub fn overload(&self, max_inflight: u64) -> bool {
        // Static call to load_average - no system state required
        let load_avg = System::load_average();
        let load1 = load_avg.one;

        // Check both CPU load and inflight request limits
        let cpu_overloaded = load1 > self.cpus as f64;
        let requests_overloaded = self.inflight.load(Ordering::Relaxed) > max_inflight;

        cpu_overloaded || requests_overloaded
    }

    /// Get current system load metrics for monitoring.
    /// Lock-free operation returning load statistics.
    pub fn load_metrics(&self) -> LoadMetrics {
        let load_avg = System::load_average();
        LoadMetrics {
            load_1min: load_avg.one,
            load_5min: load_avg.five,
            load_15min: load_avg.fifteen,
            inflight: self.inflight.load(Ordering::Relaxed),
            total_requests: self.total_requests.get(),
            cpu_count: self.cpus,
        }
    }
}

/// Load metrics for monitoring and diagnostics.
/// Zero-allocation struct with Copy trait for efficient passing.
#[derive(Debug, Clone, Copy)]
pub struct LoadMetrics {
    pub load_1min: f64,
    pub load_5min: f64,
    pub load_15min: f64,
    pub inflight: u64,
    pub total_requests: usize,
    pub cpu_count: usize,
}

impl Default for Load {
    fn default() -> Self {
        Self::new()
    }
}

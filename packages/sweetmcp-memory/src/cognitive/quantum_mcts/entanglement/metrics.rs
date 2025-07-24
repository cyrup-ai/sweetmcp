//! Performance metrics and quality assessment for quantum entanglement operations
//!
//! This module provides blazing-fast metrics collection with atomic counters
//! and lock-free performance tracking for optimal zero-allocation patterns.

// Include the metrics submodules
#[path = "metrics/mod.rs"]
mod metrics;

// Re-export everything from the metrics submodule for backward compatibility
pub use metrics::*;
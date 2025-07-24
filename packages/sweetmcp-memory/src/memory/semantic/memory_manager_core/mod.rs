//! Core memory management structures and statistics
//!
//! This module provides the core memory management functionality with zero-allocation
//! patterns and blazing-fast performance for semantic memory lifecycle operations.

pub mod config;
pub mod statistics;

// Re-export all public items for ergonomic access
pub use config::{CleanupConfig, OptimizationStrategy};
pub use statistics::MemoryStatistics;

//! Tree statistics integration module
//!
//! This module integrates all the decomposed tree statistics functionality
//! with zero allocation patterns and blazing-fast performance.

// Re-export from the decomposed tree_stats modules
pub use super::tree_stats_types::*;
pub use super::tree_stats_analyzer::*;
pub use super::tree_stats_mod::*;

// Re-export key types from other modules for backward compatibility
pub use super::types::QuantumTreeStatistics;
pub use super::metrics::{DepthStatistics, RewardStatistics, ConvergenceMetrics};
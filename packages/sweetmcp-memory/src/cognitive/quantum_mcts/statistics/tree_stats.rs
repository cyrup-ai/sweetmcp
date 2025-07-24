//! Tree statistics integration module
//!
//! This module integrates all the decomposed tree statistics functionality
//! with zero allocation patterns and blazing-fast performance.

// Re-export from the decomposed tree_stats modules
pub use super::tree_stats_types::*;
pub use super::tree_stats_analyzer::*;

// Re-export the mod functionality if it exists
pub use super::tree_stats_mod::*;
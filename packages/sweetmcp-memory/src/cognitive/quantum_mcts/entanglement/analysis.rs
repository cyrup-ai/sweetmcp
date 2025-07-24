//! Network topology analysis and influence calculations for quantum entanglement coordination module
//!
//! This module provides blazing-fast analysis of entanglement networks with
//! zero-allocation patterns and lock-free computation for optimal performance.

// Include the analysis submodules
#[path = "analysis/mod.rs"]
mod analysis;

// Re-export everything from the analysis submodule for backward compatibility
pub use analysis::*;
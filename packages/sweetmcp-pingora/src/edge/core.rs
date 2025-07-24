//! Edge service core implementation
//!
//! This module provides the main EdgeService interface and core functionality.
//! It re-exports functionality from the decomposed core modules.

// Re-export all core functionality
pub use self::core::*;

// Include the core module
#[path = "core/mod.rs"]
mod core;
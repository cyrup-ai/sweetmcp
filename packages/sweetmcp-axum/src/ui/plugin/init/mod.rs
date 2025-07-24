//! Plugin initialization system with comprehensive scaffolding
//!
//! This module provides complete plugin initialization functionality with zero
//! allocation patterns, blazing-fast performance, and production-ready scaffolding.

pub mod core;
pub mod templates;
pub mod engine;

// Re-export core types and functions for ergonomic use
pub use core::*;
pub use templates::*;
pub use engine::*;
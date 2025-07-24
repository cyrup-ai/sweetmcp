//! Zero-allocation input validation framework with SIMD-accelerated pattern matching
//!
//! This module provides comprehensive input validation for all external inputs
//! with zero-allocation, lock-free, and SIMD-accelerated patterns.

pub mod core;
pub mod rules;
pub mod engine;

// Re-export core types and functions for ergonomic use
pub use core::*;
pub use rules::*;
pub use engine::*;
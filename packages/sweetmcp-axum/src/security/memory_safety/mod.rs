//! Memory safety verification system with zero-allocation patterns
//!
//! This module provides comprehensive runtime memory safety verification for
//! production environments with zero-allocation, lock-free, and SIMD-accelerated patterns.

pub mod core;
pub mod validation;
pub mod monitoring;

// Re-export core types and functions for ergonomic use
pub use core::*;
pub use validation::*;
pub use monitoring::*;
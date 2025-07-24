//! Security module for SweetMCP Axum server
//!
//! This module provides comprehensive security functionality including:
//! - Zero-allocation input validation with SIMD-accelerated pattern matching
//! - Automated memory safety verification with real-time monitoring
//! - Lock-free validation result caching for high-performance validation
//! - Comprehensive input sanitization for all external inputs
//! - Integration with existing security audit systems
//! - Real-time validation metrics and monitoring

pub mod memory_safety;
pub mod validation;

// Re-export all security types for convenience
pub use memory_safety::*;
pub use validation::*;

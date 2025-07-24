//! Security module for SweetMCP daemon
//!
//! This module provides comprehensive security functionality including:
//! - Zero-allocation vulnerability scanning
//! - Lock-free security metrics
//! - SIMD-accelerated pattern matching
//! - CI/CD integration for security validation

pub mod audit;

pub use audit::*;

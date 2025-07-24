//! Cryptographic utilities module for secure token handling
//!
//! This module provides comprehensive cryptographic utilities including NaCl box
//! encryption for discovery tokens, token rotation, revocation list support,
//! and secure token wrappers with zero allocation patterns and blazing-fast performance.

pub mod core;
pub mod operations;
pub mod wrapper;

// Re-export core types for ergonomic use
pub use core::{
    EncryptedToken, TokenManager, TokenKeypair, TokenData, KeyInfo, TokenManagerStats,
    TOKEN_ROTATION_HOURS, TOKEN_VALIDITY_HOURS,
};

// Re-export operations types
pub use operations::{TokenMetadata, TokenChainValidation};

// Re-export wrapper types
pub use wrapper::{SecureDiscoveryToken, SecureTokenStats, SecureTokenBuilder};
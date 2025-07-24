//! Converts GraphQL, JSON-RPC, or Cap'n Proto payloads into standard JSON-RPC for MCP
//!
//! This module provides the main protocol normalization interface.
//! It re-exports functionality from the decomposed normalize modules.

// Re-export all normalization functionality
pub use self::normalize::*;

// Include the normalize module
#[path = "normalize/mod.rs"]
mod normalize;
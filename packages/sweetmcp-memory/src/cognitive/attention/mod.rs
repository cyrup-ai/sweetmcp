//! Attention mechanism module for cognitive memory management
//!
//! This module provides comprehensive attention mechanisms including multi-head
//! attention, memory scoring, and relevance calculation with zero allocation
//! patterns and blazing-fast performance.

pub mod core;
pub mod computation;
pub mod scoring;

// Re-export core types for ergonomic use
pub use core::{
    AttentionMechanism, AttentionConfig, AttentionWeights, AttentionOutput,
    AttentionMemoryUsage, AttentionContext, AttentionHead, MultiHeadAttentionState,
};

// Re-export computation utilities
pub use computation::AttentionUtils;

// Re-export scoring types
pub use scoring::{AttentionStatistics, AttentionCluster};
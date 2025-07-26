//! Attention mechanism module for cognitive memory management
//!
//! This module provides comprehensive attention mechanisms including multi-head
//! attention, memory scoring, and relevance calculation with zero allocation
//! patterns and blazing-fast performance.

pub mod computation;
pub mod scoring;

use std::collections::HashMap;

/// Configuration for attention mechanism
#[derive(Debug, Clone)]
pub struct AttentionConfig {
    pub num_heads: usize,
    pub hidden_dim: usize,
    pub dropout_rate: f32,
    pub use_causal_mask: bool,
}



/// Core attention mechanism for cognitive memory management
#[derive(Debug, Clone)]
pub struct AttentionMechanism {
    pub attention_scores: HashMap<String, f32>,
    pub num_heads: usize,
    pub head_dim: usize,
}

impl AttentionMechanism {
    /// Create a new lock-free attention mechanism
    pub fn new_lock_free(config: AttentionConfig) -> Self {
        Self {
            attention_scores: HashMap::new(),
            num_heads: config.num_heads,
            head_dim: config.hidden_dim / config.num_heads,
        }
    }
}

impl Default for AttentionMechanism {
    fn default() -> Self {
        Self {
            attention_scores: HashMap::new(),
            num_heads: 8,
            head_dim: 64,
        }
    }
}

// Re-export all public types from modules for ergonomic use
pub use computation::*;
pub use scoring::*;
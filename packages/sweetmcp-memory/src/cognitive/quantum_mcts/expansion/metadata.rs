//! Metadata structures for quantum expansion
//!
//! This module provides statistics and result structures for monitoring
//! and tracking quantum MCTS expansion operations.

use crate::cognitive::quantum::Complex64;

/// Expansion statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct ExpansionStats {
    /// Number of action vectors in the pool
    pub action_pool_size: usize,
    /// Capacity of the action pool
    pub action_pool_capacity: usize,
    /// Maximum parallel expansions allowed
    pub max_parallel: usize,
    /// Available expansion permits
    pub available_permits: usize,
}

/// Expansion result with metadata
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    /// New child node ID (if expansion occurred)
    pub child_id: Option<String>,
    /// Action that was applied
    pub action: String,
    /// Quantum amplitude of the new child
    pub child_amplitude: Complex64,
    /// Expansion success
    pub success: bool,
    /// Error message if expansion failed  
    pub error: Option<String>,
}
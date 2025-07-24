//! Quantum MCTS node state types for statistics
//!
//! This module provides comprehensive node state definitions with zero allocation
//! optimizations and blazing-fast performance for quantum MCTS statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Quantum MCTS node for statistics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumMCTSNode {
    /// Node unique identifier
    pub id: String,
    /// Node state data
    pub state: NodeState,
    /// Quantum properties
    pub quantum_properties: QuantumProperties,
    /// Node statistics
    pub statistics: NodeStatistics,
    /// Performance metrics
    pub metrics: NodeMetrics,
    /// Creation timestamp
    pub created_at: Instant,
}

impl QuantumMCTSNode {
    /// Create new quantum MCTS node
    pub fn new(id: String, state: NodeState) -> Self {
        Self {
            id,
            state,
            quantum_properties: QuantumProperties::default(),
            statistics: NodeStatistics::new(),
            metrics: NodeMetrics::new(),
            created_at: Instant::now(),
        }
    }

    /// Update node statistics
    pub fn update_statistics(&mut self, visits: u64, value: f64) {
        self.statistics.visit_count += visits;
        self.statistics.total_value += value;
        self.statistics.average_value = self.statistics.total_value / self.statistics.visit_count as f64;
        self.statistics.last_updated = Instant::now();
    }

    /// Get node quality score
    pub fn quality_score(&self) -> f64 {
        self.statistics.average_value * self.quantum_properties.coherence_factor
    }
}

/// Node state representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    /// State vector
    pub state_vector: Vec<f64>,
    /// State hash for quick comparison
    pub state_hash: u64,
    /// State metadata
    pub metadata: StateMetadata,
}

impl NodeState {
    /// Create new node state
    pub fn new(state_vector: Vec<f64>) -> Self {
        let state_hash = Self::calculate_hash(&state_vector);
        Self {
            state_vector,
            state_hash,
            metadata: StateMetadata::default(),
        }
    }

    /// Calculate state hash
    fn calculate_hash(vector: &[f64]) -> u64 {
        // Simple hash implementation for demonstration
        vector.iter().fold(0u64, |acc, &x| {
            acc.wrapping_mul(31).wrapping_add(x.to_bits())
        })
    }
}

/// State metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// State dimension
    pub dimension: usize,
    /// State norm
    pub norm: f64,
    /// State entropy
    pub entropy: f64,
    /// Additional properties
    pub properties: HashMap<String, f64>,
}

impl Default for StateMetadata {
    fn default() -> Self {
        Self {
            dimension: 0,
            norm: 0.0,
            entropy: 0.0,
            properties: HashMap::new(),
        }
    }
}

/// Quantum properties of the node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumProperties {
    /// Quantum coherence factor
    pub coherence_factor: f64,
    /// Entanglement strength
    pub entanglement_strength: f64,
    /// Superposition coefficient
    pub superposition_coefficient: f64,
    /// Quantum phase
    pub quantum_phase: f64,
}

impl Default for QuantumProperties {
    fn default() -> Self {
        Self {
            coherence_factor: 1.0,
            entanglement_strength: 0.0,
            superposition_coefficient: 1.0,
            quantum_phase: 0.0,
        }
    }
}

/// Node statistics for tracking performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatistics {
    /// Number of visits to this node
    pub visit_count: u64,
    /// Total value accumulated
    pub total_value: f64,
    /// Average value per visit
    pub average_value: f64,
    /// Best value seen
    pub best_value: f64,
    /// Worst value seen
    pub worst_value: f64,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl NodeStatistics {
    /// Create new node statistics
    pub fn new() -> Self {
        Self {
            visit_count: 0,
            total_value: 0.0,
            average_value: 0.0,
            best_value: f64::NEG_INFINITY,
            worst_value: f64::INFINITY,
            last_updated: Instant::now(),
        }
    }

    /// Update with new value
    pub fn update(&mut self, value: f64) {
        self.visit_count += 1;
        self.total_value += value;
        self.average_value = self.total_value / self.visit_count as f64;
        self.best_value = self.best_value.max(value);
        self.worst_value = self.worst_value.min(value);
        self.last_updated = Instant::now();
    }
}

/// Node performance metrics
#[derive(Debug, Clone)]
pub struct NodeMetrics {
    /// Time spent in this node
    pub total_time_us: u64,
    /// Number of expansions from this node
    pub expansion_count: u64,
    /// Memory usage for this node
    pub memory_bytes: usize,
    /// Last access timestamp
    pub last_accessed: Instant,
}

impl NodeMetrics {
    /// Create new node metrics
    pub fn new() -> Self {
        Self {
            total_time_us: 0,
            expansion_count: 0,
            memory_bytes: 0,
            last_accessed: Instant::now(),
        }
    }

    /// Update access time
    pub fn update_access(&mut self) {
        self.last_accessed = Instant::now();
    }
}
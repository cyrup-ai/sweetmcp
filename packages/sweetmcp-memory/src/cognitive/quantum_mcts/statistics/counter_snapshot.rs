//! Counter snapshot for atomic values and operation tracking
//!
//! This module provides the CounterSnapshot struct for capturing atomic counter
//! states with blazing-fast zero-allocation operations and analysis.

/// Counter snapshot for atomic values with operation analysis
#[derive(Debug, Clone, Default)]
pub struct CounterSnapshot {
    /// Total nodes
    pub nodes: usize,
    /// Total visits
    pub visits: u64,
    /// Total selections
    pub selections: u64,
    /// Total expansions
    pub expansions: u64,
    /// Total backpropagations
    pub backpropagations: u64,
    /// Total simulations
    pub simulations: u64,
}

impl CounterSnapshot {
    /// Create new counter snapshot
    pub fn new(
        nodes: usize,
        visits: u64,
        selections: u64,
        expansions: u64,
        backpropagations: u64,
        simulations: u64,
    ) -> Self {
        Self {
            nodes,
            visits,
            selections,
            expansions,
            backpropagations,
            simulations,
        }
    }
    
    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.selections + self.expansions + self.backpropagations + self.simulations
    }
    
    /// Calculate operations per node ratio
    pub fn operations_per_node(&self) -> f64 {
        if self.nodes > 0 {
            self.total_operations() as f64 / self.nodes as f64
        } else {
            0.0
        }
    }
    
    /// Calculate visits per node ratio
    pub fn visits_per_node(&self) -> f64 {
        if self.nodes > 0 {
            self.visits as f64 / self.nodes as f64
        } else {
            0.0
        }
    }
    
    /// Get performance summary string
    pub fn performance_summary(&self) -> String {
        format!(
            "Nodes: {}, Visits: {}, Ops/Node: {:.1}, Total Ops: {}",
            self.nodes,
            self.visits,
            self.operations_per_node(),
            self.total_operations()
        )
    }
}
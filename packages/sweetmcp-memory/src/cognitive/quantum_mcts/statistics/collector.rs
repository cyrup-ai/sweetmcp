//! Core statistics collector with lock-free atomic operations
//!
//! This module provides the main QuantumStatisticsCollector with blazing-fast
//! atomic counters, zero-allocation updates, and comprehensive tree analysis.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::{EntanglementGraph, QuantumMetrics},
    types::CognitiveError,
};
use super::{
    super::{
        node_state::QuantumMCTSNode,
        config::QuantumMCTSConfig,
    },
    types::QuantumTreeStatistics,
    counter_snapshot::CounterSnapshot,
    calculation_engine::CalculationEngine,
    atomic_operations::AtomicOperationsManager,
};

/// Lock-free statistics collector with atomic operations
pub struct QuantumStatisticsCollector {
    /// Configuration
    config: QuantumMCTSConfig,
    /// Atomic counters for lock-free updates
    total_nodes: AtomicUsize,
    total_visits: AtomicU64,
    total_selections: AtomicU64,
    total_expansions: AtomicU64,
    total_backpropagations: AtomicU64,
    total_simulations: AtomicU64,
    /// Atomic operations manager
    atomic_ops: AtomicOperationsManager,
}

impl QuantumStatisticsCollector {
    /// Create new statistics collector with lock-free initialization
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self {
            config,
            total_nodes: AtomicUsize::new(0),
            total_visits: AtomicU64::new(0),
            total_selections: AtomicU64::new(0),
            total_expansions: AtomicU64::new(0),
            total_backpropagations: AtomicU64::new(0),
            total_simulations: AtomicU64::new(0),
            atomic_ops: AtomicOperationsManager::new(),
        }
    }
    
    /// Collect comprehensive quantum tree statistics with zero allocation analysis
    pub async fn collect_statistics(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        entanglement_graph: &RwLock<EntanglementGraph>,
        quantum_metrics: &RwLock<QuantumMetrics>,
    ) -> Result<QuantumTreeStatistics, CognitiveError> {
        let tree_read = tree.read().await;
        let entanglement_read = entanglement_graph.read().await;
        let metrics_read = quantum_metrics.read().await;
        
        // Basic tree statistics with vectorized calculations
        let total_nodes = tree_read.len();
        let total_visits = CalculationEngine::calculate_total_visits(&tree_read);
        let total_entanglements = entanglement_read.entanglement_count();
        
        // Quantum-specific statistics
        let (avg_decoherence, max_amplitude) = CalculationEngine::calculate_quantum_stats(&tree_read);
        
        // Advanced statistics with parallel computation
        let depth_stats = CalculationEngine::calculate_depth_statistics(&tree_read).await?;
        let reward_stats = CalculationEngine::calculate_reward_statistics(&tree_read).await?;
        let convergence_metrics = CalculationEngine::calculate_convergence_metrics(&tree_read).await?;
        let performance_metrics = CalculationEngine::calculate_performance_metrics(
            &tree_read,
            self.atomic_ops.start_time(),
            &self.get_counter_values(),
        ).await?;
        
        // Update atomic counters
        self.total_nodes.store(total_nodes, Ordering::Relaxed);
        self.total_visits.store(total_visits, Ordering::Relaxed);
        
        Ok(QuantumTreeStatistics::new(
            total_nodes,
            total_visits,
            total_entanglements,
            avg_decoherence,
            max_amplitude,
            metrics_read.clone(),
            depth_stats,
            reward_stats,
            convergence_metrics,
            performance_metrics,
        ))
    }
    
    /// Record selection operation (lock-free)
    #[inline(always)]
    pub fn record_selection(&self) {
        self.total_selections.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record expansion operation (lock-free)
    #[inline(always)]
    pub fn record_expansion(&self) {
        self.total_expansions.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record backpropagation operation (lock-free)
    #[inline(always)]
    pub fn record_backpropagation(&self) {
        self.total_backpropagations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record simulation operation (lock-free)
    #[inline(always)]
    pub fn record_simulation(&self) {
        self.total_simulations.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current counter values as snapshot
    pub fn get_counter_values(&self) -> CounterSnapshot {
        CounterSnapshot::new(
            self.total_nodes.load(Ordering::Relaxed),
            self.total_visits.load(Ordering::Relaxed),
            self.total_selections.load(Ordering::Relaxed),
            self.total_expansions.load(Ordering::Relaxed),
            self.total_backpropagations.load(Ordering::Relaxed),
            self.total_simulations.load(Ordering::Relaxed),
        )
    }
    
    /// Get elapsed time since collector creation
    pub fn elapsed_time(&self) -> Duration {
        self.atomic_ops.elapsed_time()
    }
    
    /// Take statistics snapshot for trend analysis
    pub async fn take_snapshot(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        entanglement_graph: &RwLock<EntanglementGraph>,
        quantum_metrics: &RwLock<QuantumMetrics>,
    ) -> Result<(), CognitiveError> {
        let statistics = self.collect_statistics(tree, entanglement_graph, quantum_metrics).await?;
        self.atomic_ops.take_snapshot(statistics).await?;
        debug!("Statistics snapshot taken at {} nodes", statistics.total_nodes);
        Ok(())
    }
    
    /// Get statistics history for trend analysis
    pub async fn get_history(&self) -> Vec<crate::cognitive::quantum_mcts::statistics::types::StatisticsSnapshot> {
        self.atomic_ops.get_history().await
    }
    
    /// Get recent snapshots (last N entries)
    pub async fn get_recent_snapshots(&self, count: usize) -> Vec<crate::cognitive::quantum_mcts::statistics::types::StatisticsSnapshot> {
        self.atomic_ops.get_recent_snapshots(count).await
    }
    
    /// Get history size
    pub async fn history_size(&self) -> usize {
        self.atomic_ops.history_size().await
    }
    
    /// Clear history
    pub async fn clear_history(&self) {
        self.atomic_ops.clear_history().await;
    }
    
    /// Reset all counters
    pub fn reset_counters(&self) {
        self.total_nodes.store(0, Ordering::Relaxed);
        self.total_visits.store(0, Ordering::Relaxed);
        self.total_selections.store(0, Ordering::Relaxed);
        self.total_expansions.store(0, Ordering::Relaxed);
        self.total_backpropagations.store(0, Ordering::Relaxed);
        self.total_simulations.store(0, Ordering::Relaxed);
        self.atomic_ops.reset_snapshot_time();
    }
}

// CounterSnapshot is already imported above, no need to re-export
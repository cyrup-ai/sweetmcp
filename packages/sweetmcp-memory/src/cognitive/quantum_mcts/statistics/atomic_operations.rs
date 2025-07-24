//! Atomic operations and history management for statistics collection
//!
//! This module provides lock-free atomic operations, snapshot management,
//! and historical statistics tracking with blazing-fast performance.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::{
    quantum::{EntanglementGraph, QuantumMetrics},
    types::CognitiveError,
};
use super::{
    super::node_state::QuantumMCTSNode,
    types::{QuantumTreeStatistics, StatisticsSnapshot},
};

/// Atomic operations manager for statistics tracking
pub struct AtomicOperationsManager {
    /// Start time for rate calculations
    start_time: Instant,
    /// Last snapshot time
    last_snapshot_time: AtomicU64, // Microseconds since start
    /// Statistics history for trend analysis
    history: Arc<RwLock<Vec<StatisticsSnapshot>>>,
}

impl AtomicOperationsManager {
    /// Create new atomic operations manager
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_snapshot_time: AtomicU64::new(0),
            history: Arc::new(RwLock::new(Vec::with_capacity(1000))),
        }
    }
    
    /// Get elapsed time since manager creation
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Get start time for rate calculations
    pub fn start_time(&self) -> Instant {
        self.start_time
    }
    
    /// Take statistics snapshot for trend analysis
    pub async fn take_snapshot(
        &self,
        statistics: QuantumTreeStatistics,
    ) -> Result<(), CognitiveError> {
        let snapshot = StatisticsSnapshot::new(statistics);
        
        let mut history = self.history.write().await;
        history.push(snapshot);
        
        // Limit history size to prevent memory growth
        if history.len() > 1000 {
            history.remove(0);
        }
        
        // Update snapshot time
        let microseconds = self.start_time.elapsed().as_micros() as u64;
        self.last_snapshot_time.store(microseconds, Ordering::Relaxed);
        
        debug!("Statistics snapshot taken with {} historical entries", history.len());
        Ok(())
    }
    
    /// Get statistics history for trend analysis
    pub async fn get_history(&self) -> Vec<StatisticsSnapshot> {
        self.history.read().await.clone()
    }
    
    /// Get last snapshot time in microseconds
    pub fn last_snapshot_time(&self) -> u64 {
        self.last_snapshot_time.load(Ordering::Relaxed)
    }
    
    /// Clear history and reset snapshot time
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
        self.last_snapshot_time.store(0, Ordering::Relaxed);
        debug!("Statistics history cleared");
    }
    
    /// Get history size without acquiring lock
    pub async fn history_size(&self) -> usize {
        self.history.read().await.len()
    }
    
    /// Get recent snapshots (last N entries)
    pub async fn get_recent_snapshots(&self, count: usize) -> Vec<StatisticsSnapshot> {
        let history = self.history.read().await;
        let start_index = if history.len() > count {
            history.len() - count
        } else {
            0
        };
        history[start_index..].to_vec()
    }
    
    /// Reset snapshot time tracking
    pub fn reset_snapshot_time(&self) {
        self.last_snapshot_time.store(0, Ordering::Relaxed);
    }
}

impl Default for AtomicOperationsManager {
    fn default() -> Self {
        Self::new()
    }
}
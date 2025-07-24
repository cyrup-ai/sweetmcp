//! Entanglement distribution balancing operations
//!
//! This module provides comprehensive balancing operations for quantum entanglement
//! distribution with zero allocation optimizations and blazing-fast performance.

pub mod balancing;

pub use balancing::{
    NodeBalance, NetworkBalanceAnalysis, DistributionStatistics, BalanceAnalyzer,
    BalancingStrategy, BalancingStrategyType, PriorityWeights, NodeRebalancingPriority,
    RebalancingAction, StrategySelector, NetworkComplexity, PerformanceHistory, OperationResult,
    BalancingOperations, BalancingResult, OperationMetrics, CacheStatistics,
    BalancingCoordinator,
};

use std::collections::HashMap;
use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::{
    super::analysis::NetworkTopology,
    super::node_state::QuantumMCTSNode,
};

/// High-level entanglement distribution balancing facade
pub struct EntanglementDistributionBalancing {
    coordinator: BalancingCoordinator,
}

impl Default for EntanglementDistributionBalancing {
    fn default() -> Self {
        Self::new()
    }
}

impl EntanglementDistributionBalancing {
    /// Create new balancing instance with default strategy
    #[inline]
    pub fn new() -> Self {
        Self {
            coordinator: BalancingCoordinator::new(),
        }
    }

    /// Create instance with custom strategy
    #[inline]
    pub fn with_strategy(strategy: BalancingStrategy) -> Self {
        Self {
            coordinator: BalancingCoordinator::with_strategy(strategy),
        }
    }

    /// Create instance optimized for speed
    #[inline]
    pub fn speed_optimized() -> Self {
        Self::with_strategy(BalancingStrategy::speed_optimized())
    }

    /// Create instance optimized for quality
    #[inline]
    pub fn quality_optimized() -> Self {
        Self::with_strategy(BalancingStrategy::quality_optimized())
    }

    /// Create adaptive instance based on network characteristics
    #[inline]
    pub fn adaptive(topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> Self {
        Self {
            coordinator: BalancingCoordinator::adaptive(topology, analysis),
        }
    }

    /// Execute balancing operation on the quantum MCTS tree
    pub async fn balance(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        debug!("Starting entanglement distribution balancing for {} nodes", tree.len());
        self.coordinator.balance_network(tree, topology).await
    }

    /// Get current performance metrics
    #[inline]
    pub fn get_metrics(&self) -> &OperationMetrics {
        self.coordinator.get_metrics()
    }

    /// Get performance history
    #[inline]
    pub fn get_performance_history(&self) -> &PerformanceHistory {
        self.coordinator.get_performance_history()
    }

    /// Reset coordinator state
    #[inline]
    pub fn reset(&mut self) {
        self.coordinator.reset();
    }

    /// Get cache statistics
    #[inline]
    pub fn get_cache_stats(&self) -> CacheStatistics {
        self.coordinator.get_cache_stats()
    }
}

/// Convenience functions for quick balancing operations
impl EntanglementDistributionBalancing {
    /// Quick balance with default settings
    pub async fn quick_balance(
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        let mut balancer = Self::speed_optimized();
        balancer.balance(tree, topology).await
    }

    /// Quality balance with optimal settings
    pub async fn quality_balance(
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        let mut balancer = Self::quality_optimized();
        balancer.balance(tree, topology).await
    }

    /// Adaptive balance based on network conditions
    pub async fn adaptive_balance(
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        // Analyze network first
        let analysis = BalanceAnalyzer::analyze_network_balance(tree, topology)?;
        let mut balancer = Self::adaptive(topology, &analysis);
        balancer.balance(tree, topology).await
    }
}
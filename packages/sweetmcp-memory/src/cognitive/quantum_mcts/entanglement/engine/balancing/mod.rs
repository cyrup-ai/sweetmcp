//! Entanglement distribution balancing module
//!
//! This module provides comprehensive balancing operations for quantum entanglement
//! distribution with zero allocation optimizations and blazing-fast performance.

pub mod balance_analysis;
pub mod balancing_strategy;
pub mod balancing_operations;

pub use balance_analysis::{
    NodeBalance, NetworkBalanceAnalysis, DistributionStatistics, BalanceAnalyzer,
};
pub use balancing_strategy::{
    BalancingStrategy, BalancingStrategyType, PriorityWeights, NodeRebalancingPriority,
    RebalancingAction, StrategySelector, NetworkComplexity, PerformanceHistory, OperationResult,
};
pub use balancing_operations::{
    BalancingOperations, BalancingResult, OperationMetrics, CacheStatistics,
};

use std::collections::HashMap;
use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::super::super::{
    analysis::NetworkTopology,
    node_state::QuantumMCTSNode,
};

/// High-level balancing coordinator with ergonomic interface
pub struct BalancingCoordinator {
    operations: BalancingOperations,
    performance_history: PerformanceHistory,
}

impl BalancingCoordinator {
    /// Create new balancing coordinator with default strategy
    #[inline]
    pub fn new() -> Self {
        Self {
            operations: BalancingOperations::new(BalancingStrategy::default()),
            performance_history: PerformanceHistory::new(),
        }
    }

    /// Create coordinator with custom strategy
    #[inline]
    pub fn with_strategy(strategy: BalancingStrategy) -> Self {
        Self {
            operations: BalancingOperations::new(strategy),
            performance_history: PerformanceHistory::new(),
        }
    }

    /// Create coordinator with adaptive strategy based on network
    #[inline]
    pub fn adaptive(topology: &NetworkTopology, analysis: &NetworkBalanceAnalysis) -> Self {
        let strategy = BalancingStrategy::adaptive(topology, analysis);
        Self::with_strategy(strategy)
    }

    /// Execute balancing with automatic strategy selection
    pub async fn balance_network(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        topology: &NetworkTopology,
    ) -> Result<BalancingResult, CognitiveError> {
        debug!("Starting network balancing coordination");

        // Analyze current state
        let analysis = BalanceAnalyzer::analyze_network_balance(tree, topology)?;
        
        // Select optimal strategy based on conditions
        let optimal_strategy = StrategySelector::select_optimal_strategy(
            topology,
            &analysis,
            &self.performance_history,
        );
        
        // Update strategy if needed
        self.operations.update_strategy(optimal_strategy);
        
        // Execute balancing
        let result = self.operations.execute_balancing(tree, topology).await?;
        
        // Record performance for future strategy selection
        self.performance_history.add_result(result.to_operation_result());
        
        debug!("Network balancing completed: {}", result.get_summary());
        Ok(result)
    }

    /// Get current performance metrics
    #[inline]
    pub fn get_metrics(&self) -> &OperationMetrics {
        self.operations.get_metrics()
    }

    /// Get performance history
    #[inline]
    pub fn get_performance_history(&self) -> &PerformanceHistory {
        &self.performance_history
    }

    /// Clear caches and reset state
    #[inline]
    pub fn reset(&mut self) {
        self.operations.clear_cache();
        debug!("Balancing coordinator reset");
    }

    /// Get cache statistics
    #[inline]
    pub fn get_cache_stats(&self) -> CacheStatistics {
        self.operations.get_cache_stats()
    }
}

impl Default for BalancingCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macros for balancing operations
#[macro_export]
macro_rules! balance_network {
    ($tree:expr, $topology:expr) => {{
        let mut coordinator = BalancingCoordinator::new();
        coordinator.balance_network($tree, $topology).await
    }};
    
    ($tree:expr, $topology:expr, $strategy:expr) => {{
        let mut coordinator = BalancingCoordinator::with_strategy($strategy);
        coordinator.balance_network($tree, $topology).await
    }};
}

/// Convenience macro for creating balancing strategies
#[macro_export]
macro_rules! balancing_strategy {
    (speed) => {
        BalancingStrategy::speed_optimized()
    };
    
    (quality) => {
        BalancingStrategy::quality_optimized()
    };
    
    (adaptive, $topology:expr, $analysis:expr) => {
        BalancingStrategy::adaptive($topology, $analysis)
    };
    
    (custom, $ratio:expr, $max_redistributions:expr, $threshold:expr, $factor:expr) => {
        BalancingStrategy::new($ratio, $max_redistributions, $threshold, $factor)
    };
}

/// Convenience macro for balance analysis
#[macro_export]
macro_rules! analyze_balance {
    ($tree:expr, $topology:expr) => {
        BalanceAnalyzer::analyze_network_balance($tree, $topology)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balancing_coordinator_creation() {
        let coordinator = BalancingCoordinator::new();
        assert_eq!(coordinator.get_metrics().total_operations, 0);
    }

    #[test]
    fn test_balancing_strategy_macros() {
        let _speed_strategy = balancing_strategy!(speed);
        let _quality_strategy = balancing_strategy!(quality);
        let _custom_strategy = balancing_strategy!(custom, 0.8, 100, 2.0, 0.7);
    }
}
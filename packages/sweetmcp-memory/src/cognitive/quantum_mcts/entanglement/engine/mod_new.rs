//! Quantum entanglement engine coordination module
//!
//! This module coordinates all quantum entanglement engine submodules with blazing-fast performance
//! and zero allocation optimizations, integrating all engine components.

pub mod core_types;
pub mod factory;
pub mod operations;

// Re-export existing submodules
pub mod core;
pub mod optimization;
pub mod pruning;
pub mod balancing;
pub mod health;

// Re-export key types and functions for ergonomic access
pub use core_types::{
    EngineOperationResult, EngineOperationType, EngineOperationDetails,
    EngineStatistics, EnginePerformanceReport, PerformanceGrades,
};

pub use factory::{
    QuantumEntanglementEngineFactory, NetworkSize, OptimizationProfile, PerformanceRequirements,
};

pub use core::{QuantumEntanglementEngine, EngineStatus};
pub use optimization::{OptimizationResult, CreationResult};
pub use pruning::{PruningResult, PruningStrategy, StrengthStatistics, RecentPruningStatistics};
pub use balancing::{BalancingResult, NodeBalance, BalancingStrategy, NetworkBalanceAnalysis, DistributionStatistics};
pub use health::{NetworkHealthReport, HealthCheckConfig, HealthIssue, IssueSeverity};

// Common imports for all submodules
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::super::{
    core::QuantumEntanglementManager,
    analysis::{NetworkTopology, NetworkTopologyAnalyzer},
    metrics::{EntanglementMetrics, PerformanceTracker},
    config::QuantumMCTSConfig,
};
use super::super::super::{
    node_state::QuantumMCTSNode,
};

/// Comprehensive engine management facade for quantum entanglement operations
pub struct QuantumEntanglementEngineManager {
    engine: QuantumEntanglementEngine,
    factory: QuantumEntanglementEngineFactory,
    statistics: EngineStatistics,
}

impl QuantumEntanglementEngineManager {
    /// Create new engine manager with optimized configuration
    #[inline]
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        let factory = QuantumEntanglementEngineFactory;
        let engine = factory.create_balanced(config, entanglement_graph);
        let statistics = EngineStatistics::new();

        Self {
            engine,
            factory,
            statistics,
        }
    }

    /// Create engine manager with specific optimization profile
    #[inline]
    pub fn with_profile(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        profile: OptimizationProfile,
    ) -> Self {
        let factory = QuantumEntanglementEngineFactory;
        let engine = factory.create_with_profile(config, entanglement_graph, profile);
        let statistics = EngineStatistics::new();

        Self {
            engine,
            factory,
            statistics,
        }
    }

    /// Create engine manager with adaptive configuration
    #[inline]
    pub fn adaptive(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Result<Self, CognitiveError> {
        let factory = QuantumEntanglementEngineFactory;
        let engine = factory.create_adaptive(config, entanglement_graph)?;
        let statistics = EngineStatistics::new();

        Ok(Self {
            engine,
            factory,
            statistics,
        })
    }

    /// Perform comprehensive operation with statistics tracking
    #[inline]
    pub async fn perform_operation(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        operation_type: EngineOperationType,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let result = self.engine.perform_comprehensive_operation(tree, operation_type).await?;
        
        // Update statistics with operation result
        self.statistics.update_with_operation(&result);
        
        Ok(result)
    }

    /// Perform combined optimization with full statistics tracking
    #[inline]
    pub async fn perform_combined_optimization(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<EngineOperationResult, CognitiveError> {
        let start_time = Instant::now();
        let result = self.engine.perform_combined_optimization(tree, start_time).await?;
        
        // Update statistics with operation result
        self.statistics.update_with_operation(&result);
        
        Ok(result)
    }

    /// Perform automatic maintenance with comprehensive tracking
    #[inline]
    pub async fn perform_maintenance(
        &mut self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<EngineOperationResult>, CognitiveError> {
        let results = self.engine.perform_automatic_maintenance(tree).await?;
        
        // Update statistics with all operation results
        for result in &results {
            self.statistics.update_with_operation(result);
        }
        
        Ok(results)
    }

    /// Get current engine statistics
    #[inline]
    pub fn get_statistics(&self) -> &EngineStatistics {
        &self.statistics
    }

    /// Get comprehensive engine statistics
    #[inline]
    pub fn get_comprehensive_statistics(&self) -> Result<EngineStatistics, CognitiveError> {
        self.engine.get_comprehensive_statistics()
    }

    /// Create performance report
    #[inline]
    pub fn create_performance_report(&self) -> Result<EnginePerformanceReport, CognitiveError> {
        self.engine.create_performance_report()
    }

    /// Check if engine is performing optimally
    #[inline]
    pub fn is_optimal(&self) -> bool {
        self.statistics.is_optimal()
    }

    /// Get engine reference for direct access
    #[inline]
    pub fn engine(&self) -> &QuantumEntanglementEngine {
        &self.engine
    }

    /// Get mutable engine reference for direct access
    #[inline]
    pub fn engine_mut(&mut self) -> &mut QuantumEntanglementEngine {
        &mut self.engine
    }

    /// Reconfigure engine with new profile
    #[inline]
    pub fn reconfigure_with_profile(
        &mut self,
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        profile: OptimizationProfile,
    ) {
        self.engine = self.factory.create_with_profile(config, entanglement_graph, profile);
        self.statistics = EngineStatistics::new(); // Reset statistics
    }

    /// Upgrade engine based on performance requirements
    #[inline]
    pub fn upgrade_for_requirements(
        &mut self,
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        requirements: &PerformanceRequirements,
    ) {
        let recommended_profile = self.factory.recommend_configuration(requirements);
        self.reconfigure_with_profile(config, entanglement_graph, recommended_profile);
    }
}

/// Convenience macros for common engine operations
#[macro_export]
macro_rules! perform_engine_operation {
    ($manager:expr, $tree:expr, $operation:expr) => {
        $manager.perform_operation($tree, $operation).await
    };
}

#[macro_export]
macro_rules! create_engine_manager {
    ($config:expr, $graph:expr) => {
        QuantumEntanglementEngineManager::new($config, $graph)
    };
    ($config:expr, $graph:expr, $profile:expr) => {
        QuantumEntanglementEngineManager::with_profile($config, $graph, $profile)
    };
    ($config:expr, $graph:expr, adaptive) => {
        QuantumEntanglementEngineManager::adaptive($config, $graph)
    };
}

#[macro_export]
macro_rules! engine_factory {
    (high_performance, $config:expr, $graph:expr) => {
        QuantumEntanglementEngineFactory::create_high_performance($config, $graph)
    };
    (low_latency, $config:expr, $graph:expr) => {
        QuantumEntanglementEngineFactory::create_low_latency($config, $graph)
    };
    (memory_optimized, $config:expr, $graph:expr) => {
        QuantumEntanglementEngineFactory::create_memory_optimized($config, $graph)
    };
    (balanced, $config:expr, $graph:expr) => {
        QuantumEntanglementEngineFactory::create_balanced($config, $graph)
    };
}
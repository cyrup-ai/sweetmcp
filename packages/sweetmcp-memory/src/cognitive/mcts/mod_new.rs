//! MCTS module coordination
//!
//! This module provides the main coordination layer for Monte Carlo Tree Search operations,
//! integrating all submodules with blazing-fast performance and zero allocation optimizations.

pub mod types;
pub mod tree_operations;
pub mod execution;
pub mod analysis;
pub mod actions;
pub mod controller;
pub mod runner;
pub mod results;
pub mod factory;

// Re-export key types for ergonomic access
pub use types::{
    CodeState, MCTSNode, TreeStatistics, NodeStatistics, EfficiencyMetrics, ActionMetadata,
};
pub use tree_operations::{
    TreeOperations, TreeEfficiency, OptimizationResult, EfficiencyCategory,
};
pub use execution::{
    MCTSExecutor, ExecutionResult, ExecutionCategory, ExecutionRecommendation, 
    ExecutionSummary, RecommendationPriority,
};
pub use analysis::{
    TreeAnalyzer, PathInfo, NodeCriteria, NodeMatch, TreeStructureAnalysis,
    VisitStatistics, Bottleneck, BottleneckType, BottleneckSeverity,
};
pub use actions::{
    ActionGenerator, ActionApplicator, ActionCoordinator, CacheStatistics,
    ApplicationStatistics, CoordinatorStatistics,
};
pub use controller::MCTS;
pub use results::{MCTSResult, MemoryUsage, PerformanceSummary, MemoryDistribution};
pub use factory::{MCTSFactory, OptimizationProfile, PerformanceRequirements};

// Convenience macros for common MCTS operations
#[macro_export]
macro_rules! create_mcts {
    ($state:expr, $analyzer:expr, $spec:expr, $objective:expr, $tx:expr) => {
        MCTS::new($state, $analyzer, $spec, $objective, $tx).await
    };
    ($state:expr, $analyzer:expr, $spec:expr, $objective:expr, $tx:expr, $profile:expr) => {
        MCTSFactory::create_with_profile($state, $analyzer, $spec, $objective, $tx, $profile).await
    };
}

#[macro_export]
macro_rules! run_mcts {
    ($mcts:expr, $iterations:expr) => {
        $mcts.run($iterations).await
    };
    ($mcts:expr, $iterations:expr, parallel) => {
        $mcts.run_parallel($iterations).await
    };
    ($mcts:expr, $iterations:expr, adaptive) => {
        $mcts.run_adaptive($iterations, 1000, $iterations * 2).await
    };
}

#[macro_export]
macro_rules! mcts_factory {
    (speed, $($args:expr),*) => {
        MCTSFactory::create_speed_optimized($($args),*).await
    };
    (quality, $($args:expr),*) => {
        MCTSFactory::create_quality_optimized($($args),*).await
    };
    (balanced, $($args:expr),*) => {
        MCTSFactory::create_balanced($($args),*).await
    };
    (memory, $($args:expr),*) => {
        MCTSFactory::create_memory_optimized($($args),*).await
    };
    (realtime, $($args:expr),*) => {
        MCTSFactory::create_realtime($($args),*).await
    };
    (batch, $($args:expr),*) => {
        MCTSFactory::create_batch_processing($($args),*).await
    };
}

/// Comprehensive MCTS manager for high-level operations
pub struct MCTSManager {
    mcts: MCTS,
    factory: MCTSFactory,
    current_profile: OptimizationProfile,
}

impl MCTSManager {
    /// Create new MCTS manager with balanced configuration
    #[inline]
    pub async fn new(
        initial_state: CodeState,
        performance_analyzer: std::sync::Arc<crate::cognitive::performance::PerformanceAnalyzer>,
        spec: std::sync::Arc<crate::cognitive::types::OptimizationSpec>,
        user_objective: String,
        event_tx: tokio::sync::mpsc::Sender<crate::cognitive::committee::CommitteeEvent>,
    ) -> Result<Self, crate::cognitive::types::CognitiveError> {
        let factory = MCTSFactory;
        let mcts = factory.create_balanced(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
        ).await?;

        Ok(Self {
            mcts,
            factory,
            current_profile: OptimizationProfile::Balanced,
        })
    }

    /// Create MCTS manager with specific profile
    #[inline]
    pub async fn with_profile(
        initial_state: CodeState,
        performance_analyzer: std::sync::Arc<crate::cognitive::performance::PerformanceAnalyzer>,
        spec: std::sync::Arc<crate::cognitive::types::OptimizationSpec>,
        user_objective: String,
        event_tx: tokio::sync::mpsc::Sender<crate::cognitive::committee::CommitteeEvent>,
        profile: OptimizationProfile,
    ) -> Result<Self, crate::cognitive::types::CognitiveError> {
        let factory = MCTSFactory;
        let mcts = factory.create_with_profile(
            initial_state,
            performance_analyzer,
            spec,
            user_objective,
            event_tx,
            profile,
        ).await?;

        Ok(Self {
            mcts,
            factory,
            current_profile: profile,
        })
    }

    /// Run MCTS with current configuration
    #[inline]
    pub async fn run(&mut self, iterations: u64) -> Result<MCTSResult, crate::cognitive::types::CognitiveError> {
        self.mcts.run(iterations).await
    }

    /// Run parallel MCTS
    #[inline]
    pub async fn run_parallel(&mut self, iterations: u64) -> Result<MCTSResult, crate::cognitive::types::CognitiveError> {
        self.mcts.run_parallel(iterations).await
    }

    /// Run adaptive MCTS
    #[inline]
    pub async fn run_adaptive(
        &mut self,
        initial_iterations: u64,
        adaptation_interval: u64,
        max_total_iterations: u64,
    ) -> Result<MCTSResult, crate::cognitive::types::CognitiveError> {
        self.mcts.run_adaptive(initial_iterations, adaptation_interval, max_total_iterations).await
    }

    /// Get current optimization profile
    #[inline]
    pub fn current_profile(&self) -> OptimizationProfile {
        self.current_profile
    }

    /// Get MCTS reference
    #[inline]
    pub fn mcts(&self) -> &MCTS {
        &self.mcts
    }

    /// Get mutable MCTS reference
    #[inline]
    pub fn mcts_mut(&mut self) -> &mut MCTS {
        &mut self.mcts
    }

    /// Get performance summary
    #[inline]
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        self.mcts.get_performance_summary()
    }

    /// Get memory usage
    #[inline]
    pub fn get_memory_usage(&self) -> MemoryUsage {
        self.mcts.get_memory_usage()
    }

    /// Check if performance is satisfactory
    #[inline]
    pub fn is_performance_satisfactory(&self, threshold: f64) -> bool {
        self.get_performance_summary().is_satisfactory(threshold)
    }

    /// Optimize tree structure
    #[inline]
    pub fn optimize_tree(&mut self) -> tree_operations::OptimizationResult {
        self.mcts.optimize_tree()
    }

    /// Clear caches to free memory
    #[inline]
    pub fn clear_caches(&mut self) {
        self.mcts.clear_caches();
    }

    /// Reset MCTS to new initial state
    #[inline]
    pub fn reset(&mut self, initial_state: CodeState) -> Result<(), crate::cognitive::types::CognitiveError> {
        self.mcts.reset(initial_state)
    }
}
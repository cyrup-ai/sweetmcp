//! MCTS module coordination
//!
//! This module provides the main coordination layer for Monte Carlo Tree Search operations,
//! integrating all submodules with blazing-fast performance and zero allocation optimizations.

pub mod actions;
pub mod analysis;
pub mod controller;
pub mod execution;
pub mod factory;
pub mod manager;
pub mod results;
pub mod runner;
pub mod tree_operations;
pub mod types;

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
pub use manager::MCTSManager;
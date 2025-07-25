//! MCTS Manager - High-level MCTS operations coordination
//!
//! This module provides the MCTSManager for comprehensive MCTS management with
//! optimization profiles, factory integration, and performance tracking.

use crate::cognitive::mcts::{
    controller::MCTS,
    factory::{MCTSFactory, OptimizationProfile},
    results::{MCTSResult, MemoryUsage, PerformanceSummary},
    tree_operations,
    types::CodeState,
};

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
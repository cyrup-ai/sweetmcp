//! MCTS controller for main coordination and execution
//!
//! This module provides blazing-fast MCTS controller with zero allocation
//! optimizations and elegant ergonomic interfaces for Monte Carlo Tree Search operations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{debug, error, info};

use crate::cognitive::committee::{CommitteeEvent, EvaluationCommittee};
use crate::cognitive::performance::PerformanceAnalyzer;
use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;

use super::{
    types::{CodeState, MCTSNode, TreeStatistics},
    tree_operations::TreeOperations,
    execution::{MCTSExecutor, ExecutionResult},
    analysis::{TreeAnalyzer, TreeStructureAnalysis, Bottleneck},
    actions::{ActionCoordinator, CoordinatorStatistics},
};

/// Main MCTS controller with committee-based evaluation
pub struct MCTS {
    tree: HashMap<String, MCTSNode>,
    root_id: String,
    executor: MCTSExecutor,
    analyzer: TreeAnalyzer,
    action_coordinator: ActionCoordinator,
    performance_analyzer: Arc<PerformanceAnalyzer>,
    optimization_spec: Arc<OptimizationSpec>,
    user_objective: String,
    max_parallel: usize,
}

impl MCTS {
    /// Create new MCTS instance with zero allocation optimizations
    #[inline]
    pub async fn new(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<Self, CognitiveError> {
        let committee = Arc::new(EvaluationCommittee::new(event_tx, num_cpus::get().min(4)).await?);

        let root_id = "root".to_string();
        
        // Create action coordinator
        let action_coordinator = ActionCoordinator::new(
            spec.clone(),
            committee.clone(),
            user_objective.clone(),
        );

        // Get initial actions for root node
        let mut temp_coordinator = ActionCoordinator::new(
            spec.clone(),
            committee.clone(),
            user_objective.clone(),
        );
        let untried_actions = temp_coordinator.get_possible_actions(&initial_state);

        // Create root node
        let root_node = MCTSNode::new_root(initial_state, untried_actions);
        let mut tree = HashMap::new();
        tree.insert(root_id.clone(), root_node);

        // Create executor with optimized parameters
        let executor = MCTSExecutor::new(
            1.41, // sqrt(2) for UCT
            10000, // max iterations
            Some(Duration::from_secs(300)), // 5 minute timeout
        );

        Ok(Self {
            tree,
            root_id,
            executor,
            analyzer: TreeAnalyzer,
            action_coordinator,
            performance_analyzer,
            optimization_spec: spec,
            user_objective,
            max_parallel: num_cpus::get().min(8),
        })
    }

    /// Create MCTS with custom configuration
    #[inline]
    pub async fn with_config(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
        exploration_constant: f64,
        max_iterations: u64,
        time_limit: Option<Duration>,
    ) -> Result<Self, CognitiveError> {
        let committee = Arc::new(EvaluationCommittee::new(event_tx, num_cpus::get().min(4)).await?);

        let root_id = "root".to_string();
        
        let action_coordinator = ActionCoordinator::new(
            spec.clone(),
            committee.clone(),
            user_objective.clone(),
        );

        let mut temp_coordinator = ActionCoordinator::new(
            spec.clone(),
            committee.clone(),
            user_objective.clone(),
        );
        let untried_actions = temp_coordinator.get_possible_actions(&initial_state);

        let root_node = MCTSNode::new_root(initial_state, untried_actions);
        let mut tree = HashMap::new();
        tree.insert(root_id.clone(), root_node);

        let executor = MCTSExecutor::new(exploration_constant, max_iterations, time_limit);

        Ok(Self {
            tree,
            root_id,
            executor,
            analyzer: TreeAnalyzer,
            action_coordinator,
            performance_analyzer,
            optimization_spec: spec,
            user_objective,
            max_parallel: num_cpus::get().min(8),
        })
    }

    /// Get best modification found so far
    #[inline]
    pub fn best_modification(&self) -> Option<CodeState> {
        self.executor.get_best_modification(&self.tree, &self.root_id)
    }

    /// Get statistics about the search tree
    #[inline]
    pub fn get_statistics(&self) -> TreeStatistics {
        TreeStatistics::from_tree(&self.tree, &self.root_id)
    }

    /// Get comprehensive tree analysis
    #[inline]
    pub fn get_tree_analysis(&self) -> TreeStructureAnalysis {
        self.analyzer.analyze_tree_structure(&self.tree, &self.root_id)
    }

    /// Get best path from root to best leaf
    #[inline]
    pub fn get_best_path(&self) -> Vec<String> {
        self.executor.get_best_path(&self.tree, &self.root_id)
    }

    /// Find bottlenecks in the search tree
    #[inline]
    pub fn find_bottlenecks(&self) -> Vec<Bottleneck> {
        self.analyzer.find_bottlenecks(&self.tree, &self.root_id)
    }

    /// Validate MCTS state consistency
    #[inline]
    pub fn validate_state(&self) -> Result<(), CognitiveError> {
        self.executor.validate_execution_state(&self.tree, &self.root_id)
    }

    /// Optimize tree structure for better performance
    #[inline]
    pub fn optimize_tree(&mut self) -> super::tree_operations::OptimizationResult {
        let tree_ops = TreeOperations::default();
        tree_ops.optimize_tree_structure(&mut self.tree, &self.root_id)
    }

    /// Clear caches to free memory
    #[inline]
    pub fn clear_caches(&mut self) {
        self.action_coordinator.clear_caches();
    }

    /// Get tree reference for direct access
    #[inline]
    pub fn tree(&self) -> &HashMap<String, MCTSNode> {
        &self.tree
    }

    /// Get mutable tree reference for direct access
    #[inline]
    pub fn tree_mut(&mut self) -> &mut HashMap<String, MCTSNode> {
        &mut self.tree
    }

    /// Get root node ID
    #[inline]
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Get executor reference
    #[inline]
    pub fn executor(&self) -> &MCTSExecutor {
        &self.executor
    }

    /// Get mutable executor reference
    #[inline]
    pub fn executor_mut(&mut self) -> &mut MCTSExecutor {
        &mut self.executor
    }

    /// Get analyzer reference
    #[inline]
    pub fn analyzer(&self) -> &TreeAnalyzer {
        &self.analyzer
    }

    /// Get action coordinator reference
    #[inline]
    pub fn action_coordinator(&self) -> &ActionCoordinator {
        &self.action_coordinator
    }

    /// Get mutable action coordinator reference
    #[inline]
    pub fn action_coordinator_mut(&mut self) -> &mut ActionCoordinator {
        &mut self.action_coordinator
    }

    /// Get performance analyzer reference
    #[inline]
    pub fn performance_analyzer(&self) -> &Arc<PerformanceAnalyzer> {
        &self.performance_analyzer
    }

    /// Get optimization spec reference
    #[inline]
    pub fn optimization_spec(&self) -> &Arc<OptimizationSpec> {
        &self.optimization_spec
    }

    /// Get user objective
    #[inline]
    pub fn user_objective(&self) -> &str {
        &self.user_objective
    }

    /// Get maximum parallel operations
    #[inline]
    pub fn max_parallel(&self) -> usize {
        self.max_parallel
    }

    /// Set maximum parallel operations
    #[inline]
    pub fn set_max_parallel(&mut self, max_parallel: usize) {
        self.max_parallel = max_parallel.max(1).min(num_cpus::get());
    }

    /// Check if tree is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get tree size
    #[inline]
    pub fn tree_size(&self) -> usize {
        self.tree.len()
    }

    /// Check if root node exists
    #[inline]
    pub fn has_root(&self) -> bool {
        self.tree.contains_key(&self.root_id)
    }

    /// Reset MCTS to initial state
    #[inline]
    pub fn reset(&mut self, initial_state: CodeState) -> Result<(), CognitiveError> {
        // Clear existing tree
        self.tree.clear();

        // Get initial actions for new root node
        let untried_actions = self.action_coordinator.get_possible_actions(&initial_state);

        // Create new root node
        let root_node = MCTSNode::new_root(initial_state, untried_actions);
        self.tree.insert(self.root_id.clone(), root_node);

        // Clear caches
        self.clear_caches();

        debug!("MCTS reset to initial state");
        Ok(())
    }
}
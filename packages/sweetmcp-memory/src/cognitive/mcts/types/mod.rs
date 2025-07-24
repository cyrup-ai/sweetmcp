//! MCTS types coordination module
//!
//! This module provides a high-level coordination facade for MCTS type operations
//! with zero allocation optimizations and elegant ergonomic interfaces.

pub mod node_types;
pub mod tree_types;
pub mod action_types;

use std::collections::HashMap;
use std::time::Instant;

// Re-export core types for ergonomic usage
pub use node_types::{
    CodeState, MCTSNode, NodeMetadata, QualityMetrics
};
pub use tree_types::{
    MCTSTree, TreeConfig, TreeMetrics, OptimizationCache, CachedStatistics, TreeStatistics
};
pub use action_types::{
    ActionMetadata, NodeStatistics, EfficiencyMetrics,
    ActionSpace, ActionConstraint, ActionContext, ActionResult, ActionResultMetadata,
    ActionHistoryEntry, ActionSpaceMetadata, ActionSpaceStatistics
};

/// High-level coordinator for MCTS type operations
pub struct MCTSTypeCoordinator {
    /// Tree instance for coordinated operations
    tree: MCTSTree,
    /// Action space for available actions
    action_space: ActionSpace,
    /// Coordination metrics
    metrics: CoordinationMetrics,
    /// Configuration
    config: CoordinatorConfig,
}

impl MCTSTypeCoordinator {
    /// Create new MCTS type coordinator
    #[inline]
    pub fn new(root_state: CodeState, actions: Vec<String>) -> Self {
        let root_node = MCTSNode::create_root(root_state);
        let tree_config = TreeConfig::new();
        let tree = MCTSTree::new(root_node, tree_config);
        let action_space = ActionSpace::new(actions);
        
        Self {
            tree,
            action_space,
            metrics: CoordinationMetrics::new(),
            config: CoordinatorConfig::default(),
        }
    }

    /// Create coordinator with custom configuration
    #[inline]
    pub fn with_config(
        root_state: CodeState,
        actions: Vec<String>,
        config: CoordinatorConfig,
    ) -> Self {
        let mut coordinator = Self::new(root_state, actions);
        coordinator.config = config;
        coordinator
    }

    /// Execute coordinated node expansion with action validation
    #[inline]
    pub fn expand_node(
        &mut self,
        node_index: usize,
        action: String,
        new_state: CodeState,
    ) -> Result<usize, String> {
        let start_time = Instant::now();
        
        // Validate action availability
        let node = self.tree.get_node(node_index)
            .ok_or("Invalid node index")?;
        
        let context = ActionContext::new(
            node.depth + 1,
            node.visits,
            new_state.memory,
        );
        
        let valid_actions = self.action_space.get_valid_actions(&context);
        if !valid_actions.contains(&action) {
            return Err(format!("Action '{}' is not valid in current context", action));
        }
        
        // Create and add child node
        let child_node = MCTSNode::create_child(
            new_state,
            node_index,
            action.clone(),
            node.depth + 1,
        );
        
        let child_index = self.tree.add_node(child_node);
        
        // Update parent node
        if let Some(parent) = self.tree.get_node_mut(node_index) {
            parent.add_child(child_index);
            parent.pop_untried_action(); // Remove the action that was just tried
        }
        
        // Record action execution
        let execution_time = start_time.elapsed();
        let result = ActionResult::success(0.0, execution_time); // Reward will be updated during backpropagation
        self.action_space.record_action(action, result);
        
        // Update metrics
        self.metrics.record_expansion(execution_time);
        
        Ok(child_index)
    }

    /// Execute coordinated node selection using UCB1 and action weights
    #[inline]
    pub fn select_best_child(&self, node_index: usize) -> Option<usize> {
        let node = self.tree.get_node(node_index)?;
        
        let mut best_child = None;
        let mut best_score = f64::NEG_INFINITY;
        
        for &child_index in &node.children {
            if let Some(child) = self.tree.get_node(child_index) {
                // Calculate UCB1 value
                let ucb1_value = child.ucb1_value(
                    node.visits,
                    self.tree.config.exploration_constant,
                );
                
                // Apply action weight if available
                let action_weight = if let Some(ref action) = child.action_taken {
                    self.action_space.get_action_weight(action)
                } else {
                    1.0
                };
                
                let weighted_score = ucb1_value * action_weight;
                
                if weighted_score > best_score {
                    best_score = weighted_score;
                    best_child = Some(child_index);
                }
            }
        }
        
        best_child
    }

    /// Execute coordinated backpropagation with action weight updates
    #[inline]
    pub fn backpropagate(&mut self, node_index: usize, reward: f64) -> Result<(), String> {
        let start_time = Instant::now();
        let mut current_index = node_index;
        let mut propagation_count = 0;
        
        // Traverse up the tree updating nodes
        while let Some(node) = self.tree.get_node_mut(current_index) {
            node.update(reward);
            
            // Update action weights based on performance
            if let Some(ref action) = node.action_taken.clone() {
                let performance = reward.clamp(-1.0, 1.0);
                self.action_space.update_action_weight(action, performance);
            }
            
            propagation_count += 1;
            
            if let Some(parent_index) = node.parent {
                current_index = parent_index;
            } else {
                break; // Reached root
            }
        }
        
        let execution_time = start_time.elapsed();
        self.metrics.record_backpropagation(execution_time, propagation_count);
        
        Ok(())
    }

    /// Get best action sequence from root to best leaf
    #[inline]
    pub fn get_best_action_sequence(&self) -> Vec<String> {
        let mut sequence = Vec::new();
        let mut current_index = self.tree.root_index;
        
        while let Some(best_child_index) = self.tree.get_best_child(current_index) {
            if let Some(child) = self.tree.get_node(best_child_index) {
                if let Some(ref action) = child.action_taken {
                    sequence.push(action.clone());
                }
                current_index = best_child_index;
            } else {
                break;
            }
        }
        
        sequence
    }

    /// Execute tree optimization with action space pruning
    #[inline]
    pub fn optimize_structures(&mut self) -> OptimizationResult {
        let start_time = Instant::now();
        
        // Optimize tree structure
        self.tree.optimize_structure();
        
        // Prune underperforming actions
        let pruned_actions = self.action_space.prune_actions(
            self.config.min_action_success_rate,
            self.config.min_action_usage_count,
        );
        
        // Prune tree nodes
        let pruned_nodes = self.tree.prune_tree(
            self.config.min_node_visits,
            self.config.min_node_reward,
        );
        
        let execution_time = start_time.elapsed();
        self.metrics.record_optimization(execution_time);
        
        OptimizationResult {
            pruned_nodes,
            pruned_actions,
            execution_time,
            memory_saved: self.estimate_memory_saved(pruned_nodes, pruned_actions),
        }
    }

    /// Get comprehensive performance statistics
    #[inline]
    pub fn get_performance_statistics(&self) -> PerformanceStatistics {
        let tree_stats = self.tree.metrics.clone();
        let action_stats = self.action_space.get_statistics();
        
        PerformanceStatistics {
            tree_metrics: tree_stats,
            action_metrics: action_stats,
            coordination_metrics: self.metrics.clone(),
            overall_efficiency: self.calculate_overall_efficiency(),
        }
    }

    /// Calculate overall system efficiency
    #[inline]
    fn calculate_overall_efficiency(&self) -> f64 {
        let tree_efficiency = self.tree.metrics.efficiency_score();
        let action_efficiency = self.action_space.get_statistics().efficiency_score();
        let coordination_efficiency = self.metrics.efficiency_score();
        
        (tree_efficiency * 0.4 + action_efficiency * 0.4 + coordination_efficiency * 0.2)
            .clamp(0.0, 1.0)
    }

    /// Estimate memory saved from optimization
    #[inline]
    fn estimate_memory_saved(&self, pruned_nodes: usize, pruned_actions: usize) -> usize {
        let node_size = std::mem::size_of::<MCTSNode>();
        let action_size = 64; // Estimated average action string size
        
        pruned_nodes * node_size + pruned_actions * action_size
    }

    /// Get tree reference for read operations
    #[inline]
    pub fn tree(&self) -> &MCTSTree {
        &self.tree
    }

    /// Get action space reference for read operations
    #[inline]
    pub fn action_space(&self) -> &ActionSpace {
        &self.action_space
    }

    /// Update coordinator configuration
    #[inline]
    pub fn update_config(&mut self, config: CoordinatorConfig) {
        self.config = config;
    }

    /// Get current configuration
    #[inline]
    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }
}

/// Coordination metrics for performance tracking
#[derive(Debug, Clone)]
pub struct CoordinationMetrics {
    pub total_expansions: u32,
    pub total_backpropagations: u32,
    pub total_optimizations: u32,
    pub total_execution_time_ms: u64,
    pub average_expansion_time_ms: f64,
    pub average_backpropagation_time_ms: f64,
    pub creation_time: Instant,
    pub last_operation_time: Instant,
}

impl CoordinationMetrics {
    /// Create new coordination metrics
    #[inline]
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            total_expansions: 0,
            total_backpropagations: 0,
            total_optimizations: 0,
            total_execution_time_ms: 0,
            average_expansion_time_ms: 0.0,
            average_backpropagation_time_ms: 0.0,
            creation_time: now,
            last_operation_time: now,
        }
    }

    /// Record expansion operation
    #[inline]
    pub fn record_expansion(&mut self, execution_time: std::time::Duration) {
        self.total_expansions += 1;
        let time_ms = execution_time.as_millis() as u64;
        self.total_execution_time_ms += time_ms;
        
        // Update running average
        let alpha = 0.1;
        self.average_expansion_time_ms = alpha * time_ms as f64 + 
            (1.0 - alpha) * self.average_expansion_time_ms;
        
        self.last_operation_time = Instant::now();
    }

    /// Record backpropagation operation
    #[inline]
    pub fn record_backpropagation(&mut self, execution_time: std::time::Duration, _node_count: u32) {
        self.total_backpropagations += 1;
        let time_ms = execution_time.as_millis() as u64;
        self.total_execution_time_ms += time_ms;
        
        // Update running average
        let alpha = 0.1;
        self.average_backpropagation_time_ms = alpha * time_ms as f64 + 
            (1.0 - alpha) * self.average_backpropagation_time_ms;
        
        self.last_operation_time = Instant::now();
    }

    /// Record optimization operation
    #[inline]
    pub fn record_optimization(&mut self, execution_time: std::time::Duration) {
        self.total_optimizations += 1;
        let time_ms = execution_time.as_millis() as u64;
        self.total_execution_time_ms += time_ms;
        self.last_operation_time = Instant::now();
    }

    /// Calculate operations per second
    #[inline]
    pub fn operations_per_second(&self) -> f64 {
        let age_seconds = self.creation_time.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            (self.total_expansions + self.total_backpropagations) as f64 / age_seconds
        } else {
            0.0
        }
    }

    /// Calculate efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        let ops_per_second = self.operations_per_second();
        let time_efficiency = 1.0 / (1.0 + self.average_expansion_time_ms * 0.001);
        let throughput_score = (ops_per_second / 100.0).clamp(0.0, 1.0);
        
        (time_efficiency * 0.6 + throughput_score * 0.4).clamp(0.0, 1.0)
    }

    /// Get age in seconds
    #[inline]
    pub fn age_seconds(&self) -> f64 {
        self.creation_time.elapsed().as_secs_f64()
    }
}

impl Default for CoordinationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Coordinator configuration
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    pub min_action_success_rate: f64,
    pub min_action_usage_count: usize,
    pub min_node_visits: u32,
    pub min_node_reward: f64,
    pub optimization_frequency: u32,
    pub enable_action_weight_updates: bool,
    pub enable_tree_pruning: bool,
    pub max_memory_usage_mb: usize,
}

impl CoordinatorConfig {
    /// Create new coordinator configuration with default values
    #[inline]
    pub fn new() -> Self {
        Self {
            min_action_success_rate: 0.3,
            min_action_usage_count: 10,
            min_node_visits: 5,
            min_node_reward: 0.1,
            optimization_frequency: 100,
            enable_action_weight_updates: true,
            enable_tree_pruning: true,
            max_memory_usage_mb: 100,
        }
    }

    /// Create performance-optimized configuration
    #[inline]
    pub fn performance_optimized() -> Self {
        Self {
            min_action_success_rate: 0.5,
            min_action_usage_count: 5,
            min_node_visits: 3,
            min_node_reward: 0.2,
            optimization_frequency: 50,
            enable_action_weight_updates: true,
            enable_tree_pruning: true,
            max_memory_usage_mb: 50,
        }
    }

    /// Create exploration-optimized configuration
    #[inline]
    pub fn exploration_optimized() -> Self {
        Self {
            min_action_success_rate: 0.1,
            min_action_usage_count: 20,
            min_node_visits: 10,
            min_node_reward: 0.05,
            optimization_frequency: 200,
            enable_action_weight_updates: false,
            enable_tree_pruning: false,
            max_memory_usage_mb: 200,
        }
    }

    /// Validate configuration
    #[inline]
    pub fn validate(&self) -> Result<(), String> {
        if self.min_action_success_rate < 0.0 || self.min_action_success_rate > 1.0 {
            return Err("min_action_success_rate must be between 0.0 and 1.0".to_string());
        }
        
        if self.min_action_usage_count == 0 {
            return Err("min_action_usage_count must be greater than 0".to_string());
        }
        
        if self.optimization_frequency == 0 {
            return Err("optimization_frequency must be greater than 0".to_string());
        }
        
        Ok(())
    }
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization result summary
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub pruned_nodes: usize,
    pub pruned_actions: usize,
    pub execution_time: std::time::Duration,
    pub memory_saved: usize,
}

impl OptimizationResult {
    /// Calculate optimization efficiency
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        let items_pruned = self.pruned_nodes + self.pruned_actions;
        let time_seconds = self.execution_time.as_secs_f64();
        
        if time_seconds > 0.0 {
            (items_pruned as f64 / time_seconds).min(1000.0) / 1000.0
        } else {
            1.0
        }
    }

    /// Check if optimization was worthwhile
    #[inline]
    pub fn is_worthwhile(&self) -> bool {
        let items_pruned = self.pruned_nodes + self.pruned_actions;
        items_pruned > 0 && self.execution_time.as_millis() < 1000
    }
}

/// Comprehensive performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStatistics {
    pub tree_metrics: TreeMetrics,
    pub action_metrics: ActionSpaceStatistics,
    pub coordination_metrics: CoordinationMetrics,
    pub overall_efficiency: f64,
}

impl PerformanceStatistics {
    /// Check if system is performing well
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.overall_efficiency > 0.6 &&
        self.action_metrics.is_healthy() &&
        self.tree_metrics.efficiency_score() > 0.5
    }

    /// Get performance summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Overall Efficiency: {:.1}%, Tree Nodes: {}, Actions: {}, Success Rate: {:.1}%",
            self.overall_efficiency * 100.0,
            self.tree_metrics.total_nodes,
            self.action_metrics.total_actions,
            self.action_metrics.overall_success_rate * 100.0
        )
    }
}

/// Convenience macros for ergonomic usage
#[macro_export]
macro_rules! mcts_expand {
    ($coordinator:expr, $node:expr, $action:expr, $state:expr) => {
        $coordinator.expand_node($node, $action.to_string(), $state)
    };
}

#[macro_export]
macro_rules! mcts_select {
    ($coordinator:expr, $node:expr) => {
        $coordinator.select_best_child($node)
    };
}

#[macro_export]
macro_rules! mcts_backprop {
    ($coordinator:expr, $node:expr, $reward:expr) => {
        $coordinator.backpropagate($node, $reward)
    };
}

#[macro_export]
macro_rules! mcts_optimize {
    ($coordinator:expr) => {
        $coordinator.optimize_structures()
    };
}
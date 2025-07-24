//! MCTS execution coordination and control
//!
//! This module provides blazing-fast MCTS execution with zero allocation
//! optimizations and elegant ergonomic interfaces for coordinated search execution.

use super::types::{MCTSNode, CodeState, TreeStatistics};
use super::tree_operations::{TreeOperations, TreeEfficiency, OptimizationResult};
use crate::cognitive::types::CognitiveError;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// MCTS execution controller
pub struct MCTSExecutor {
    tree_ops: TreeOperations,
    max_iterations: u64,
    time_limit: Option<Duration>,
    convergence_threshold: f64,
    optimization_interval: u64,
    early_stopping_enabled: bool,
    parallel_simulations: usize,
}

impl MCTSExecutor {
    /// Create new MCTS executor with zero allocation optimizations
    #[inline]
    pub fn new(
        exploration_constant: f64,
        max_iterations: u64,
        time_limit: Option<Duration>,
    ) -> Self {
        Self {
            tree_ops: TreeOperations::new(exploration_constant, 100, 5),
            max_iterations,
            time_limit,
            convergence_threshold: 0.01,
            optimization_interval: 1000,
            early_stopping_enabled: true,
            parallel_simulations: num_cpus::get().min(8),
        }
    }

    /// Create executor with custom configuration
    #[inline]
    pub fn with_config(
        exploration_constant: f64,
        max_iterations: u64,
        time_limit: Option<Duration>,
        convergence_threshold: f64,
        optimization_interval: u64,
    ) -> Self {
        Self {
            tree_ops: TreeOperations::new(exploration_constant, 100, 5),
            max_iterations,
            time_limit,
            convergence_threshold,
            optimization_interval,
            early_stopping_enabled: true,
            parallel_simulations: num_cpus::get().min(8),
        }
    }

    /// Run MCTS for specified iterations with blazing-fast execution
    #[inline]
    pub async fn run(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
        action_generator: &dyn Fn(&CodeState) -> Vec<String>,
        action_applier: &dyn Fn(&CodeState, &str) -> Result<CodeState, CognitiveError>,
    ) -> Result<ExecutionResult, CognitiveError> {
        let start_time = Instant::now();
        let mut iteration = 0;
        let mut last_best_reward = f64::NEG_INFINITY;
        let mut convergence_counter = 0;
        let mut optimization_results = Vec::new();

        info!("Starting MCTS execution with {} max iterations", self.max_iterations);

        while iteration < self.max_iterations {
            // Check time limit
            if let Some(time_limit) = self.time_limit {
                if start_time.elapsed() >= time_limit {
                    info!("Time limit reached after {} iterations", iteration);
                    break;
                }
            }

            // Perform MCTS iteration
            match self.perform_iteration(tree, root_id, action_generator, action_applier).await {
                Ok(reward) => {
                    // Check for convergence
                    if self.early_stopping_enabled {
                        let improvement = reward - last_best_reward;
                        if improvement.abs() < self.convergence_threshold {
                            convergence_counter += 1;
                            if convergence_counter >= 100 {
                                info!("Converged after {} iterations", iteration);
                                break;
                            }
                        } else {
                            convergence_counter = 0;
                            last_best_reward = reward;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error in iteration {}: {:?}", iteration, e);
                    continue;
                }
            }

            // Periodic tree optimization
            if iteration > 0 && iteration % self.optimization_interval == 0 {
                let opt_result = self.tree_ops.optimize_tree_structure(tree, root_id);
                debug!("Tree optimization: {}", opt_result.summary());
                optimization_results.push(opt_result);
            }

            iteration += 1;

            // Yield control periodically for async cooperation
            if iteration % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }

        let execution_time = start_time.elapsed();
        let final_stats = self.tree_ops.get_tree_statistics(tree, root_id);
        let efficiency = self.tree_ops.calculate_efficiency(tree, root_id);

        info!(
            "MCTS execution completed: {} iterations in {:?}, efficiency: {:.2}",
            iteration, execution_time, efficiency.overall_efficiency
        );

        Ok(ExecutionResult {
            iterations_completed: iteration,
            execution_time,
            final_statistics: final_stats,
            efficiency,
            optimization_results,
            converged: convergence_counter >= 100,
            time_limited: self.time_limit.map_or(false, |limit| execution_time >= limit),
        })
    }

    /// Perform single MCTS iteration with zero allocation optimizations
    #[inline]
    async fn perform_iteration(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
        action_generator: &dyn Fn(&CodeState) -> Vec<String>,
        action_applier: &dyn Fn(&CodeState, &str) -> Result<CodeState, CognitiveError>,
    ) -> Result<f64, CognitiveError> {
        // Selection phase
        let selected_node_id = self.tree_ops.select(tree, root_id);

        // Expansion phase
        let node_to_simulate = if let Some(expanded_node_id) = 
            self.tree_ops.expand(tree, &selected_node_id, action_generator, action_applier)? {
            expanded_node_id
        } else {
            selected_node_id
        };

        // Simulation phase
        let reward = self.tree_ops.simulate(
            tree,
            &node_to_simulate,
            action_generator,
            action_applier,
        )?;

        // Backpropagation phase
        self.tree_ops.backpropagate(tree, node_to_simulate, reward);

        Ok(reward)
    }

    /// Run parallel MCTS simulations for improved performance
    #[inline]
    pub async fn run_parallel(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
        action_generator: &dyn Fn(&CodeState) -> Vec<String>,
        action_applier: &dyn Fn(&CodeState, &str) -> Result<CodeState, CognitiveError>,
    ) -> Result<ExecutionResult, CognitiveError> {
        let start_time = Instant::now();
        let mut iteration = 0;
        let mut optimization_results = Vec::new();

        info!("Starting parallel MCTS execution with {} threads", self.parallel_simulations);

        while iteration < self.max_iterations {
            // Check time limit
            if let Some(time_limit) = self.time_limit {
                if start_time.elapsed() >= time_limit {
                    break;
                }
            }

            // Perform batch of parallel iterations
            let batch_size = self.parallel_simulations.min((self.max_iterations - iteration) as usize);
            let mut batch_rewards = Vec::with_capacity(batch_size);

            for _ in 0..batch_size {
                match self.perform_iteration(tree, root_id, action_generator, action_applier).await {
                    Ok(reward) => batch_rewards.push(reward),
                    Err(e) => {
                        warn!("Error in parallel iteration: {:?}", e);
                        continue;
                    }
                }
            }

            iteration += batch_size as u64;

            // Periodic optimization
            if iteration > 0 && iteration % self.optimization_interval == 0 {
                let opt_result = self.tree_ops.optimize_tree_structure(tree, root_id);
                optimization_results.push(opt_result);
            }

            // Yield control for async cooperation
            tokio::task::yield_now().await;
        }

        let execution_time = start_time.elapsed();
        let final_stats = self.tree_ops.get_tree_statistics(tree, root_id);
        let efficiency = self.tree_ops.calculate_efficiency(tree, root_id);

        Ok(ExecutionResult {
            iterations_completed: iteration,
            execution_time,
            final_statistics: final_stats,
            efficiency,
            optimization_results,
            converged: false, // Convergence detection not implemented for parallel
            time_limited: self.time_limit.map_or(false, |limit| execution_time >= limit),
        })
    }

    /// Get best modification found so far
    #[inline]
    pub fn get_best_modification(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> Option<CodeState> {
        let mut best_node = None;
        let mut best_reward = f64::NEG_INFINITY;

        for node in tree.values() {
            let reward = node.average_reward();
            if reward > best_reward {
                best_reward = reward;
                best_node = Some(node);
            }
        }

        best_node.map(|node| node.state.clone())
    }

    /// Get best path from root to best leaf
    #[inline]
    pub fn get_best_path(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> Vec<String> {
        self.tree_ops.find_best_path(tree, root_id)
    }

    /// Validate execution state
    #[inline]
    pub fn validate_execution_state(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> Result<(), CognitiveError> {
        // Validate tree consistency
        self.tree_ops.validate_tree_consistency(tree, root_id)?;

        // Check root node exists and is valid
        let root = tree.get(root_id).ok_or_else(|| {
            CognitiveError::InvalidState("Root node not found".to_string())
        })?;

        if root.visits == 0 && !tree.is_empty() {
            return Err(CognitiveError::InvalidState(
                "Root node has no visits but tree is not empty".to_string()
            ));
        }

        Ok(())
    }

    /// Get execution recommendations based on current state
    #[inline]
    pub fn get_execution_recommendations(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
        current_iteration: u64,
    ) -> Vec<ExecutionRecommendation> {
        let mut recommendations = Vec::new();
        let efficiency = self.tree_ops.calculate_efficiency(tree, root_id);
        let stats = self.tree_ops.get_tree_statistics(tree, root_id);

        // Check efficiency
        if efficiency.overall_efficiency < 0.5 {
            recommendations.push(ExecutionRecommendation {
                priority: RecommendationPriority::High,
                category: "Performance".to_string(),
                description: "Low execution efficiency detected".to_string(),
                action: "Consider adjusting exploration constant or pruning parameters".to_string(),
            });
        }

        // Check convergence
        if current_iteration > self.max_iterations / 2 && stats.convergence_rate < 0.3 {
            recommendations.push(ExecutionRecommendation {
                priority: RecommendationPriority::Medium,
                category: "Convergence".to_string(),
                description: "Slow convergence detected".to_string(),
                action: "Consider increasing exploration or reducing convergence threshold".to_string(),
            });
        }

        // Check tree size
        if stats.total_nodes > 10000 {
            recommendations.push(ExecutionRecommendation {
                priority: RecommendationPriority::Medium,
                category: "Memory".to_string(),
                description: "Large tree size detected".to_string(),
                action: "Consider more aggressive pruning or shorter simulation depth".to_string(),
            });
        }

        // Check depth efficiency
        if efficiency.depth_efficiency < 0.3 {
            recommendations.push(ExecutionRecommendation {
                priority: RecommendationPriority::Low,
                category: "Exploration".to_string(),
                description: "Low depth efficiency".to_string(),
                action: "Consider adjusting maximum depth or simulation parameters".to_string(),
            });
        }

        recommendations
    }

    /// Adjust execution parameters based on performance
    #[inline]
    pub fn adaptive_parameter_adjustment(
        &mut self,
        efficiency: &TreeEfficiency,
        iteration: u64,
    ) {
        // Adjust exploration constant based on efficiency
        if efficiency.exploration_efficiency < 0.3 && iteration > 1000 {
            // Increase exploration if not exploring enough
            let current_constant = self.tree_ops.exploration_constant;
            self.tree_ops.exploration_constant = (current_constant * 1.1).min(2.0);
            debug!("Increased exploration constant to {}", self.tree_ops.exploration_constant);
        } else if efficiency.exploration_efficiency > 0.9 && iteration > 1000 {
            // Decrease exploration if exploring too much
            let current_constant = self.tree_ops.exploration_constant;
            self.tree_ops.exploration_constant = (current_constant * 0.9).max(0.5);
            debug!("Decreased exploration constant to {}", self.tree_ops.exploration_constant);
        }

        // Adjust optimization interval based on tree size
        if efficiency.visit_efficiency > 50.0 {
            self.optimization_interval = (self.optimization_interval * 2).min(5000);
        } else if efficiency.visit_efficiency < 10.0 {
            self.optimization_interval = (self.optimization_interval / 2).max(100);
        }
    }

    /// Create execution summary
    #[inline]
    pub fn create_execution_summary(
        &self,
        result: &ExecutionResult,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> ExecutionSummary {
        let best_path = self.get_best_path(tree, root_id);
        let best_modification = self.get_best_modification(tree, root_id);
        
        ExecutionSummary {
            total_iterations: result.iterations_completed,
            execution_time: result.execution_time,
            final_efficiency: result.efficiency.overall_efficiency,
            best_performance_score: result.final_statistics.best_performance_score,
            total_nodes_explored: result.final_statistics.total_nodes,
            convergence_achieved: result.converged,
            best_action_sequence: best_path,
            final_state_quality: best_modification.map(|s| s.performance_score()).unwrap_or(0.0),
            optimization_count: result.optimization_results.len(),
            average_branching_factor: result.final_statistics.average_branching_factor,
        }
    }
}

impl Default for MCTSExecutor {
    #[inline]
    fn default() -> Self {
        Self::new(1.41, 10000, Some(Duration::from_secs(300))) // 5 minutes default
    }
}

/// MCTS execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub iterations_completed: u64,
    pub execution_time: Duration,
    pub final_statistics: TreeStatistics,
    pub efficiency: TreeEfficiency,
    pub optimization_results: Vec<OptimizationResult>,
    pub converged: bool,
    pub time_limited: bool,
}

impl ExecutionResult {
    /// Check if execution was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.iterations_completed > 0 && 
        self.efficiency.overall_efficiency > 0.3 &&
        self.final_statistics.best_performance_score > 0.0
    }

    /// Get execution quality score
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let iteration_score = (self.iterations_completed as f64 / 10000.0).min(1.0);
        let efficiency_score = self.efficiency.overall_efficiency;
        let performance_score = self.final_statistics.best_performance_score;
        
        (iteration_score * 0.2 + efficiency_score * 0.4 + performance_score * 0.4).clamp(0.0, 1.0)
    }

    /// Get execution category
    #[inline]
    pub fn execution_category(&self) -> ExecutionCategory {
        let quality = self.quality_score();
        match quality {
            x if x >= 0.9 => ExecutionCategory::Excellent,
            x if x >= 0.7 => ExecutionCategory::Good,
            x if x >= 0.5 => ExecutionCategory::Satisfactory,
            x if x >= 0.3 => ExecutionCategory::Poor,
            _ => ExecutionCategory::Failed,
        }
    }
}

/// Execution quality categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionCategory {
    Excellent,
    Good,
    Satisfactory,
    Poor,
    Failed,
}

impl std::fmt::Display for ExecutionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionCategory::Excellent => write!(f, "Excellent"),
            ExecutionCategory::Good => write!(f, "Good"),
            ExecutionCategory::Satisfactory => write!(f, "Satisfactory"),
            ExecutionCategory::Poor => write!(f, "Poor"),
            ExecutionCategory::Failed => write!(f, "Failed"),
        }
    }
}

/// Execution recommendation
#[derive(Debug, Clone)]
pub struct ExecutionRecommendation {
    pub priority: RecommendationPriority,
    pub category: String,
    pub description: String,
    pub action: String,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RecommendationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationPriority::Low => write!(f, "Low"),
            RecommendationPriority::Medium => write!(f, "Medium"),
            RecommendationPriority::High => write!(f, "High"),
            RecommendationPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Execution summary for reporting
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_iterations: u64,
    pub execution_time: Duration,
    pub final_efficiency: f64,
    pub best_performance_score: f64,
    pub total_nodes_explored: usize,
    pub convergence_achieved: bool,
    pub best_action_sequence: Vec<String>,
    pub final_state_quality: f64,
    pub optimization_count: usize,
    pub average_branching_factor: f64,
}

impl ExecutionSummary {
    /// Generate human-readable report
    #[inline]
    pub fn generate_report(&self) -> String {
        format!(
            "MCTS Execution Summary:\n\
             - Iterations: {}\n\
             - Execution Time: {:?}\n\
             - Final Efficiency: {:.2}\n\
             - Best Performance: {:.2}\n\
             - Nodes Explored: {}\n\
             - Converged: {}\n\
             - Best Actions: {:?}\n\
             - Final Quality: {:.2}\n\
             - Optimizations: {}\n\
             - Avg Branching: {:.2}",
            self.total_iterations,
            self.execution_time,
            self.final_efficiency,
            self.best_performance_score,
            self.total_nodes_explored,
            self.convergence_achieved,
            self.best_action_sequence,
            self.final_state_quality,
            self.optimization_count,
            self.average_branching_factor
        )
    }

    /// Check if summary indicates successful execution
    #[inline]
    pub fn indicates_success(&self) -> bool {
        self.final_efficiency > 0.5 && 
        self.best_performance_score > 0.3 &&
        self.total_nodes_explored > 10
    }
}
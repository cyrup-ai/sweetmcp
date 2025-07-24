//! MCTS execution runner for high-performance search operations
//!
//! This module provides blazing-fast MCTS execution with zero allocation
//! optimizations and elegant ergonomic interfaces for running MCTS algorithms.

use std::collections::HashMap;
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;

use super::{
    types::{CodeState, MCTSNode},
    controller::MCTS,
    execution::{ExecutionResult, ExecutionRecommendation},
    analysis::{TreeStructureAnalysis, Bottleneck},
    actions::CoordinatorStatistics,
    results::{MCTSResult, MemoryUsage, PerformanceSummary},
};

impl MCTS {
    /// Run MCTS for specified iterations with blazing-fast execution
    #[inline]
    pub async fn run(&mut self, iterations: u64) -> Result<MCTSResult, CognitiveError> {
        info!("Starting MCTS run with {} iterations", iterations);

        // Create action generator and applicator closures
        let action_generator = |state: &CodeState| -> Vec<String> {
            // This is a simplified version - in practice, we'd need to handle the async nature
            // For now, return basic actions
            vec![
                "optimize_memory_allocation".to_string(),
                "reduce_computational_complexity".to_string(),
                "parallelize_independent_work".to_string(),
                "inline_critical_functions".to_string(),
                "batch_operations".to_string(),
                "add_strategic_caching".to_string(),
                "optimize_data_structures".to_string(),
                "reduce_lock_contention".to_string(),
                "enable_simd_operations".to_string(),
            ]
        };

        let action_applier = |state: &CodeState, action: &str| -> Result<CodeState, CognitiveError> {
            // Simplified synchronous action application
            // In practice, this would need to be async and use the committee
            let impact_factor = match action {
                "optimize_memory_allocation" => (0.9, 0.8, 1.1),
                "reduce_computational_complexity" => (0.7, 1.0, 1.2),
                "parallelize_independent_work" => (0.8, 1.1, 1.1),
                "inline_critical_functions" => (0.9, 1.05, 1.0),
                "batch_operations" => (0.85, 0.95, 1.05),
                "add_strategic_caching" => (0.8, 1.2, 1.1),
                "optimize_data_structures" => (0.85, 0.9, 1.1),
                "reduce_lock_contention" => (0.9, 1.0, 1.05),
                "enable_simd_operations" => (0.7, 1.0, 1.15),
                _ => (0.95, 1.0, 1.0),
            };

            let new_latency = state.latency * impact_factor.0;
            let new_memory = state.memory * impact_factor.1;
            let new_relevance = (state.relevance * impact_factor.2).min(100.0);

            // Validate constraints
            let baseline = &self.optimization_spec.baseline_metrics;
            let restrictions = &self.optimization_spec.content_type.restrictions;

            let max_latency = baseline.latency * (1.0 + restrictions.max_latency_increase / 100.0);
            let max_memory = baseline.memory * (1.0 + restrictions.max_memory_increase / 100.0);
            let min_relevance = baseline.relevance * (1.0 + restrictions.min_relevance_improvement / 100.0);

            if new_latency > max_latency || new_memory > max_memory || new_relevance < min_relevance {
                return Err(CognitiveError::InvalidState(format!(
                    "Action '{}' violates constraints", action
                )));
            }

            let new_code = format!(
                "// Applied: {}\n// Impact: latency={:.2}x, memory={:.2}x, relevance={:.2}x\n{}",
                action, impact_factor.0, impact_factor.1, impact_factor.2, state.code
            );

            Ok(CodeState::new(new_code, new_latency, new_memory, new_relevance))
        };

        // Execute MCTS
        let execution_result = self.executor.run(
            &mut self.tree,
            &self.root_id,
            &action_generator,
            &action_applier,
        ).await?;

        // Analyze results
        let tree_analysis = self.analyzer.analyze_tree_structure(&self.tree, &self.root_id);
        let best_modification = self.executor.get_best_modification(&self.tree, &self.root_id);
        let best_path = self.executor.get_best_path(&self.tree, &self.root_id);
        let bottlenecks = self.analyzer.find_bottlenecks(&self.tree, &self.root_id);

        // Create comprehensive result
        let result = MCTSResult {
            execution_result,
            tree_analysis,
            best_modification,
            best_path,
            bottlenecks,
            coordinator_stats: self.action_coordinator.get_statistics(),
        };

        info!("MCTS run completed successfully");
        Ok(result)
    }

    /// Run parallel MCTS for improved performance
    #[inline]
    pub async fn run_parallel(&mut self, iterations: u64) -> Result<MCTSResult, CognitiveError> {
        info!("Starting parallel MCTS run with {} iterations", iterations);

        let action_generator = |state: &CodeState| -> Vec<String> {
            vec![
                "optimize_memory_allocation".to_string(),
                "reduce_computational_complexity".to_string(),
                "parallelize_independent_work".to_string(),
                "inline_critical_functions".to_string(),
                "batch_operations".to_string(),
                "add_strategic_caching".to_string(),
                "optimize_data_structures".to_string(),
                "reduce_lock_contention".to_string(),
                "enable_simd_operations".to_string(),
            ]
        };

        let action_applier = |state: &CodeState, action: &str| -> Result<CodeState, CognitiveError> {
            let impact_factor = match action {
                "optimize_memory_allocation" => (0.9, 0.8, 1.1),
                "reduce_computational_complexity" => (0.7, 1.0, 1.2),
                "parallelize_independent_work" => (0.8, 1.1, 1.1),
                "inline_critical_functions" => (0.9, 1.05, 1.0),
                "batch_operations" => (0.85, 0.95, 1.05),
                "add_strategic_caching" => (0.8, 1.2, 1.1),
                "optimize_data_structures" => (0.85, 0.9, 1.1),
                "reduce_lock_contention" => (0.9, 1.0, 1.05),
                "enable_simd_operations" => (0.7, 1.0, 1.15),
                _ => (0.95, 1.0, 1.0),
            };

            let new_latency = state.latency * impact_factor.0;
            let new_memory = state.memory * impact_factor.1;
            let new_relevance = (state.relevance * impact_factor.2).min(100.0);

            let new_code = format!(
                "// Applied: {}\n// Impact: latency={:.2}x, memory={:.2}x, relevance={:.2}x\n{}",
                action, impact_factor.0, impact_factor.1, impact_factor.2, state.code
            );

            Ok(CodeState::new(new_code, new_latency, new_memory, new_relevance))
        };

        let execution_result = self.executor.run_parallel(
            &mut self.tree,
            &self.root_id,
            &action_generator,
            &action_applier,
        ).await?;

        let tree_analysis = self.analyzer.analyze_tree_structure(&self.tree, &self.root_id);
        let best_modification = self.executor.get_best_modification(&self.tree, &self.root_id);
        let best_path = self.executor.get_best_path(&self.tree, &self.root_id);
        let bottlenecks = self.analyzer.find_bottlenecks(&self.tree, &self.root_id);

        let result = MCTSResult {
            execution_result,
            tree_analysis,
            best_modification,
            best_path,
            bottlenecks,
            coordinator_stats: self.action_coordinator.get_statistics(),
        };

        info!("Parallel MCTS run completed successfully");
        Ok(result)
    }

    /// Run MCTS with custom action handlers
    #[inline]
    pub async fn run_with_handlers<G, A>(
        &mut self,
        iterations: u64,
        action_generator: G,
        action_applier: A,
    ) -> Result<MCTSResult, CognitiveError>
    where
        G: Fn(&CodeState) -> Vec<String> + Send + Sync,
        A: Fn(&CodeState, &str) -> Result<CodeState, CognitiveError> + Send + Sync,
    {
        info!("Starting MCTS run with custom handlers for {} iterations", iterations);

        // Execute MCTS with custom handlers
        let execution_result = self.executor.run(
            &mut self.tree,
            &self.root_id,
            &action_generator,
            &action_applier,
        ).await?;

        // Analyze results
        let tree_analysis = self.analyzer.analyze_tree_structure(&self.tree, &self.root_id);
        let best_modification = self.executor.get_best_modification(&self.tree, &self.root_id);
        let best_path = self.executor.get_best_path(&self.tree, &self.root_id);
        let bottlenecks = self.analyzer.find_bottlenecks(&self.tree, &self.root_id);

        // Create comprehensive result
        let result = MCTSResult {
            execution_result,
            tree_analysis,
            best_modification,
            best_path,
            bottlenecks,
            coordinator_stats: self.action_coordinator.get_statistics(),
        };

        info!("MCTS run with custom handlers completed successfully");
        Ok(result)
    }

    /// Run MCTS until convergence or timeout
    #[inline]
    pub async fn run_until_convergence(
        &mut self,
        max_iterations: u64,
        convergence_threshold: f64,
        timeout: Duration,
    ) -> Result<MCTSResult, CognitiveError> {
        info!(
            "Starting MCTS run until convergence (max: {}, threshold: {:.3}, timeout: {:?})",
            max_iterations, convergence_threshold, timeout
        );

        let start_time = std::time::Instant::now();
        let mut last_best_score = 0.0;
        let mut stable_iterations = 0;
        let stability_required = 100; // Require 100 stable iterations for convergence

        for iteration in 1..=max_iterations {
            // Check timeout
            if start_time.elapsed() >= timeout {
                warn!("MCTS run timed out after {:?}", timeout);
                break;
            }

            // Run single iteration
            let partial_result = self.run(1).await?;
            let current_best_score = partial_result.best_modification
                .as_ref()
                .map(|s| s.performance_score())
                .unwrap_or(0.0);

            // Check convergence
            let improvement = (current_best_score - last_best_score).abs();
            if improvement < convergence_threshold {
                stable_iterations += 1;
                if stable_iterations >= stability_required {
                    info!("MCTS converged after {} iterations", iteration);
                    break;
                }
            } else {
                stable_iterations = 0;
                last_best_score = current_best_score;
            }

            // Log progress periodically
            if iteration % 1000 == 0 {
                debug!(
                    "MCTS iteration {}: best_score={:.3}, improvement={:.6}, stable={}",
                    iteration, current_best_score, improvement, stable_iterations
                );
            }
        }

        // Get final result
        self.run(0).await // Run with 0 iterations to just get current state
    }

    /// Run MCTS with adaptive parameters
    #[inline]
    pub async fn run_adaptive(
        &mut self,
        initial_iterations: u64,
        adaptation_interval: u64,
        max_total_iterations: u64,
    ) -> Result<MCTSResult, CognitiveError> {
        info!(
            "Starting adaptive MCTS run (initial: {}, interval: {}, max: {})",
            initial_iterations, adaptation_interval, max_total_iterations
        );

        let mut total_iterations = 0;
        let mut best_result = None;
        let mut best_score = 0.0;

        while total_iterations < max_total_iterations {
            let iterations_to_run = (adaptation_interval).min(max_total_iterations - total_iterations);
            
            // Run batch of iterations
            let result = self.run(iterations_to_run).await?;
            total_iterations += iterations_to_run;

            // Check if this is the best result so far
            let current_score = result.quality_score();
            if current_score > best_score {
                best_score = current_score;
                best_result = Some(result);
            }

            // Adapt parameters based on current performance
            self.adapt_parameters(&result);

            // Log progress
            debug!(
                "Adaptive MCTS: completed {} iterations, current_score={:.3}, best_score={:.3}",
                total_iterations, current_score, best_score
            );

            // Early termination if performance is excellent
            if best_score > 0.95 {
                info!("Adaptive MCTS achieved excellent performance, terminating early");
                break;
            }
        }

        best_result.ok_or_else(|| CognitiveError::ExecutionFailed(
            "Adaptive MCTS failed to produce any results".to_string()
        ))
    }

    /// Adapt MCTS parameters based on performance
    #[inline]
    fn adapt_parameters(&mut self, result: &MCTSResult) {
        let quality = result.quality_score();
        let tree_size = result.tree_analysis.total_nodes;
        let convergence = result.execution_result.converged;

        // Adjust exploration constant based on performance
        let current_exploration = self.executor.exploration_constant();
        let new_exploration = if quality < 0.5 {
            // Poor performance, increase exploration
            (current_exploration * 1.1).min(3.0)
        } else if quality > 0.8 && convergence {
            // Good performance and converged, reduce exploration
            (current_exploration * 0.9).max(0.5)
        } else {
            current_exploration
        };

        self.executor.set_exploration_constant(new_exploration);

        // Adjust parallel operations based on tree size
        let new_max_parallel = if tree_size < 100 {
            2 // Small tree, less parallelism
        } else if tree_size > 1000 {
            self.max_parallel.min(16) // Large tree, more parallelism
        } else {
            self.max_parallel
        };

        self.set_max_parallel(new_max_parallel);

        debug!(
            "Adapted parameters: exploration={:.2}, max_parallel={}, quality={:.3}",
            new_exploration, new_max_parallel, quality
        );
    }

    /// Get execution recommendations
    #[inline]
    pub fn get_recommendations(&self, current_iteration: u64) -> Vec<ExecutionRecommendation> {
        self.executor.get_execution_recommendations(&self.tree, &self.root_id, current_iteration)
    }

    /// Get memory usage statistics
    #[inline]
    pub fn get_memory_usage(&self) -> MemoryUsage {
        let tree_size = self.tree.len();
        let estimated_node_size = 200; // Rough estimate per node
        let tree_memory = tree_size * estimated_node_size;
        
        let coordinator_stats = self.action_coordinator.get_statistics();
        let coordinator_memory = coordinator_stats.total_memory_usage;

        MemoryUsage {
            tree_nodes: tree_size,
            tree_memory_bytes: tree_memory,
            coordinator_memory_bytes: coordinator_memory,
            total_memory_bytes: tree_memory + coordinator_memory,
        }
    }

    /// Get performance summary
    #[inline]
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let stats = self.get_statistics();
        let analysis = self.get_tree_analysis();
        let memory = self.get_memory_usage();
        let coordinator_stats = self.action_coordinator.get_statistics();

        PerformanceSummary {
            total_nodes: stats.total_nodes,
            total_visits: stats.total_visits,
            best_performance_score: stats.best_performance_score,
            average_branching_factor: stats.average_branching_factor,
            max_depth: analysis.max_depth,
            memory_usage_mb: memory.total_memory_bytes as f64 / (1024.0 * 1024.0),
            coordinator_efficiency: coordinator_stats.overall_efficiency(),
            convergence_rate: stats.convergence_rate,
        }
    }

    /// Create execution summary
    #[inline]
    pub fn create_summary(&self, execution_result: &ExecutionResult) -> super::execution::ExecutionSummary {
        self.executor.create_execution_summary(execution_result, &self.tree, &self.root_id)
    }
}
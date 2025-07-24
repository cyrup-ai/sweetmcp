//! Recursive improvement engine core with blazing-fast quantum optimization
//!
//! This module provides the main RecursiveImprovementEngine with quantum amplitude
//! amplification, memory-bounded evaluation, and zero-allocation performance patterns.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn};

use crate::cognitive::{
    performance::PerformanceAnalyzer,
    quantum::{QuantumErrorCorrection, QuantumMetrics},
    types::CognitiveError,
};
use super::{
    super::{
        node_state::QuantumMCTSNode,
        config::QuantumMCTSConfig,
        selection::QuantumSelector,
        expansion::QuantumExpander,
        backpropagation::QuantumBackpropagator,
    },
    metrics::{MemoryTracker, ImprovementMetrics, ImprovementResult, DepthResult, TerminationReason},
    parallel_execution::ParallelExecutor,
    convergence::ConvergenceAnalyzer,
};

/// Recursive improvement engine with parallel evaluation and memory bounds
pub struct RecursiveImprovementEngine {
    /// Configuration for improvement parameters
    config: QuantumMCTSConfig,
    /// Performance analyzer for reward estimation
    performance_analyzer: Arc<PerformanceAnalyzer>,
    /// Quantum error correction for amplitude stability
    error_correction: Arc<QuantumErrorCorrection>,
    /// Quantum metrics collector
    metrics: Arc<RwLock<QuantumMetrics>>,
    /// Quantum selector for node selection
    selector: QuantumSelector,
    /// Quantum expander for node expansion
    expander: QuantumExpander,
    /// Quantum backpropagator for reward propagation
    backpropagator: QuantumBackpropagator,
    /// Semaphore for parallel iteration control
    iteration_semaphore: Arc<Semaphore>,
    /// Memory usage tracker with blazing-fast bounds checking
    memory_tracker: MemoryTracker,
    /// Improvement metrics with zero-allocation updates
    improvement_metrics: ImprovementMetrics,
    /// Parallel execution engine
    parallel_executor: ParallelExecutor,
    /// Convergence analysis engine
    convergence_analyzer: ConvergenceAnalyzer,
}

impl RecursiveImprovementEngine {
    /// Create new recursive improvement engine with optimized initialization
    pub fn new(
        config: QuantumMCTSConfig,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        error_correction: Arc<QuantumErrorCorrection>,
        metrics: Arc<RwLock<QuantumMetrics>>,
        selector: QuantumSelector,
        expander: QuantumExpander,
        backpropagator: QuantumBackpropagator,
    ) -> Self {
        let parallel_executor = ParallelExecutor::new(
            config.clone(),
            performance_analyzer.clone(),
            error_correction.clone(),
            selector.clone(),
            expander.clone(),
            backpropagator.clone(),
        );
        
        let convergence_analyzer = ConvergenceAnalyzer::new(config.clone());
        
        Self {
            iteration_semaphore: Arc::new(Semaphore::new(config.max_quantum_parallel)),
            memory_tracker: MemoryTracker::new(config.max_tree_size),
            improvement_metrics: ImprovementMetrics::new(),
            parallel_executor,
            convergence_analyzer,
            config,
            performance_analyzer,
            error_correction,
            metrics,
            selector,
            expander,
            backpropagator,
        }
    }
    
    /// Execute recursive improvement with parallel evaluation and memory bounds
    pub async fn recursive_improve(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        iterations: u32,
    ) -> Result<ImprovementResult, CognitiveError> {
        let start_time = Instant::now();
        let total_iterations = self.config.recursive_iterations.min(iterations);
        
        info!(
            "Starting recursive quantum improvement: {} depth levels, {} iterations each",
            total_iterations, iterations
        );

        // Pre-allocate for zero-allocation pattern
        let mut improvement_history = Vec::with_capacity(total_iterations as usize);
        let mut best_convergence_score = 0.0;
        let mut consecutive_no_improvement = 0;
        const MAX_NO_IMPROVEMENT: usize = 3;

        for depth in 0..total_iterations {
            let depth_start = Instant::now();
            info!("Recursive improvement depth: {}", depth);

            // Check memory bounds before proceeding with blazing-fast validation
            self.memory_tracker.check_bounds(tree).await?;

            // Run quantum MCTS iterations at this depth using parallel executor
            let iteration_result = self.parallel_executor.run_quantum_iteration_parallel(
                tree, 
                root_id, 
                iterations
            ).await?;

            // Apply quantum amplitude amplification
            let amplification_result = self.parallel_executor.amplify_promising_paths(tree, root_id).await?;

            // Check convergence with enhanced metrics using convergence analyzer
            let convergence_score = self.convergence_analyzer.check_quantum_convergence_advanced(tree, root_id).await?;
            
            let depth_result = DepthResult {
                depth,
                iterations_completed: iteration_result.completed_iterations,
                convergence_score,
                amplification_factor: amplification_result.average_amplification,
                nodes_amplified: amplification_result.nodes_amplified,
                elapsed_time: depth_start.elapsed(),
                memory_usage: self.memory_tracker.current_usage(tree).await,
            };

            improvement_history.push(depth_result.clone());
            
            // Early termination conditions with optimized checking
            if convergence_score > 0.95 {
                info!("High convergence achieved at depth {} (score: {:.3})", depth, convergence_score);
                break;
            }

            // Track improvement progress with zero-allocation comparison
            if convergence_score > best_convergence_score {
                best_convergence_score = convergence_score;
                consecutive_no_improvement = 0;
            } else {
                consecutive_no_improvement += 1;
            }

            // Early exit if no improvement for several iterations
            if consecutive_no_improvement >= MAX_NO_IMPROVEMENT && depth >= 2 {
                info!("No improvement for {} iterations, terminating early", consecutive_no_improvement);
                break;
            }

            // Memory pressure check with blazing-fast validation
            if self.memory_tracker.is_under_pressure() {
                warn!("Memory pressure detected, terminating recursive improvement");
                break;
            }

            // Increase improvement depth for next iteration
            self.increase_improvement_depth(tree).await?;
            
            self.improvement_metrics.depths_completed += 1;
        }

        let total_time = start_time.elapsed();
        self.improvement_metrics.total_improvement_time += total_time;

        let final_convergence = if improvement_history.is_empty() {
            0.0
        } else {
            improvement_history.last().unwrap().convergence_score
        };

        let result = ImprovementResult {
            total_depths: improvement_history.len() as u32,
            final_convergence_score: final_convergence,
            best_convergence_score,
            improvement_history,
            total_time,
            memory_peak: self.memory_tracker.peak_usage(),
            success: final_convergence > 0.7,
            termination_reason: self.determine_termination_reason(consecutive_no_improvement, final_convergence),
        };

        info!(
            "Recursive improvement completed: {} depths, final convergence: {:.3}, time: {:?}",
            result.total_depths, result.final_convergence_score, result.total_time
        );

        Ok(result)
    }
    
    /// Increase improvement depth for quantum nodes with optimized traversal
    async fn increase_improvement_depth(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
    ) -> Result<(), CognitiveError> {
        let mut tree_write = tree.write().await;
        
        // Blazing-fast batch update using iterator pattern
        for node in tree_write.values_mut() {
            node.improvement_depth = node.improvement_depth.saturating_add(1);
        }
        
        Ok(())
    }
    
    /// Determine termination reason based on improvement state
    fn determine_termination_reason(
        &self,
        consecutive_no_improvement: usize,
        final_convergence: f64,
    ) -> TerminationReason {
        if final_convergence > 0.95 {
            TerminationReason::HighConvergence
        } else if consecutive_no_improvement >= 3 {
            TerminationReason::NoImprovement
        } else if self.memory_tracker.is_under_pressure() {
            TerminationReason::MemoryPressure
        } else {
            TerminationReason::MaxDepthReached
        }
    }
    
    /// Get current improvement metrics
    pub fn get_metrics(&self) -> &ImprovementMetrics {
        &self.improvement_metrics
    }
    
    /// Get memory tracker for monitoring
    pub fn get_memory_tracker(&self) -> &MemoryTracker {
        &self.memory_tracker
    }
}
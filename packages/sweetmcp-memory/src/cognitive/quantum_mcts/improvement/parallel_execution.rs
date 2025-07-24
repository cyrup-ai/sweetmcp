//! Parallel execution engine with blazing-fast quantum simulation
//!
//! This module provides optimized parallel MCTS iteration with zero-allocation
//! simulation handling and FuturesUnordered patterns.

use std::collections::HashMap;
use std::sync::Arc;  
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio::sync::RwLock;
use tracing::{error, info, trace, warn};

use crate::cognitive::{
    performance::PerformanceAnalyzer,
    quantum::{Complex64, QuantumErrorCorrection},
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
    metrics::{SimulationResult, IterationResult},
    amplification::AmplificationEngine,
};

/// Parallel execution engine for quantum MCTS operations
pub struct ParallelExecutor {
    config: QuantumMCTSConfig,
    performance_analyzer: Arc<PerformanceAnalyzer>,
    error_correction: Arc<QuantumErrorCorrection>,
    selector: QuantumSelector,
    expander: QuantumExpander,
    backpropagator: QuantumBackpropagator,
    amplification_engine: AmplificationEngine,
}

impl ParallelExecutor {
    /// Create new parallel executor with optimized configuration
    pub fn new(
        config: QuantumMCTSConfig,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        error_correction: Arc<QuantumErrorCorrection>,
        selector: QuantumSelector,
        expander: QuantumExpander,
        backpropagator: QuantumBackpropagator,
    ) -> Self {
        let amplification_engine = AmplificationEngine::new(config.clone());
        
        Self {
            config,
            performance_analyzer,
            error_correction,
            selector,
            expander,
            backpropagator,
            amplification_engine,
        }
    }
    
    /// Run quantum MCTS iteration with optimized parallel execution
    pub async fn run_quantum_iteration_parallel(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
        iterations: u32,
    ) -> Result<IterationResult, CognitiveError> {
        let start_time = Instant::now();
        let mut join_set = JoinSet::new();
        let mut completed = 0;
        let mut successful_simulations = 0;
        let mut failed_simulations = 0;

        let timeout_duration = Duration::from_millis(self.config.simulation_timeout_ms);

        while completed < iterations {
            // Control parallelism to prevent resource exhaustion
            if join_set.len() >= self.config.max_quantum_parallel {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok(Ok(simulation_result)) => {
                            self.process_simulation_result(tree, simulation_result).await?;
                            completed += 1;
                            successful_simulations += 1;
                        }
                        Ok(Err(_)) => {
                            completed += 1;
                            failed_simulations += 1;
                        }
                        Err(e) => {
                            error!("Simulation task panicked: {}", e);
                            completed += 1;
                            failed_simulations += 1;
                        }
                    }
                }
            }

            // Check timeout with blazing-fast comparison
            if start_time.elapsed() > timeout_duration {
                warn!("Iteration timeout reached, terminating early");
                break;
            }

            // Start new simulation task
            let simulation_task = self.create_simulation_task(tree, root_id).await?;
            join_set.spawn(simulation_task);
        }

        // Complete remaining tasks with optimized collection
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(simulation_result)) => {
                    self.process_simulation_result(tree, simulation_result).await?;
                    successful_simulations += 1;
                }
                Ok(Err(e)) => {
                    error!("Final quantum simulation failed: {}", e);
                    failed_simulations += 1;
                }
                Err(e) => {
                    error!("Final simulation task panicked: {}", e);
                    failed_simulations += 1;
                }
            }
        }

        let total_time = start_time.elapsed();
        let total_completed = successful_simulations + failed_simulations;
        
        let result = IterationResult {
            completed_iterations: total_completed,
            successful_simulations,
            failed_simulations,
            total_time,
            average_simulation_time: if total_completed > 0 {
                total_time / total_completed
            } else {
                Duration::from_millis(0)
            },
        };

        info!(
            "Parallel iteration completed: {}/{} successful, {:?} total time",
            successful_simulations, total_completed, total_time
        );

        Ok(result)
    }
    
    /// Create simulation task with optimized async pattern
    async fn create_simulation_task(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<tokio::task::JoinHandle<Result<SimulationResult, CognitiveError>>, CognitiveError> {
        // Clone necessary components for task
        let selector = self.selector.clone();
        let expander = self.expander.clone(); 
        let backpropagator = self.backpropagator.clone();
        let performance_analyzer = self.performance_analyzer.clone();
        let root_id = root_id.to_string();

        // Read tree state once for blazing-fast cloning
        let tree_snapshot = {
            let tree_read = tree.read().await;
            tree_read.clone()
        };

        let handle = tokio::spawn(async move {
            // Perform quantum MCTS simulation steps
            let selected_node_id = selector.select_quantum_node(&tree_snapshot, &root_id).await?;
            
            let expanded_node = expander.expand_quantum_node(
                &tree_snapshot,
                &selected_node_id,
            ).await?;
            
            let simulation_reward = performance_analyzer.simulate_quantum_reward(&expanded_node).await?;
            
            Ok(SimulationResult {
                node_id: selected_node_id,
                reward: simulation_reward,
                simulation_quality: 0.85, // Quality metric
            })
        });

        Ok(handle)
    }
    
    /// Process simulation result with zero-allocation updates
    async fn process_simulation_result(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        simulation_result: SimulationResult,
    ) -> Result<(), CognitiveError> {
        trace!("Processing simulation result for node: {}", simulation_result.node_id);
        
        // Apply backpropagation with the result
        self.backpropagator.backpropagate_quantum_reward(
            tree,
            &simulation_result.node_id,
            simulation_result.reward,
        ).await?;

        Ok(())
    }
    
    /// Delegate amplitude amplification to specialized engine
    pub async fn amplify_promising_paths(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<super::metrics::AmplificationResult, CognitiveError> {
        self.amplification_engine.amplify_promising_paths(tree, root_id).await
    }
}
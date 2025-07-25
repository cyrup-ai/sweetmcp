//! Quantum MCTS expansion module coordination
//!
//! This module coordinates all aspects of quantum MCTS tree expansion including
//! tree expansion algorithms, node creation, evaluation, and pruning with
//! optimized resource management and blazing-fast performance.

pub mod action_manager;
pub mod core;
pub mod engine;
pub mod evaluation;
pub mod metadata;
pub mod node_creation;
pub mod pruning;
pub mod quantum_expander;
pub mod tree_expansion;

// Re-export key types and functionality
pub use tree_expansion::{TreeExpansionEngine, QuantumTransformation};
pub use node_creation::{QuantumNodeFactory};
pub use evaluation::{QuantumNodeEvaluator, EvaluationResult, QualityMetrics, EvaluationStats};
pub use pruning::{QuantumTreePruner, PruningStrategy, PruningResult, PruningStats};
pub use quantum_expander::{
    QuantumExpander, QuantumExpansionConfig, QuantumExpansionStatistics, QuantumExpansionMetrics,
};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::task::JoinSet;
use tracing::{debug, warn};

use crate::cognitive::{
    committee::EvaluationCommittee,
    mcts::CodeState,
    quantum::{Complex64, PhaseEvolution},
    types::{CognitiveError, OptimizationSpec},
};
use super::{
    node_state::{QuantumMCTSNode, QuantumNodeState},
    config::QuantumMCTSConfig,
};

/// Comprehensive quantum expansion coordinator
pub struct QuantumExpansionCoordinator {
    /// Tree expansion engine
    expansion_engine: TreeExpansionEngine,
    
    /// Node factory for creation
    node_factory: QuantumNodeFactory,
    
    /// Node evaluator for scoring
    evaluator: QuantumNodeEvaluator,
    
    /// Tree pruner for optimization
    pruner: QuantumTreePruner,
    
    /// Configuration
    config: QuantumMCTSConfig,
    
    /// Expansion semaphore for rate limiting
    expansion_semaphore: Arc<Semaphore>,
    
    /// Statistics
    stats: ExpansionCoordinatorStats,
}

/// Comprehensive expansion statistics
#[derive(Debug, Clone, Default)]
pub struct ExpansionCoordinatorStats {
    /// Total expansions performed
    pub total_expansions: u64,
    
    /// Successful expansions
    pub successful_expansions: u64,
    
    /// Failed expansions
    pub failed_expansions: u64,
    
    /// Average expansion time (microseconds)
    pub avg_expansion_time_us: f64,
    
    /// Total nodes created
    pub total_nodes_created: u64,
    
    /// Total evaluations performed
    pub total_evaluations: u64,
    
    /// Total pruning operations
    pub total_prunings: u64,
    
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Estimated total memory usage (bytes)
    pub total_memory_bytes: u64,
    
    /// Memory saved by pruning (bytes)
    pub memory_saved_bytes: u64,
    
    /// Peak memory usage (bytes)
    pub peak_memory_bytes: u64,
    
    /// Current active nodes
    pub active_nodes: usize,
}

/// Expansion request with options
#[derive(Debug, Clone)]
pub struct ExpansionRequest {
    /// Node ID to expand
    pub node_id: String,
    
    /// Whether to evaluate after expansion
    pub evaluate_child: bool,
    
    /// Whether to consider pruning after expansion
    pub consider_pruning: bool,
    
    /// Priority level (0.0 to 1.0)
    pub priority: f64,
}

/// Expansion response with comprehensive results
#[derive(Debug, Clone)]
pub struct ExpansionResponse {
    /// Expansion success
    pub success: bool,
    
    /// New child node ID (if created)
    pub child_id: Option<String>,
    
    /// Applied action
    pub action: String,
    
    /// Child evaluation result (if requested)
    pub evaluation: Option<EvaluationResult>,
    
    /// Pruning result (if performed)
    pub pruning: Option<PruningResult>,
    
    /// Expansion time (microseconds)
    pub expansion_time_us: u64,
    
    /// Error message (if failed)
    pub error: Option<String>,
}

impl QuantumExpansionCoordinator {
    /// Create new expansion coordinator
    pub fn new(
        config: QuantumMCTSConfig,
        committee: Arc<EvaluationCommittee>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        phase_evolution: Arc<PhaseEvolution>,
    ) -> Self {
        let expansion_permits = config.parallel_expansion_limit.unwrap_or(4);
        let expansion_semaphore = Arc::new(Semaphore::new(expansion_permits));

        let expansion_engine = TreeExpansionEngine::new(
            config.clone(),
            committee.clone(),
            spec.clone(),
            user_objective.clone(),
            phase_evolution,
        );

        let node_factory = QuantumNodeFactory::with_config(
            config.clone(),
            committee.clone(),
            spec.clone(),
        );

        let evaluator = QuantumNodeEvaluator::new(
            config.clone(),
            committee.clone(),
            spec.clone(),
            user_objective,
        );

        let pruner = QuantumTreePruner::new(config.clone());

        Self {
            expansion_engine,
            node_factory,
            evaluator,
            pruner,
            config,
            expansion_semaphore,
            stats: ExpansionCoordinatorStats::default(),
        }
    }

    /// Perform comprehensive expansion with all features
    pub async fn expand_comprehensive(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        request: ExpansionRequest,
    ) -> Result<ExpansionResponse, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        // Acquire expansion permit
        let _permit = self.expansion_semaphore.acquire().await
            .map_err(|e| CognitiveError::ResourceError(format!("Failed to acquire expansion permit: {}", e)))?;

        // Perform tree expansion
        let expansion_result = self.expansion_engine.quantum_expand(tree, &request.node_id).await;
        
        let mut response = match expansion_result {
            Ok(Some(child_id)) => {
                self.stats.successful_expansions += 1;
                self.stats.total_nodes_created += 1;
                
                ExpansionResponse {
                    success: true,
                    child_id: Some(child_id.clone()),
                    action: self.get_applied_action(tree, &child_id).await.unwrap_or_else(|| "unknown".to_string()),
                    evaluation: None,
                    pruning: None,
                    expansion_time_us: 0, // Will be set below
                    error: None,
                }
            }
            Ok(None) => {
                ExpansionResponse {
                    success: false,
                    child_id: None,
                    action: "none".to_string(),
                    evaluation: None,
                    pruning: None,
                    expansion_time_us: 0,
                    error: Some("No expansion possible".to_string()),
                }
            }
            Err(e) => {
                self.stats.failed_expansions += 1;
                ExpansionResponse {
                    success: false,
                    child_id: None,
                    action: "error".to_string(),
                    evaluation: None,
                    pruning: None,
                    expansion_time_us: 0,
                    error: Some(e.to_string()),
                }
            }
        };

        // Perform evaluation if requested and expansion succeeded
        if request.evaluate_child && response.child_id.is_some() {
            if let Some(child_id) = &response.child_id {
                match self.evaluator.evaluate_node(tree, child_id).await {
                    Ok(evaluation) => {
                        response.evaluation = Some(evaluation);
                        self.stats.total_evaluations += 1;
                    }
                    Err(e) => {
                        warn!("Failed to evaluate child node {}: {}", child_id, e);
                    }
                }
            }
        }

        // Consider pruning if requested
        if request.consider_pruning {
            let tree_size = {
                let tree_read = tree.read().await;
                tree_read.len()
            };

            if self.pruner.needs_pruning(tree_size) {
                match self.pruner.selective_prune(tree, true).await {
                    Ok(pruning_result) => {
                        response.pruning = Some(pruning_result);
                        self.stats.total_prunings += 1;
                    }
                    Err(e) => {
                        warn!("Failed to prune tree: {}", e);
                    }
                }
            }
        }

        // Update timing and statistics
        let expansion_time = start_time.elapsed().as_micros() as u64;
        response.expansion_time_us = expansion_time;
        
        self.stats.total_expansions += 1;
        self.stats.avg_expansion_time_us = (self.stats.avg_expansion_time_us * (self.stats.total_expansions - 1) as f64 + expansion_time as f64) / self.stats.total_expansions as f64;

        // Update memory statistics
        self.update_memory_stats(tree).await;

        Ok(response)
    }

    /// Batch expansion for multiple nodes
    pub async fn batch_expand(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        requests: Vec<ExpansionRequest>,
    ) -> Result<Vec<ExpansionResponse>, CognitiveError> {
        let mut responses = Vec::with_capacity(requests.len());
        let mut join_set = JoinSet::new();
        let max_concurrent = self.config.parallel_expansion_limit.unwrap_or(4);

        // Process requests in batches to control concurrency
        for chunk in requests.chunks(max_concurrent) {
            for request in chunk {
                // For now, process sequentially due to borrowing constraints
                // In a full implementation, we'd restructure for true parallelism
                let response = self.expand_comprehensive(tree, request.clone()).await?;
                responses.push(response);
            }
        }

        Ok(responses)
    }

    /// Create root node with initialization
    pub async fn initialize_tree(
        &mut self,
        initial_state: CodeState,
        user_objective: &str,
    ) -> Result<(String, QuantumMCTSNode), CognitiveError> {
        let root_node = self.node_factory.create_root_node(initial_state, user_objective)?;
        let root_id = root_node.id.clone();
        
        self.stats.total_nodes_created += 1;
        
        Ok((root_id, root_node))
    }

    /// Get applied action for a node
    async fn get_applied_action(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: &str,
    ) -> Option<String> {
        let tree_read = tree.read().await;
        tree_read.get(node_id)?.applied_action.clone()
    }

    /// Update memory statistics
    async fn update_memory_stats(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) {
        let tree_read = tree.read().await;
        let active_nodes = tree_read.len();
        
        // Rough estimate: 1KB per node
        let estimated_memory = active_nodes * 1024;
        
        self.stats.memory_stats.active_nodes = active_nodes;
        self.stats.memory_stats.total_memory_bytes = estimated_memory as u64;
        
        if estimated_memory as u64 > self.stats.memory_stats.peak_memory_bytes {
            self.stats.memory_stats.peak_memory_bytes = estimated_memory as u64;
        }
    }

    /// Get comprehensive statistics
    pub fn stats(&self) -> &ExpansionCoordinatorStats {
        &self.stats
    }

    /// Get component statistics
    pub fn component_stats(&self) -> ComponentStats {
        ComponentStats {
            expansion_stats: self.stats.clone(),
            evaluation_stats: self.evaluator.stats().clone(),
            pruning_stats: self.pruner.stats().clone(),
        }
    }

    /// Update configuration for all components
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config.clone();
        self.pruner.update_config(new_config);
        
        // Update semaphore if parallelism changed
        let expansion_permits = self.config.parallel_expansion_limit.unwrap_or(4);
        self.expansion_semaphore = Arc::new(Semaphore::new(expansion_permits));
    }

    /// Cleanup resources and optimize memory usage
    pub async fn cleanup(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) {
        // Perform aggressive pruning
        if let Err(e) = self.pruner.prune_tree(tree, PruningStrategy::LRU).await {
            warn!("Cleanup pruning failed: {}", e);
        }

        // Clear evaluation cache
        self.evaluator.clear_cache();

        // Reset some statistics
        self.stats.memory_stats.total_memory_bytes = 0;
    }

    /// Health check for the expansion system
    pub fn health_check(&self) -> HealthStatus {
        let mut issues = Vec::new();

        // Check success rate
        let total_attempts = self.stats.successful_expansions + self.stats.failed_expansions;
        if total_attempts > 0 {
            let success_rate = self.stats.successful_expansions as f64 / total_attempts as f64;
            if success_rate < 0.8 {
                issues.push(format!("Low expansion success rate: {:.2}%", success_rate * 100.0));
            }
        }

        // Check memory usage
        if self.stats.memory_stats.total_memory_bytes > 100 * 1024 * 1024 { // 100MB
            issues.push("High memory usage detected".to_string());
        }

        // Check average expansion time
        if self.stats.avg_expansion_time_us > 10000.0 { // 10ms
            issues.push("High average expansion time".to_string());
        }

        HealthStatus {
            healthy: issues.is_empty(),
            issues,
            uptime_stats: self.stats.clone(),
        }
    }
}

/// Component statistics aggregation
#[derive(Debug, Clone)]
pub struct ComponentStats {
    pub expansion_stats: ExpansionCoordinatorStats,
    pub evaluation_stats: EvaluationStats,
    pub pruning_stats: PruningStats,
}

/// Health status for the expansion system
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub uptime_stats: ExpansionCoordinatorStats,
}

/// High-level expansion interface for external use
pub struct QuantumExpansionManager {
    coordinator: QuantumExpansionCoordinator,
}

impl QuantumExpansionManager {
    /// Create new expansion manager
    pub fn new(
        config: QuantumMCTSConfig,
        committee: Arc<EvaluationCommittee>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        phase_evolution: Arc<PhaseEvolution>,
    ) -> Self {
        Self {
            coordinator: QuantumExpansionCoordinator::new(
                config, committee, spec, user_objective, phase_evolution
            ),
        }
    }

    /// Simple expansion interface
    pub async fn expand(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: &str,
    ) -> Result<Option<String>, CognitiveError> {
        let request = ExpansionRequest {
            node_id: node_id.to_string(),
            evaluate_child: false,
            consider_pruning: false,
            priority: 1.0,
        };

        let response = self.coordinator.expand_comprehensive(tree, request).await?;
        Ok(response.child_id)
    }

    /// Expansion with evaluation
    pub async fn expand_and_evaluate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: &str,
    ) -> Result<(Option<String>, Option<EvaluationResult>), CognitiveError> {
        let request = ExpansionRequest {
            node_id: node_id.to_string(),
            evaluate_child: true,
            consider_pruning: false,
            priority: 1.0,
        };

        let response = self.coordinator.expand_comprehensive(tree, request).await?;
        Ok((response.child_id, response.evaluation))
    }

    /// Get statistics
    pub fn stats(&self) -> &ExpansionCoordinatorStats {
        self.coordinator.stats()
    }
}
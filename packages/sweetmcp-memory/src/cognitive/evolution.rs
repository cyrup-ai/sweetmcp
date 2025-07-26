// src/cognitive/evolution.rs
//! Self-optimizing component using MCTS with committee evaluation

use crate::cognitive::committee::{CommitteeEvent, EvaluationCommittee};
use crate::cognitive::mcts::{CodeState, MCTS};
use crate::cognitive::performance::PerformanceAnalyzer;
use crate::cognitive::quantum::ml_decoder::ComplexityMetrics;
use crate::cognitive::performance::PerformanceMetrics;
use crate::cognitive::types::{
    CognitiveError, OptimizationOutcome, OptimizationType,
    PendingOptimizationResult, OptimizationResult,
};
use crate::vector::async_vector_optimization::OptimizationSpec;

// Re-export removed to fix circular import (EvolutionRules already available via types module)
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, oneshot};
use tracing::{error, info};

pub trait CodeEvolution {
    fn evolve_routing_logic(&self) -> PendingOptimizationResult;
}

#[derive(Clone)]
pub struct CognitiveCodeEvolution {
    initial_state: CodeState,
    spec: Arc<OptimizationSpec>,
    user_objective: String,
}

impl CognitiveCodeEvolution {
    pub fn new(
        spec: Arc<OptimizationSpec>,
        initial_code: String,
        initial_latency: f64,
        initial_memory: f64,
        initial_relevance: f64,
    ) -> Self {
        let initial_state = CodeState {
            code: initial_code,
            latency: initial_latency,
            memory: initial_memory,
            relevance: initial_relevance,
            memory_usage: 0.0,
            complexity_score: 0.0,
            metadata: std::collections::HashMap::new(),
        };

        Self {
            initial_state,
            spec,
            user_objective: "General optimization".to_string(),
        }
    }

    /// Comprehensive optimization method for iteration management
    pub async fn optimize(&self) -> Result<OptimizationResult, CognitiveError> {
        // Run the existing evolution logic
        let pending_result = self.evolve_routing_logic();
        let optimization_outcome = pending_result.wait_for_result().await?;

        // Create a result with the expected fields
        let optimized_state = self.get_optimized_state().await?;
        
        Ok(OptimizationResult {
            optimized_code: optimized_state.code,
            final_latency: optimized_state.latency,
            final_memory: optimized_state.memory,
            final_relevance: optimized_state.relevance,
            outcome: optimization_outcome,
        })
    }

    async fn get_optimized_state(&self) -> Result<CodeState, CognitiveError> {
        // For now, return a slightly improved version of the initial state
        // This would be replaced with actual optimization logic
        let mut state = self.initial_state.clone();
        state.latency *= 0.95; // 5% improvement
        state.memory *= 0.98; // 2% improvement
        state.relevance *= 1.1; // 10% improvement
        Ok(state)
    }
}

impl CodeEvolution for CognitiveCodeEvolution {
    fn evolve_routing_logic(&self) -> PendingOptimizationResult {
        let (tx, rx) = oneshot::channel();
        let initial_state = self.initial_state.clone();
        let spec = Arc::clone(&self.spec);
        let user_objective = self.user_objective.clone();

        tokio::spawn(async move {
            // Create event channel for committee
            let (event_tx, mut event_rx) = mpsc::channel(256);

            // Spawn event logger
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    match event {
                        CommitteeEvent::ConsensusReached {
                            action,
                            factors,
                            rounds_taken,
                        } => {
                            info!(
                                "Committee consensus on '{}' after {} rounds: latency={:.2}, memory={:.2}, relevance={:.2}, confidence={:.2}",
                                action,
                                rounds_taken,
                                factors.latency_factor,
                                factors.memory_factor,
                                factors.relevance_factor,
                                factors.confidence
                            );
                        }
                        CommitteeEvent::SteeringDecision {
                            feedback,
                            continue_rounds,
                        } => {
                            info!(
                                "Committee steering: {} (continue: {})",
                                feedback, continue_rounds
                            );
                        }
                        _ => {} // Log other events at debug level
                    }
                }
            });

            // Create committee
            let committee = match EvaluationCommittee::new(event_tx.clone(), 4).await {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    error!("Failed to create committee: {}", e);
                    let _ = tx.send(Err(e));
                    return;
                }
            };

            // Create performance analyzer with committee
            let performance_analyzer = Arc::new(
                PerformanceAnalyzer::new(spec.clone(), committee.clone(), user_objective.clone())
                    .await,
            );

            // Create and run MCTS
            let mut mcts = match MCTS::new(
                initial_state.clone(),
                performance_analyzer.clone(),
                spec.clone(),
                user_objective.clone(),
                event_tx,
            )
            .await
            {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to create MCTS: {}", e);
                    let _ = tx.send(Err(e));
                    return;
                }
            };

            // Run MCTS iterations
            if let Err(e) = mcts.run(1000).await {
                error!("MCTS execution failed: {}", e);
                let _ = tx.send(Err(e));
                return;
            }

            // Get best modification
            if let Some(best_state) = mcts.best_modification() {
                // Calculate improvements
                let latency_improvement =
                    (initial_state.latency - best_state.latency) / initial_state.latency * 100.0;
                let memory_improvement =
                    (initial_state.memory - best_state.memory) / initial_state.memory * 100.0;
                let relevance_improvement = (best_state.relevance - initial_state.relevance)
                    / initial_state.relevance
                    * 100.0;

                // Check if improvements are significant
                if latency_improvement > 5.0
                    || memory_improvement > 5.0
                    || relevance_improvement > 10.0
                {
                    let outcome = OptimizationOutcome::Success {
                        improvements: vec![
                            format!("Latency improved by {:.1}%", latency_improvement),
                            format!("Memory improved by {:.1}%", memory_improvement),
                            format!("Relevance improved by {:.1}%", relevance_improvement),
                        ],
                        performance_gain: (latency_improvement
                            + memory_improvement
                            + relevance_improvement)
                            / 3.0,
                        quality_score: 0.8,
                        metadata: std::collections::HashMap::new(),
                    };

                    info!(
                        "Applied optimization: latency improved {:.1}%, memory improved {:.1}%, relevance improved {:.1}%",
                        latency_improvement, memory_improvement, relevance_improvement
                    );

                    // Get statistics
                    let stats = mcts.get_statistics();
                    info!(
                        "MCTS explored {} nodes with {} total visits, max depth {}, best path: {:?}",
                        stats.total_nodes, stats.total_visits, stats.max_depth, stats.best_path
                    );

                    let _ = tx.send(Ok(outcome));
                } else {
                    info!("No significant improvement found");
                    let _ = tx.send(Ok(OptimizationOutcome::Failure {
                        errors: vec!["No significant improvement found".to_string()],
                        root_cause: "Improvement threshold not met".to_string(),
                        suggestions: vec!["Try different optimization parameters".to_string()],
                    }));
                }
            } else {
                info!("No modifications found");
                let _ = tx.send(Ok(OptimizationOutcome::Failure {
                    errors: vec!["No modifications found".to_string()],
                    root_cause: "MCTS search returned no results".to_string(),
                    suggestions: vec!["Check input parameters".to_string()],
                }));
            }
        });

        PendingOptimizationResult::new(rx)
    }
}

/// Evolution engine for managing optimization processes
pub struct EvolutionEngine {
    state_manager: Arc<RwLock<CodeState>>,
    generation_count: u64,
    optimization_spec: Arc<OptimizationSpec>,
}

impl EvolutionEngine {
    pub fn new(initial_state: CodeState, max_generations: u64) -> Self {
        Self {
            state_manager: Arc::new(RwLock::new(initial_state)),
            generation_count: 0,
            optimization_spec: Arc::new(OptimizationSpec {
                objective: "Optimize for performance".to_string(),
                constraints: vec!["Max memory: 256MB".to_string()],
                success_criteria: vec!["Improve latency by 10%".to_string()],
                optimization_type: OptimizationType::Performance,
                timeout_ms: Some(300_000),
                max_iterations: Some(50),
                target_quality: 0.8,
                content_type: crate::cognitive::types::ContentType {
                    format: "rust".to_string(),
                    restrictions: crate::cognitive::types::Restrictions {
                        compiler: "rustc".to_string(),
                        max_latency_increase: 20.0,
                        max_memory_increase: 30.0,
                        min_relevance_improvement: 10.0,
                    },
                },
                baseline_metrics: crate::cognitive::types::BaselineMetrics {
                    latency: 100.0,
                    memory: 256.0,
                    relevance: 0.5,
                },
                evolution_rules: crate::cognitive::types::EvolutionRules {
                    build_on_previous: true,
                    new_axis_per_iteration: false,
                    max_cumulative_latency_increase: 50.0,
                    min_action_diversity: 0.3,
                    validation_required: true,
                },
            }),
        }
    }

    /// Create a new lock-free evolution engine with default initial state
    pub fn new_lock_free(evolution_rate: f64) -> Self {
        let initial_state = CodeState {
            source_code: "".to_string(),
            complexity_metrics: ComplexityMetrics {
                cyclomatic_complexity: 1,
                cognitive_complexity: 1,
                lines_of_code: 0,
                nested_depth: 0,
                maintainability_index: 100.0,
            },
            performance_metrics: PerformanceMetrics {
                execution_time: 0.0,
                memory_usage: 0,
                cpu_usage: 0.0,
                io_operations: 0,
                network_calls: 0,
            },
            memory_usage: 0,
            complexity_score: 0.0,
            metadata: crate::cognitive::mcts::types::node_types::CodeMetadata::default(),
        };

        let mut instance = Self::new(initial_state, 100);
        
        // Configure for lock-free operation with the provided evolution rate  
        if let Ok(mut spec) = Arc::try_unwrap(instance.optimization_spec) {
            spec.target_quality = evolution_rate.min(1.0);
            instance.optimization_spec = Arc::new(spec);
        }
        
        instance
    }

    pub async fn evolve(&mut self) -> Result<bool, CognitiveError> {
        self.generation_count += 1;
        // Basic evolution logic - would be expanded with actual MCTS
        Ok(true)
    }

    pub fn get_generation_count(&self) -> u64 {
        self.generation_count
    }

    pub async fn get_current_state(&self) -> CodeState {
        self.state_manager.read().await.clone()
    }
}

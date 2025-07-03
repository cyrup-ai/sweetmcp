// src/cognitive/evolution.rs
//! Self-optimizing component using MCTS with committee evaluation

use crate::cognitive::committee::{CommitteeEvent, EvaluationCommittee};
use crate::cognitive::compiler::{CompiledCode, RuntimeCompiler};
use crate::cognitive::mcts::{CodeState, MCTS};
use crate::cognitive::performance::PerformanceAnalyzer;
use crate::cognitive::types::{
    CognitiveError, OptimizationOutcome, OptimizationSpec, PendingOptimizationResult,
};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
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
        initial_code: String,
        initial_latency: f64,
        initial_memory: f64,
        initial_relevance: f64,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
    ) -> Result<Self, CognitiveError> {
        let initial_state = CodeState {
            code: initial_code,
            latency: initial_latency,
            memory: initial_memory,
            relevance: initial_relevance,
        };

        Ok(Self {
            initial_state,
            spec,
            user_objective,
        })
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
                    let outcome = OptimizationOutcome {
                        applied: true,
                        latency_improvement,
                        memory_improvement,
                        relevance_improvement,
                        iteration: 0, // Set by orchestrator
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
                    let _ = tx.send(Ok(OptimizationOutcome {
                        applied: false,
                        latency_improvement: 0.0,
                        memory_improvement: 0.0,
                        relevance_improvement: 0.0,
                        iteration: 0,
                    }));
                }
            } else {
                info!("No modifications found");
                let _ = tx.send(Ok(OptimizationOutcome {
                    applied: false,
                    latency_improvement: 0.0,
                    memory_improvement: 0.0,
                    relevance_improvement: 0.0,
                    iteration: 0,
                }));
            }
        });

        PendingOptimizationResult::new(rx)
    }
}

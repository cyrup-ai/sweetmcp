// src/cognitive/performance.rs
//! Performance analysis for code states using committee evaluation

use crate::cognitive::committee::{CommitteeEvent, EvaluationCommittee};
use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::ImpactFactors;
use crate::cognitive::types::{CognitiveError, OptimizationSpec};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Tracks performance metrics across evaluations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub evaluations: Vec<StateEvaluation>,
    pub timestamp: Instant,
}

/// Single state evaluation result
#[derive(Debug, Clone)]
pub struct StateEvaluation {
    pub state: CodeState,
    pub impact_factors: ImpactFactors,
    pub objective_score: f64,
    pub timestamp: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            evaluations: Vec::new(),
            timestamp: Instant::now(),
        }
    }
}

/// Analyzes performance and calculates rewards using committee evaluations
pub struct PerformanceAnalyzer {
    spec: Arc<OptimizationSpec>,
    metrics_history: Arc<RwLock<PerformanceMetrics>>,
    committee: Arc<EvaluationCommittee>,
    user_objective: String,
}

impl PerformanceAnalyzer {
    pub async fn new(
        spec: Arc<OptimizationSpec>,
        committee: Arc<EvaluationCommittee>,
        user_objective: String,
    ) -> Self {
        Self {
            spec,
            metrics_history: Arc::new(RwLock::new(PerformanceMetrics::default())),
            committee,
            user_objective,
        }
    }

    /// Estimate reward for a state using committee evaluation
    pub async fn estimate_reward(&self, state: &CodeState) -> Result<f64, CognitiveError> {
        // Ask committee to evaluate the state against the objective
        let evaluation_action = "evaluate_current_state";
        let factors = self
            .committee
            .evaluate_action(state, evaluation_action, &self.spec, &self.user_objective)
            .await?;

        // Calculate reward based on how well the state meets the objectives
        let reward = self.calculate_reward_from_factors(&factors, state);

        // Store evaluation in history
        let evaluation = StateEvaluation {
            state: state.clone(),
            impact_factors: factors.clone(),
            objective_score: reward,
            timestamp: Instant::now(),
        };

        self.metrics_history
            .write()
            .await
            .evaluations
            .push(evaluation);

        debug!(
            "State evaluation: latency={:.2}, memory={:.2}, relevance={:.2}, reward={:.3}",
            state.latency, state.memory, state.relevance, reward
        );

        Ok(reward)
    }

    /// Calculate reward from impact factors
    fn calculate_reward_from_factors(&self, factors: &ImpactFactors, state: &CodeState) -> f64 {
        let spec = &self.spec;

        // Check if we're within constraints
        let latency_ok = state.latency
            <= spec.baseline_metrics.latency
                * (1.0 + spec.content_type.restrictions.max_latency_increase / 100.0);
        let memory_ok = state.memory
            <= spec.baseline_metrics.memory
                * (1.0 + spec.content_type.restrictions.max_memory_increase / 100.0);
        let relevance_ok = state.relevance
            >= spec.baseline_metrics.relevance
                * (1.0 + spec.content_type.restrictions.min_relevance_improvement / 100.0);

        if !latency_ok || !memory_ok || !relevance_ok {
            return 0.0; // Constraint violation
        }

        // Calculate improvement scores
        let latency_improvement =
            (spec.baseline_metrics.latency - state.latency) / spec.baseline_metrics.latency;
        let memory_improvement =
            (spec.baseline_metrics.memory - state.memory) / spec.baseline_metrics.memory;
        let relevance_improvement =
            (state.relevance - spec.baseline_metrics.relevance) / spec.baseline_metrics.relevance;

        // Weight by confidence from committee
        let base_reward =
            (latency_improvement * 0.3 + memory_improvement * 0.3 + relevance_improvement * 0.4)
                .max(0.0);
        let weighted_reward = base_reward * factors.confidence;

        weighted_reward
    }

    /// Get performance trend analysis
    pub async fn analyze_trend(&self) -> PerformanceTrend {
        let metrics = self.metrics_history.read().await;

        if metrics.evaluations.len() < 2 {
            return PerformanceTrend::Insufficient;
        }

        // Calculate trend over last N evaluations
        let window = 10.min(metrics.evaluations.len());
        let recent = &metrics.evaluations[metrics.evaluations.len() - window..];

        let mut improving = 0;
        let mut degrading = 0;

        for i in 1..recent.len() {
            if recent[i].objective_score > recent[i - 1].objective_score * 1.05 {
                improving += 1;
            } else if recent[i].objective_score < recent[i - 1].objective_score * 0.95 {
                degrading += 1;
            }
        }

        if improving > degrading * 2 {
            PerformanceTrend::Improving
        } else if degrading > improving * 2 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }

    /// Get best state found so far
    pub async fn get_best_state(&self) -> Option<CodeState> {
        let metrics = self.metrics_history.read().await;

        metrics
            .evaluations
            .iter()
            .max_by(|a, b| a.objective_score.partial_cmp(&b.objective_score).unwrap())
            .map(|eval| eval.state.clone())
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
    Insufficient,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_performance_analyzer() {
        let spec = Arc::new(OptimizationSpec {
            content_type: crate::cognitive::types::ContentType {
                format: "rust".to_string(),
                restrictions: crate::cognitive::types::Restrictions {
                    compiler: "rustc".to_string(),
                    max_latency_increase: 20.0,
                    max_memory_increase: 30.0,
                    min_relevance_improvement: 40.0,
                },
            },
            constraints: crate::cognitive::types::Constraints {
                size: "single function".to_string(),
                style: "idiomatic".to_string(),
                schemas: vec![],
            },
            evolution_rules: crate::cognitive::types::EvolutionRules {
                build_on_previous: true,
                new_axis_per_iteration: true,
                max_cumulative_latency_increase: 20.0,
                min_action_diversity: 30.0,
                validation_required: true,
            },
            baseline_metrics: crate::cognitive::types::BaselineMetrics {
                latency: 10.0,
                memory: 100.0,
                relevance: 50.0,
            },
        });

        let (tx, _rx) = mpsc::channel(64);
        let committee = Arc::new(EvaluationCommittee::new(tx, 2).await.unwrap());

        let analyzer =
            PerformanceAnalyzer::new(spec, committee, "Optimize for search relevance".to_string())
                .await;

        let state = CodeState {
            code: "fn search() { /* optimized */ }".to_string(),
            latency: 8.0,
            memory: 90.0,
            relevance: 75.0,
        };

        match analyzer.estimate_reward(&state).await {
            Ok(reward) => {
                assert!(reward > 0.0, "Good state should have positive reward");
            }
            Err(e) => {
                // This is expected if no LLM models are configured
                println!("Test skipped: {}", e);
            }
        }
    }
}

// tests/test_quantum_mcts.rs
//! Integration tests for quantum MCTS recursive improvement

use std::sync::Arc;
use surreal_memory::cognitive::{
    committee::CommitteeEvent,
    mcts::CodeState,
    performance::PerformanceAnalyzer,
    quantum_mcts::{QuantumMCTS, QuantumMCTSConfig},
    quantum_orchestrator::{QuantumOrchestrationConfig, QuantumOrchestrator},
    types::{ContentRestrictions, ContentType, OptimizationSpec},
};
use tokio::sync::mpsc;

#[tokio::test]
async fn test_quantum_mcts_basic() {
    // Setup
    let (event_tx, mut event_rx) = mpsc::channel(100);

    let initial_state = CodeState {
        code: r#"
fn process_data(items: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if item > 0 {
            result.push(item * 2);
        }
    }
    result
}
"#
        .to_string(),
        latency: 100.0,
        memory: 50.0,
        relevance: 80.0,
    };

    let spec = Arc::new(OptimizationSpec {
        content_type: ContentType {
            restrictions: ContentRestrictions {
                max_latency_increase: 10.0,
                max_memory_increase: 20.0,
                min_relevance_improvement: 5.0,
            },
        },
        baseline_metrics: initial_state.clone(),
        user_objective: "Optimize for performance while maintaining accuracy".to_string(),
    });

    let performance_analyzer = Arc::new(PerformanceAnalyzer::new());
    let config = QuantumMCTSConfig::default();

    // Create quantum MCTS
    let mut quantum_mcts = QuantumMCTS::new(
        initial_state,
        performance_analyzer,
        spec,
        "Optimize data processing".to_string(),
        event_tx,
        config,
    )
    .await
    .unwrap();

    // Run recursive improvement
    quantum_mcts.recursive_improve(50).await.unwrap();

    // Get results
    let best_modification = quantum_mcts.best_quantum_modification().await;
    assert!(best_modification.is_some());

    let stats = quantum_mcts.get_quantum_statistics().await;
    assert!(stats.total_nodes > 1);
    assert!(stats.total_visits > 0);

    // Verify quantum properties
    assert!(stats.max_amplitude > 0.0);
    assert!(stats.avg_decoherence < 1.0);
}

#[tokio::test]
async fn test_quantum_orchestrator() {
    // Setup
    let (event_tx, mut event_rx) = mpsc::channel(100);

    let initial_state = CodeState {
        code: r#"
async fn fetch_data(urls: Vec<String>) -> Vec<Result<String, Error>> {
    let mut results = Vec::new();
    for url in urls {
        match fetch(url).await {
            Ok(data) => results.push(Ok(data)),
            Err(e) => results.push(Err(e)),
        }
    }
    results
}
"#
        .to_string(),
        latency: 200.0,
        memory: 100.0,
        relevance: 75.0,
    };

    let spec = Arc::new(OptimizationSpec {
        content_type: ContentType {
            restrictions: ContentRestrictions {
                max_latency_increase: 5.0,
                max_memory_increase: 10.0,
                min_relevance_improvement: 10.0,
            },
        },
        baseline_metrics: initial_state.clone(),
        user_objective: "Parallelize async operations for better performance".to_string(),
    });

    let performance_analyzer = Arc::new(PerformanceAnalyzer::new());
    let orchestration_config = QuantumOrchestrationConfig {
        max_recursive_depth: 3,
        improvement_threshold: 0.03,
        coherence_time_ms: 100,
        parallel_circuits: 2,
        convergence_epsilon: 0.01,
        max_iterations_per_depth: 30,
    };
    let mcts_config = QuantumMCTSConfig::default();

    // Create orchestrator
    let orchestrator = QuantumOrchestrator::new(
        orchestration_config,
        mcts_config,
        performance_analyzer,
        event_tx,
    )
    .await
    .unwrap();

    // Run recursive improvement
    let outcome = orchestrator
        .run_recursive_improvement(initial_state, spec, "Optimize async fetching".to_string())
        .await
        .unwrap();

    // Verify results
    assert!(outcome.improvement_percentage > 0.0);
    assert!(!outcome.optimized_code.is_empty());
    assert!(
        outcome
            .applied_techniques
            .contains(&"quantum_mcts".to_string())
    );

    // Check improvement history
    let history = orchestrator.get_improvement_history().await;
    assert!(!history.is_empty());

    // Visualize evolution
    let visualization = orchestrator.visualize_evolution().await.unwrap();
    assert!(visualization.contains("Quantum Recursive Improvement"));
}

#[tokio::test]
async fn test_quantum_convergence() {
    // Setup
    let (event_tx, _) = mpsc::channel(100);

    let initial_state = CodeState {
        code: "fn simple() -> i32 { 42 }".to_string(),
        latency: 10.0,
        memory: 5.0,
        relevance: 100.0,
    };

    let spec = Arc::new(OptimizationSpec {
        content_type: ContentType {
            restrictions: ContentRestrictions {
                max_latency_increase: 1.0,
                max_memory_increase: 1.0,
                min_relevance_improvement: 0.0,
            },
        },
        baseline_metrics: initial_state.clone(),
        user_objective: "Already optimal code".to_string(),
    });

    let performance_analyzer = Arc::new(PerformanceAnalyzer::new());
    let config = QuantumMCTSConfig {
        recursive_iterations: 2,
        amplitude_threshold: 0.1,
        ..Default::default()
    };

    // Create quantum MCTS
    let mut quantum_mcts = QuantumMCTS::new(
        initial_state,
        performance_analyzer,
        spec,
        "Test convergence".to_string(),
        event_tx,
        config,
    )
    .await
    .unwrap();

    // Run and expect quick convergence
    quantum_mcts.recursive_improve(20).await.unwrap();

    let stats = quantum_mcts.get_quantum_statistics().await;
    assert!(stats.total_nodes < 50); // Should converge quickly
}

#[tokio::test]
async fn test_quantum_entanglement_effects() {
    // Setup
    let (event_tx, _) = mpsc::channel(100);

    let initial_state = CodeState {
        code: r#"
fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = a.len();
    let m = b[0].len();
    let p = b.len();
    let mut result = vec![vec![0.0; m]; n];
    
    for i in 0..n {
        for j in 0..m {
            for k in 0..p {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}
"#
        .to_string(),
        latency: 500.0,
        memory: 200.0,
        relevance: 90.0,
    };

    let spec = Arc::new(OptimizationSpec {
        content_type: ContentType {
            restrictions: ContentRestrictions {
                max_latency_increase: 0.0,
                max_memory_increase: 50.0,
                min_relevance_improvement: 0.0,
            },
        },
        baseline_metrics: initial_state.clone(),
        user_objective: "Optimize matrix multiplication with cache efficiency".to_string(),
    });

    let performance_analyzer = Arc::new(PerformanceAnalyzer::new());
    let config = QuantumMCTSConfig {
        entanglement_strength: 0.9,
        quantum_exploration: 3.0,
        ..Default::default()
    };

    // Create quantum MCTS
    let mut quantum_mcts = QuantumMCTS::new(
        initial_state,
        performance_analyzer,
        spec,
        "Matrix optimization".to_string(),
        event_tx,
        config,
    )
    .await
    .unwrap();

    // Run improvement
    quantum_mcts.recursive_improve(100).await.unwrap();

    let stats = quantum_mcts.get_quantum_statistics().await;

    // Verify entanglement was created
    assert!(stats.total_entanglements > 0);
    assert!(stats.total_entanglements as f64 / stats.total_nodes as f64 > 0.1);
}

#[cfg(test)]
mod performance_mock {
    use super::*;

    impl PerformanceAnalyzer {
        pub fn new() -> Self {
            Self {
                // Mock implementation
            }
        }

        pub async fn estimate_reward(
            &self,
            state: &CodeState,
        ) -> Result<f64, Box<dyn std::error::Error>> {
            // Simple reward calculation for testing
            let latency_score = 100.0 / state.latency;
            let memory_score = 50.0 / state.memory;
            let relevance_score = state.relevance / 100.0;

            Ok(latency_score * 0.5 + memory_score * 0.3 + relevance_score * 0.2)
        }
    }
}

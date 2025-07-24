//! MCTS integration specifics extracted from quantum orchestrator

use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

use crate::cognitive::{
    committee::CommitteeEvent,
    evolution::CognitiveCodeEvolution,
    performance::PerformanceAnalyzer,
    quantum::{QuantumConfig, QuantumRouter},
    quantum_mcts::QuantumMCTSConfig,
    types::CognitiveError,
};

use super::{
    config::{QuantumOrchestrationConfig, RecursiveState},
    recursive_improvement::RecursiveImprovement,
};

/// Quantum orchestrator for recursive improvement
pub struct QuantumOrchestrator {
    /// Configuration
    config: QuantumOrchestrationConfig,
    /// Quantum MCTS config
    mcts_config: QuantumMCTSConfig,
    /// Performance analyzer
    performance_analyzer: Arc<PerformanceAnalyzer>,
    /// Event channel
    event_tx: mpsc::Sender<CommitteeEvent>,
    /// Recursive improvement engine
    recursive_improvement: RecursiveImprovement,
}

impl QuantumOrchestrator {
    pub async fn new(
        config: QuantumOrchestrationConfig,
        mcts_config: QuantumMCTSConfig,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<Self, CognitiveError> {
        let quantum_config = QuantumConfig::default();
        let quantum_router = Arc::new(QuantumRouter::new(quantum_config));
        let evolution_engine = Arc::new(CognitiveCodeEvolution::new());

        let recursive_improvement = RecursiveImprovement::new(
            config.clone(),
            quantum_router,
            evolution_engine,
        );

        Ok(Self {
            config,
            mcts_config,
            performance_analyzer,
            event_tx: event_tx.clone(),
            recursive_improvement,
        })
    }

    /// Run recursive quantum improvement
    pub async fn run_recursive_improvement(
        &self,
        initial_state: crate::cognitive::mcts::CodeState,
        spec: Arc<crate::cognitive::types::OptimizationSpec>,
        user_objective: String,
    ) -> Result<crate::cognitive::types::OptimizationOutcome, CognitiveError> {
        self.recursive_improvement
            .run_recursive_improvement(
                initial_state,
                spec,
                user_objective,
                &self.mcts_config,
                self.performance_analyzer.clone(),
                self.event_tx.clone(),
            )
            .await
    }

    /// Get recursive improvement history
    pub async fn get_improvement_history(&self) -> Vec<RecursiveState> {
        self.recursive_improvement.get_improvement_history().await
    }

    /// Visualize quantum evolution
    pub async fn visualize_evolution(&self) -> Result<String, CognitiveError> {
        self.recursive_improvement.visualize_evolution().await
    }
}
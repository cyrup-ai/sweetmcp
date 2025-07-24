// src/cognitive/quantum_mcts/tree_operations.rs
//! Tree operations, selection, expansion, and backpropagation logic

use rand::Rng;
use std::collections::HashMap;
use tokio::task::JoinSet;
use tracing::{error, info};

use crate::cognitive::{
    mcts::CodeState,
    quantum::{Complex64, EntanglementType, MeasurementBasis},
    types::CognitiveError,
};

use super::core::{QuantumMCTS, QuantumMCTSNode, QuantumNodeState};

impl QuantumMCTS {
    /// Quantum selection using superposition and entanglement
    pub(crate) async fn quantum_select(&self) -> Result<String, CognitiveError> {
        let tree = self.tree.read().await;
        let mut current_id = self.root_id.clone();

        loop {
            let node = tree
                .get(&current_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found".to_string()))?;

            // Check terminal or expansion needed
            if node.is_terminal || !node.untried_actions.is_empty() {
                return Ok(current_id);
            }

            if node.children.is_empty() {
                return Ok(current_id);
            }

            // Quantum UCT selection
            let selected_child = self.quantum_uct_select(node, &tree).await?;
            current_id = selected_child;
        }
    }

    /// Quantum UCT selection with superposition
    async fn quantum_uct_select(
        &self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<String, CognitiveError> {
        let parent_visits = node.visits as f64;
        let mut quantum_scores: Vec<(String, f64)> = Vec::new();

        for (_action, child_id) in &node.children {
            let child = tree
                .get(child_id)
                .ok_or_else(|| CognitiveError::InvalidState("Child not found".to_string()))?;

            // Calculate quantum UCT score
            let quantum_score = if child.visits == 0 {
                f64::INFINITY
            } else {
                let exploitation = child.quantum_reward.norm() / child.visits as f64;
                let exploration = self.config.quantum_exploration
                    * ((parent_visits.ln()) / (child.visits as f64)).sqrt();
                let quantum_bonus =
                    child.amplitude.norm() * (1.0 - child.quantum_state.decoherence);

                exploitation + exploration + quantum_bonus
            };

            quantum_scores.push((child_id.clone(), quantum_score));
        }

        // Select using quantum measurement
        let selected = self.quantum_measure_selection(quantum_scores).await?;
        Ok(selected)
    }

    /// Quantum measurement for selection
    async fn quantum_measure_selection(
        &self,
        scores: Vec<(String, f64)>,
    ) -> Result<String, CognitiveError> {
        if scores.is_empty() {
            return Err(CognitiveError::InvalidState(
                "No children to select".to_string(),
            ));
        }

        // Convert scores to probability amplitudes
        let total_score: f64 = scores.iter().map(|(_, s)| s.exp()).sum();
        let probabilities: Vec<f64> = scores.iter().map(|(_, s)| s.exp() / total_score).collect();

        // Quantum measurement
        let mut rng = rand::thread_rng();
        let measurement = rng.gen_range(0.0..1.0);
        let mut cumulative = 0.0;

        for (i, p) in probabilities.iter().enumerate() {
            cumulative += p;
            if measurement < cumulative {
                return Ok(scores[i].0.clone());
            }
        }

        Ok(scores.last().unwrap().0.clone())
    }

    /// Quantum expansion with superposition
    pub(crate) async fn quantum_expand(&self, node_id: &str) -> Result<Option<String>, CognitiveError> {
        let mut tree = self.tree.write().await;

        let node = tree
            .get_mut(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Node not found".to_string()))?;

        if node.untried_actions.is_empty() {
            return Ok(None);
        }

        // Select action using quantum superposition
        let action_idx = self
            .quantum_action_selection(&node.quantum_state.superposition)
            .await?;
        let action = node.untried_actions.remove(action_idx);

        // Apply quantum transformation
        let parent_state = node.quantum_state.clone();
        let new_quantum_state = self.apply_quantum_action(&parent_state, &action).await?;

        // Create child with quantum properties
        let child_id = format!("{}-q{}", node_id, tree.len());
        let child_node = QuantumMCTSNode {
            id: child_id.clone(),
            visits: 0,
            amplitude: self.calculate_child_amplitude(&node.amplitude, &action),
            quantum_reward: Complex64::new(0.0, 0.0),
            children: HashMap::new(),
            parent: Some(node_id.to_string()),
            quantum_state: new_quantum_state,
            untried_actions: Self::get_quantum_actions(
                &new_quantum_state.classical_state,
                &self.spec,
            ),
            is_terminal: false,
            applied_action: Some(action.clone()),
            improvement_depth: node.improvement_depth,
        };

        // Add to tree
        tree.insert(child_id.clone(), child_node);
        tree.get_mut(node_id)
            .unwrap()
            .children
            .insert(action, child_id.clone());

        // Create entanglement if appropriate
        self.create_entanglement(&child_id, &tree).await?;

        Ok(Some(child_id))
    }

    /// Quantum action selection using superposition
    async fn quantum_action_selection(
        &self,
        superposition: &crate::cognitive::quantum::SuperpositionState,
    ) -> Result<usize, CognitiveError> {
        // Measure the superposition state
        let measurement = MeasurementBasis::computational();
        let probabilities = superposition.measure(&measurement)?;

        // Select based on quantum probabilities
        let mut rng = rand::thread_rng();
        let selection = rng.gen_range(0.0..1.0);
        let mut cumulative = 0.0;

        for (i, &p) in probabilities.iter().enumerate() {
            cumulative += p;
            if selection < cumulative {
                return Ok(i);
            }
        }

        Ok(probabilities.len() - 1)
    }

    /// Apply quantum action with transformation
    async fn apply_quantum_action(
        &self,
        state: &QuantumNodeState,
        action: &str,
    ) -> Result<QuantumNodeState, CognitiveError> {
        // Get classical transformation
        let new_classical = self
            .committee
            .evaluate_action(
                &state.classical_state,
                action,
                &self.spec,
                &self.user_objective,
            )
            .await?;

        // Apply quantum evolution
        let mut new_superposition = state.superposition.clone();
        new_superposition.evolve(self.phase_evolution.compute(0.1))?;

        // Update phase
        let new_phase = state.phase + self.config.phase_evolution_rate;

        // Calculate decoherence
        let new_decoherence = state.decoherence + 0.01; // Simple model

        Ok(QuantumNodeState {
            classical_state: CodeState {
                code: format!("// Quantum: {}\n{}", action, state.classical_state.code),
                latency: state.classical_state.latency * 0.95,
                memory: state.classical_state.memory * 0.95,
                relevance: state.classical_state.relevance * 1.02,
            },
            superposition: new_superposition,
            entanglements: state.entanglements.clone(),
            phase: new_phase,
            decoherence: new_decoherence,
        })
    }

    /// Calculate child amplitude
    pub(crate) fn calculate_child_amplitude(&self, parent_amplitude: &Complex64, action: &str) -> Complex64 {
        // Action-dependent phase shift
        let phase_shift = match action {
            "optimize_hot_paths" => 0.1,
            "reduce_allocations" => 0.15,
            "improve_cache_locality" => 0.2,
            _ => 0.05,
        };

        let phase = Complex64::new(0.0, phase_shift);
        *parent_amplitude * phase.exp() * 0.9 // Amplitude decay
    }

    /// Create entanglement between related nodes
    pub(crate) async fn create_entanglement(
        &self,
        node_id: &str,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<(), CognitiveError> {
        let node = tree
            .get(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Node not found".to_string()))?;

        // Find nodes with similar quantum states
        let mut entanglement_graph = self.entanglement_graph.write().await;

        for (other_id, other_node) in tree.iter() {
            if other_id != node_id && self.should_entangle(node, other_node) {
                entanglement_graph.add_entanglement(
                    node_id.to_string(),
                    other_id.to_string(),
                    EntanglementType::Weak,
                    self.config.entanglement_strength,
                )?;
            }
        }

        Ok(())
    }

    /// Check if nodes should be entangled
    fn should_entangle(&self, node1: &QuantumMCTSNode, node2: &QuantumMCTSNode) -> bool {
        // Entangle if similar improvement depth and low decoherence
        let depth_similar =
            (node1.improvement_depth as i32 - node2.improvement_depth as i32).abs() <= 1;
        let both_coherent = node1.quantum_state.decoherence < self.config.decoherence_threshold
            && node2.quantum_state.decoherence < self.config.decoherence_threshold;

        depth_similar && both_coherent
    }

    /// Run single quantum MCTS iteration
    pub(crate) async fn run_quantum_iteration(&self, iterations: u32) -> Result<(), CognitiveError> {
        let mut join_set = JoinSet::new();
        let mut completed = 0;

        while completed < iterations {
            if join_set.len() >= self.config.max_quantum_parallel {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok(Ok((node_id, reward))) => {
                            self.quantum_backpropagate(node_id, reward).await?;
                            completed += 1;
                        }
                        Ok(Err(e)) => error!("Quantum simulation failed: {}", e),
                        Err(e) => error!("Task panicked: {}", e),
                    }
                }
            }

            // Quantum selection
            let selected = self.quantum_select().await?;

            // Quantum expansion
            let node_to_simulate = match self.quantum_expand(&selected).await? {
                Some(child_id) => child_id,
                None => selected,
            };

            // Clone for async simulation
            let tree = self.tree.clone();
            let performance_analyzer = self.performance_analyzer.clone();
            let error_correction = self.error_correction.clone();

            join_set.spawn(async move {
                let tree_read = tree.read().await;
                let node = tree_read
                    .get(&node_to_simulate)
                    .ok_or_else(|| CognitiveError::InvalidState("Node not found".to_string()))?;

                // Quantum simulation with error correction
                let raw_reward = performance_analyzer
                    .estimate_reward(&node.quantum_state.classical_state)
                    .await?;

                let quantum_reward = Complex64::new(raw_reward, node.quantum_state.phase.sin());
                let corrected_reward = error_correction.correct_amplitude(quantum_reward)?;

                Ok((node_to_simulate, corrected_reward))
            });
        }

        // Complete remaining tasks
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((node_id, reward))) => {
                    self.quantum_backpropagate(node_id, reward).await?;
                }
                Ok(Err(e)) => error!("Final quantum simulation failed: {}", e),
                Err(e) => error!("Final task panicked: {}", e),
            }
        }

        Ok(())
    }

    /// Quantum backpropagation with entanglement effects
    pub(crate) async fn quantum_backpropagate(
        &self,
        mut node_id: String,
        reward: Complex64,
    ) -> Result<(), CognitiveError> {
        let mut tree = self.tree.write().await;
        let entanglement_graph = self.entanglement_graph.read().await;

        // Direct backpropagation
        while let Some(node) = tree.get_mut(&node_id) {
            node.visits += 1;
            node.quantum_reward += reward * node.amplitude;

            // Entanglement effects
            if let Ok(entangled) = entanglement_graph.get_entangled(&node_id) {
                for (entangled_id, strength) in entangled {
                    if let Some(entangled_node) = tree.get_mut(&entangled_id) {
                        entangled_node.quantum_reward += reward * strength * 0.5;
                    }
                }
            }

            match &node.parent {
                Some(parent_id) => node_id = parent_id.clone(),
                None => break,
            }
        }

        Ok(())
    }

    /// Amplify promising paths using quantum amplitude amplification
    pub(crate) async fn amplify_promising_paths(&self) -> Result<(), CognitiveError> {
        let mut tree = self.tree.write().await;

        // Find high-reward paths
        let mut promising_nodes: Vec<(String, f64)> = Vec::new();

        for (id, node) in tree.iter() {
            if node.visits > 0 {
                let avg_reward = node.quantum_reward.norm() / node.visits as f64;
                if avg_reward > self.config.amplitude_threshold {
                    promising_nodes.push((id.clone(), avg_reward));
                }
            }
        }

        // Sort by reward
        promising_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Amplify top paths
        let amplification_factor = 1.2;
        for (node_id, _) in promising_nodes.iter().take(10) {
            if let Some(node) = tree.get_mut(node_id) {
                node.amplitude *= amplification_factor;
                // Reduce decoherence for promising paths
                node.quantum_state.decoherence *= 0.9;
            }
        }

        Ok(())
    }
}
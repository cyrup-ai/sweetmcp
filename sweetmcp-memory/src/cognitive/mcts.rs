// src/cognitive/mcts.rs
//! MCTS for exploring code modifications with committee-based evaluation

use crate::cognitive::committee::{CommitteeEvent, EvaluationCommittee};
use crate::cognitive::performance::PerformanceAnalyzer;
use crate::cognitive::types::{CognitiveError, ImpactFactors, OptimizationSpec};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

/// Codebase state with metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeState {
    pub code: String,
    pub latency: f64,
    pub memory: f64,
    pub relevance: f64,
}

/// MCTS node representing a state in the search tree
#[derive(Debug)]
struct MCTSNode {
    visits: u64,
    total_reward: f64,
    children: HashMap<String, String>, // action -> child_id
    parent: Option<String>,
    state: CodeState,
    untried_actions: Vec<String>,
    is_terminal: bool,
    applied_action: Option<String>, // Action that led to this state
}

/// MCTS controller with committee-based evaluation
pub struct MCTS {
    tree: HashMap<String, MCTSNode>,
    root_id: String,
    performance_analyzer: Arc<PerformanceAnalyzer>,
    committee: Arc<EvaluationCommittee>,
    spec: Arc<OptimizationSpec>,
    user_objective: String,
    max_parallel: usize,
    exploration_constant: f64,
}

impl MCTS {
    pub async fn new(
        initial_state: CodeState,
        performance_analyzer: Arc<PerformanceAnalyzer>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        event_tx: mpsc::Sender<CommitteeEvent>,
    ) -> Result<Self, CognitiveError> {
        let committee = Arc::new(EvaluationCommittee::new(event_tx, num_cpus::get().min(4)).await?);

        let root_id = "root".to_string();
        let untried_actions = Self::get_possible_actions(&initial_state, &spec);

        let tree = HashMap::from([(
            root_id.clone(),
            MCTSNode {
                visits: 0,
                total_reward: 0.0,
                children: HashMap::new(),
                parent: None,
                state: initial_state,
                untried_actions,
                is_terminal: false,
                applied_action: None,
            },
        )]);

        Ok(Self {
            tree,
            root_id,
            performance_analyzer,
            committee,
            spec,
            user_objective,
            max_parallel: num_cpus::get().min(8),
            exploration_constant: 1.41, // sqrt(2) for UCT
        })
    }

    /// Select a node to expand using UCT (Upper Confidence Bound for Trees)
    #[inline]
    fn select(&self) -> String {
        let mut current_id = self.root_id.clone();

        loop {
            let node = &self.tree[&current_id];

            // If terminal or has untried actions, return this node
            if node.is_terminal || !node.untried_actions.is_empty() {
                return current_id;
            }

            // If no children, this is terminal
            if node.children.is_empty() {
                return current_id;
            }

            // Select best child using UCT
            let mut best_uct = f64::NEG_INFINITY;
            let mut best_child = None;
            let parent_visits = node.visits as f64;

            for (_action, child_id) in &node.children {
                let child = &self.tree[child_id];
                let uct = if child.visits == 0 {
                    f64::INFINITY // Prioritize unvisited nodes
                } else {
                    let exploitation = child.total_reward / child.visits as f64;
                    let exploration = self.exploration_constant
                        * ((parent_visits.ln()) / (child.visits as f64)).sqrt();
                    exploitation + exploration
                };

                if uct > best_uct {
                    best_uct = uct;
                    best_child = Some(child_id.clone());
                }
            }

            current_id = best_child.expect("Should have found a child");
        }
    }

    /// Expand a node by trying an untried action
    async fn expand(&mut self, node_id: &str) -> Result<Option<String>, CognitiveError> {
        let node = self
            .tree
            .get_mut(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Node not found".to_string()))?;

        if node.untried_actions.is_empty() {
            return Ok(None);
        }

        // Pop a random untried action
        let action_idx = rand::thread_rng().gen_range(0..node.untried_actions.len());
        let action = node.untried_actions.remove(action_idx);

        // Apply action to get new state
        let parent_state = node.state.clone();
        let new_state = self.apply_action(&parent_state, &action).await?;

        // Create child node
        let child_id = format!("{}-{}", node_id, self.tree.len());
        let child_node = MCTSNode {
            visits: 0,
            total_reward: 0.0,
            children: HashMap::new(),
            parent: Some(node_id.to_string()),
            state: new_state,
            untried_actions: Self::get_possible_actions(&new_state, &self.spec),
            is_terminal: false, // Will be determined later
            applied_action: Some(action.clone()),
        };

        self.tree.insert(child_id.clone(), child_node);
        self.tree
            .get_mut(node_id)
            .unwrap()
            .children
            .insert(action, child_id.clone());

        Ok(Some(child_id))
    }

    /// Simulate from a node to estimate reward
    async fn simulate(&self, node_id: &str) -> Result<f64, CognitiveError> {
        let node = &self.tree[node_id];

        // Use actual performance analysis
        let reward = self
            .performance_analyzer
            .estimate_reward(&node.state)
            .await?;

        debug!(
            "Simulated reward for node {} (action: {:?}): {:.3}",
            node_id, node.applied_action, reward
        );

        Ok(reward)
    }

    /// Backpropagate reward up the tree
    #[inline]
    fn backpropagate(&mut self, mut node_id: String, reward: f64) {
        while let Some(node) = self.tree.get_mut(&node_id) {
            node.visits += 1;
            node.total_reward += reward;

            match &node.parent {
                Some(parent_id) => node_id = parent_id.clone(),
                None => break,
            }
        }
    }

    /// Run MCTS for specified iterations
    pub async fn run(&mut self, iterations: u64) -> Result<(), CognitiveError> {
        let mut join_set = JoinSet::new();
        let mut completed_iterations = 0;

        while completed_iterations < iterations {
            // Limit parallel tasks
            if join_set.len() >= self.max_parallel {
                if let Some(result) = join_set.join_next().await {
                    match result {
                        Ok(Ok((node_id, reward))) => {
                            self.backpropagate(node_id, reward);
                            completed_iterations += 1;

                            if completed_iterations % 100 == 0 {
                                info!(
                                    "MCTS progress: {}/{} iterations",
                                    completed_iterations, iterations
                                );
                            }
                        }
                        Ok(Err(e)) => error!("Simulation failed: {}", e),
                        Err(e) => error!("Task panicked: {}", e),
                    }
                }
            }

            // Selection
            let selected = self.select();

            // Expansion
            let node_to_simulate = match self.expand(&selected).await? {
                Some(child_id) => child_id,
                None => selected, // No expansion possible, simulate current node
            };

            // Clone necessary data for async simulation
            let node = self.tree[&node_to_simulate].clone();
            let performance_analyzer = Arc::clone(&self.performance_analyzer);

            // Spawn simulation task
            join_set.spawn(async move {
                let reward = performance_analyzer.estimate_reward(&node.state).await?;
                Ok((node_to_simulate, reward))
            });
        }

        // Wait for remaining tasks
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((node_id, reward))) => {
                    self.backpropagate(node_id, reward);
                }
                Ok(Err(e)) => error!("Final simulation failed: {}", e),
                Err(e) => error!("Final task panicked: {}", e),
            }
        }

        info!("MCTS completed {} iterations", iterations);
        Ok(())
    }

    /// Get the best modification found
    pub fn best_modification(&self) -> Option<CodeState> {
        let root = &self.tree[&self.root_id];

        // Find child with highest average reward
        root.children
            .values()
            .filter_map(|child_id| {
                let child = &self.tree[child_id];
                if child.visits > 0 {
                    Some((child, child.total_reward / child.visits as f64))
                } else {
                    None
                }
            })
            .max_by(|(_, a_score), (_, b_score)| {
                a_score
                    .partial_cmp(b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(child, _)| child.state.clone())
    }

    /// Get statistics about the search tree
    pub fn get_statistics(&self) -> TreeStatistics {
        let total_nodes = self.tree.len();
        let total_visits: u64 = self.tree.values().map(|n| n.visits).sum();
        let max_depth = self.calculate_max_depth();

        let best_path = self.get_best_path();
        let explored_actions: Vec<String> = self
            .tree
            .values()
            .filter_map(|n| n.applied_action.clone())
            .collect();

        TreeStatistics {
            total_nodes,
            total_visits,
            max_depth,
            best_path,
            explored_actions,
        }
    }

    fn calculate_max_depth(&self) -> usize {
        let mut max_depth = 0;

        for node in self.tree.values() {
            let mut depth = 0;
            let mut current = node.parent.as_ref();

            while let Some(parent_id) = current {
                depth += 1;
                current = self.tree.get(parent_id).and_then(|n| n.parent.as_ref());
            }

            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    fn get_best_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        let mut current_id = self.root_id.clone();

        loop {
            let node = &self.tree[&current_id];

            // Find best child
            if let Some((action, child_id)) = node
                .children
                .iter()
                .filter_map(|(action, child_id)| {
                    let child = &self.tree[child_id];
                    if child.visits > 0 {
                        Some((action, child_id, child.total_reward / child.visits as f64))
                    } else {
                        None
                    }
                })
                .max_by(|(_, _, a_score), (_, _, b_score)| {
                    a_score
                        .partial_cmp(b_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(action, child_id, _)| (action.clone(), child_id.clone()))
            {
                path.push(action);
                current_id = child_id;
            } else {
                break;
            }
        }

        path
    }

    /// Get possible actions for a state
    fn get_possible_actions(state: &CodeState, spec: &OptimizationSpec) -> Vec<String> {
        let mut actions = Vec::new();

        // Basic optimization actions
        actions.push("optimize_hot_paths".to_string());
        actions.push("reduce_allocations".to_string());
        actions.push("improve_cache_locality".to_string());
        actions.push("parallelize_independent_work".to_string());
        actions.push("inline_critical_functions".to_string());
        actions.push("batch_operations".to_string());
        actions.push("add_strategic_caching".to_string());
        actions.push("optimize_data_structures".to_string());
        actions.push("reduce_lock_contention".to_string());
        actions.push("enable_simd_operations".to_string());

        // Conditional actions based on current state
        if state.latency > spec.baseline_metrics.latency * 1.1 {
            actions.push("aggressive_latency_optimization".to_string());
        }

        if state.memory > spec.baseline_metrics.memory * 1.1 {
            actions.push("aggressive_memory_optimization".to_string());
        }

        if state.relevance < spec.baseline_metrics.relevance * 0.9 {
            actions.push("improve_algorithm_accuracy".to_string());
        }

        actions
    }

    /// Apply an action to a state using committee evaluation
    async fn apply_action(
        &self,
        state: &CodeState,
        action: &str,
    ) -> Result<CodeState, CognitiveError> {
        // Get impact factors from committee
        let factors = self
            .committee
            .evaluate_action(state, action, &self.spec, &self.user_objective)
            .await?;

        // Apply factors to current metrics
        let new_latency = state.latency * factors.latency_factor;
        let new_memory = state.memory * factors.memory_factor;
        let new_relevance = (state.relevance * factors.relevance_factor).min(100.0);

        // Validate against constraints
        let max_latency = self.spec.baseline_metrics.latency
            * (1.0 + self.spec.content_type.restrictions.max_latency_increase / 100.0);
        let max_memory = self.spec.baseline_metrics.memory
            * (1.0 + self.spec.content_type.restrictions.max_memory_increase / 100.0);
        let min_relevance = self.spec.baseline_metrics.relevance
            * (1.0
                + self
                    .spec
                    .content_type
                    .restrictions
                    .min_relevance_improvement
                    / 100.0);

        if new_latency > max_latency || new_memory > max_memory || new_relevance < min_relevance {
            return Err(CognitiveError::InvalidState(format!(
                "Action '{}' violates constraints: latency={:.2} (max={:.2}), memory={:.2} (max={:.2}), relevance={:.2} (min={:.2})",
                action,
                new_latency,
                max_latency,
                new_memory,
                max_memory,
                new_relevance,
                min_relevance
            )));
        }

        // Create new code that reflects the action
        let new_code = format!(
            "// Applied: {} (confidence: {:.2})\n// Impact: latency={:.2}x, memory={:.2}x, relevance={:.2}x\n{}",
            action,
            factors.confidence,
            factors.latency_factor,
            factors.memory_factor,
            factors.relevance_factor,
            state.code
        );

        Ok(CodeState {
            code: new_code,
            latency: new_latency,
            memory: new_memory,
            relevance: new_relevance,
        })
    }
}

/// Statistics about the MCTS tree
#[derive(Debug, Serialize)]
pub struct TreeStatistics {
    pub total_nodes: usize,
    pub total_visits: u64,
    pub max_depth: usize,
    pub best_path: Vec<String>,
    pub explored_actions: Vec<String>,
}

// Ensure Node is cloneable for async operations
impl Clone for MCTSNode {
    fn clone(&self) -> Self {
        Self {
            visits: self.visits,
            total_reward: self.total_reward,
            children: self.children.clone(),
            parent: self.parent.clone(),
            state: self.state.clone(),
            untried_actions: self.untried_actions.clone(),
            is_terminal: self.is_terminal,
            applied_action: self.applied_action.clone(),
        }
    }
}

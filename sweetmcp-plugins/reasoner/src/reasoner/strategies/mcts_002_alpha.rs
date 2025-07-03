use crate::state::StateManager;
use crate::strategies::base::{
    AsyncPath, BaseStrategy, ClearedSignal, Metric, MetricStream, Reasoning, Strategy,
};
use crate::strategies::mcts::MonteCarloTreeSearchStrategy;
use crate::types::{ReasoningRequest, ReasoningResponse, ThoughtNode};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
// Tracing is imported for error logging in case of future extensions
#[allow(unused_imports)]
use tracing;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyGuidedNode {
    #[serde(flatten)]
    pub base: ThoughtNode,
    pub visits: usize,
    #[serde(rename = "totalReward")]
    pub total_reward: f64,
    #[serde(rename = "untriedActions")]
    pub untried_actions: Option<Vec<String>>,
    #[serde(rename = "policyScore")]
    pub policy_score: f64, // Policy network prediction
    #[serde(rename = "valueEstimate")]
    pub value_estimate: f64, // Value network estimate
    #[serde(rename = "priorActionProbs")]
    pub prior_action_probs: HashMap<String, f64>, // Action probabilities
    pub puct: Option<f64>, // PUCT score for selection
    #[serde(rename = "actionHistory")]
    pub action_history: Option<Vec<String>>, // Track sequence of actions
    #[serde(rename = "noveltyScore")]
    pub novelty_score: Option<f64>, // Measure of thought novelty
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetrics {
    #[serde(rename = "averagePolicyScore")]
    pub average_policy_score: f64,
    #[serde(rename = "averageValueEstimate")]
    pub average_value_estimate: f64,
    #[serde(rename = "actionDistribution")]
    pub action_distribution: HashMap<String, usize>,
    #[serde(rename = "explorationStats")]
    pub exploration_stats: ExplorationStats,
    #[serde(rename = "convergenceMetrics")]
    pub convergence_metrics: ConvergenceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationStats {
    pub temperature: f64,
    #[serde(rename = "explorationRate")]
    pub exploration_rate: f64,
    #[serde(rename = "noveltyBonus")]
    pub novelty_bonus: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceMetrics {
    #[serde(rename = "policyEntropy")]
    pub policy_entropy: f64,
    #[serde(rename = "valueStability")]
    pub value_stability: f64,
}

pub struct MCTS002AlphaStrategy {
    base: BaseStrategy,
    temperature: f64,
    exploration_rate: Arc<Mutex<f64>>,
    learning_rate: f64,
    novelty_bonus: f64,
    policy_metrics: Arc<Mutex<PolicyMetrics>>,
    simulation_count: usize,
}

impl MCTS002AlphaStrategy {
    pub fn new(state_manager: Arc<StateManager>, num_simulations: Option<usize>) -> Self {
        let num_simulations = num_simulations.unwrap_or(crate::types::CONFIG.num_simulations);

        let policy_metrics = PolicyMetrics {
            average_policy_score: 0.0,
            average_value_estimate: 0.0,
            action_distribution: HashMap::new(),
            exploration_stats: ExplorationStats {
                temperature: 1.0,
                exploration_rate: 2.0_f64.sqrt(),
                novelty_bonus: 0.2,
            },
            convergence_metrics: ConvergenceMetrics {
                policy_entropy: 0.0,
                value_stability: 0.0,
            },
        };

        Self {
            base: BaseStrategy::new(state_manager),
            temperature: 1.0,
            exploration_rate: Arc::new(Mutex::new(2.0_f64.sqrt())),
            learning_rate: 0.1,
            novelty_bonus: 0.2,
            policy_metrics: Arc::new(Mutex::new(policy_metrics)),
            simulation_count: num_simulations,
        }
    }

    // Renamed for clarity, now uses hashing for better uniqueness representation
    pub fn get_thought_identifier(&self, thought: &str) -> String {
        // Use a hash of the thought content as a more robust identifier
        // than just the first few words.
        let mut hasher = DefaultHasher::new();
        thought.hash(&mut hasher);
        hasher.finish().to_string()
    }

    // Now async to use semantic coherence
    pub async fn calculate_policy_score(
        &self,
        node: &PolicyGuidedNode,
        parent: Option<&PolicyGuidedNode>,
    ) -> f64 {
        // Combine multiple policy factors
        let depth_factor = (-0.1 * node.base.depth as f64).exp();

        // Use async semantic coherence
        let parent_alignment = if let Some(p) = parent {
            match self
                .base
                .calculate_semantic_coherence(&p.base.thought, &node.base.thought)
                .await // Await the async call
            {
                Ok(score) => score,
                Err(_) => 0.5, // Default on error
            }
        } else {
            1.0 // No parent, max alignment
        };
        let novelty_bonus = node.novelty_score.unwrap_or(0.0); // Novelty is also heuristic

        // Weights are heuristic
        0.4 * depth_factor + 0.4 * parent_alignment + 0.2 * novelty_bonus
    }

    // Now async (though not strictly needed if policy_score already calculated coherence)
    pub async fn estimate_value(&self, node: &PolicyGuidedNode) -> f64 {
        // Combine immediate score with future potential
        let immediate_value = node.base.score; // Base score from evaluate_thought
        let depth_potential =
            1.0 - (node.base.depth as f64 / crate::types::CONFIG.max_depth as f64); // Heuristic potential
        let novelty_value = node.novelty_score.unwrap_or(0.0); // Heuristic novelty

        // Weights are heuristic
        0.5 * immediate_value + 0.3 * depth_potential + 0.2 * novelty_value
    }

    pub fn calculate_novelty(&self, node: &PolicyGuidedNode) -> f64 {
        // Measure thought novelty based on thought identifier history
        let thought_history = match &node.action_history { // Reusing action_history field for thought identifiers
            Some(history) => history,
            None => return 0.0, // No history, no novelty score
        };

        if thought_history.is_empty() {
            return 0.0;
        }

        let unique_thoughts = thought_history.iter().collect::<HashSet<_>>().len();
        let history_length = thought_history.len();
        // Avoid division by zero if history_length is somehow 0 despite check
        let uniqueness_ratio = if history_length > 0 {
             unique_thoughts as f64 / history_length as f64
        } else {
            0.0
        };

        // Combine with linguistic novelty
        let re = regex::Regex::new(r"[.!?;]|therefore|because|if|then")
            .expect("Failed to compile regex pattern");
        let complexity_score = re.find_iter(&node.base.thought).count() as f64 / 10.0; // Simple complexity heuristic

        // Weights are heuristic
        0.7 * uniqueness_ratio + 0.3 * complexity_score
    }

    // Simple word overlap coherence (used as placeholder)
    #[allow(dead_code)]
    fn thought_coherence(&self, thought1: &str, thought2: &str) -> f64 {
        let words1: HashSet<String> = thought1
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let words2: HashSet<String> = thought2
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        intersection as f64 / union as f64
    }

    async fn run_policy_guided_search(
        &self,
        node: PolicyGuidedNode,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..self.simulation_count {
            let selected_node = self.select_with_puct(node.clone()).await?;
            let expanded_node = self.expand_with_policy(selected_node).await?;
            let reward = self.simulate_with_value_guidance(&expanded_node).await?;
            let expanded_node_clone = expanded_node.clone(); // Clone before moving
            self.backpropagate_with_policy_update(expanded_node, reward)
                .await?;

            // Adapt exploration rate
            self.adapt_exploration_rate(&expanded_node_clone).await;
        }

        Ok(())
    }

    async fn select_with_puct(
        &self,
        root: PolicyGuidedNode,
    ) -> Result<PolicyGuidedNode, Box<dyn std::error::Error + Send + Sync>> {
        let mut node = root;

        while !node.base.children.is_empty() {
            let mut children = Vec::new();
            for id in &node.base.children {
                if let Ok(Some(child_node)) = self.base.get_node(id).await {
                    if let Ok(policy_child) = self.thought_to_policy(child_node).await {
                        children.push(policy_child);
                    }
                }
            }

            if children.is_empty() {
                break;
            }

            node = self.select_best_puct_child(children).await;
        }

        Ok(node)
    }

    async fn select_best_puct_child(&self, nodes: Vec<PolicyGuidedNode>) -> PolicyGuidedNode {
        let total_visits: usize = nodes.iter().map(|node| node.visits).sum();
        let exploration_rate = *self.exploration_rate.lock().await;

        nodes
            .into_iter()
            .fold(None, |best: Option<PolicyGuidedNode>, mut node| {
                let exploitation = node.value_estimate;
                let exploration = ((total_visits as f64).max(1.0).ln() / node.visits.max(1) as f64).sqrt(); // Avoid ln(0) or div by zero
                let policy_term = node.policy_score * exploration_rate;
                let novelty_bonus = node.novelty_score.unwrap_or(0.0) * self.novelty_bonus;

                let puct = exploitation + exploration * policy_term + novelty_bonus;
                node.puct = Some(puct);

                match best {
                    None => Some(node),
                    Some(best_node) => {
                        if puct > best_node.puct.unwrap_or(0.0) {
                            Some(node)
                        } else {
                            Some(best_node)
                        }
                    }
                }
            })
            .unwrap_or_else(|| {
                // If no nodes provided, return a default MCTSNode
                MCTSNode {
                    base: ThoughtNode {
                        id: "default".to_string(),
                        thought: "Default selection".to_string(),
                        depth: 0,
                        score: 0.0,
                        children: vec![],
                        parent_id: None,
                        is_complete: false,
                    },
                    visits: 1,
                    total_reward: 0.0,
                    untried_actions: Some(vec![]),
                }
            })
    }

    async fn expand_with_policy(
        &self,
        node: PolicyGuidedNode,
    ) -> Result<PolicyGuidedNode, Box<dyn std::error::Error + Send + Sync>> {
        if node.base.is_complete {
            return Ok(node);
        }

        let new_node_id = Uuid::new_v4().to_string();
        let parent_prefix = node.base.thought.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
        // Simple heuristic continuation
        let new_thought = format!(
            "Based on '{}...', the policy suggests exploring...",
             parent_prefix
        );

        let base_node = ThoughtNode {
            id: new_node_id.clone(),
            thought: new_thought.clone(),
            depth: node.base.depth + 1,
            score: 0.0, // Will be evaluated later
            children: vec![],
            parent_id: Some(node.base.id.clone()),
            is_complete: false,
        };

        // Update history with the new thought's identifier
        let thought_identifier = self.get_thought_identifier(&new_thought);
        let action_history = match &node.action_history { // Reusing field name
            Some(history) => {
                let mut new_history = history.clone();
                new_history.push(thought_identifier);
                Some(new_history)
            }
            None => Some(vec![thought_identifier]),
        };

        let mut new_node = PolicyGuidedNode {
            base: base_node.clone(),
            visits: 1,
            total_reward: 0.0,
            untried_actions: Some(vec![]),
            policy_score: 0.0,
            value_estimate: 0.0,
            prior_action_probs: HashMap::new(),
            puct: None,
            action_history,
            novelty_score: None,
        };

        new_node.novelty_score = Some(self.calculate_novelty(&new_node));
        // Await the async calculation
        new_node.policy_score = self.calculate_policy_score(&new_node, Some(&node)).await;
        new_node.base.score = self.base.evaluate_thought(&new_node.base, Some(&node.base));
        // Await the async calculation
        new_node.value_estimate = self.estimate_value(&new_node).await;

        // Save the base node
        if let Err(e) = self.base.save_node(base_node).await {
            return Err(Box::new(e));
        }

        Ok(new_node)
    }

    async fn simulate_with_value_guidance(
        &self,
        node: &PolicyGuidedNode,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut current = node.clone();
        let mut total_reward = 0.0;
        let mut depth = 0;

        while !current.base.is_complete && depth < crate::types::CONFIG.max_depth {
            // Await the async value estimate
            let reward = self.estimate_value(&current).await;
            total_reward += reward;

            // Expansion uses heuristic generation
            if let Ok(expanded) = self.expand_with_policy(current).await {
                current = expanded;
                depth += 1;
            } else {
                break;
            }
        }

        if depth == 0 {
            return Ok(node.value_estimate);
        }

        Ok(total_reward / depth as f64)
    }

    async fn backpropagate_with_policy_update(
        &self,
        node: PolicyGuidedNode,
        reward: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current_opt = Some(node);

        while let Some(mut current) = current_opt {
            // Update node stats
            current.visits += 1;
            current.total_reward += reward;

            // Update value estimate with temporal difference
            // Await the async value estimate but not using it directly (just for future expansion)
            let _current_value_estimate = self.estimate_value(&current).await; // Get current estimate
            let new_value = (1.0 - self.learning_rate) * current.value_estimate + self.learning_rate * reward;
            current.value_estimate = new_value;

            // Update action probabilities
            // Update action probabilities (using thought identifier as key)
            if let Some(parent_id) = &current.base.parent_id {
                if let Ok(Some(parent_node)) = self.base.get_node(parent_id).await {
                    if let Ok(mut parent) = self.thought_to_policy(parent_node).await {
                        let thought_key = self.get_thought_identifier(&current.base.thought);
                        let current_prob =
                            *parent.prior_action_probs.get(&thought_key).unwrap_or(&0.0);
                        // Simple update rule, could be more sophisticated (e.g., based on PUCT)
                        let new_prob = current_prob + self.learning_rate * (reward - current_prob);
                        parent.prior_action_probs.insert(thought_key, new_prob);

                        // Save updated parent
                        if let Err(e) = self.base.save_node(parent.base.clone()).await {
                            return Err(crate::strategies::base::ReasoningError::Other(
                                format!("Failed to save parent node: {}", e),
                            ).into());
                        }
                    }
                }
            }

            // Save updated node
            if let Err(e) = self.base.save_node(current.base.clone()).await {
                return Err(crate::strategies::base::ReasoningError::Other(
                    format!("Failed to save current node: {}", e),
                ).into());
            }

            // Move to parent
            if let Some(parent_id) = &current.base.parent_id {
                if let Ok(Some(parent_node)) = self.base.get_node(parent_id).await {
                    current_opt = match self.thought_to_policy(parent_node).await {
                        Ok(parent) => Some(parent),
                        Err(_) => None,
                    };
                } else {
                    current_opt = None;
                }
            } else {
                current_opt = None;
            }
        }

        Ok(())
    }

    async fn adapt_exploration_rate(&self, node: &PolicyGuidedNode) {
        let success_rate = node.total_reward / node.visits as f64;
        let target_rate = 0.6;

        let mut exploration_rate = self.exploration_rate.lock().await;
        if success_rate > target_rate {
            // Reduce exploration when doing well
            *exploration_rate = (0.5f64).max(*exploration_rate * 0.95);
        } else {
            // Increase exploration when results are poor
            *exploration_rate = (2.0f64).min(*exploration_rate / 0.95);
        }
    }

    // Extract an action-like identifier from a thought
    fn extract_action(&self, thought: &str) -> String {
        // Simple approach: Use first 3 words to create an action identifier
        thought
            .split_whitespace()
            .take(3)
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase()
    }

    async fn update_policy_metrics(
        &self,
        node: &PolicyGuidedNode,
        parent: &PolicyGuidedNode,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics = self.policy_metrics.lock().await;

        // Update running averages
        metrics.average_policy_score = (metrics.average_policy_score + node.policy_score) / 2.0;
        metrics.average_value_estimate =
            (metrics.average_value_estimate + node.value_estimate) / 2.0;

        // Update action distribution
        let action = self.extract_action(&node.base.thought);
        let count = metrics.action_distribution.entry(action).or_insert(0);
        *count += 1;

        // Update exploration stats
        let exploration_rate = *self.exploration_rate.lock().await; // Read the current rate
        metrics.exploration_stats = ExplorationStats {
            temperature: self.temperature,
            exploration_rate,
            novelty_bonus: self.novelty_bonus,
        };

        // Calculate policy entropy and value stability
        let probs: Vec<f64> = parent.prior_action_probs.values().copied().collect();
        metrics.convergence_metrics = ConvergenceMetrics {
            policy_entropy: self.calculate_entropy(&probs),
            value_stability: (node.value_estimate - parent.value_estimate).abs(),
        };

        Ok(())
    }

    fn calculate_entropy(&self, probs: &[f64]) -> f64 {
        let sum: f64 = probs.iter().sum();
        if sum == 0.0 {
            return 0.0;
        }

        -probs
            .iter()
            .map(|&p| {
                let norm = p / sum;
                if norm <= 0.0 {
                    0.0
                } else {
                    norm * (norm + 1e-10).log2()
                }
            })
            .sum::<f64>()
    }

    // Now async
    async fn calculate_policy_enhanced_score(&self, path: &[ThoughtNode]) -> f64 {
        if path.is_empty() {
            return 0.0;
        }

        let mut total_score = 0.0;

        for node in path {
            // Attempt to get policy info (awaiting the conversion)
            // Note: This still uses the *initial* default scores unless the node
            // was previously processed and its PolicyGuidedNode state persisted.
            if let Ok(policy_node) = self.thought_to_policy(node.clone()).await {
                let base_score = node.score;
                // Using initial default scores from thought_to_policy
                let policy_bonus = policy_node.policy_score; // Default 0.5
                let value_bonus = policy_node.value_estimate; // Default 0.5
                let novelty_bonus = policy_node.novelty_score.unwrap_or(0.0) * self.novelty_bonus; // Default 0.5 * bonus

                // Combine base score with placeholder bonuses
                total_score += (base_score + policy_bonus + value_bonus + novelty_bonus) / 4.0;
            } else {
                // Fallback to just the base score if conversion fails (shouldn't happen often)
                total_score += node.score;
            }
        }

        total_score / path.len() as f64
    }

    pub async fn thought_to_policy(
        &self,
        node: ThoughtNode,
    ) -> Result<PolicyGuidedNode, Box<dyn std::error::Error + Send + Sync>> {
        // Convert ThoughtNode to PolicyGuidedNode with default initial values
        Ok(PolicyGuidedNode {
            base: node,
            visits: 1,
            total_reward: 0.0,
            untried_actions: Some(vec![]),
            policy_score: 0.5,   // Default value
            value_estimate: 0.5, // Default value
            prior_action_probs: HashMap::new(),
            puct: None,
            action_history: Some(vec![]),
            novelty_score: Some(0.5), // Default value
        })
    }
}

// Add Clone implementation for MCTS002AlphaStrategy
impl Clone for MCTS002AlphaStrategy {
    fn clone(&self) -> Self {
        Self {
            base: BaseStrategy::new(Arc::clone(&self.base.state_manager)),
            temperature: self.temperature,
            exploration_rate: Arc::clone(&self.exploration_rate),
            learning_rate: self.learning_rate,
            novelty_bonus: self.novelty_bonus,
            policy_metrics: Arc::clone(&self.policy_metrics),
            simulation_count: self.simulation_count,
        }
    }
}

impl Strategy for MCTS002AlphaStrategy {
    fn process_thought(&self, request: ReasoningRequest) -> Reasoning {
        let (tx, rx) = mpsc::channel(1);
        let self_clone = self.clone();

        tokio::spawn(async move {
            // Use MonteCarloTreeSearchStrategy as a base
            let mcts = MonteCarloTreeSearchStrategy::new(
                Arc::clone(&self_clone.base.state_manager),
                Some(self_clone.simulation_count),
            );

            // Get base MCTS response - manually await the stream to get first item
            let mut mcts_reasoning = mcts.process_thought(request.clone());
            let base_response = match mcts_reasoning.next().await {
                Some(Ok(response)) => response,
                _ => {
                    let _ = tx
                        .send(Err(crate::strategies::base::ReasoningError::Other(
                            "Failed to get base MCTS response".into(),
                        )))
                        .await;
                    return;
                }
            };

            let node_id = Uuid::new_v4().to_string();
            let parent_node = match &request.parent_id {
                Some(parent_id) => {
                    if let Ok(Some(node)) = self_clone.base.get_node(parent_id).await {
                        match self_clone.thought_to_policy(node).await {
                            Ok(policy_node) => Some(policy_node),
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                }
                None => None,
            };

            let mut base_node = ThoughtNode {
                id: node_id.clone(),
                thought: request.thought.clone(),
                depth: request.thought_number - 1,
                score: 0.0,
                children: vec![],
                parent_id: request.parent_id.clone(),
                is_complete: !request.next_thought_needed,
            };

            // Create thought identifier history from parent or initialize new one
            let thought_identifier = self_clone.get_thought_identifier(&request.thought);
            let action_history = match &parent_node { // Reusing field name
                Some(parent) => {
                    let mut history = parent.action_history.clone().unwrap_or_default();
                    history.push(thought_identifier);
                    Some(history)
                }
                None => Some(vec![thought_identifier]),
            };

            // Initialize PolicyGuidedNode
            let mut node = PolicyGuidedNode {
                base: base_node.clone(),
                visits: 1,
                total_reward: 0.0,
                untried_actions: Some(vec![]),
                policy_score: 0.0,
                value_estimate: 0.0,
                prior_action_probs: HashMap::new(),
                puct: None,
                action_history,
                novelty_score: None,
            };

            // Initialize node with policy guidance
            node.base.score = self_clone
                .base
                .evaluate_thought(&node.base, parent_node.as_ref().map(|p| &p.base));
            node.visits = 1;
            node.total_reward = node.base.score;
            // Await async calculations
            node.policy_score = self_clone.calculate_policy_score(&node, parent_node.as_ref()).await;
            node.value_estimate = self_clone.estimate_value(&node).await;
            node.novelty_score = Some(self_clone.calculate_novelty(&node));
            base_node.score = node.base.score;

            // Save the node
            if let Err(e) = self_clone.base.save_node(base_node.clone()).await {
                let _ = tx.send(Err(crate::strategies::base::ReasoningError::Other(
                    format!("Failed to save base node: {}", e),
                ))).await;
                return;
            }

            // Update parent if exists
            if let Some(mut parent) = parent_node {
                parent.base.children.push(node.base.id.clone());
                if let Err(e) = self_clone.base.save_node(parent.base.clone()).await {
                    let _ = tx.send(Err(crate::strategies::base::ReasoningError::Other(
                        format!("Failed to save parent node: {}", e),
                    ))).await;
                    return;
                }
                let _ = self_clone.update_policy_metrics(&node, &parent).await;
            }

            // Run policy-guided search
            if !node.base.is_complete {
                let _ = self_clone.run_policy_guided_search(node.clone()).await;
            }

            // Calculate enhanced path statistics
            let current_path = self_clone.base.state_manager.get_path(&node_id).await;
            let enhanced_score = self_clone.calculate_policy_enhanced_score(&current_path).await;

            let response = ReasoningResponse {
                node_id: base_response.node_id,
                thought: base_response.thought,
                score: enhanced_score,
                depth: base_response.depth,
                is_complete: base_response.is_complete,
                next_thought_needed: base_response.next_thought_needed,
                possible_paths: base_response.possible_paths,
                best_score: Some(base_response.best_score.unwrap_or(0.0).max(enhanced_score)),
                strategy_used: None, // Will be set by reasoner
            };

            let _ = tx.send(Ok(response)).await;
        });

        Reasoning::new(rx)
    }

    fn get_best_path(&self) -> AsyncPath {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Delegate to MonteCarloTreeSearchStrategy
            let mcts = MonteCarloTreeSearchStrategy::new(
                Arc::clone(&self_clone.base.state_manager),
                Some(self_clone.simulation_count),
            );

            // Use the async path from MCTS but await it to get the result
            let path = mcts.get_best_path().await;
            let _ = tx.send(path);
        });

        AsyncPath::new(rx)
    }

    fn get_metrics(&self) -> MetricStream {
        let self_clone = self.clone();
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let base_metrics = match self_clone.base.get_base_metrics().await {
                Ok(metrics) => metrics,
                _ => Metric {
                    name: String::from("MCTS-002-Alpha"),
                    nodes_explored: 0,
                    average_score: 0.0,
                    max_depth: 0,
                    active: None,
                    extra: Default::default(),
                },
            };

            let mut metrics = base_metrics;
            let policy_metrics = self_clone.policy_metrics.lock().await.clone();
            let exploration_rate = *self_clone.exploration_rate.lock().await;

            let policy_stats = serde_json::json!({
                "averages": {
                    "policy_score": policy_metrics.average_policy_score,
                    "value_estimate": policy_metrics.average_value_estimate,
                },
                "temperature": self_clone.temperature,
                "exploration_rate": exploration_rate,
                "learning_rate": self_clone.learning_rate,
                "novelty_bonus": self_clone.novelty_bonus,
                "policy_entropy": policy_metrics.convergence_metrics.policy_entropy,
                "value_stability": policy_metrics.convergence_metrics.value_stability,
            });

            metrics.name = "MCTS-002-Alpha (Policy Enhanced)".to_string();
            metrics
                .extra
                .insert("temperature".to_string(), self_clone.temperature.into());
            metrics
                .extra
                .insert("exploration_rate".to_string(), exploration_rate.into());
            metrics
                .extra
                .insert("learning_rate".to_string(), self_clone.learning_rate.into());
            metrics
                .extra
                .insert("policy_stats".to_string(), policy_stats);

            let _ = tx.send(Ok(metrics)).await;
        });

        MetricStream::new(rx)
    }

    fn clear(&self) -> ClearedSignal {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Reset exploration rate and policy metrics
            let mut exploration_rate = self_clone.exploration_rate.lock().await;
            *exploration_rate = 2.0_f64.sqrt();

            let mut metrics = self_clone.policy_metrics.lock().await;
            *metrics = PolicyMetrics {
                average_policy_score: 0.0,
                average_value_estimate: 0.0,
                action_distribution: HashMap::new(),
                exploration_stats: ExplorationStats {
                    temperature: self_clone.temperature,
                    exploration_rate: *exploration_rate,
                    novelty_bonus: self_clone.novelty_bonus,
                },
                convergence_metrics: ConvergenceMetrics {
                    policy_entropy: 0.0,
                    value_stability: 0.0,
                },
            };

            let _ = tx.send(Ok(()));
        });

        ClearedSignal::new(rx)
    }
}

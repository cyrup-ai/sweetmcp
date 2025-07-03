use crate::state::StateManager;
use crate::strategies::base::{
    AsyncPath, BaseStrategy, ClearedSignal, Metric, MetricStream, Reasoning, Strategy,
};
use crate::strategies::experiments::mcts_002_alpha::{MCTS002AlphaStrategy, PolicyGuidedNode};
use crate::types::{ReasoningRequest, ReasoningResponse, ThoughtNode};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing;
use uuid::Uuid;

// Queue implementation for bidirectional search
struct Queue<T> {
    items: VecDeque<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
        }
    }

    fn enqueue(&mut self, item: T) {
        self.items.push_back(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn size(&self) -> usize {
        self.items.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidirectionalPolicyNode {
    #[serde(flatten)]
    pub base: PolicyGuidedNode,
    pub g: f64,                 // A* cost from start
    pub h: f64,                 // A* heuristic to goal
    pub f: f64,                 // A* f = g + h
    pub parent: Option<String>, // For path reconstruction
    // Match TypeScript exactly - 'forward' | 'backward' only
    #[serde(rename = "direction")]
    pub direction: Option<String>, // Must be "forward" or "backward" only
    #[serde(rename = "searchDepth")]
    pub search_depth: Option<usize>, // Depth within the search
    #[serde(rename = "meetingPoint")]
    pub meeting_point: Option<bool>, // If true, node is a meeting point
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidirectionalStats {
    #[serde(rename = "forwardExplorationRate")]
    pub forward_exploration_rate: f64,
    #[serde(rename = "backwardExplorationRate")]
    pub backward_exploration_rate: f64,
    #[serde(rename = "meetingPoints")]
    pub meeting_points: usize,
    #[serde(rename = "pathQuality")]
    pub path_quality: f64,
}

pub struct MCTS002AltAlphaStrategy {
    base: BaseStrategy,
    inner_strategy: MCTS002AlphaStrategy,
    start_node: Arc<Mutex<Option<BidirectionalPolicyNode>>>,
    goal_node: Arc<Mutex<Option<BidirectionalPolicyNode>>>,
    bidirectional_stats: Arc<Mutex<BidirectionalStats>>,
    #[allow(dead_code)]
    simulation_count: usize,
}

impl MCTS002AltAlphaStrategy {
    pub fn new(state_manager: Arc<StateManager>, num_simulations: Option<usize>) -> Self {
        let num_simulations = num_simulations.unwrap_or(crate::types::CONFIG.num_simulations);

        let bidirectional_stats = BidirectionalStats {
            forward_exploration_rate: 2.0_f64.sqrt(),
            backward_exploration_rate: 2.0_f64.sqrt(),
            meeting_points: 0,
            path_quality: 0.0,
        };

        Self {
            base: BaseStrategy::new(Arc::clone(&state_manager)),
            inner_strategy: MCTS002AlphaStrategy::new(
                Arc::clone(&state_manager),
                Some(num_simulations),
            ),
            start_node: Arc::new(Mutex::new(None)),
            goal_node: Arc::new(Mutex::new(None)),
            bidirectional_stats: Arc::new(Mutex::new(bidirectional_stats)),
            simulation_count: num_simulations,
        }
    }

    fn get_action_key(&self, thought: &str) -> String {
        // Same as the extract_action method in MCTS002AlphaStrategy
        thought
            .split_whitespace()
            .take(3)
            .collect::<Vec<&str>>()
            .join("_")
            .to_lowercase()
    }

    #[allow(dead_code)]
    async fn create_bidirectional_node(
        &self,
        thought_node: ThoughtNode,
        direction: &str,
        search_depth: usize,
        parent_g: f64,
    ) -> Result<BidirectionalPolicyNode, Box<dyn std::error::Error + Send + Sync>> {
        // Convert to PolicyGuidedNode first
        let policy_node = self.inner_strategy.thought_to_policy(thought_node).await?;

        // Calculate value estimate directly (same as inner_strategy.estimate_value)
        // This avoids dealing with the async Result return type in estimate_value
        let immediate_value = policy_node.base.score;
        let depth_potential = 1.0 - (policy_node.base.depth as f64 / crate::types::CONFIG.max_depth as f64);
        let novelty_value = policy_node.novelty_score.unwrap_or(0.0);
        
        // Same weights as in the original function
        let value_estimate = 0.5 * immediate_value + 0.3 * depth_potential + 0.2 * novelty_value;

        Ok(BidirectionalPolicyNode {
            base: policy_node,
            g: parent_g + 1.0,
            h: 1.0 - value_estimate, // Heuristic is inverse of value
            f: parent_g + 1.0 + (1.0 - value_estimate),
            parent: None,
            direction: Some(direction.to_string()),
            search_depth: Some(search_depth),
            meeting_point: None,
        })
    }

    async fn search_level(
        &self,
        queue: &mut Queue<BidirectionalPolicyNode>,
        visited: &mut HashMap<String, BidirectionalPolicyNode>,
        other_visited: &HashMap<String, BidirectionalPolicyNode>,
        direction: &str,
    ) -> Option<BidirectionalPolicyNode> {
        let level_size = queue.size();

        for _ in 0..level_size {
            if let Some(mut current) = queue.dequeue() {
                // Check if we've found a meeting point
                if other_visited.contains_key(&current.base.base.id) {
                    current.meeting_point = Some(true);
                    let mut stats = self.bidirectional_stats.lock().await;
                    stats.meeting_points += 1;

                    // Save the updated node
                    if let Err(_) = self.base.save_node(current.base.base.clone()).await {
                        return Some(current);  // Return meeting point despite error, just log it
                    }

                    return Some(current);
                }

                // Get neighbors based on direction and policy scores
                let mut neighbors = Vec::new();
                if direction == "forward" {
                    // Forward direction: get children
                    for id in &current.base.base.children {
                        if let Ok(Some(child_node)) = self.base.get_node(id).await {
                            if let Ok(policy_child) =
                                self.inner_strategy.thought_to_policy(child_node).await
                            {
                                let search_depth = current.search_depth.unwrap_or(0) + 1;
                                let parent_g = current.g;

                                let mut bi_child = BidirectionalPolicyNode {
                                    base: policy_child,
                                    g: parent_g + 1.0,
                                    h: 0.0, // Will calculate below
                                    f: 0.0, // Will calculate below
                                    parent: Some(current.base.base.id.clone()),
                                    direction: Some(direction.to_string()),
                                    search_depth: Some(search_depth),
                                    meeting_point: None,
                                };

                                // Calculate h and f
                                bi_child.h = 1.0 - bi_child.base.value_estimate;
                                bi_child.f = bi_child.g + bi_child.h;

                                neighbors.push(bi_child);
                            }
                        }
                    }
                } else {
                    // Backward direction: get parent
                    if let Some(parent_id) = &current.base.base.parent_id {
                        if let Ok(Some(parent_node)) = self.base.get_node(parent_id).await {
                            if let Ok(policy_parent) =
                                self.inner_strategy.thought_to_policy(parent_node).await
                            {
                                let search_depth = current.search_depth.unwrap_or(0) + 1;
                                let parent_g = current.g;

                                let mut bi_parent = BidirectionalPolicyNode {
                                    base: policy_parent,
                                    g: parent_g + 1.0,
                                    h: 0.0, // Will calculate below
                                    f: 0.0, // Will calculate below
                                    parent: Some(current.base.base.id.clone()),
                                    direction: Some(direction.to_string()),
                                    search_depth: Some(search_depth),
                                    meeting_point: None,
                                };

                                // Calculate h and f
                                bi_parent.h = 1.0 - bi_parent.base.value_estimate;
                                bi_parent.f = bi_parent.g + bi_parent.h;

                                neighbors.push(bi_parent);
                            }
                        }
                    }
                }

                // Sort neighbors by policy score
                neighbors.sort_by(|a, b| {
                    b.base
                        .policy_score
                        .partial_cmp(&a.base.policy_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for neighbor in neighbors {
                    if !visited.contains_key(&neighbor.base.base.id) {
                        visited.insert(neighbor.base.base.id.clone(), neighbor.clone());

                        // Save updated node
                        if let Err(e) = self.base.save_node(neighbor.base.base.clone()).await {
                            tracing::error!("Error saving neighbor node: {}", e);
                            continue;  // Skip this neighbor if we can't save it
                        }

                        queue.enqueue(neighbor);
                    }
                }
            }
        }

        None
    }

    async fn bidirectional_search(
        &self,
        start: BidirectionalPolicyNode,
        goal: BidirectionalPolicyNode,
    ) -> Result<Vec<BidirectionalPolicyNode>, Box<dyn std::error::Error + Send + Sync>> {
        let mut forward_queue = Queue::new();
        let mut backward_queue = Queue::new();
        let mut forward_visited = HashMap::new();
        let mut backward_visited = HashMap::new();

        forward_queue.enqueue(start.clone());
        backward_queue.enqueue(goal.clone());
        forward_visited.insert(start.base.base.id.clone(), start);
        backward_visited.insert(goal.base.base.id.clone(), goal);

        while !forward_queue.is_empty() && !backward_queue.is_empty() {
            // Search from both directions with policy guidance
            if let Some(meeting_point) = self
                .search_level(
                    &mut forward_queue,
                    &mut forward_visited,
                    &backward_visited,
                    "forward",
                )
                .await
            {
                let path =
                    self.reconstruct_path(meeting_point, &forward_visited, &backward_visited);
                self.update_bidirectional_stats(&path).await;
                return Ok(path);
            }

            if let Some(back_meeting_point) = self
                .search_level(
                    &mut backward_queue,
                    &mut backward_visited,
                    &forward_visited,
                    "backward",
                )
                .await
            {
                let path =
                    self.reconstruct_path(back_meeting_point, &forward_visited, &backward_visited);
                self.update_bidirectional_stats(&path).await;
                return Ok(path);
            }

            // Adapt exploration rates based on progress
            self.adapt_bidirectional_exploration(&forward_visited, &backward_visited)
                .await;
        }

        Ok(vec![])
    }

    fn reconstruct_path(
        &self,
        meeting_point: BidirectionalPolicyNode,
        forward_visited: &HashMap<String, BidirectionalPolicyNode>,
        backward_visited: &HashMap<String, BidirectionalPolicyNode>,
    ) -> Vec<BidirectionalPolicyNode> {
        let mut path = vec![meeting_point.clone()];

        // Reconstruct forward path
        let mut current = meeting_point.clone();
        while let Some(parent_id) = &current.parent {
            if let Some(parent) = forward_visited.get(parent_id) {
                path.insert(0, parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }

        // Reconstruct backward path
        current = meeting_point;
        while let Some(parent_id) = &current.parent {
            if let Some(parent) = backward_visited.get(parent_id) {
                path.push(parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }

        path
    }

    async fn update_path_with_policy_guidance(
        &self,
        path: &[BidirectionalPolicyNode],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path_bonus = 0.2;

        for node in path {
            let mut updated_node = node.clone();

            // Boost both policy and value estimates for nodes along the path
            updated_node.base.policy_score += path_bonus;
            updated_node.base.value_estimate = (updated_node.base.value_estimate + 1.0) / 2.0;

            // Update action probabilities with path information
            if let Some(parent_id) = &updated_node.base.base.parent_id {
                if let Ok(Some(parent_node)) = self.base.get_node(parent_id).await {
                    if let Ok(mut policy_parent) = self
                        .inner_strategy
                        .thought_to_policy(parent_node.clone())
                        .await
                    {
                        let action_key = self.get_action_key(&updated_node.base.base.thought);
                        let current_prob = *policy_parent
                            .prior_action_probs
                            .get(&action_key)
                            .unwrap_or(&0.0);
                        let new_prob = current_prob.max(0.8); // Strong preference for path actions
                        policy_parent
                            .prior_action_probs
                            .insert(action_key, new_prob);

                        // Save updated parent node
                        let updated_parent = policy_parent.base.clone();
                        if let Err(e) = self.base.save_node(updated_parent).await {
                            tracing::error!("Error saving updated parent: {}", e);
                        }
                    }
                }
            }

            // Save updated node
            let base_node = updated_node.base.base.clone();
            if let Err(e) = self.base.save_node(base_node).await {
                tracing::error!("Error saving base node: {}", e);
            }
        }

        // Update path quality metric
        if !path.is_empty() {
            let path_quality = path
                .iter()
                .map(|node| node.base.policy_score + node.base.value_estimate)
                .sum::<f64>()
                / (path.len() as f64 * 2.0);

            let mut stats = self.bidirectional_stats.lock().await;
            stats.path_quality = path_quality;
        }

        Ok(())
    }

    async fn adapt_bidirectional_exploration(
        &self,
        forward_visited: &HashMap<String, BidirectionalPolicyNode>,
        backward_visited: &HashMap<String, BidirectionalPolicyNode>,
    ) {
        // Skip if either map is empty
        if forward_visited.is_empty() || backward_visited.is_empty() {
            return;
        }

        // Adjust exploration rates based on search progress
        let forward_progress = forward_visited
            .values()
            .map(|node| node.base.policy_score)
            .sum::<f64>()
            / forward_visited.len() as f64;

        let backward_progress = backward_visited
            .values()
            .map(|node| node.base.policy_score)
            .sum::<f64>()
            / backward_visited.len() as f64;

        // Increase exploration in the direction making less progress
        let mut stats = self.bidirectional_stats.lock().await;
        if forward_progress > backward_progress {
            stats.backward_exploration_rate *= 1.05;
            stats.forward_exploration_rate *= 0.95;
        } else {
            stats.forward_exploration_rate *= 1.05;
            stats.backward_exploration_rate *= 0.95;
        }
    }

    async fn update_bidirectional_stats(&self, path: &[BidirectionalPolicyNode]) {
        if path.is_empty() {
            return;
        }

        let forward_nodes: Vec<&BidirectionalPolicyNode> = path
            .iter()
            .filter(|n| n.direction.as_deref() == Some("forward"))
            .collect();

        let backward_nodes: Vec<&BidirectionalPolicyNode> = path
            .iter()
            .filter(|n| n.direction.as_deref() == Some("backward"))
            .collect();

        // Skip if either direction is missing
        if forward_nodes.is_empty() || backward_nodes.is_empty() {
            return;
        }

        // Update exploration rates based on path composition
        let forward_quality = forward_nodes
            .iter()
            .map(|n| n.base.policy_score)
            .sum::<f64>()
            / forward_nodes.len() as f64;

        let backward_quality = backward_nodes
            .iter()
            .map(|n| n.base.policy_score)
            .sum::<f64>()
            / backward_nodes.len() as f64;

        let mut stats = self.bidirectional_stats.lock().await;
        stats.path_quality = (forward_quality + backward_quality) / 2.0;
    }

    fn calculate_bidirectional_policy_score(
        &self,
        path: &[ThoughtNode],
        stats: &BidirectionalStats,
    ) -> f64 {
        if path.is_empty() {
            return 0.0; // No path, no score
        }

        let mut total_score = 0.0;

        for node in path {
            // Use the node's evaluated score
            let node_score = node.score;

            // Add a small bonus based on estimated direction exploration rate
            let direction_bonus = if node.depth % 2 == 0 {
                stats.forward_exploration_rate * 0.1
            } else {
                stats.backward_exploration_rate * 0.1
            };
            total_score += node_score + direction_bonus;
        }

        total_score / path.len() as f64
    }
}

// Add Clone implementation for MCTS002AltAlphaStrategy
impl Clone for MCTS002AltAlphaStrategy {
    fn clone(&self) -> Self {
        Self {
            base: BaseStrategy::new(Arc::clone(&self.base.state_manager)),
            inner_strategy: self.inner_strategy.clone(),
            start_node: Arc::clone(&self.start_node),
            goal_node: Arc::clone(&self.goal_node),
            bidirectional_stats: Arc::clone(&self.bidirectional_stats),
            simulation_count: self.simulation_count,
        }
    }
}

impl Strategy for MCTS002AltAlphaStrategy {
    fn process_thought(&self, request: ReasoningRequest) -> Reasoning {
        let (tx, rx) = mpsc::channel(1);
        let self_clone = self.clone();

        tokio::spawn(async move {
            // First get the base response from MCTS002Alpha
            let mut mcts_reasoning = self_clone.inner_strategy.process_thought(request.clone());
            let base_response = match mcts_reasoning.next().await {
                Some(Ok(response)) => response,
                _ => {
                    let _ = tx
                        .send(Err(crate::strategies::base::ReasoningError::Other(
                            "Failed to get base response from MCTS002Alpha".into(),
                        )))
                        .await;
                    return;
                }
            };

            let _node_id = Uuid::new_v4().to_string();

            // Process the thought using standard MCTS002Alpha
            let policy_response = base_response.clone();

            // Track start and goal nodes for bidirectional search
            if request.parent_id.is_none() {
                // This is a start node
                if let Ok(Some(node)) = self_clone.base.get_node(&policy_response.node_id).await {
                    if let Ok(policy_node) = self_clone.inner_strategy.thought_to_policy(node).await
                    {
                        let mut bi_node = BidirectionalPolicyNode {
                            base: policy_node,
                            g: 0.0,
                            h: 0.0,
                            f: 0.0,
                            parent: None,
                            direction: Some("forward".to_string()),
                            search_depth: Some(0),
                            meeting_point: None,
                        };

                        // Calculate h and f
                        bi_node.h = 1.0 - bi_node.base.value_estimate;
                        bi_node.f = bi_node.g + bi_node.h;

                        let mut start_node = self_clone.start_node.lock().await;
                        *start_node = Some(bi_node);
                    }
                }
            }

            if !request.next_thought_needed {
                // This is a goal node
                if let Ok(Some(node)) = self_clone.base.get_node(&policy_response.node_id).await {
                    if let Ok(policy_node) = self_clone.inner_strategy.thought_to_policy(node).await
                    {
                        let mut bi_node = BidirectionalPolicyNode {
                            base: policy_node,
                            g: 0.0,
                            h: 0.0,
                            f: 0.0,
                            parent: None,
                            direction: Some("backward".to_string()),
                            search_depth: Some(0),
                            meeting_point: None,
                        };

                        // Calculate h and f
                        bi_node.h = 1.0 - bi_node.base.value_estimate;
                        bi_node.f = bi_node.g + bi_node.h;

                        let mut goal_node = self_clone.goal_node.lock().await;
                        *goal_node = Some(bi_node);
                    }
                }
            }

            // Run bidirectional search if we have both endpoints
            let mut path = Vec::new();
            {
                let start_node = self_clone.start_node.lock().await;
                let goal_node = self_clone.goal_node.lock().await;

                if let (Some(start), Some(goal)) = (start_node.clone(), goal_node.clone()) {
                    if let Ok(found_path) = self_clone.bidirectional_search(start, goal).await {
                        path = found_path;
                    }
                }
            }

            if !path.is_empty() {
                let _ = self_clone.update_path_with_policy_guidance(&path).await;
            }

            // Calculate enhanced path statistics
            let current_path = self_clone
                .base
                .state_manager
                .get_path(&policy_response.node_id)
                .await;
            let stats = self_clone.bidirectional_stats.lock().await.clone();
            let enhanced_score =
                self_clone.calculate_bidirectional_policy_score(&current_path, &stats);

            let response = ReasoningResponse {
                node_id: policy_response.node_id,
                thought: policy_response.thought,
                score: enhanced_score,
                depth: policy_response.depth,
                is_complete: policy_response.is_complete,
                next_thought_needed: policy_response.next_thought_needed,
                possible_paths: policy_response.possible_paths,
                best_score: Some(
                    policy_response
                        .best_score
                        .unwrap_or(0.0)
                        .max(enhanced_score),
                ),
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
            // Use inner strategy for best path calculation
            let inner_path = self_clone.inner_strategy.get_best_path().await;
            let _ = tx.send(inner_path);
        });

        AsyncPath::new(rx)
    }

    fn get_metrics(&self) -> MetricStream {
        let self_clone = self.clone();
        let (tx, rx) = mpsc::channel(1);

        tokio::spawn(async move {
            let mut inner_metrics_stream = self_clone.inner_strategy.get_metrics();
            
            // Get the metrics from the stream (should be just one item)
            let base_metrics = match inner_metrics_stream.next().await {
                Some(Ok(metrics)) => metrics,
                _ => Metric {
                    name: String::from("MCTS-002Alt-Alpha"),
                    nodes_explored: 0,
                    average_score: 0.0,
                    max_depth: 0,
                    active: None,
                    extra: Default::default(),
                },
            };

            let mut metrics = base_metrics;
            let nodes = self_clone.base.state_manager.get_all_nodes().await;

            // Build approximated bidirectional metrics
            let stats = self_clone.bidirectional_stats.lock().await.clone();
            let start_node = self_clone.start_node.lock().await;
            let goal_node = self_clone.goal_node.lock().await;
            // Split nodes by direction (approximate since we don't store direction with nodes)
            // This remains an approximation based on depth parity.
            let (forward_nodes, backward_nodes): (Vec<&ThoughtNode>, Vec<&ThoughtNode>) =
                nodes.iter().partition(|n| n.depth % 2 == 0); // Approximation

            let bidirectional_metrics = serde_json::json!({
                "forward_search_approx": { // Explicitly indicate approximation
                    "nodes_explored": forward_nodes.len(),
                    "average_score": forward_nodes.iter().map(|n| n.score).sum::<f64>() / forward_nodes.len().max(1) as f64,
                    "exploration_rate": stats.forward_exploration_rate
                },
                "backward_search_approx": { // Explicitly indicate approximation
                    "nodes_explored": backward_nodes.len(),
                    "average_score": backward_nodes.iter().map(|n| n.score).sum::<f64>() / backward_nodes.len().max(1) as f64,
                    "exploration_rate": stats.backward_exploration_rate
                },
                "meeting_points": {
                    "count": stats.meeting_points,
                }, // Keep reported meeting points from stats
                "path_quality": stats.path_quality
            });

            metrics.name = "MCTS-002Alt-Alpha (Bidirectional + Policy Enhanced)".to_string();
            metrics
                .extra
                .insert("has_start_node".to_string(), start_node.is_some().into());
            metrics
                .extra
                .insert("has_goal_node".to_string(), goal_node.is_some().into());
            metrics
                .extra
                .insert("bidirectional_metrics".to_string(), bidirectional_metrics);

            let _ = tx.send(Ok(metrics)).await;
        });

        MetricStream::new(rx)
    }

    fn clear(&self) -> ClearedSignal {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Clear the inner strategy first
            let inner_clear = self_clone.inner_strategy.clear();
            let _ = inner_clear.await; // Wait for inner clear to complete

            // Reset bidirectional search state
            let mut start_node = self_clone.start_node.lock().await;
            *start_node = None;

            let mut goal_node = self_clone.goal_node.lock().await;
            *goal_node = None;

            let mut stats = self_clone.bidirectional_stats.lock().await;
            *stats = BidirectionalStats {
                forward_exploration_rate: 2.0_f64.sqrt(),
                backward_exploration_rate: 2.0_f64.sqrt(),
                meeting_points: 0,
                path_quality: 0.0,
            };

            let _ = tx.send(Ok(()));
        });

        ClearedSignal::new(rx)
    }
}

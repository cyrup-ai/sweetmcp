use crate::state::StateManager;
use crate::strategies::base::{
    AsyncPath, BaseStrategy, ClearedSignal, Metric, MetricStream, Reasoning, Strategy,
};
use crate::types::{ReasoningRequest, ReasoningResponse, ThoughtNode, CONFIG};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCTSNode {
    #[serde(flatten)]
    pub base: ThoughtNode,
    pub visits: usize,
    pub total_reward: f64,
    pub untried_actions: Option<Vec<String>>,
}

pub struct MonteCarloTreeSearchStrategy {
    base: BaseStrategy,
    exploration_constant: f64,
    simulation_depth: usize,
    num_simulations: usize,
    root: Arc<Mutex<Option<MCTSNode>>>,
}

impl MonteCarloTreeSearchStrategy {
    pub fn new(state_manager: Arc<StateManager>, num_simulations: Option<usize>) -> Self {
        Self {
            base: BaseStrategy::new(state_manager),
            exploration_constant: 2.0_f64.sqrt(),
            simulation_depth: CONFIG.max_depth,
            num_simulations: num_simulations
                .unwrap_or(CONFIG.num_simulations)
                .max(1)
                .min(150),
            root: Arc::new(Mutex::new(None)),
        }
    }

    async fn run_simulations(
        &self,
        node: MCTSNode,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..self.num_simulations {
            let selected_node = self.select(node.clone()).await?;
            let expanded_node = self.expand(selected_node).await?;
            let reward = self.simulate(&expanded_node).await?;
            self.backpropagate(expanded_node, reward).await?;
        }
        Ok(())
    }

    async fn select(
        &self,
        node: MCTSNode,
    ) -> Result<MCTSNode, Box<dyn std::error::Error + Send + Sync>> {
        let mut current = node;

        while !current.base.children.is_empty()
            && current
                .untried_actions
                .as_ref()
                .map_or(true, |a| a.is_empty())
        {
            let mut children = Vec::new();
            for id in &current.base.children {
                if let Ok(Some(child_node)) = self.base.get_node(id).await {
                    // Convert ThoughtNode to MCTSNode
                    let mcts_child = self.thought_to_mcts(child_node).await?;
                    children.push(mcts_child);
                }
            }

            if children.is_empty() {
                break;
            }

            current = self.select_best_uct(children);
        }

        Ok(current)
    }

    async fn expand(
        &self,
        node: MCTSNode,
    ) -> Result<MCTSNode, Box<dyn std::error::Error + Send + Sync>> {
        if node.base.is_complete || node.base.depth >= self.simulation_depth {
            return Ok(node);
        }

        // Create a new thought node as expansion
        let new_node_id = Uuid::new_v4().to_string();
        let parent_prefix = node.base.thought.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
        // Simple heuristic continuation - better than a pure placeholder
        let new_thought = format!(
            "Considering '{}...', a possible next step is...",
            parent_prefix
        );
        let mut new_node = ThoughtNode {
            id: new_node_id.clone(),
            thought: new_thought,
            depth: node.base.depth + 1,
            score: 0.0,
            children: vec![],
            parent_id: Some(node.base.id.clone()),
            is_complete: false,
        };

        new_node.score = self.base.evaluate_thought(&new_node, Some(&node.base));

        // Save the new node
        if let Err(e) = self.base.save_node(new_node.clone()).await {
            return Err(Box::new(e));
        }

        // Update parent
        let mut parent = node.clone();
        parent.base.children.push(new_node_id.clone());

        // Extract base node and save it
        let parent_base = parent.base.clone();
        if let Err(e) = self.base.save_node(parent_base).await {
            return Err(Box::new(e));
        }

        // Convert to MCTSNode and return
        let mcts_node = MCTSNode {
            base: new_node,
            visits: 1,
            total_reward: 0.0,
            untried_actions: Some(vec![]),
        };

        Ok(mcts_node)
    }

    async fn simulate(
        &self,
        node: &MCTSNode,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut current = node.clone();
        let mut total_score = current.base.score;
        let mut depth = current.base.depth;

        while depth < self.simulation_depth && !current.base.is_complete {
            let simulated_node_id = Uuid::new_v4().to_string();
            let current_prefix = current.base.thought.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
            // Simple heuristic continuation for simulation
            let simulated_thought = format!(
                "If we follow '{}...', perhaps...",
                 current_prefix
            );
            let mut simulated_node = ThoughtNode {
                id: simulated_node_id,
                thought: simulated_thought,
                depth: depth + 1,
                score: 0.0, // Score will be evaluated below
                children: vec![],
                parent_id: Some(current.base.id.clone()),
                is_complete: depth + 1 >= self.simulation_depth,
            };

            simulated_node.score = self
                .base
                .evaluate_thought(&simulated_node, Some(&current.base));
            total_score += simulated_node.score;

            // Update current to the simulated node
            current = MCTSNode {
                base: simulated_node,
                visits: 1,
                total_reward: 0.0,
                untried_actions: Some(vec![]),
            };

            depth += 1;
        }

        Ok(total_score / (depth - node.base.depth + 1) as f64)
    }

    async fn backpropagate(
        &self,
        node: MCTSNode,
        reward: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current_opt = Some(node);

        while let Some(mut current) = current_opt {
            // Update node stats
            current.visits += 1;
            current.total_reward += reward;

            // Save the updated node
            let updated_node = ThoughtNode {
                id: current.base.id.clone(),
                thought: current.base.thought.clone(),
                score: current.base.score,
                depth: current.base.depth,
                children: current.base.children.clone(),
                parent_id: current.base.parent_id.clone(),
                is_complete: current.base.is_complete,
            };

            if let Err(e) = self.base.save_node(updated_node).await {
                return Err(Box::new(e));
            }

            // Move to parent
            if let Some(parent_id) = &current.base.parent_id {
                if let Ok(Some(parent_node)) = self.base.get_node(parent_id).await {
                    current_opt = Some(self.thought_to_mcts(parent_node).await?);
                } else {
                    current_opt = None;
                }
            } else {
                current_opt = None;
            }
        }

        Ok(())
    }

    fn select_best_uct(&self, nodes: Vec<MCTSNode>) -> MCTSNode {
        let total_visits: usize = nodes.iter().map(|node| node.visits).sum();

        nodes
            .into_iter()
            .fold(None, |best: Option<MCTSNode>, node| {
                let exploitation = node.total_reward / node.visits as f64;
                let exploration = ((total_visits as f64).ln() / node.visits as f64).sqrt();
                let uct = exploitation + self.exploration_constant * exploration;

                match best {
                    None => Some(node),
                    Some(best_node) => {
                        let best_exploitation = best_node.total_reward / best_node.visits as f64;
                        let best_exploration =
                            ((total_visits as f64).ln() / best_node.visits as f64).sqrt();
                        let best_uct =
                            best_exploitation + self.exploration_constant * best_exploration;

                        if uct > best_uct {
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

    fn calculate_path_score(&self, path: &[ThoughtNode]) -> f64 {
        if path.is_empty() {
            return 0.0;
        }

        path.iter().map(|node| node.score).sum::<f64>() / path.len() as f64
    }

    async fn calculate_possible_paths(&self, node: &MCTSNode) -> usize {
        // Count actual paths explored down to simulation depth
        self.count_paths_recursive(&node.base.id, node.base.depth)
            .await
    }

    async fn count_paths_recursive(&self, node_id: &str, current_depth: usize) -> usize {
        // Use Box::pin to avoid infinitely sized future with recursion
        async fn inner(
            this: &MonteCarloTreeSearchStrategy,
            node_id: &str,
            current_depth: usize,
        ) -> usize {
            if current_depth >= this.simulation_depth {
                return 1; // Reached max depth for path counting
            }

            let children = this.base.state_manager.get_children(node_id).await;

            if children.is_empty() {
                return 1; // Leaf node in the explored tree
            }

            let mut count = 0;
            for child in children {
                // Ensure we don't exceed simulation depth in recursion
                if child.depth < this.simulation_depth {
                    count += Box::pin(inner(this, &child.id, child.depth)).await;
                } else {
                    count += 1; // Child is at max depth, counts as one path end
                }
            }
            count
        }
        
        Box::pin(inner(self, node_id, current_depth)).await
    }

    async fn thought_to_mcts(
        &self,
        node: ThoughtNode,
    ) -> Result<MCTSNode, Box<dyn std::error::Error + Send + Sync>> {
        Ok(MCTSNode {
            base: node,
            visits: 1,
            total_reward: 0.0,
            untried_actions: Some(vec![]),
        })
    }
}

// Add Clone implementation for MonteCarloTreeSearchStrategy
impl Clone for MonteCarloTreeSearchStrategy {
    fn clone(&self) -> Self {
        Self {
            base: BaseStrategy::new(Arc::clone(&self.base.state_manager)),
            exploration_constant: self.exploration_constant,
            simulation_depth: self.simulation_depth,
            num_simulations: self.num_simulations,
            root: Arc::clone(&self.root),
        }
    }
}

impl Strategy for MonteCarloTreeSearchStrategy {
    fn process_thought(&self, request: ReasoningRequest) -> Reasoning {
        let (tx, rx) = mpsc::channel(1);
        let self_clone = self.clone();

        tokio::spawn(async move {
            let node_id = Uuid::new_v4().to_string();
            let parent_node = match &request.parent_id {
                Some(parent_id) => self_clone.base.get_node(parent_id).await.unwrap_or(None),
                None => None,
            };

            let mut node = ThoughtNode {
                id: node_id.clone(),
                thought: request.thought.clone(),
                depth: request.thought_number - 1,
                score: 0.0,
                children: vec![],
                parent_id: request.parent_id.clone(),
                is_complete: !request.next_thought_needed,
            };

            // Initialize node
            node.score = self_clone
                .base
                .evaluate_thought(&node, parent_node.as_ref());
            if let Err(e) = self_clone.base.save_node(node.clone()).await {
                tracing::error!("Error saving node: {}", e);
            }

            // Update parent if exists
            if let Some(mut parent) = parent_node {
                parent.children.push(node.id.clone());
                if let Err(e) = self_clone.base.save_node(parent).await {
                    tracing::error!("Error saving parent node: {}", e);
                }
            }

            // Create MCTS node
            let mcts_node = MCTSNode {
                base: node.clone(),
                visits: 1,
                total_reward: node.score,
                untried_actions: Some(vec![]),
            };

            // If this is a root node, store it
            if node.parent_id.is_none() {
                let mut root = self_clone.root.lock().await;
                *root = Some(mcts_node.clone());
            }

            // Run MCTS simulations
            if !node.is_complete {
                let _ = self_clone.run_simulations(mcts_node).await;
            }

            // Calculate path statistics
            let current_path = self_clone.base.state_manager.get_path(&node_id).await;
            let path_score = self_clone.calculate_path_score(&current_path);

            // Calculate possible paths
            let mcts_node_for_paths = MCTSNode {
                base: node.clone(),
                visits: 1,
                total_reward: node.score,
                untried_actions: Some(vec![]),
            };
            let possible_paths = self_clone.calculate_possible_paths(&mcts_node_for_paths).await;

            let response = ReasoningResponse {
                node_id: node.id,
                thought: node.thought,
                score: node.score,
                depth: node.depth,
                is_complete: node.is_complete,
                next_thought_needed: request.next_thought_needed,
                possible_paths: Some(possible_paths),
                best_score: Some(path_score),
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
            let root_opt = self_clone.root.lock().await.clone();

            if let Some(root) = root_opt {
                let children = self_clone
                    .base
                    .state_manager
                    .get_children(&root.base.id)
                    .await;
                if children.is_empty() {
                    let _ = tx.send(Ok(vec![]));
                    return;
                }

                let mut best_child: Option<ThoughtNode> = None;
                let mut max_visits = 0;

                for child in children {
                    let child_id = child.id.clone();
                    if let Ok(Some(child_node)) = self_clone.base.get_node(&child_id).await {
                        let mcts_child = match self_clone.thought_to_mcts(child_node).await {
                            Ok(node) => node,
                            Err(_) => continue,
                        };

                        if mcts_child.visits > max_visits {
                            max_visits = mcts_child.visits;
                            best_child = Some(mcts_child.base);
                        }
                    }
                }

                if let Some(best) = best_child {
                    let path = self_clone.base.state_manager.get_path(&best.id).await;
                    let _ = tx.send(Ok(path));
                    return;
                }
            }

            let _ = tx.send(Ok(vec![]));
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
                    name: String::from("Monte Carlo Tree Search"),
                    nodes_explored: 0,
                    average_score: 0.0,
                    max_depth: 0,
                    active: None,
                    extra: Default::default(),
                },
            };

            let mut metrics = base_metrics;

            let root_visits = match &*self_clone.root.lock().await {
                Some(root) => root.visits,
                None => 0,
            };

            metrics.name = "Monte Carlo Tree Search".to_string();
            metrics.extra.insert(
                "simulation_depth".to_string(),
                self_clone.simulation_depth.into(),
            );
            metrics.extra.insert(
                "num_simulations".to_string(),
                self_clone.num_simulations.into(),
            );
            metrics.extra.insert(
                "exploration_constant".to_string(),
                self_clone.exploration_constant.into(),
            );
            metrics
                .extra
                .insert("total_simulations".to_string(), root_visits.into());

            let _ = tx.send(Ok(metrics)).await;
        });

        MetricStream::new(rx)
    }

    fn clear(&self) -> ClearedSignal {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut root = self_clone.root.lock().await;
            *root = None;
            let _ = tx.send(Ok(()));
        });

        ClearedSignal::new(rx)
    }
}

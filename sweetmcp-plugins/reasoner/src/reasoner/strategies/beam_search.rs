use crate::state::StateManager;
use crate::strategies::base::{AsyncPath, BaseStrategy, ClearedSignal, MetricStream, Reasoning, Strategy, AsyncTask, TaskStream};
use crate::types::{ReasoningRequest, ReasoningResponse, StrategyMetrics, ThoughtNode, CONFIG};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

pub struct BeamSearchStrategy {
    base: BaseStrategy,
    beam_width: usize,
    beams: Arc<Mutex<HashMap<usize, Vec<ThoughtNode>>>>,
}

impl BeamSearchStrategy {
    pub fn new(state_manager: Arc<StateManager>, beam_width: Option<usize>) -> Self {
        Self {
            base: BaseStrategy::new(state_manager),
            beam_width: beam_width.unwrap_or(CONFIG.beam_width),
            beams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn calculate_possible_paths(&self) -> usize {
        let beams = self.beams.lock().await;
        let mut total_paths = 0;

        let depths: Vec<usize> = beams.keys().copied().collect();
        for depth in &depths {
            let beam = match beams.get(depth) {
                Some(beam) => beam,
                None => {
                    tracing::error!("Beam at depth {} not found", depth);
                    return 0; // Return 0 paths if beam not found
                }
            };
            let next_beam = beams.get(&(depth + 1));

            if let Some(next_beam) = next_beam {
                total_paths += beam.len() * next_beam.len();
            } else {
                total_paths += beam.len();
            }
        }

        total_paths
    }
}

impl Strategy for BeamSearchStrategy {
    fn process_thought(&self, request: ReasoningRequest) -> Reasoning {
        let self_clone = self.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        tokio::spawn(async move {
            let node_id = Uuid::new_v4().to_string();
            let parent_node = match &request.parent_id {
                Some(parent_id) => {
                    match self_clone.base.get_node(parent_id).await {
                        Ok(node) => node,
                        Err(e) => {
                            let _ = tx.send(Err(crate::strategies::base::ReasoningError::Other(
                                format!("Failed to get parent node: {}", e)
                            )));
                            return;
                        }
                    }
                },
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

            // Evaluate and score the node
            node.score = self_clone
                .base
                .evaluate_thought(&node, parent_node.as_ref());
            if let Err(e) = self_clone.base.save_node(node.clone()).await {
                let _ = tx.send(Err(crate::strategies::base::ReasoningError::Other(
                    format!("Failed to save node: {}", e)
                )));
                return;
            }

            // Update parent if exists
            if let Some(mut parent) = parent_node {
                parent.children.push(node.id.clone());
                if let Err(e) = self_clone.base.save_node(parent).await {
                    let _ = tx.send(Err(crate::strategies::base::ReasoningError::Other(
                        format!("Failed to save parent node: {}", e)
                    )));
                    return;
                }
            }

            // Manage beam at current depth
            let mut beams = self_clone.beams.lock().await;
            let current_beam = beams.entry(node.depth).or_insert_with(Vec::new);
            current_beam.push(node.clone());
            current_beam.sort_by(|a, b| {
                match b.score.partial_cmp(&a.score) {
                    Some(order) => order,
                    None => std::cmp::Ordering::Equal, // Handle NaN values by treating them as equal
                }
            });

            // Prune beam to maintain beam width
            if current_beam.len() > self_clone.beam_width {
                *current_beam = current_beam[0..self_clone.beam_width].to_vec();
            }
            drop(beams);

            // Calculate path statistics
            let current_path = self_clone.base.state_manager.get_path(&node_id).await;
            let path_score = if current_path.is_empty() {
                0.0
            } else {
                current_path.iter().map(|n| n.score).sum::<f64>() / current_path.len() as f64
            };

            // Get best beam score from all beams
            let beams = self_clone.beams.lock().await;
            let best_beam_score = beams
                .values()
                .flat_map(|nodes| nodes.iter().map(|n| n.score))
                .fold(f64::NEG_INFINITY, f64::max);
            drop(beams);

            // Calculate possible paths
            let possible_paths = self_clone.calculate_possible_paths().await;

            let response = ReasoningResponse {
                node_id: node.id,
                thought: node.thought,
                score: node.score,
                depth: node.depth,
                is_complete: node.is_complete,
                next_thought_needed: request.next_thought_needed,
                possible_paths: Some(possible_paths),
                best_score: Some(path_score.max(best_beam_score)),
                strategy_used: None, // Will be set by reasoner
            };

            if let Err(_) = tx.send(Ok(response)).await {
                // Channel closed, receiver dropped
            }
        });

        TaskStream::new(rx)
    }

    fn get_best_path(&self) -> AsyncPath {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let beams = self_clone.beams.lock().await;

            // Find the deepest beam
            let max_depth = beams.keys().max().copied();

            if let Some(depth) = max_depth {
                if let Some(deepest_beam) = beams.get(&depth) {
                    if !deepest_beam.is_empty() {
                        // Get the best scoring node from deepest beam
                        let best_node_id = deepest_beam
                            .iter()
                            .max_by(|a, b| {
                                match a.score.partial_cmp(&b.score) {
                                    Some(ordering) => ordering,
                                    None => std::cmp::Ordering::Equal, // Handle NaN values
                                }
                            })
                            .expect("Deepest beam should contain at least one element")
                            .id
                            .clone();

                        drop(beams);
                        let path = self_clone.base.state_manager.get_path(&best_node_id).await;
                        let _ = tx.send(Ok(path));
                        return;
                    }
                }
            }

            let _ = tx.send(Ok(vec![]));
        });

        AsyncTask::new(rx)
    }

    fn get_metrics(&self) -> MetricStream {
        let self_clone = self.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        tokio::spawn(async move {
            let base_metrics = self_clone
                .base
                .get_base_metrics()
                .await
                .unwrap_or_else(|_| StrategyMetrics {
                    name: String::from("Beam Search"),
                    nodes_explored: 0,
                    average_score: 0.0,
                    max_depth: 0,
                    active: None,
                    extra: Default::default(),
                });

            let mut metrics = base_metrics;

            let beams = self_clone.beams.lock().await;
            let active_beams = beams.len();
            let total_beam_nodes = beams.values().map(|beam| beam.len()).sum::<usize>();
            drop(beams);

            metrics.name = "Beam Search".to_string();
            metrics
                .extra
                .insert("beam_width".to_string(), self_clone.beam_width.into());
            metrics
                .extra
                .insert("active_beams".to_string(), active_beams.into());
            metrics
                .extra
                .insert("total_beam_nodes".to_string(), total_beam_nodes.into());

            if let Err(_) = tx.send(Ok(metrics)).await {
                // Channel closed, receiver dropped
            }
        });

        TaskStream::new(rx)
    }

    fn clear(&self) -> ClearedSignal {
        let self_clone = self.clone();
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut beams = self_clone.beams.lock().await;
            beams.clear();
            let _ = tx.send(Ok(()));
        });

        AsyncTask::new(rx)
    }
}

// Add Clone implementation for BeamSearchStrategy
impl Clone for BeamSearchStrategy {
    fn clone(&self) -> Self {
        Self {
            base: BaseStrategy::new(Arc::clone(&self.base.state_manager)),
            beam_width: self.beam_width,
            beams: Arc::clone(&self.beams),
        }
    }
}

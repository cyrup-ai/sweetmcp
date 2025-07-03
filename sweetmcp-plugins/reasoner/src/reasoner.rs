use crate::state::StateManager;
use crate::strategies::factory::{ReasoningStrategy, StrategyFactory};
use crate::types::{ReasoningRequest, ReasoningResponse, ReasoningStats, ThoughtNode, CONFIG};
use futures::StreamExt;
use std::sync::Arc;

pub struct Reasoner {
    state_manager: Arc<StateManager>,
}

impl Reasoner {
    pub fn new(cache_size: Option<usize>) -> Self {
        let cache_size = cache_size.unwrap_or(CONFIG.cache_size);
        Self {
            state_manager: Arc::new(StateManager::new(cache_size)),
        }
    }

    pub async fn process_thought(&self, request: ReasoningRequest) -> ReasoningResponse {
        // Determine which strategy to use
        let strategy_type = match &request.strategy_type {
            Some(strategy_str) => ReasoningStrategy::from_str(strategy_str)
                .unwrap_or_else(|| {
                    ReasoningStrategy::from_str(CONFIG.default_strategy)
                        .unwrap_or(ReasoningStrategy::BeamSearch)
                }),
            None => ReasoningStrategy::from_str(CONFIG.default_strategy)
                .unwrap_or(ReasoningStrategy::BeamSearch),
        };

        // Create strategy instance
        let strategy = StrategyFactory::create_strategy(
            strategy_type,
            Arc::clone(&self.state_manager),
            request.beam_width,
            request.num_simulations,
        );

        // Process thought with selected strategy - convert stream to single item
        let mut reasoning_stream = strategy.process_thought(request);
        
        // Take the first item from the stream
        let response_result = match reasoning_stream.next().await {
            Some(result) => result,
            None => {
                return ReasoningResponse {
                    node_id: "error".to_string(),
                    thought: "Error: Empty response stream".to_string(),
                    score: 0.0,
                    depth: 0,
                    is_complete: true,
                    next_thought_needed: false,
                    possible_paths: None,
                    best_score: None,
                    strategy_used: Some(strategy_type.as_str().to_string()),
                };
            }
        };

        let mut response = match response_result {
            Ok(response) => response,
            Err(e) => {
                return ReasoningResponse {
                    node_id: "error".to_string(),
                    thought: format!("Error processing thought: {}", e),
                    score: 0.0,
                    depth: 0,
                    is_complete: true,
                    next_thought_needed: false,
                    possible_paths: None,
                    best_score: None,
                    strategy_used: Some(strategy_type.as_str().to_string()),
                };
            }
        };

        // Add strategy info to response
        response.strategy_used = Some(strategy_type.as_str().to_string());

        response
    }

    pub async fn get_best_reasoning_path(&self, strategy_type: Option<&str>) -> Vec<ThoughtNode> {
        // Determine which strategy to use
        let strategy_type = match strategy_type {
            Some(strategy_str) => ReasoningStrategy::from_str(strategy_str)
                .unwrap_or_else(|| {
                    ReasoningStrategy::from_str(CONFIG.default_strategy)
                        .unwrap_or(ReasoningStrategy::BeamSearch)
                }),
            None => ReasoningStrategy::from_str(CONFIG.default_strategy)
                .unwrap_or(ReasoningStrategy::BeamSearch),
        };

        // Create strategy instance
        let strategy = StrategyFactory::create_strategy(
            strategy_type,
            Arc::clone(&self.state_manager),
            None,
            None,
        );

        // Get the best path and handle errors
        let async_path = strategy.get_best_path();
        match async_path.await {
            Ok(path) => path,
            Err(_) => Vec::new(),
        }
    }

    pub async fn get_stats(&self, strategy_types: Vec<&str>) -> ReasoningStats {
        let nodes = self.state_manager.get_all_nodes().await;

        let total_nodes = nodes.len();
        let avg_score = if nodes.is_empty() {
            0.0
        } else {
            nodes.iter().map(|n| n.score).sum::<f64>() / total_nodes as f64
        };

        let max_depth = nodes.iter().map(|n| n.depth).max().unwrap_or(0);

        // Calculate branching factor
        let mut parent_counts = std::collections::HashMap::new();
        for node in &nodes {
            if let Some(parent_id) = &node.parent_id {
                *parent_counts.entry(parent_id.clone()).or_insert(0) += 1;
            }
        }

        let branching_factor = if parent_counts.is_empty() {
            0.0
        } else {
            parent_counts.values().sum::<usize>() as f64 / parent_counts.len() as f64
        };

        // Get metrics from all requested strategies
        let mut strategy_metrics = std::collections::HashMap::new();

        for strategy_str in strategy_types {
            if let Some(strategy_type) = ReasoningStrategy::from_str(strategy_str) {
                let strategy = StrategyFactory::create_strategy(
                    strategy_type,
                    Arc::clone(&self.state_manager),
                    None,
                    None,
                );

                // Get metrics from stream
                let mut metrics_stream = strategy.get_metrics();
                if let Some(Ok(metrics)) = metrics_stream.next().await {
                    strategy_metrics.insert(strategy_str.to_string(), metrics);
                }
            }
        }

        ReasoningStats {
            total_nodes,
            average_score: avg_score,
            max_depth,
            branching_factor,
            strategy_metrics,
        }
    }

    pub async fn clear(&self) {
        // Clear the state manager
        self.state_manager.clear().await;

        // Also clear each strategy
        for strategy_str in &["beam_search", "mcts", "mcts_002_alpha", "mcts_002alt_alpha"] {
            if let Some(strategy_type) = ReasoningStrategy::from_str(strategy_str) {
                let strategy = StrategyFactory::create_strategy(
                    strategy_type,
                    Arc::clone(&self.state_manager),
                    None,
                    None,
                );

                // Call clear and await the result
                let _ = strategy.clear().await;
            }
        }
    }
}

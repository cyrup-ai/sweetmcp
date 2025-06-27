use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtNode {
    pub id: String,
    pub thought: String,
    pub score: f64,
    pub depth: usize,
    pub children: Vec<String>, // Store child IDs
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>, // Store parent ID
    #[serde(rename = "isComplete")]
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningRequest {
    pub thought: String,
    #[serde(rename = "thoughtNumber")]
    pub thought_number: usize,
    #[serde(rename = "totalThoughts")]
    pub total_thoughts: usize,
    #[serde(rename = "nextThoughtNeeded")]
    pub next_thought_needed: bool,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>, // For branching thoughts
    #[serde(rename = "strategyType")]
    pub strategy_type: Option<String>, // Strategy to use for reasoning
    #[serde(rename = "beamWidth")]
    pub beam_width: Option<usize>, // Number of top paths to maintain (n-sampling)
    #[serde(rename = "numSimulations")]
    pub num_simulations: Option<usize>, // Number of MCTS simulations to run
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResponse {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub thought: String,
    pub score: f64,
    pub depth: usize,
    #[serde(rename = "isComplete")]
    pub is_complete: bool,
    #[serde(rename = "nextThoughtNeeded")]
    pub next_thought_needed: bool,
    #[serde(rename = "possiblePaths")]
    pub possible_paths: Option<usize>,
    #[serde(rename = "bestScore")]
    pub best_score: Option<f64>,
    #[serde(rename = "strategyUsed")]
    pub strategy_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStats {
    #[serde(rename = "totalNodes")]
    pub total_nodes: usize,
    #[serde(rename = "averageScore")]
    pub average_score: f64,
    #[serde(rename = "maxDepth")]
    pub max_depth: usize,
    #[serde(rename = "branchingFactor")]
    pub branching_factor: f64,
    #[serde(rename = "strategyMetrics")]
    pub strategy_metrics: HashMap<String, StrategyMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub name: String,
    #[serde(rename = "nodesExplored")]
    pub nodes_explored: usize,
    #[serde(rename = "averageScore")]
    pub average_score: f64,
    #[serde(rename = "maxDepth")]
    pub max_depth: usize,
    pub active: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub struct Config {
    pub beam_width: usize,
    pub max_depth: usize,
    pub min_score: f64,
    pub temperature: f64,
    pub cache_size: usize,
    pub default_strategy: &'static str,
    pub num_simulations: usize,
}

pub const CONFIG: Config = Config {
    beam_width: 3,                   // Keep top 3 paths
    max_depth: 5,                    // Reasonable depth limit
    min_score: 0.5,                  // Threshold for path viability
    temperature: 0.7,                // For thought diversity
    cache_size: 1000,                // LRU cache size
    default_strategy: "beam_search", // Match TypeScript reference default strategy
    num_simulations: 50,             // Default number of MCTS simulations
};

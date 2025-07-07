// Import Extism PDK for plugin development
use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;
use uuid::Uuid;

// Core types for the MCP reasoner

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

// Simplified reasoner for the WASM plugin. In a real implementation,
// this would include all the strategy implementations.
pub struct SimpleReasoner {
    nodes: HashMap<String, ThoughtNode>,
}

impl SimpleReasoner {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn process_thought(&mut self, request: ReasoningRequest) -> ReasoningResponse {
        // Generate a unique ID for this thought
        let node_id = Uuid::new_v4().to_string();

        // Default strategy
        let strategy = request
            .strategy_type
            .unwrap_or_else(|| "beam_search".to_string());

        // Calculate score (in a real implementation, this would use the selected strategy)
        let score = 0.7 + (request.thought_number as f64 * 0.05);

        // Create the node
        let node = ThoughtNode {
            id: node_id.clone(),
            thought: request.thought.clone(),
            score,
            depth: request.thought_number,
            children: Vec::new(),
            parent_id: request.parent_id.clone(),
            is_complete: !request.next_thought_needed,
        };

        // Add to parent's children if it exists
        if let Some(parent_id) = &request.parent_id {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children.push(node_id.clone());
            }
        }

        // Store the node
        self.nodes.insert(node_id.clone(), node.clone());

        // Generate response
        ReasoningResponse {
            node_id,
            thought: request.thought,
            score,
            depth: request.thought_number,
            is_complete: !request.next_thought_needed,
            next_thought_needed: request.next_thought_needed,
            possible_paths: Some(1),
            best_score: Some(score),
            strategy_used: Some(strategy),
        }
    }

    pub fn get_stats(&self, strategy_types: Vec<&str>) -> ReasoningStats {
        let total_nodes = self.nodes.len();
        let average_score = if total_nodes > 0 {
            self.nodes.values().map(|n| n.score).sum::<f64>() / total_nodes as f64
        } else {
            0.0
        };

        let max_depth = self.nodes.values().map(|n| n.depth).max().unwrap_or(0);

        // Calculate branching factor
        let mut parent_counts = HashMap::new();
        for node in self.nodes.values() {
            if let Some(parent_id) = &node.parent_id {
                *parent_counts.entry(parent_id.clone()).or_insert(0) += 1;
            }
        }

        let branching_factor = if parent_counts.is_empty() {
            0.0
        } else {
            parent_counts.values().sum::<usize>() as f64 / parent_counts.len() as f64
        };

        // Create strategy metrics
        let mut strategy_metrics = HashMap::new();
        for strategy in strategy_types {
            let metrics = StrategyMetrics {
                name: strategy.to_string(),
                nodes_explored: total_nodes,
                average_score,
                max_depth,
                active: Some(true),
                extra: HashMap::new(),
            };

            strategy_metrics.insert(strategy.to_string(), metrics);
        }

        ReasoningStats {
            total_nodes,
            average_score,
            max_depth,
            branching_factor,
            strategy_metrics,
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

// Track plugin state (singleton pattern)
static REASONER: OnceLock<Mutex<SimpleReasoner>> = OnceLock::new();

fn get_reasoner() -> &'static Mutex<SimpleReasoner> {
    REASONER.get_or_init(|| Mutex::new(SimpleReasoner::new()))
}

// Extism plugin exports

#[derive(Debug, Serialize, Deserialize)]
struct EnhancedResponse {
    #[serde(rename = "thoughtNumber")]
    thought_number: usize,
    #[serde(rename = "totalThoughts")]
    total_thoughts: usize,
    #[serde(rename = "nextThoughtNeeded")]
    next_thought_needed: bool,
    thought: String,
    #[serde(rename = "nodeId")]
    node_id: String,
    score: f64,
    #[serde(rename = "strategyUsed")]
    strategy_used: String,
    stats: ReasoningStats,
}

#[plugin_fn]
pub fn process_thought(input: String) -> FnResult<String> {
    // Parse the input JSON
    let request: ReasoningRequest = serde_json::from_str(&input)?;

    // Get the reasoner singleton
    let reasoner = get_reasoner();

    // Process the thought
    let response = match reasoner.lock() {
        Ok(mut reasoner) => reasoner.process_thought(request.clone()),
        Err(e) => {
            return Ok(serde_json::json!({
                "is_error": true,
                "content": [{
                    "type": "text",
                    "text": format!("Failed to lock reasoner: {}", e)
                }]
            })
            .to_string());
        }
    };

    // Get stats for the used strategy
    let strategy = response
        .strategy_used
        .clone()
        .unwrap_or("beam_search".to_string());
    let stats = match reasoner.lock() {
        Ok(reasoner) => reasoner.get_stats(vec![&strategy]),
        Err(e) => {
            return Ok(serde_json::json!({
                "is_error": true,
                "content": [{
                    "type": "text",
                    "text": format!("Failed to lock reasoner for stats: {}", e)
                }]
            })
            .to_string());
        }
    };

    // Create the enhanced response
    let enhanced_response = EnhancedResponse {
        thought_number: request.thought_number,
        total_thoughts: request.total_thoughts,
        next_thought_needed: request.next_thought_needed,
        thought: request.thought.clone(),
        node_id: response.node_id,
        score: response.score,
        strategy_used: strategy,
        stats,
    };

    // Serialize and return
    Ok(serde_json::to_string(&enhanced_response)?)
}

#[plugin_fn]
pub fn clear(_: String) -> FnResult<String> {
    // Get the reasoner singleton and clear it
    let reasoner = get_reasoner();
    match reasoner.lock() {
        Ok(mut reasoner) => reasoner.clear(),
        Err(e) => {
            return Err(extism_pdk::Error::msg(format!(
                "Failed to lock reasoner for clearing: {}",
                e
            ))
            .into());
        }
    };

    Ok("Reasoner state cleared".to_string())
}

// Plugin manifest for tool definition
#[plugin_fn]
pub fn manifest(_: String) -> FnResult<String> {
    let manifest = serde_json::json!({
        "name": "mcp-reasoner",
        "description": "Advanced reasoning tool with multiple strategies including Beam Search and Monte Carlo Tree Search",
        "functions": [
            {
                "name": "process_thought",
                "description": "Process a thought with the reasoning engine",
                "inputs": [{
                    "name": "request",
                    "description": "Reasoning request",
                }],
                "outputs": [{
                    "name": "response",
                    "description": "Enhanced reasoning response",
                }]
            },
            {
                "name": "clear",
                "description": "Clear the reasoner state",
                "inputs": [],
                "outputs": [{
                    "name": "message",
                    "description": "Status message",
                }]
            }
        ],
        "config": {
            "schema": {
                "thought": {
                    "type": "string",
                    "description": "Current reasoning step"
                },
                "thoughtNumber": {
                    "type": "integer",
                    "description": "Current step number",
                    "minimum": 1
                },
                "totalThoughts": {
                    "type": "integer",
                    "description": "Total expected steps",
                    "minimum": 1
                },
                "nextThoughtNeeded": {
                    "type": "boolean",
                    "description": "Whether another step is needed"
                },
                "parentId": {
                    "type": ["string", "null"],
                    "description": "Optional parent thought ID for branching"
                },
                "strategyType": {
                    "type": ["string", "null"],
                    "enum": ["beam_search", "mcts", "mcts_002_alpha", "mcts_002alt_alpha", null],
                    "description": "Reasoning strategy to use (beam_search, mcts, mcts_002_alpha, or mcts_002alt_alpha)"
                },
                "beamWidth": {
                    "type": ["integer", "null"],
                    "description": "Number of top paths to maintain (n-sampling). Defaults if null",
                    "minimum": 1,
                    "maximum": 10
                },
                "numSimulations": {
                    "type": ["integer", "null"],
                    "description": "Number of MCTS simulations to run. Defaults if null",
                    "minimum": 1,
                    "maximum": 150
                }
            },
            "required": [
                "thought",
                "thoughtNumber",
                "totalThoughts",
                "nextThoughtNeeded"
            ]
        }
    });

    Ok(serde_json::to_string(&manifest)?)
}

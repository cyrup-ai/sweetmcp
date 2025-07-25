//! MCTS node type definitions and operations
//!
//! This module provides blazing-fast node structures with zero allocation
//! optimizations and elegant ergonomic interfaces for MCTS node operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Codebase state with performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeState {
    pub code: String,
    pub latency: f64,
    pub memory: f64,
    pub relevance: f64,
    pub memory_usage: f64,
    pub complexity_score: f64,
    pub metadata: CodeMetadata,
}

/// Metadata for code state tracking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub applied_actions: Vec<String>,
    pub optimization_level: f64,
    pub parallelization_level: f64,
    pub risk_level: f64,
}

impl Default for CodeMetadata {
    fn default() -> Self {
        Self {
            applied_actions: Vec::new(),
            optimization_level: 0.0,
            parallelization_level: 0.0,
            risk_level: 0.0,
        }
    }
}

impl CodeState {
    /// Create new code state with zero allocation for numeric operations
    #[inline]
    pub fn new(code: String, latency: f64, memory: f64, relevance: f64) -> Self {
        Self {
            code,
            latency,
            memory,
            relevance,
            memory_usage: memory / 1000.0, // Convert to usage ratio
            complexity_score: 5.0, // Default complexity
            metadata: CodeMetadata::default(),
        }
    }

    /// Create new code state with all fields
    #[inline]
    pub fn with_full_metrics(
        code: String, 
        latency: f64, 
        memory: f64, 
        relevance: f64,
        memory_usage: f64,
        complexity_score: f64,
        metadata: CodeMetadata,
    ) -> Self {
        Self {
            code,
            latency,
            memory,
            relevance,
            memory_usage,
            complexity_score,
            metadata,
        }
    }

    /// Calculate overall performance score with blazing-fast computation
    #[inline]
    pub fn performance_score(&self) -> f64 {
        // Weighted score: lower latency and memory are better, higher relevance is better
        let latency_score = 1.0 / (1.0 + self.latency);
        let memory_score = 1.0 / (1.0 + self.memory);
        let relevance_score = self.relevance / 100.0;
        
        (latency_score * 0.4 + memory_score * 0.3 + relevance_score * 0.3).clamp(0.0, 1.0)
    }

    /// Check if state meets performance constraints
    #[inline]
    pub fn meets_constraints(&self, max_latency: f64, max_memory: f64, min_relevance: f64) -> bool {
        self.latency <= max_latency && self.memory <= max_memory && self.relevance >= min_relevance
    }

    /// Calculate distance to target metrics
    #[inline]
    pub fn distance_to_target(&self, target_latency: f64, target_memory: f64, target_relevance: f64) -> f64 {
        let latency_diff = (self.latency - target_latency).abs();
        let memory_diff = (self.memory - target_memory).abs();
        let relevance_diff = (self.relevance - target_relevance).abs();
        
        (latency_diff + memory_diff + relevance_diff) / 3.0
    }

    /// Generate cache key for deterministic caching with zero allocation
    #[inline]
    pub fn cache_key(&self) -> String {
        // Create deterministic cache key based on state metrics
        // Using format! for now, but could be optimized with pre-allocated buffer
        format!("{}_{:.3}_{:.3}_{:.3}", 
            self.code.len(), 
            self.latency, 
            self.memory, 
            self.relevance
        )
    }

    /// Update metrics with new measurements
    #[inline]
    pub fn update_metrics(&mut self, new_latency: f64, new_memory: f64, new_relevance: f64) {
        // Exponential moving average for smooth updates
        let alpha = 0.3;
        self.latency = alpha * new_latency + (1.0 - alpha) * self.latency;
        self.memory = alpha * new_memory + (1.0 - alpha) * self.memory;
        self.relevance = alpha * new_relevance + (1.0 - alpha) * self.relevance;
    }

    /// Check if state is better than another state
    #[inline]
    pub fn is_better_than(&self, other: &CodeState) -> bool {
        self.performance_score() > other.performance_score()
    }

    /// Calculate improvement ratio compared to baseline
    #[inline]
    pub fn improvement_ratio(&self, baseline: &CodeState) -> f64 {
        let current_score = self.performance_score();
        let baseline_score = baseline.performance_score();
        
        if baseline_score > 0.0 {
            (current_score - baseline_score) / baseline_score
        } else {
            current_score
        }
    }
}

/// MCTS node with optimized memory layout
#[derive(Debug, Clone)]
pub struct MCTSNode {
    pub state: CodeState,
    pub visits: u32,
    pub total_reward: f64,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub action_taken: Option<String>,
    pub is_terminal: bool,
    pub depth: u16,
    pub untried_actions: Vec<String>,
    pub node_metadata: NodeMetadata,
}

impl MCTSNode {
    /// Create new MCTS node with zero allocation initialization
    #[inline]
    pub fn new(state: CodeState, parent: Option<usize>, action_taken: Option<String>, depth: u16) -> Self {
        Self {
            state,
            visits: 0,
            total_reward: 0.0,
            children: Vec::new(),
            parent,
            action_taken,
            is_terminal: false,
            depth,
            untried_actions: Vec::new(),
            node_metadata: NodeMetadata::new(),
        }
    }

    /// Create root node for MCTS tree
    #[inline]
    pub fn create_root(state: CodeState) -> Self {
        Self::new(state, None, None, 0)
    }

    /// Create root node for MCTS tree (alias for backward compatibility)
    #[inline]
    pub fn new_root(state: CodeState) -> Self {
        Self::create_root(state)
    }

    /// Create child node with optimized initialization
    #[inline]
    pub fn create_child(
        state: CodeState,
        parent_index: usize,
        action: String,
        depth: u16,
    ) -> Self {
        Self::new(state, Some(parent_index), Some(action), depth)
    }

    /// Calculate UCB1 value for node selection with blazing-fast computation
    #[inline]
    pub fn ucb1_value(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }

        let exploitation = self.total_reward / self.visits as f64;
        let exploration = exploration_constant * 
            ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        
        exploitation + exploration
    }

    /// Update node statistics with new reward
    #[inline]
    pub fn update(&mut self, reward: f64) {
        self.visits += 1;
        self.total_reward += reward;
        self.node_metadata.last_update_time = std::time::Instant::now();
        self.node_metadata.update_count += 1;
    }

    /// Get average reward for this node
    #[inline]
    pub fn average_reward(&self) -> f64 {
        if self.visits > 0 {
            self.total_reward / self.visits as f64
        } else {
            0.0
        }
    }

    /// Check if node is fully expanded
    #[inline]
    pub fn is_fully_expanded(&self) -> bool {
        self.untried_actions.is_empty()
    }

    /// Check if node is leaf (no children)
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Add child node index
    #[inline]
    pub fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }

    /// Remove untried action and return it
    #[inline]
    pub fn pop_untried_action(&mut self) -> Option<String> {
        self.untried_actions.pop()
    }

    /// Add untried action
    #[inline]
    pub fn add_untried_action(&mut self, action: String) {
        self.untried_actions.push(action);
    }

    /// Get confidence interval for node value
    #[inline]
    pub fn confidence_interval(&self, confidence_level: f64) -> (f64, f64) {
        if self.visits < 2 {
            return (0.0, 0.0);
        }

        let mean = self.average_reward();
        let std_error = (mean * (1.0 - mean) / self.visits as f64).sqrt();
        let z_score = match confidence_level {
            0.95 => 1.96,
            0.99 => 2.576,
            _ => 1.645, // 90% confidence
        };

        let margin = z_score * std_error;
        (mean - margin, mean + margin)
    }

    /// Calculate node priority for selection
    #[inline]
    pub fn selection_priority(&self, parent_visits: u32, exploration_constant: f64) -> f64 {
        let ucb1 = self.ucb1_value(parent_visits, exploration_constant);
        let depth_bonus = 1.0 / (1.0 + self.depth as f64 * 0.1);
        let state_quality = self.state.performance_score();
        
        ucb1 * 0.7 + depth_bonus * 0.2 + state_quality * 0.1
    }

    /// Check if node should be pruned based on performance
    #[inline]
    pub fn should_prune(&self, min_visits: u32, min_reward: f64) -> bool {
        self.visits >= min_visits && self.average_reward() < min_reward
    }

    /// Get node efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.visits == 0 {
            return 0.0;
        }
        
        let reward_efficiency = self.average_reward();
        let visit_efficiency = 1.0 / (1.0 + self.visits as f64 * 0.01);
        let depth_efficiency = 1.0 / (1.0 + self.depth as f64 * 0.05);
        
        reward_efficiency * 0.6 + visit_efficiency * 0.2 + depth_efficiency * 0.2
    }
}

/// Node metadata for enhanced tracking
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub creation_time: std::time::Instant,
    pub last_update_time: std::time::Instant,
    pub update_count: u32,
    pub expansion_count: u32,
    pub simulation_count: u32,
    pub backpropagation_count: u32,
    pub quality_metrics: QualityMetrics,
    pub performance_history: Vec<f64>,
}

impl NodeMetadata {
    /// Create new node metadata with current timestamp
    #[inline]
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            creation_time: now,
            last_update_time: now,
            update_count: 0,
            expansion_count: 0,
            simulation_count: 0,
            backpropagation_count: 0,
            quality_metrics: QualityMetrics::new(),
            performance_history: Vec::new(),
        }
    }

    /// Record expansion operation
    #[inline]
    pub fn record_expansion(&mut self) {
        self.expansion_count += 1;
        self.last_update_time = std::time::Instant::now();
    }

    /// Record simulation operation
    #[inline]
    pub fn record_simulation(&mut self) {
        self.simulation_count += 1;
        self.last_update_time = std::time::Instant::now();
    }

    /// Record backpropagation operation
    #[inline]
    pub fn record_backpropagation(&mut self) {
        self.backpropagation_count += 1;
        self.last_update_time = std::time::Instant::now();
    }

    /// Add performance measurement to history
    #[inline]
    pub fn add_performance_measurement(&mut self, performance: f64) {
        self.performance_history.push(performance);
        
        // Keep only recent measurements for memory efficiency
        if self.performance_history.len() > 100 {
            self.performance_history.remove(0);
        }
    }

    /// Calculate average performance from history
    #[inline]
    pub fn average_performance(&self) -> f64 {
        if self.performance_history.is_empty() {
            return 0.0;
        }
        
        self.performance_history.iter().sum::<f64>() / self.performance_history.len() as f64
    }

    /// Get performance trend (positive = improving, negative = degrading)
    #[inline]
    pub fn performance_trend(&self) -> f64 {
        if self.performance_history.len() < 2 {
            return 0.0;
        }
        
        let recent_half = self.performance_history.len() / 2;
        let recent_avg: f64 = self.performance_history[recent_half..].iter().sum::<f64>() 
            / (self.performance_history.len() - recent_half) as f64;
        let early_avg: f64 = self.performance_history[..recent_half].iter().sum::<f64>() 
            / recent_half as f64;
        
        recent_avg - early_avg
    }

    /// Calculate node age in milliseconds
    #[inline]
    pub fn age_ms(&self) -> u64 {
        self.creation_time.elapsed().as_millis() as u64
    }

    /// Calculate time since last update in milliseconds
    #[inline]
    pub fn time_since_last_update_ms(&self) -> u64 {
        self.last_update_time.elapsed().as_millis() as u64
    }
}

/// Quality metrics for node evaluation
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub stability_score: f64,
    pub convergence_rate: f64,
    pub exploration_diversity: f64,
    pub prediction_accuracy: f64,
}

impl QualityMetrics {
    /// Create new quality metrics with default values
    #[inline]
    pub fn new() -> Self {
        Self {
            stability_score: 0.5,
            convergence_rate: 0.5,
            exploration_diversity: 0.5,
            prediction_accuracy: 0.5,
        }
    }

    /// Update stability score based on reward variance
    #[inline]
    pub fn update_stability(&mut self, reward_variance: f64) {
        self.stability_score = (1.0 / (1.0 + reward_variance)).clamp(0.0, 1.0);
    }

    /// Update convergence rate based on improvement rate
    #[inline]
    pub fn update_convergence(&mut self, improvement_rate: f64) {
        self.convergence_rate = improvement_rate.clamp(0.0, 1.0);
    }

    /// Update exploration diversity based on action variety
    #[inline]
    pub fn update_diversity(&mut self, unique_actions: usize, total_actions: usize) {
        if total_actions > 0 {
            self.exploration_diversity = (unique_actions as f64 / total_actions as f64).clamp(0.0, 1.0);
        }
    }

    /// Update prediction accuracy based on actual vs predicted outcomes
    #[inline]
    pub fn update_accuracy(&mut self, predicted: f64, actual: f64) {
        let error = (predicted - actual).abs();
        let accuracy = 1.0 / (1.0 + error);
        
        // Exponential moving average
        let alpha = 0.2;
        self.prediction_accuracy = alpha * accuracy + (1.0 - alpha) * self.prediction_accuracy;
    }

    /// Calculate overall quality score
    #[inline]
    pub fn overall_quality(&self) -> f64 {
        (self.stability_score * 0.3 + 
         self.convergence_rate * 0.3 + 
         self.exploration_diversity * 0.2 + 
         self.prediction_accuracy * 0.2).clamp(0.0, 1.0)
    }
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}
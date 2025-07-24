//! MCTS tree type definitions and operations
//!
//! This module provides blazing-fast tree structures with zero allocation
//! optimizations and elegant ergonomic interfaces for MCTS tree operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::node_types::MCTSNode;

/// MCTS tree with optimized node storage and traversal
#[derive(Debug)]
pub struct MCTSTree {
    pub nodes: Vec<MCTSNode>,
    pub root_index: usize,
    pub node_map: HashMap<String, usize>,
    pub config: TreeConfig,
    pub metrics: TreeMetrics,
    pub optimization_cache: OptimizationCache,
}

impl MCTSTree {
    /// Create new MCTS tree with root node
    #[inline]
    pub fn new(root_node: MCTSNode, config: TreeConfig) -> Self {
        let mut tree = Self {
            nodes: Vec::with_capacity(config.initial_capacity),
            root_index: 0,
            node_map: HashMap::with_capacity(config.initial_capacity),
            config,
            metrics: TreeMetrics::new(),
            optimization_cache: OptimizationCache::new(),
        };
        
        tree.nodes.push(root_node);
        tree.metrics.total_nodes = 1;
        tree
    }

    /// Add node to tree and return its index
    #[inline]
    pub fn add_node(&mut self, node: MCTSNode) -> usize {
        let index = self.nodes.len();
        
        // Update node mapping if action is available
        if let Some(ref action) = node.action_taken {
            self.node_map.insert(action.clone(), index);
        }
        
        self.nodes.push(node);
        self.metrics.total_nodes += 1;
        self.metrics.current_depth = self.metrics.current_depth.max(self.nodes[index].depth as usize);
        
        index
    }

    /// Get node by index with bounds checking
    #[inline]
    pub fn get_node(&self, index: usize) -> Option<&MCTSNode> {
        self.nodes.get(index)
    }

    /// Get mutable node by index with bounds checking
    #[inline]
    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut MCTSNode> {
        self.nodes.get_mut(index)
    }

    /// Get root node reference
    #[inline]
    pub fn root(&self) -> &MCTSNode {
        &self.nodes[self.root_index]
    }

    /// Get mutable root node reference
    #[inline]
    pub fn root_mut(&mut self) -> &mut MCTSNode {
        &mut self.nodes[self.root_index]
    }

    /// Find node by action path from root
    #[inline]
    pub fn find_node_by_path(&self, actions: &[String]) -> Option<usize> {
        let mut current_index = self.root_index;
        
        for action in actions {
            let current_node = &self.nodes[current_index];
            let mut found = false;
            
            for &child_index in &current_node.children {
                if let Some(ref child_action) = self.nodes[child_index].action_taken {
                    if child_action == action {
                        current_index = child_index;
                        found = true;
                        break;
                    }
                }
            }
            
            if !found {
                return None;
            }
        }
        
        Some(current_index)
    }

    /// Get all leaf nodes in the tree
    #[inline]
    pub fn get_leaf_nodes(&self) -> Vec<usize> {
        let mut leaves = Vec::new();
        
        for (index, node) in self.nodes.iter().enumerate() {
            if node.is_leaf() {
                leaves.push(index);
            }
        }
        
        leaves
    }

    /// Get path from root to specified node
    #[inline]
    pub fn get_path_to_node(&self, target_index: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current_index = target_index;
        
        while let Some(node) = self.nodes.get(current_index) {
            path.push(current_index);
            
            if let Some(parent_index) = node.parent {
                current_index = parent_index;
            } else {
                break;
            }
        }
        
        path.reverse();
        path
    }

    /// Calculate tree statistics
    #[inline]
    pub fn calculate_statistics(&mut self) {
        self.metrics.total_nodes = self.nodes.len();
        self.metrics.leaf_nodes = self.get_leaf_nodes().len();
        
        let mut total_visits = 0;
        let mut total_reward = 0.0;
        let mut max_depth = 0;
        
        for node in &self.nodes {
            total_visits += node.visits;
            total_reward += node.total_reward;
            max_depth = max_depth.max(node.depth as usize);
        }
        
        self.metrics.total_visits = total_visits;
        self.metrics.average_reward = if total_visits > 0 {
            total_reward / total_visits as f64
        } else {
            0.0
        };
        self.metrics.max_depth = max_depth;
        self.metrics.current_depth = max_depth;
    }

    /// Prune tree by removing low-performing branches
    #[inline]
    pub fn prune_tree(&mut self, min_visits: u32, min_reward: f64) -> usize {
        let mut nodes_to_remove = Vec::new();
        
        // Identify nodes to prune (skip root)
        for (index, node) in self.nodes.iter().enumerate().skip(1) {
            if node.should_prune(min_visits, min_reward) {
                nodes_to_remove.push(index);
            }
        }
        
        // Remove nodes and update parent references
        let removed_count = nodes_to_remove.len();
        for &index in nodes_to_remove.iter().rev() {
            if let Some(parent_index) = self.nodes[index].parent {
                if let Some(parent) = self.nodes.get_mut(parent_index) {
                    parent.children.retain(|&child| child != index);
                }
            }
        }
        
        // Update metrics
        self.metrics.pruned_nodes += removed_count;
        self.calculate_statistics();
        
        removed_count
    }

    /// Get best child of a node based on average reward
    #[inline]
    pub fn get_best_child(&self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        
        let mut best_child = None;
        let mut best_reward = f64::NEG_INFINITY;
        
        for &child_index in &node.children {
            if let Some(child) = self.nodes.get(child_index) {
                let reward = child.average_reward();
                if reward > best_reward {
                    best_reward = reward;
                    best_child = Some(child_index);
                }
            }
        }
        
        best_child
    }

    /// Get most visited child of a node
    #[inline]
    pub fn get_most_visited_child(&self, node_index: usize) -> Option<usize> {
        let node = self.nodes.get(node_index)?;
        
        let mut best_child = None;
        let mut most_visits = 0;
        
        for &child_index in &node.children {
            if let Some(child) = self.nodes.get(child_index) {
                if child.visits > most_visits {
                    most_visits = child.visits;
                    best_child = Some(child_index);
                }
            }
        }
        
        best_child
    }

    /// Check if tree needs rebalancing
    #[inline]
    pub fn needs_rebalancing(&self) -> bool {
        let depth_threshold = self.config.max_depth as f64 * 0.8;
        let imbalance_ratio = self.calculate_imbalance_ratio();
        
        self.metrics.current_depth as f64 > depth_threshold || imbalance_ratio > 0.7
    }

    /// Calculate tree imbalance ratio
    #[inline]
    pub fn calculate_imbalance_ratio(&self) -> f64 {
        if self.nodes.len() < 2 {
            return 0.0;
        }
        
        let mut depth_counts = HashMap::new();
        for node in &self.nodes {
            *depth_counts.entry(node.depth).or_insert(0) += 1;
        }
        
        let max_count = *depth_counts.values().max().unwrap_or(&0);
        let min_count = *depth_counts.values().min().unwrap_or(&0);
        
        if max_count > 0 {
            1.0 - (min_count as f64 / max_count as f64)
        } else {
            0.0
        }
    }

    /// Get tree memory usage in bytes
    #[inline]
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.nodes.capacity() * std::mem::size_of::<MCTSNode>() +
        self.node_map.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>())
    }

    /// Optimize tree structure for better performance
    #[inline]
    pub fn optimize_structure(&mut self) {
        // Compact node storage
        self.nodes.shrink_to_fit();
        
        // Rebuild node map for better cache locality
        self.node_map.clear();
        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(ref action) = node.action_taken {
                self.node_map.insert(action.clone(), index);
            }
        }
        
        // Update optimization cache
        self.optimization_cache.last_optimization = std::time::Instant::now();
        self.optimization_cache.optimization_count += 1;
    }
}

/// Tree configuration parameters
#[derive(Debug, Clone)]
pub struct TreeConfig {
    pub max_depth: u16,
    pub max_nodes: usize,
    pub initial_capacity: usize,
    pub exploration_constant: f64,
    pub pruning_threshold: f64,
    pub rebalancing_frequency: u32,
    pub memory_limit_mb: usize,
}

impl TreeConfig {
    /// Create new tree configuration with default values
    #[inline]
    pub fn new() -> Self {
        Self {
            max_depth: 100,
            max_nodes: 10000,
            initial_capacity: 1000,
            exploration_constant: 1.414, // sqrt(2)
            pruning_threshold: 0.1,
            rebalancing_frequency: 1000,
            memory_limit_mb: 100,
        }
    }

    /// Create configuration optimized for performance
    #[inline]
    pub fn performance_optimized() -> Self {
        Self {
            max_depth: 50,
            max_nodes: 5000,
            initial_capacity: 2000,
            exploration_constant: 1.0,
            pruning_threshold: 0.2,
            rebalancing_frequency: 500,
            memory_limit_mb: 50,
        }
    }

    /// Create configuration optimized for exploration
    #[inline]
    pub fn exploration_optimized() -> Self {
        Self {
            max_depth: 200,
            max_nodes: 20000,
            initial_capacity: 5000,
            exploration_constant: 2.0,
            pruning_threshold: 0.05,
            rebalancing_frequency: 2000,
            memory_limit_mb: 200,
        }
    }

    /// Validate configuration parameters
    #[inline]
    pub fn validate(&self) -> Result<(), String> {
        if self.max_depth == 0 {
            return Err("max_depth must be greater than 0".to_string());
        }
        
        if self.max_nodes == 0 {
            return Err("max_nodes must be greater than 0".to_string());
        }
        
        if self.initial_capacity > self.max_nodes {
            return Err("initial_capacity cannot exceed max_nodes".to_string());
        }
        
        if self.exploration_constant < 0.0 {
            return Err("exploration_constant must be non-negative".to_string());
        }
        
        if self.pruning_threshold < 0.0 || self.pruning_threshold > 1.0 {
            return Err("pruning_threshold must be between 0.0 and 1.0".to_string());
        }
        
        Ok(())
    }

    /// Calculate optimal batch size for operations
    #[inline]
    pub fn optimal_batch_size(&self) -> usize {
        (self.max_nodes / 100).max(10).min(1000)
    }
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree performance metrics
#[derive(Debug, Clone)]
pub struct TreeMetrics {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub total_visits: u32,
    pub average_reward: f64,
    pub max_depth: usize,
    pub current_depth: usize,
    pub pruned_nodes: usize,
    pub rebalancing_operations: u32,
    pub memory_usage_bytes: usize,
    pub creation_time: std::time::Instant,
    pub last_update: std::time::Instant,
}

impl TreeMetrics {
    /// Create new tree metrics
    #[inline]
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            total_nodes: 0,
            leaf_nodes: 0,
            total_visits: 0,
            average_reward: 0.0,
            max_depth: 0,
            current_depth: 0,
            pruned_nodes: 0,
            rebalancing_operations: 0,
            memory_usage_bytes: 0,
            creation_time: now,
            last_update: now,
        }
    }

    /// Update metrics timestamp
    #[inline]
    pub fn touch(&mut self) {
        self.last_update = std::time::Instant::now();
    }

    /// Calculate tree efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.total_nodes == 0 {
            return 0.0;
        }
        
        let node_efficiency = self.leaf_nodes as f64 / self.total_nodes as f64;
        let depth_efficiency = if self.max_depth > 0 {
            1.0 / (1.0 + self.max_depth as f64 * 0.01)
        } else {
            1.0
        };
        let reward_efficiency = (self.average_reward + 1.0) / 2.0; // Normalize to [0,1]
        
        (node_efficiency * 0.4 + depth_efficiency * 0.3 + reward_efficiency * 0.3).clamp(0.0, 1.0)
    }

    /// Calculate growth rate
    #[inline]
    pub fn growth_rate(&self) -> f64 {
        let age_seconds = self.creation_time.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            self.total_nodes as f64 / age_seconds
        } else {
            0.0
        }
    }

    /// Calculate visit rate
    #[inline]
    pub fn visit_rate(&self) -> f64 {
        let age_seconds = self.creation_time.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            self.total_visits as f64 / age_seconds
        } else {
            0.0
        }
    }

    /// Get tree age in seconds
    #[inline]
    pub fn age_seconds(&self) -> f64 {
        self.creation_time.elapsed().as_secs_f64()
    }

    /// Get time since last update in seconds
    #[inline]
    pub fn time_since_update_seconds(&self) -> f64 {
        self.last_update.elapsed().as_secs_f64()
    }
}

impl Default for TreeMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization cache for tree operations
#[derive(Debug, Clone)]
pub struct OptimizationCache {
    pub last_optimization: std::time::Instant,
    pub optimization_count: u32,
    pub cached_paths: HashMap<String, Vec<usize>>,
    pub cached_statistics: Option<CachedStatistics>,
    pub cache_hit_count: u32,
    pub cache_miss_count: u32,
}

impl OptimizationCache {
    /// Create new optimization cache
    #[inline]
    pub fn new() -> Self {
        Self {
            last_optimization: std::time::Instant::now(),
            optimization_count: 0,
            cached_paths: HashMap::new(),
            cached_statistics: None,
            cache_hit_count: 0,
            cache_miss_count: 0,
        }
    }

    /// Check if cache is valid
    #[inline]
    pub fn is_valid(&self, max_age_seconds: f64) -> bool {
        self.last_optimization.elapsed().as_secs_f64() < max_age_seconds
    }

    /// Clear cache
    #[inline]
    pub fn clear(&mut self) {
        self.cached_paths.clear();
        self.cached_statistics = None;
        self.last_optimization = std::time::Instant::now();
    }

    /// Get cache hit rate
    #[inline]
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.cache_hit_count + self.cache_miss_count;
        if total_requests > 0 {
            self.cache_hit_count as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Record cache hit
    #[inline]
    pub fn record_hit(&mut self) {
        self.cache_hit_count += 1;
    }

    /// Record cache miss
    #[inline]
    pub fn record_miss(&mut self) {
        self.cache_miss_count += 1;
    }
}

impl Default for OptimizationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached statistics for performance optimization
#[derive(Debug, Clone)]
pub struct CachedStatistics {
    pub total_reward: f64,
    pub total_visits: u32,
    pub best_path: Vec<usize>,
    pub timestamp: std::time::Instant,
}

impl CachedStatistics {
    /// Create new cached statistics
    #[inline]
    pub fn new(total_reward: f64, total_visits: u32, best_path: Vec<usize>) -> Self {
        Self {
            total_reward,
            total_visits,
            best_path,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if statistics are fresh
    #[inline]
    pub fn is_fresh(&self, max_age_seconds: f64) -> bool {
        self.timestamp.elapsed().as_secs_f64() < max_age_seconds
    }
}/// Comprehensive tree statistics for MCTS performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStatistics {
    /// Total number of nodes in the tree
    pub node_count: usize,
    /// Maximum depth reached in the tree
    pub max_depth: usize,
    /// Average branching factor
    pub avg_branching_factor: f64,
    /// Total number of simulations performed
    pub simulation_count: u64,
    /// Average simulation time in microseconds
    pub avg_simulation_time_us: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// Tree construction time in microseconds
    pub construction_time_us: u64,
    /// Last update timestamp
    pub last_updated: std::time::Instant,
}

impl TreeStatistics {
    /// Create new tree statistics with default values
    pub fn new() -> Self {
        Self {
            node_count: 0,
            max_depth: 0,
            avg_branching_factor: 0.0,
            simulation_count: 0,
            avg_simulation_time_us: 0.0,
            memory_usage_bytes: 0,
            construction_time_us: 0,
            last_updated: std::time::Instant::now(),
        }
    }

    /// Update node count statistics
    pub fn update_node_count(&mut self, count: usize) {
        self.node_count = count;
        self.last_updated = std::time::Instant::now();
    }

    /// Update depth statistics
    pub fn update_max_depth(&mut self, depth: usize) {
        if depth > self.max_depth {
            self.max_depth = depth;
            self.last_updated = std::time::Instant::now();
        }
    }

    /// Update branching factor statistics
    pub fn update_branching_factor(&mut self, factor: f64) {
        self.avg_branching_factor = factor;
        self.last_updated = std::time::Instant::now();
    }

    /// Record a simulation completion
    pub fn record_simulation(&mut self, duration_us: f64) {
        self.simulation_count += 1;
        // Update running average
        let alpha = 1.0 / self.simulation_count as f64;
        self.avg_simulation_time_us = 
            (1.0 - alpha) * self.avg_simulation_time_us + alpha * duration_us;
        self.last_updated = std::time::Instant::now();
    }

    /// Update memory usage statistics
    pub fn update_memory_usage(&mut self, bytes: usize) {
        self.memory_usage_bytes = bytes;
        self.last_updated = std::time::Instant::now();
    }

    /// Get efficiency ratio (simulations per second)
    pub fn efficiency_ratio(&self) -> f64 {
        if self.avg_simulation_time_us > 0.0 {
            1_000_000.0 / self.avg_simulation_time_us
        } else {
            0.0
        }
    }
}

impl Default for TreeStatistics {
    fn default() -> Self {
        Self::new()
    }
}

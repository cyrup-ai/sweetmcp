//! Tree pruning and optimization for quantum MCTS expansion
//!
//! This module provides intelligent tree pruning algorithms with quantum-aware
//! node selection, memory optimization, and performance-oriented tree maintenance
//! for efficient MCTS tree management.

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// Quantum-aware tree pruning engine
pub struct QuantumTreePruner {
    /// Configuration for pruning parameters
    config: QuantumMCTSConfig,
    
    /// Maximum tree size before pruning
    max_tree_size: usize,
    
    /// Minimum node visits before pruning consideration
    min_visits_threshold: u32,
    
    /// Amplitude threshold for quantum pruning
    amplitude_threshold: f64,
    
    /// Pruning statistics
    stats: PruningStats,
}

/// Node priority for pruning decisions
#[derive(Debug, Clone)]
struct NodePriority {
    node_id: String,
    priority_score: f64,
    amplitude_magnitude: f64,
    visit_count: u32,
    depth: usize,
}

impl PartialEq for NodePriority {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for NodePriority {}

impl PartialOrd for NodePriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodePriority {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority score means higher priority (reverse for min-heap behavior)
        other.priority_score.partial_cmp(&self.priority_score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Pruning statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct PruningStats {
    /// Total pruning operations performed
    pub total_prunings: u64,
    
    /// Total nodes pruned
    pub nodes_pruned: u64,
    
    /// Average pruning time (microseconds)
    pub avg_pruning_time_us: f64,
    
    /// Memory saved by pruning (estimated bytes)
    pub memory_saved_bytes: u64,
    
    /// Last pruning timestamp
    pub last_pruning: Option<std::time::Instant>,
}

/// Pruning strategy types
#[derive(Debug, Clone, Copy)]
pub enum PruningStrategy {
    /// Prune based on visit count
    VisitBased,
    
    /// Prune based on quantum amplitude
    AmplitudeBased,
    
    /// Prune based on node depth
    DepthBased,
    
    /// Hybrid strategy combining multiple factors
    Hybrid,
    
    /// Prune least recently used nodes
    LRU,
}

impl QuantumTreePruner {
    /// Create new quantum tree pruner
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self {
            max_tree_size: config.max_tree_size.unwrap_or(10000),
            min_visits_threshold: config.min_visits_for_pruning.unwrap_or(5),
            amplitude_threshold: config.amplitude_pruning_threshold.unwrap_or(0.01),
            config,
            stats: PruningStats::default(),
        }
    }

    /// Check if tree needs pruning
    pub fn needs_pruning(&self, tree_size: usize) -> bool {
        tree_size > self.max_tree_size
    }

    /// Perform intelligent tree pruning
    pub async fn prune_tree(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        strategy: PruningStrategy,
    ) -> Result<PruningResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        
        let initial_size = {
            let tree_read = tree.read().await;
            tree_read.len()
        };

        if !self.needs_pruning(initial_size) {
            return Ok(PruningResult {
                nodes_pruned: 0,
                initial_size,
                final_size: initial_size,
                memory_saved_bytes: 0,
                pruning_time_us: 0,
            });
        }

        // Identify nodes for pruning
        let nodes_to_prune = self.identify_pruning_candidates(tree, strategy).await?;
        
        // Perform actual pruning
        let pruned_count = self.execute_pruning(tree, &nodes_to_prune).await?;
        
        let final_size = {
            let tree_read = tree.read().await;
            tree_read.len()
        };

        let pruning_time = start_time.elapsed().as_micros() as f64;
        let estimated_memory_saved = pruned_count * 1024; // Rough estimate

        // Update statistics
        self.stats.total_prunings += 1;
        self.stats.nodes_pruned += pruned_count as u64;
        self.stats.avg_pruning_time_us = (self.stats.avg_pruning_time_us * (self.stats.total_prunings - 1) as f64 + pruning_time) / self.stats.total_prunings as f64;
        self.stats.memory_saved_bytes += estimated_memory_saved;
        self.stats.last_pruning = Some(start_time);

        debug!("Pruned {} nodes from tree ({}→{})", pruned_count, initial_size, final_size);

        Ok(PruningResult {
            nodes_pruned: pruned_count,
            initial_size,
            final_size,
            memory_saved_bytes: estimated_memory_saved,
            pruning_time_us: pruning_time as u64,
        })
    }

    /// Identify candidates for pruning based on strategy
    async fn identify_pruning_candidates(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        strategy: PruningStrategy,
    ) -> Result<Vec<String>, CognitiveError> {
        let tree_read = tree.read().await;
        let mut candidates = Vec::new();

        match strategy {
            PruningStrategy::VisitBased => {
                candidates = self.identify_low_visit_nodes(&tree_read);
            }
            PruningStrategy::AmplitudeBased => {
                candidates = self.identify_low_amplitude_nodes(&tree_read);
            }
            PruningStrategy::DepthBased => {
                candidates = self.identify_deep_nodes(&tree_read);
            }
            PruningStrategy::Hybrid => {
                candidates = self.identify_hybrid_candidates(&tree_read);
            }
            PruningStrategy::LRU => {
                candidates = self.identify_lru_nodes(&tree_read);
            }
        }

        // Ensure we don't prune the root node
        candidates.retain(|id| !id.starts_with("root"));

        // Limit pruning to reasonable amount
        let target_prune_count = (tree_read.len() / 4).min(1000); // Prune up to 25% or 1000 nodes
        candidates.truncate(target_prune_count);

        Ok(candidates)
    }

    /// Identify nodes with low visit counts
    fn identify_low_visit_nodes(&self, tree: &HashMap<String, QuantumMCTSNode>) -> Vec<String> {
        let mut priority_heap = BinaryHeap::new();

        for (node_id, node) in tree.iter() {
            if node.visit_count < self.min_visits_threshold {
                priority_heap.push(NodePriority {
                    node_id: node_id.clone(),
                    priority_score: -(node.visit_count as f64), // Negative for min-heap behavior
                    amplitude_magnitude: node.amplitude.norm(),
                    visit_count: node.visit_count,
                    depth: node_id.matches('_').count(), // Rough depth estimation
                });
            }
        }

        priority_heap.into_iter().map(|p| p.node_id).collect()
    }

    /// Identify nodes with low quantum amplitudes
    fn identify_low_amplitude_nodes(&self, tree: &HashMap<String, QuantumMCTSNode>) -> Vec<String> {
        let mut priority_heap = BinaryHeap::new();

        for (node_id, node) in tree.iter() {
            let amplitude_magnitude = node.amplitude.norm();
            if amplitude_magnitude < self.amplitude_threshold {
                priority_heap.push(NodePriority {
                    node_id: node_id.clone(),
                    priority_score: -amplitude_magnitude, // Negative for min-heap behavior
                    amplitude_magnitude,
                    visit_count: node.visit_count,
                    depth: node_id.matches('_').count(),
                });
            }
        }

        priority_heap.into_iter().map(|p| p.node_id).collect()
    }

    /// Identify nodes at excessive depth
    fn identify_deep_nodes(&self, tree: &HashMap<String, QuantumMCTSNode>) -> Vec<String> {
        let max_depth = self.config.max_tree_depth.unwrap_or(20);
        let mut deep_nodes = Vec::new();

        for (node_id, _node) in tree.iter() {
            let depth = node_id.matches('_').count(); // Rough depth estimation
            if depth > max_depth {
                deep_nodes.push(node_id.clone());
            }
        }

        // Sort by depth (deepest first)
        deep_nodes.sort_by(|a, b| {
            let depth_a = a.matches('_').count();
            let depth_b = b.matches('_').count();
            depth_b.cmp(&depth_a)
        });

        deep_nodes
    }

    /// Identify candidates using hybrid strategy
    fn identify_hybrid_candidates(&self, tree: &HashMap<String, QuantumMCTSNode>) -> Vec<String> {
        let mut priority_heap = BinaryHeap::new();

        for (node_id, node) in tree.iter() {
            let amplitude_magnitude = node.amplitude.norm();
            let depth = node_id.matches('_').count();
            let visit_count = node.visit_count;

            // Hybrid score combining multiple factors
            let amplitude_score = if amplitude_magnitude < self.amplitude_threshold { -2.0 } else { 0.0 };
            let visit_score = if visit_count < self.min_visits_threshold { -1.0 } else { 0.0 };
            let depth_score = if depth > 15 { -0.5 } else { 0.0 };

            let hybrid_score = amplitude_score + visit_score + depth_score;

            if hybrid_score < -0.5 { // Only consider nodes with poor scores
                priority_heap.push(NodePriority {
                    node_id: node_id.clone(),
                    priority_score: hybrid_score,
                    amplitude_magnitude,
                    visit_count,
                    depth,
                });
            }
        }

        priority_heap.into_iter().map(|p| p.node_id).collect()
    }

    /// Identify least recently used nodes
    fn identify_lru_nodes(&self, tree: &HashMap<String, QuantumMCTSNode>) -> Vec<String> {
        let mut nodes_with_time: Vec<(String, std::time::Instant)> = tree.iter()
            .map(|(id, node)| (id.clone(), node.last_update))
            .collect();

        // Sort by last update time (oldest first)
        nodes_with_time.sort_by(|a, b| a.1.cmp(&b.1));

        // Take oldest 25%
        let lru_count = (nodes_with_time.len() / 4).max(1);
        nodes_with_time.into_iter()
            .take(lru_count)
            .map(|(id, _)| id)
            .collect()
    }

    /// Execute the actual pruning of identified nodes
    async fn execute_pruning(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        nodes_to_prune: &[String],
    ) -> Result<usize, CognitiveError> {
        let mut tree_write = tree.write().await;
        let mut pruned_count = 0;

        for node_id in nodes_to_prune {
            if let Some(node) = tree_write.remove(node_id) {
                pruned_count += 1;
                
                // Update parent nodes to remove references to pruned children
                if let Some(parent_id) = &node.parent_id {
                    if let Some(parent_node) = tree_write.get_mut(parent_id) {
                        parent_node.remove_child(node_id);
                    }
                }
                
                // Recursively prune orphaned children
                let children_to_prune: Vec<String> = node.children.values().cloned().collect();
                for child_id in children_to_prune {
                    if tree_write.contains_key(&child_id) {
                        self.prune_subtree(&mut tree_write, &child_id, &mut pruned_count);
                    }
                }
            }
        }

        Ok(pruned_count)
    }

    /// Recursively prune a subtree
    fn prune_subtree(
        &self,
        tree: &mut HashMap<String, QuantumMCTSNode>,
        root_id: &str,
        pruned_count: &mut usize,
    ) {
        if let Some(node) = tree.remove(root_id) {
            *pruned_count += 1;
            
            // Recursively prune children
            for child_id in node.children.values() {
                self.prune_subtree(tree, child_id, pruned_count);
            }
        }
    }

    /// Perform selective pruning to maintain tree balance
    pub async fn selective_prune(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        preserve_promising: bool,
    ) -> Result<PruningResult, CognitiveError> {
        if preserve_promising {
            // Use hybrid strategy to preserve promising nodes
            self.prune_tree(tree, PruningStrategy::Hybrid).await
        } else {
            // Use amplitude-based pruning for aggressive memory management
            self.prune_tree(tree, PruningStrategy::AmplitudeBased).await
        }
    }

    /// Get pruning statistics
    pub fn stats(&self) -> &PruningStats {
        &self.stats
    }

    /// Update pruning configuration
    pub fn update_config(&mut self, config: QuantumMCTSConfig) {
        self.config = config;
        self.max_tree_size = self.config.max_tree_size.unwrap_or(10000);
        self.min_visits_threshold = self.config.min_visits_for_pruning.unwrap_or(5);
        self.amplitude_threshold = self.config.amplitude_pruning_threshold.unwrap_or(0.01);
    }

    /// Reset pruning statistics
    pub fn reset_stats(&mut self) {
        self.stats = PruningStats::default();
    }
}

/// Result of a pruning operation
#[derive(Debug, Clone)]
pub struct PruningResult {
    /// Number of nodes pruned
    pub nodes_pruned: usize,
    
    /// Initial tree size before pruning
    pub initial_size: usize,
    
    /// Final tree size after pruning
    pub final_size: usize,
    
    /// Estimated memory saved in bytes
    pub memory_saved_bytes: u64,
    
    /// Time taken for pruning in microseconds
    pub pruning_time_us: u64,
}

impl PruningResult {
    /// Calculate pruning efficiency (nodes pruned per microsecond)
    pub fn efficiency(&self) -> f64 {
        if self.pruning_time_us > 0 {
            self.nodes_pruned as f64 / self.pruning_time_us as f64
        } else {
            0.0
        }
    }

    /// Calculate memory efficiency (bytes saved per node pruned)
    pub fn memory_efficiency(&self) -> f64 {
        if self.nodes_pruned > 0 {
            self.memory_saved_bytes as f64 / self.nodes_pruned as f64
        } else {
            0.0
        }
    }

    /// Calculate pruning ratio (fraction of tree pruned)
    pub fn pruning_ratio(&self) -> f64 {
        if self.initial_size > 0 {
            self.nodes_pruned as f64 / self.initial_size as f64
        } else {
            0.0
        }
    }
}

/// Extension trait for QuantumMCTSNode to support pruning operations
trait PruningNodeExt {
    fn remove_child(&mut self, child_id: &str);
}

impl PruningNodeExt for QuantumMCTSNode {
    fn remove_child(&mut self, child_id: &str) {
        self.children.retain(|_, id| id != child_id);
    }
}
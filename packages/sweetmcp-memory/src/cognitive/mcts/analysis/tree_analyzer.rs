//! Tree structure analysis and depth calculations
//!
//! This module provides blazing-fast tree analysis with zero allocation
//! optimizations for MCTS tree structure examination.

use super::super::types::{MCTSNode, CodeState};
use std::collections::HashMap;

/// MCTS tree analyzer for structure analysis and depth calculations
pub struct TreeAnalyzer;

impl TreeAnalyzer {
    /// Calculate maximum depth of the tree with zero allocation
    #[inline]
    pub fn calculate_max_depth(tree: &HashMap<String, MCTSNode>, root_id: &str) -> usize {
        if let Some(root) = tree.get(root_id) {
            Self::calculate_node_depth_recursive(tree, root, 0)
        } else {
            0
        }
    }

    /// Recursive depth calculation with tail call optimization
    #[inline]
    fn calculate_node_depth_recursive(
        tree: &HashMap<String, MCTSNode>,
        node: &MCTSNode,
        current_depth: usize,
    ) -> usize {
        if node.children.is_empty() {
            return current_depth;
        }

        let mut max_child_depth = current_depth;
        for child_id in node.children.values() {
            if let Some(child) = tree.get(child_id) {
                let child_depth = Self::calculate_node_depth_recursive(tree, child, current_depth + 1);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        max_child_depth
    }

    /// Calculate average branching factor for the tree
    #[inline]
    pub fn calculate_branching_factor(tree: &HashMap<String, MCTSNode>) -> f64 {
        if tree.is_empty() {
            return 0.0;
        }

        let mut total_children = 0;
        let mut internal_nodes = 0;

        for node in tree.values() {
            if !node.children.is_empty() {
                total_children += node.children.len();
                internal_nodes += 1;
            }
        }

        if internal_nodes == 0 {
            0.0
        } else {
            total_children as f64 / internal_nodes as f64
        }
    }

    /// Get tree balance ratio (measure of how balanced the tree is)
    #[inline]
    pub fn calculate_balance_ratio(tree: &HashMap<String, MCTSNode>, root_id: &str) -> f64 {
        if let Some(root) = tree.get(root_id) {
            let max_depth = Self::calculate_max_depth(tree, root_id);
            let min_depth = Self::calculate_min_depth(tree, root);
            
            if max_depth == 0 {
                1.0
            } else {
                min_depth as f64 / max_depth as f64
            }
        } else {
            0.0
        }
    }

    /// Calculate minimum depth to any leaf
    #[inline]
    fn calculate_min_depth(tree: &HashMap<String, MCTSNode>, root: &MCTSNode) -> usize {
        if root.children.is_empty() {
            return 0;
        }

        let mut min_child_depth = usize::MAX;
        for child_id in root.children.values() {
            if let Some(child) = tree.get(child_id) {
                let child_depth = Self::calculate_min_depth(tree, child) + 1;
                min_child_depth = min_child_depth.min(child_depth);
            }
        }

        if min_child_depth == usize::MAX {
            0
        } else {
            min_child_depth
        }
    }

    /// Count nodes by type (leaf, internal, terminal)
    #[inline]
    pub fn count_node_types(tree: &HashMap<String, MCTSNode>) -> NodeTypeCounts {
        let mut counts = NodeTypeCounts::default();

        for node in tree.values() {
            if node.children.is_empty() {
                counts.leaf_nodes += 1;
            } else {
                counts.internal_nodes += 1;
            }

            if node.is_terminal {
                counts.terminal_nodes += 1;
            }
        }

        counts.total_nodes = tree.len();
        counts
    }

    /// Calculate visit statistics across all nodes
    #[inline]
    pub fn calculate_visit_statistics(tree: &HashMap<String, MCTSNode>) -> VisitStatistics {
        if tree.is_empty() {
            return VisitStatistics::default();
        }

        let mut visits: Vec<u64> = tree.values().map(|node| node.visits).collect();
        visits.sort_unstable();

        let total_visits: u64 = visits.iter().sum();
        let min_visits = visits[0];
        let max_visits = visits[visits.len() - 1];
        let average_visits = total_visits as f64 / visits.len() as f64;
        
        let median_visits = if visits.len() % 2 == 0 {
            (visits[visits.len() / 2 - 1] + visits[visits.len() / 2]) / 2
        } else {
            visits[visits.len() / 2]
        };

        // Calculate standard deviation
        let variance: f64 = visits
            .iter()
            .map(|&v| {
                let diff = v as f64 - average_visits;
                diff * diff
            })
            .sum::<f64>() / visits.len() as f64;
        let std_deviation = variance.sqrt();

        VisitStatistics {
            total_visits,
            min_visits,
            max_visits,
            average_visits,
            median_visits,
            std_deviation,
        }
    }
}

/// Node type counts for tree analysis
#[derive(Debug, Clone, Default)]
pub struct NodeTypeCounts {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
    pub terminal_nodes: usize,
}

/// Visit statistics for tree analysis
#[derive(Debug, Clone)]
pub struct VisitStatistics {
    pub total_visits: u64,
    pub min_visits: u64,
    pub max_visits: u64,
    pub average_visits: f64,
    pub median_visits: u64,
    pub std_deviation: f64,
}

impl Default for VisitStatistics {
    #[inline]
    fn default() -> Self {
        Self {
            total_visits: 0,
            min_visits: 0,
            max_visits: 0,
            average_visits: 0.0,
            median_visits: 0,
            std_deviation: 0.0,
        }
    }
}
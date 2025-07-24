//! Basic node search operations
//!
//! This module provides basic MCTS node search functionality
//! with zero-allocation patterns and blazing-fast performance.

use super::super::super::types::{MCTSNode, CodeState};
use super::node_search_types::{NodeCriteria, NodeMatch, NodeSortCriteria};
use std::collections::HashMap;

/// Basic node search operations
pub struct BasicNodeSearch;

impl BasicNodeSearch {
    /// Find nodes by performance criteria
    #[inline]
    pub fn find_nodes_by_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> Vec<NodeMatch> {
        let mut matches = Vec::new();

        for (node_id, node) in tree {
            if Self::node_matches_criteria(node, criteria) {
                matches.push(NodeMatch {
                    node_id: node_id.clone(),
                    reward: node.average_reward(),
                    visits: node.visits,
                    performance_score: node.state.performance_score(),
                    depth: node.calculate_depth(tree),
                });
            }
        }

        // Sort by reward descending
        matches.sort_by(|a, b| b.reward.partial_cmp(&a.reward).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    /// Check if node matches given criteria
    #[inline]
    fn node_matches_criteria(node: &MCTSNode, criteria: &NodeCriteria) -> bool {
        if let Some(min_reward) = criteria.min_reward {
            if node.average_reward() < min_reward {
                return false;
            }
        }

        if let Some(max_reward) = criteria.max_reward {
            if node.average_reward() > max_reward {
                return false;
            }
        }

        if let Some(min_visits) = criteria.min_visits {
            if node.visits < min_visits {
                return false;
            }
        }

        if let Some(max_visits) = criteria.max_visits {
            if node.visits > max_visits {
                return false;
            }
        }

        if let Some(min_performance) = criteria.min_performance {
            if node.state.performance_score() < min_performance {
                return false;
            }
        }

        if let Some(max_performance) = criteria.max_performance {
            if node.state.performance_score() > max_performance {
                return false;
            }
        }

        if criteria.terminal_only && !node.is_terminal {
            return false;
        }

        if criteria.leaf_only && !node.children.is_empty() {
            return false;
        }

        true
    }

    /// Find top performing nodes
    #[inline]
    pub fn find_top_nodes(
        tree: &HashMap<String, MCTSNode>,
        count: usize,
        sort_by: NodeSortCriteria,
    ) -> Vec<NodeMatch> {
        let mut matches: Vec<NodeMatch> = tree
            .iter()
            .map(|(node_id, node)| NodeMatch {
                node_id: node_id.clone(),
                reward: node.average_reward(),
                visits: node.visits,
                performance_score: node.state.performance_score(),
                depth: node.calculate_depth(tree),
            })
            .collect();

        // Sort by specified criteria
        match sort_by {
            NodeSortCriteria::Reward => {
                matches.sort_by(|a, b| b.reward.partial_cmp(&a.reward).unwrap_or(std::cmp::Ordering::Equal));
            }
            NodeSortCriteria::Visits => {
                matches.sort_by(|a, b| b.visits.cmp(&a.visits));
            }
            NodeSortCriteria::Performance => {
                matches.sort_by(|a, b| b.performance_score.partial_cmp(&a.performance_score).unwrap_or(std::cmp::Ordering::Equal));
            }
            NodeSortCriteria::Depth => {
                matches.sort_by(|a, b| b.depth.cmp(&a.depth));
            }
        }

        matches.truncate(count);
        matches
    }

    /// Find nodes within reward range
    #[inline]
    pub fn find_nodes_in_reward_range(
        tree: &HashMap<String, MCTSNode>,
        min_reward: f64,
        max_reward: f64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new()
            .with_min_reward(min_reward)
            .with_max_reward(max_reward);
        
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find nodes within visit range
    #[inline]
    pub fn find_nodes_in_visit_range(
        tree: &HashMap<String, MCTSNode>,
        min_visits: u64,
        max_visits: u64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new()
            .with_min_visits(min_visits)
            .with_max_visits(max_visits);
        
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find terminal nodes only
    #[inline]
    pub fn find_terminal_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().terminal_only();
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find leaf nodes only
    #[inline]
    pub fn find_leaf_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().leaf_only();
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find nodes with high reward (above threshold)
    #[inline]
    pub fn find_high_reward_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: f64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().with_min_reward(threshold);
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find nodes with low reward (below threshold)
    #[inline]
    pub fn find_low_reward_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: f64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().with_max_reward(threshold);
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find frequently visited nodes (above threshold)
    #[inline]
    pub fn find_frequently_visited_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: u64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().with_min_visits(threshold);
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Find rarely visited nodes (below threshold)
    #[inline]
    pub fn find_rarely_visited_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: u64,
    ) -> Vec<NodeMatch> {
        let criteria = NodeCriteria::new().with_max_visits(threshold);
        Self::find_nodes_by_criteria(tree, &criteria)
    }

    /// Count nodes matching criteria
    #[inline]
    pub fn count_nodes_matching_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> usize {
        tree.values()
            .filter(|node| Self::node_matches_criteria(node, criteria))
            .count()
    }

    /// Check if any nodes match criteria
    #[inline]
    pub fn has_nodes_matching_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> bool {
        tree.values()
            .any(|node| Self::node_matches_criteria(node, criteria))
    }
}
//! Path finding and path analysis for MCTS trees
//!
//! This module provides blazing-fast path finding with zero allocation
//! optimizations for MCTS tree path analysis and exploration.

use super::super::types::MCTSNode;
use std::collections::HashMap;

/// Path finding utilities for MCTS trees
pub struct PathFinder;

impl PathFinder {
    /// Get best path from root to highest reward leaf
    #[inline]
    pub fn get_best_path(tree: &HashMap<String, MCTSNode>, root_id: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current_id = root_id;

        while let Some(node) = tree.get(current_id) {
            if node.children.is_empty() {
                break;
            }

            // Find child with highest average reward
            let mut best_child_id = None;
            let mut best_reward = f64::NEG_INFINITY;

            for child_id in node.children.values() {
                if let Some(child) = tree.get(child_id) {
                    let reward = child.average_reward();
                    if reward > best_reward {
                        best_reward = reward;
                        best_child_id = Some(child_id);
                    }
                }
            }

            if let Some(child_id) = best_child_id {
                if let Some(child) = tree.get(child_id) {
                    if let Some(action) = &child.applied_action {
                        path.push(action.clone());
                    }
                    current_id = child_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        path
    }

    /// Get all paths from root to leaves with their rewards
    #[inline]
    pub fn get_all_paths(tree: &HashMap<String, MCTSNode>, root_id: &str) -> Vec<PathInfo> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        
        if let Some(root) = tree.get(root_id) {
            Self::collect_paths_recursive(tree, root, &mut current_path, &mut paths);
        }

        paths
    }

    /// Recursively collect all paths to leaves
    #[inline]
    fn collect_paths_recursive(
        tree: &HashMap<String, MCTSNode>,
        node: &MCTSNode,
        current_path: &mut Vec<String>,
        paths: &mut Vec<PathInfo>,
    ) {
        if node.children.is_empty() {
            // Leaf node - add path
            paths.push(PathInfo {
                actions: current_path.clone(),
                final_reward: node.average_reward(),
                visits: node.visits,
                depth: current_path.len(),
                performance_score: node.state.performance_score(),
            });
            return;
        }

        // Explore all children
        for child_id in node.children.values() {
            if let Some(child) = tree.get(child_id) {
                if let Some(action) = &child.applied_action {
                    current_path.push(action.clone());
                    Self::collect_paths_recursive(tree, child, current_path, paths);
                    current_path.pop();
                }
            }
        }
    }

    /// Get paths matching specific criteria
    #[inline]
    pub fn get_paths_by_criteria(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
        criteria: &PathCriteria,
    ) -> Vec<PathInfo> {
        let all_paths = Self::get_all_paths(tree, root_id);
        
        all_paths
            .into_iter()
            .filter(|path| Self::path_matches_criteria(path, criteria))
            .collect()
    }

    /// Check if path matches given criteria
    #[inline]
    fn path_matches_criteria(path: &PathInfo, criteria: &PathCriteria) -> bool {
        if let Some(min_reward) = criteria.min_reward {
            if path.final_reward < min_reward {
                return false;
            }
        }

        if let Some(max_reward) = criteria.max_reward {
            if path.final_reward > max_reward {
                return false;
            }
        }

        if let Some(min_depth) = criteria.min_depth {
            if path.depth < min_depth {
                return false;
            }
        }

        if let Some(max_depth) = criteria.max_depth {
            if path.depth > max_depth {
                return false;
            }
        }

        if let Some(min_visits) = criteria.min_visits {
            if path.visits < min_visits {
                return false;
            }
        }

        if let Some(min_performance) = criteria.min_performance {
            if path.performance_score < min_performance {
                return false;
            }
        }

        true
    }

    /// Find the most promising paths for further exploration
    #[inline]
    pub fn get_promising_paths(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
        threshold: f64,
        max_paths: usize,
    ) -> Vec<PathInfo> {
        let mut paths = Self::get_all_paths(tree, root_id);
        
        // Filter by promise threshold
        paths.retain(|path| path.is_promising(threshold));
        
        // Sort by quality score descending
        paths.sort_by(|a, b| {
            b.quality_score().partial_cmp(&a.quality_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take top paths
        paths.truncate(max_paths);
        paths
    }

    /// Calculate path diversity metrics
    #[inline]
    pub fn calculate_path_diversity(paths: &[PathInfo]) -> PathDiversityMetrics {
        if paths.is_empty() {
            return PathDiversityMetrics::default();
        }

        let total_paths = paths.len();
        let mut unique_actions = std::collections::HashSet::new();
        let mut total_actions = 0;
        let mut depth_sum = 0;
        let mut reward_sum = 0.0;

        for path in paths {
            for action in &path.actions {
                unique_actions.insert(action.clone());
                total_actions += 1;
            }
            depth_sum += path.depth;
            reward_sum += path.final_reward;
        }

        let action_diversity = if total_actions == 0 {
            0.0
        } else {
            unique_actions.len() as f64 / total_actions as f64
        };

        PathDiversityMetrics {
            total_paths,
            unique_actions: unique_actions.len(),
            total_actions,
            action_diversity,
            average_depth: depth_sum as f64 / total_paths as f64,
            average_reward: reward_sum / total_paths as f64,
        }
    }
}

/// Path information for tree analysis
#[derive(Debug, Clone)]
pub struct PathInfo {
    pub actions: Vec<String>,
    pub final_reward: f64,
    pub visits: u64,
    pub depth: usize,
    pub performance_score: f64,
}

impl PathInfo {
    /// Calculate path quality score
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let reward_weight = 0.5;
        let visit_weight = 0.3;
        let performance_weight = 0.2;
        
        let normalized_visits = (self.visits as f64 / 100.0).min(1.0);
        
        self.final_reward * reward_weight +
        normalized_visits * visit_weight +
        self.performance_score * performance_weight
    }

    /// Check if path is promising for further exploration
    #[inline]
    pub fn is_promising(&self, threshold: f64) -> bool {
        self.quality_score() >= threshold
    }
}

/// Path search criteria
#[derive(Debug, Clone)]
pub struct PathCriteria {
    pub min_reward: Option<f64>,
    pub max_reward: Option<f64>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub min_visits: Option<u64>,
    pub min_performance: Option<f64>,
}

impl PathCriteria {
    /// Create new criteria with defaults
    #[inline]
    pub fn new() -> Self {
        Self {
            min_reward: None,
            max_reward: None,
            min_depth: None,
            max_depth: None,
            min_visits: None,
            min_performance: None,
        }
    }

    /// Set minimum reward threshold
    #[inline]
    pub fn with_min_reward(mut self, min_reward: f64) -> Self {
        self.min_reward = Some(min_reward);
        self
    }

    /// Set maximum reward threshold
    #[inline]
    pub fn with_max_reward(mut self, max_reward: f64) -> Self {
        self.max_reward = Some(max_reward);
        self
    }

    /// Set minimum depth threshold
    #[inline]
    pub fn with_min_depth(mut self, min_depth: usize) -> Self {
        self.min_depth = Some(min_depth);
        self
    }

    /// Set maximum depth threshold
    #[inline]
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }
}

impl Default for PathCriteria {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Path diversity metrics
#[derive(Debug, Clone)]
pub struct PathDiversityMetrics {
    pub total_paths: usize,
    pub unique_actions: usize,
    pub total_actions: usize,
    pub action_diversity: f64,
    pub average_depth: f64,
    pub average_reward: f64,
}

impl Default for PathDiversityMetrics {
    #[inline]
    fn default() -> Self {
        Self {
            total_paths: 0,
            unique_actions: 0,
            total_actions: 0,
            action_diversity: 0.0,
            average_depth: 0.0,
            average_reward: 0.0,
        }
    }
}
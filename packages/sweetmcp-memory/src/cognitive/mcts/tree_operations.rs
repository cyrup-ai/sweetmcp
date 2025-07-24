//! MCTS tree operations and algorithms
//!
//! This module provides blazing-fast MCTS tree operations with zero allocation
//! optimizations and elegant ergonomic interfaces for tree traversal and manipulation.

use super::types::{MCTSNode, CodeState, TreeStatistics};
use crate::cognitive::types::CognitiveError;
use std::collections::HashMap;
use rand::Rng;

/// MCTS tree operations handler
pub struct TreeOperations {
    exploration_constant: f64,
    max_depth: usize,
    simulation_rollouts: usize,
}

impl TreeOperations {
    /// Create new tree operations handler
    #[inline]
    pub fn new(exploration_constant: f64, max_depth: usize, simulation_rollouts: usize) -> Self {
        Self {
            exploration_constant,
            max_depth,
            simulation_rollouts,
        }
    }

    /// Select a node to expand using UCT (Upper Confidence Bound for Trees)
    #[inline]
    pub fn select(&self, tree: &HashMap<String, MCTSNode>, root_id: &str) -> String {
        let mut current_id = root_id.to_string();

        loop {
            let node = &tree[&current_id];

            // If terminal or has untried actions, return this node
            if node.is_terminal || !node.untried_actions.is_empty() {
                return current_id;
            }

            // If no children, this is terminal
            if node.children.is_empty() {
                return current_id;
            }

            // Select best child using UCT with zero allocation
            let mut best_uct = f64::NEG_INFINITY;
            let mut best_child = None;
            let parent_visits = node.visits;

            for child_id in node.children.values() {
                let child = &tree[child_id];
                let uct = child.uct_value(parent_visits, self.exploration_constant);

                if uct > best_uct {
                    best_uct = uct;
                    best_child = Some(child_id.clone());
                }
            }

            if let Some(child_id) = best_child {
                current_id = child_id;
            } else {
                return current_id;
            }
        }
    }

    /// Expand a node by trying an untried action
    #[inline]
    pub fn expand(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        node_id: &str,
        action_generator: &dyn Fn(&CodeState) -> Vec<String>,
        action_applier: &dyn Fn(&CodeState, &str) -> Result<CodeState, CognitiveError>,
    ) -> Result<Option<String>, CognitiveError> {
        let node = tree.get_mut(node_id).ok_or_else(|| {
            CognitiveError::InvalidState(format!("Node not found: {}", node_id))
        })?;

        // Check if node can be expanded
        if node.is_terminal || node.untried_actions.is_empty() {
            return Ok(None);
        }

        // Get next untried action
        let action = node.pop_untried_action().ok_or_else(|| {
            CognitiveError::InvalidState("No untried actions available".to_string())
        })?;

        // Apply action to get new state
        let new_state = action_applier(&node.state, &action)?;

        // Generate child ID
        let child_id = format!("{}_{}", node_id, tree.len());

        // Generate possible actions for new state
        let possible_actions = action_generator(&new_state);

        // Create child node
        let child_node = MCTSNode::new_child(
            new_state,
            node_id.to_string(),
            action.clone(),
            possible_actions,
        );

        // Add child to tree
        tree.insert(child_id.clone(), child_node);

        // Update parent's children mapping
        let parent_node = tree.get_mut(node_id).unwrap();
        parent_node.add_child(action, child_id.clone());

        Ok(Some(child_id))
    }

    /// Simulate from a node to estimate reward
    #[inline]
    pub fn simulate(
        &self,
        tree: &HashMap<String, MCTSNode>,
        node_id: &str,
        action_generator: &dyn Fn(&CodeState) -> Vec<String>,
        action_applier: &dyn Fn(&CodeState, &str) -> Result<CodeState, CognitiveError>,
    ) -> Result<f64, CognitiveError> {
        let node = tree.get(node_id).ok_or_else(|| {
            CognitiveError::InvalidState(format!("Node not found: {}", node_id))
        })?;

        let mut current_state = node.state.clone();
        let mut depth = 0;
        let mut total_reward = 0.0;

        // Perform multiple rollouts for better estimation
        for _ in 0..self.simulation_rollouts {
            let mut rollout_state = current_state.clone();
            let mut rollout_depth = 0;
            let mut rollout_reward = rollout_state.performance_score();

            // Simulate random actions until max depth or terminal state
            while rollout_depth < self.max_depth {
                let possible_actions = action_generator(&rollout_state);
                
                if possible_actions.is_empty() {
                    break;
                }

                // Select random action
                let action_index = rand::thread_rng().gen_range(0..possible_actions.len());
                let action = &possible_actions[action_index];

                // Apply action
                match action_applier(&rollout_state, action) {
                    Ok(new_state) => {
                        rollout_state = new_state;
                        rollout_reward = rollout_state.performance_score();
                        rollout_depth += 1;
                    }
                    Err(_) => break, // Invalid action, terminate rollout
                }
            }

            total_reward += rollout_reward;
        }

        // Return average reward across rollouts
        Ok(total_reward / self.simulation_rollouts as f64)
    }

    /// Backpropagate reward up the tree
    #[inline]
    pub fn backpropagate(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        mut node_id: String,
        reward: f64,
    ) {
        while let Some(node) = tree.get_mut(&node_id) {
            node.update(reward);
            
            // Move to parent
            if let Some(parent_id) = node.parent.clone() {
                node_id = parent_id;
            } else {
                break;
            }
        }
    }

    /// Get best child by UCT value
    #[inline]
    pub fn get_best_child_uct(
        &self,
        tree: &HashMap<String, MCTSNode>,
        node_id: &str,
    ) -> Option<String> {
        let node = tree.get(node_id)?;
        
        let mut best_uct = f64::NEG_INFINITY;
        let mut best_child = None;

        for child_id in node.children.values() {
            let child = tree.get(child_id)?;
            let uct = child.uct_value(node.visits, self.exploration_constant);

            if uct > best_uct {
                best_uct = uct;
                best_child = Some(child_id.clone());
            }
        }

        best_child
    }

    /// Get best child by average reward
    #[inline]
    pub fn get_best_child_reward(
        &self,
        tree: &HashMap<String, MCTSNode>,
        node_id: &str,
    ) -> Option<String> {
        let node = tree.get(node_id)?;
        
        let mut best_reward = f64::NEG_INFINITY;
        let mut best_child = None;

        for child_id in node.children.values() {
            let child = tree.get(child_id)?;
            let reward = child.average_reward();

            if reward > best_reward {
                best_reward = reward;
                best_child = Some(child_id.clone());
            }
        }

        best_child
    }

    /// Check if node is fully expanded
    #[inline]
    pub fn is_fully_expanded(&self, tree: &HashMap<String, MCTSNode>, node_id: &str) -> bool {
        tree.get(node_id)
            .map(|node| node.is_fully_expanded())
            .unwrap_or(false)
    }

    /// Get node depth from root
    #[inline]
    pub fn get_node_depth(&self, tree: &HashMap<String, MCTSNode>, node_id: &str) -> usize {
        tree.get(node_id)
            .map(|node| node.calculate_depth(tree))
            .unwrap_or(0)
    }

    /// Prune tree to remove low-value branches
    #[inline]
    pub fn prune_tree(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
        min_visits: u64,
        min_reward: f64,
    ) -> usize {
        let mut nodes_to_remove = Vec::new();
        
        // Identify nodes to prune
        for (node_id, node) in tree.iter() {
            if node_id != root_id && 
               (node.visits < min_visits || node.average_reward() < min_reward) {
                nodes_to_remove.push(node_id.clone());
            }
        }

        // Remove identified nodes and update parent references
        let removed_count = nodes_to_remove.len();
        for node_id in &nodes_to_remove {
            if let Some(node) = tree.get(node_id) {
                // Remove from parent's children
                if let Some(parent_id) = &node.parent {
                    if let Some(parent) = tree.get_mut(parent_id) {
                        parent.children.retain(|_, child_id| child_id != node_id);
                    }
                }
            }
            tree.remove(node_id);
        }

        removed_count
    }

    /// Balance tree by redistributing visits
    #[inline]
    pub fn balance_tree(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
        target_balance: f64,
    ) {
        let root_visits = tree.get(root_id).map(|n| n.visits).unwrap_or(0);
        if root_visits == 0 {
            return;
        }

        // Calculate visit distribution
        let mut total_child_visits = 0;
        let mut child_visits = Vec::new();

        if let Some(root) = tree.get(root_id) {
            for child_id in root.children.values() {
                if let Some(child) = tree.get(child_id) {
                    child_visits.push((child_id.clone(), child.visits));
                    total_child_visits += child.visits;
                }
            }
        }

        if total_child_visits == 0 {
            return;
        }

        // Redistribute visits to achieve target balance
        let target_visits_per_child = (total_child_visits as f64 * target_balance) as u64;
        
        for (child_id, current_visits) in child_visits {
            if let Some(child) = tree.get_mut(&child_id) {
                if current_visits < target_visits_per_child {
                    let additional_visits = target_visits_per_child - current_visits;
                    child.visits += additional_visits;
                    child.total_reward += child.average_reward() * additional_visits as f64;
                }
            }
        }
    }

    /// Get tree statistics with zero allocation optimizations
    #[inline]
    pub fn get_tree_statistics(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> TreeStatistics {
        TreeStatistics::from_tree(tree, root_id)
    }

    /// Validate tree consistency
    #[inline]
    pub fn validate_tree_consistency(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> Result<(), CognitiveError> {
        // Check root exists
        if !tree.contains_key(root_id) {
            return Err(CognitiveError::InvalidState("Root node not found".to_string()));
        }

        // Validate all nodes
        for (node_id, node) in tree {
            // Check parent-child consistency
            for child_id in node.children.values() {
                let child = tree.get(child_id).ok_or_else(|| {
                    CognitiveError::InvalidState(format!("Child node not found: {}", child_id))
                })?;

                if child.parent.as_ref() != Some(node_id) {
                    return Err(CognitiveError::InvalidState(format!(
                        "Parent-child inconsistency: {} -> {}",
                        node_id, child_id
                    )));
                }
            }

            // Check parent reference
            if let Some(parent_id) = &node.parent {
                let parent = tree.get(parent_id).ok_or_else(|| {
                    CognitiveError::InvalidState(format!("Parent node not found: {}", parent_id))
                })?;

                if !parent.children.values().any(|child_id| child_id == node_id) {
                    return Err(CognitiveError::InvalidState(format!(
                        "Parent doesn't reference child: {} -> {}",
                        parent_id, node_id
                    )));
                }
            }

            // Check visit consistency
            if node.visits > 0 && node.total_reward.is_nan() {
                return Err(CognitiveError::InvalidState(format!(
                    "Invalid reward for node: {}",
                    node_id
                )));
            }
        }

        Ok(())
    }

    /// Find path from root to best leaf
    #[inline]
    pub fn find_best_path(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> Vec<String> {
        let mut path = Vec::new();
        let mut current_id = root_id.to_string();

        while let Some(node) = tree.get(&current_id) {
            if let Some(best_child_id) = self.get_best_child_reward(tree, &current_id) {
                if let Some(child) = tree.get(&best_child_id) {
                    if let Some(action) = &child.applied_action {
                        path.push(action.clone());
                    }
                    current_id = best_child_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        path
    }

    /// Calculate tree efficiency metrics
    #[inline]
    pub fn calculate_efficiency(
        &self,
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> TreeEfficiency {
        let stats = self.get_tree_statistics(tree, root_id);
        
        let visit_efficiency = if stats.total_nodes > 0 {
            stats.total_visits as f64 / stats.total_nodes as f64
        } else {
            0.0
        };

        let depth_efficiency = if stats.max_depth > 0 {
            stats.best_performance_score / stats.max_depth as f64
        } else {
            0.0
        };

        let exploration_efficiency = stats.convergence_rate;

        TreeEfficiency {
            visit_efficiency,
            depth_efficiency,
            exploration_efficiency,
            overall_efficiency: (visit_efficiency * 0.4 + 
                               depth_efficiency * 0.3 + 
                               exploration_efficiency * 0.3).clamp(0.0, 1.0),
        }
    }

    /// Optimize tree structure for better performance
    #[inline]
    pub fn optimize_tree_structure(
        &self,
        tree: &mut HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> OptimizationResult {
        let initial_nodes = tree.len();
        let initial_efficiency = self.calculate_efficiency(tree, root_id);

        // Prune low-value branches
        let pruned_nodes = self.prune_tree(tree, root_id, 5, 0.1);

        // Balance visit distribution
        self.balance_tree(tree, root_id, 0.8);

        let final_efficiency = self.calculate_efficiency(tree, root_id);

        OptimizationResult {
            initial_nodes,
            final_nodes: tree.len(),
            pruned_nodes,
            initial_efficiency: initial_efficiency.overall_efficiency,
            final_efficiency: final_efficiency.overall_efficiency,
            improvement: final_efficiency.overall_efficiency - initial_efficiency.overall_efficiency,
        }
    }
}

impl Default for TreeOperations {
    #[inline]
    fn default() -> Self {
        Self::new(1.41, 100, 5) // sqrt(2), reasonable defaults
    }
}

/// Tree efficiency metrics
#[derive(Debug, Clone)]
pub struct TreeEfficiency {
    pub visit_efficiency: f64,
    pub depth_efficiency: f64,
    pub exploration_efficiency: f64,
    pub overall_efficiency: f64,
}

impl TreeEfficiency {
    /// Check if efficiency meets threshold
    #[inline]
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.overall_efficiency >= threshold
    }

    /// Get efficiency category
    #[inline]
    pub fn efficiency_category(&self) -> EfficiencyCategory {
        match self.overall_efficiency {
            x if x >= 0.9 => EfficiencyCategory::Excellent,
            x if x >= 0.7 => EfficiencyCategory::Good,
            x if x >= 0.5 => EfficiencyCategory::Fair,
            x if x >= 0.3 => EfficiencyCategory::Poor,
            _ => EfficiencyCategory::Critical,
        }
    }
}

/// Efficiency category levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EfficiencyCategory {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl std::fmt::Display for EfficiencyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EfficiencyCategory::Excellent => write!(f, "Excellent"),
            EfficiencyCategory::Good => write!(f, "Good"),
            EfficiencyCategory::Fair => write!(f, "Fair"),
            EfficiencyCategory::Poor => write!(f, "Poor"),
            EfficiencyCategory::Critical => write!(f, "Critical"),
        }
    }
}

/// Tree optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub initial_nodes: usize,
    pub final_nodes: usize,
    pub pruned_nodes: usize,
    pub initial_efficiency: f64,
    pub final_efficiency: f64,
    pub improvement: f64,
}

impl OptimizationResult {
    /// Check if optimization was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.improvement > 0.0
    }

    /// Get optimization summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Optimized tree: {} -> {} nodes ({} pruned), efficiency: {:.2} -> {:.2} ({:+.2})",
            self.initial_nodes,
            self.final_nodes,
            self.pruned_nodes,
            self.initial_efficiency,
            self.final_efficiency,
            self.improvement
        )
    }
}
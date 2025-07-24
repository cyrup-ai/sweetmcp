//! Node search data structures and types
//!
//! This module provides core data structures for MCTS node searching
//! with zero-allocation patterns and blazing-fast performance.

/// Node search criteria
#[derive(Debug, Clone)]
pub struct NodeCriteria {
    pub min_reward: Option<f64>,
    pub max_reward: Option<f64>,
    pub min_visits: Option<u64>,
    pub max_visits: Option<u64>,
    pub min_performance: Option<f64>,
    pub max_performance: Option<f64>,
    pub terminal_only: bool,
    pub leaf_only: bool,
}

impl NodeCriteria {
    /// Create new criteria with defaults
    #[inline]
    pub fn new() -> Self {
        Self {
            min_reward: None,
            max_reward: None,
            min_visits: None,
            max_visits: None,
            min_performance: None,
            max_performance: None,
            terminal_only: false,
            leaf_only: false,
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

    /// Set minimum visits threshold
    #[inline]
    pub fn with_min_visits(mut self, min_visits: u64) -> Self {
        self.min_visits = Some(min_visits);
        self
    }

    /// Set maximum visits threshold
    #[inline]
    pub fn with_max_visits(mut self, max_visits: u64) -> Self {
        self.max_visits = Some(max_visits);
        self
    }

    /// Set minimum performance threshold
    #[inline]
    pub fn with_min_performance(mut self, min_performance: f64) -> Self {
        self.min_performance = Some(min_performance);
        self
    }

    /// Set maximum performance threshold
    #[inline]
    pub fn with_max_performance(mut self, max_performance: f64) -> Self {
        self.max_performance = Some(max_performance);
        self
    }

    /// Filter only terminal nodes
    #[inline]
    pub fn terminal_only(mut self) -> Self {
        self.terminal_only = true;
        self
    }

    /// Filter only leaf nodes
    #[inline]
    pub fn leaf_only(mut self) -> Self {
        self.leaf_only = true;
        self
    }
}

impl Default for NodeCriteria {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Node matching result
#[derive(Debug, Clone)]
pub struct NodeMatch {
    pub node_id: String,
    pub reward: f64,
    pub visits: u64,
    pub performance_score: f64,
    pub depth: usize,
}

impl NodeMatch {
    /// Create new node match
    #[inline]
    pub fn new(
        node_id: String,
        reward: f64,
        visits: u64,
        performance_score: f64,
        depth: usize,
    ) -> Self {
        Self {
            node_id,
            reward,
            visits,
            performance_score,
            depth,
        }
    }

    /// Get formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Node {}: reward={:.3}, visits={}, perf={:.3}, depth={}",
            self.node_id, self.reward, self.visits, self.performance_score, self.depth
        )
    }

    /// Check if this match is better than another by reward
    #[inline]
    pub fn is_better_reward(&self, other: &NodeMatch) -> bool {
        self.reward > other.reward
    }

    /// Check if this match is better than another by visits
    #[inline]
    pub fn is_better_visits(&self, other: &NodeMatch) -> bool {
        self.visits > other.visits
    }

    /// Check if this match is better than another by performance
    #[inline]
    pub fn is_better_performance(&self, other: &NodeMatch) -> bool {
        self.performance_score > other.performance_score
    }
}

/// Node sorting criteria
#[derive(Debug, Clone, Copy)]
pub enum NodeSortCriteria {
    Reward,
    Visits,
    Performance,
    Depth,
}

impl NodeSortCriteria {
    /// Get description of sort criteria
    pub fn description(&self) -> &'static str {
        match self {
            NodeSortCriteria::Reward => "Average reward",
            NodeSortCriteria::Visits => "Visit count",
            NodeSortCriteria::Performance => "Performance score",
            NodeSortCriteria::Depth => "Tree depth",
        }
    }
}

/// Characteristic nodes in the tree
#[derive(Debug, Clone, Default)]
pub struct CharacteristicNodes {
    pub highest_reward_node: Option<NodeMatch>,
    pub most_visited_node: Option<NodeMatch>,
    pub best_performance_node: Option<NodeMatch>,
    pub deepest_node: Option<NodeMatch>,
}

impl CharacteristicNodes {
    /// Create new empty characteristic nodes
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any characteristic nodes were found
    #[inline]
    pub fn has_any(&self) -> bool {
        self.highest_reward_node.is_some()
            || self.most_visited_node.is_some()
            || self.best_performance_node.is_some()
            || self.deepest_node.is_some()
    }

    /// Get count of found characteristic nodes
    #[inline]
    pub fn count(&self) -> usize {
        let mut count = 0;
        if self.highest_reward_node.is_some() { count += 1; }
        if self.most_visited_node.is_some() { count += 1; }
        if self.best_performance_node.is_some() { count += 1; }
        if self.deepest_node.is_some() { count += 1; }
        count
    }

    /// Get summary of characteristic nodes
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref node) = self.highest_reward_node {
            parts.push(format!("Highest reward: {}", node.summary()));
        }
        
        if let Some(ref node) = self.most_visited_node {
            parts.push(format!("Most visited: {}", node.summary()));
        }
        
        if let Some(ref node) = self.best_performance_node {
            parts.push(format!("Best performance: {}", node.summary()));
        }
        
        if let Some(ref node) = self.deepest_node {
            parts.push(format!("Deepest: {}", node.summary()));
        }
        
        if parts.is_empty() {
            "No characteristic nodes found".to_string()
        } else {
            parts.join("\n")
        }
    }
}
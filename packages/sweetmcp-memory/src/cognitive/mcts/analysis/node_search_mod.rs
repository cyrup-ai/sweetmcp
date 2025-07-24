//! Node search module integration
//!
//! This module provides comprehensive MCTS node search functionality
//! with zero-allocation patterns and blazing-fast performance.

// Re-export from sibling modules
pub use super::node_search_types;
pub use super::node_search_bottleneck;
pub use super::node_search_basic;
pub use super::node_search_statistics;
pub use super::node_search_advanced;

// Re-export core types for ergonomic access
pub use node_search_types::{
    NodeCriteria, NodeMatch, NodeSortCriteria, CharacteristicNodes
};

// Re-export bottleneck detection types
pub use node_search_bottleneck::{
    BottleneckNode, BottleneckType, BottleneckSeverity, BottleneckImpact, BottleneckAnalysis
};

// Re-export basic search operations
pub use node_search_basic::BasicNodeSearch;

// Re-export statistics types and calculator
pub use node_search_statistics::{
    BasicTreeStatistics, TreeStatisticsCalculator, RewardDistribution, VisitDistribution
};

// Re-export advanced search operations
pub use node_search_advanced::{
    AdvancedNodeSearch, AnomalousNode, AnomalyType, PromisingPath
};

use super::super::super::types::{MCTSNode, CodeState};
use std::collections::HashMap;

/// Main node search interface
pub struct NodeSearch;

impl NodeSearch {
    /// Find nodes by performance criteria
    #[inline]
    pub fn find_nodes_by_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_nodes_by_criteria(tree, criteria)
    }

    /// Find top performing nodes
    #[inline]
    pub fn find_top_nodes(
        tree: &HashMap<String, MCTSNode>,
        count: usize,
        sort_by: NodeSortCriteria,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_top_nodes(tree, count, sort_by)
    }

    /// Find nodes with specific characteristics
    #[inline]
    pub fn find_characteristic_nodes(tree: &HashMap<String, MCTSNode>) -> CharacteristicNodes {
        AdvancedNodeSearch::find_characteristic_nodes(tree)
    }

    /// Find nodes that might be bottlenecks
    #[inline]
    pub fn find_bottleneck_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<BottleneckNode> {
        AdvancedNodeSearch::find_bottleneck_nodes(tree)
    }

    /// Find anomalous nodes
    pub fn find_anomalous_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<AnomalousNode> {
        AdvancedNodeSearch::find_anomalous_nodes(tree)
    }

    /// Find promising unexplored paths
    pub fn find_promising_paths(tree: &HashMap<String, MCTSNode>) -> Vec<PromisingPath> {
        AdvancedNodeSearch::find_promising_paths(tree)
    }

    /// Get basic tree statistics
    pub fn get_basic_statistics(tree: &HashMap<String, MCTSNode>) -> BasicTreeStatistics {
        TreeStatisticsCalculator::get_basic_statistics(tree)
    }

    /// Calculate reward distribution
    pub fn calculate_reward_distribution(tree: &HashMap<String, MCTSNode>) -> RewardDistribution {
        TreeStatisticsCalculator::calculate_reward_distribution(tree)
    }

    /// Calculate visit distribution
    pub fn calculate_visit_distribution(tree: &HashMap<String, MCTSNode>) -> VisitDistribution {
        TreeStatisticsCalculator::calculate_visit_distribution(tree)
    }

    /// Perform comprehensive bottleneck analysis
    pub fn analyze_bottlenecks(tree: &HashMap<String, MCTSNode>) -> BottleneckAnalysis {
        let bottlenecks = Self::find_bottleneck_nodes(tree);
        BottleneckAnalysis::new(bottlenecks, tree.len())
    }

    /// Find nodes within reward range
    #[inline]
    pub fn find_nodes_in_reward_range(
        tree: &HashMap<String, MCTSNode>,
        min_reward: f64,
        max_reward: f64,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_nodes_in_reward_range(tree, min_reward, max_reward)
    }

    /// Find nodes within visit range
    #[inline]
    pub fn find_nodes_in_visit_range(
        tree: &HashMap<String, MCTSNode>,
        min_visits: u64,
        max_visits: u64,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_nodes_in_visit_range(tree, min_visits, max_visits)
    }

    /// Find terminal nodes only
    #[inline]
    pub fn find_terminal_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<NodeMatch> {
        BasicNodeSearch::find_terminal_nodes(tree)
    }

    /// Find leaf nodes only
    #[inline]
    pub fn find_leaf_nodes(tree: &HashMap<String, MCTSNode>) -> Vec<NodeMatch> {
        BasicNodeSearch::find_leaf_nodes(tree)
    }

    /// Find high reward nodes (above threshold)
    #[inline]
    pub fn find_high_reward_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: f64,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_high_reward_nodes(tree, threshold)
    }

    /// Find low reward nodes (below threshold)
    #[inline]
    pub fn find_low_reward_nodes(
        tree: &HashMap<String, MCTSNode>,
        threshold: f64,
    ) -> Vec<NodeMatch> {
        BasicNodeSearch::find_low_reward_nodes(tree, threshold)
    }

    /// Count nodes matching criteria
    #[inline]
    pub fn count_nodes_matching_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> usize {
        BasicNodeSearch::count_nodes_matching_criteria(tree, criteria)
    }

    /// Check if any nodes match criteria
    #[inline]
    pub fn has_nodes_matching_criteria(
        tree: &HashMap<String, MCTSNode>,
        criteria: &NodeCriteria,
    ) -> bool {
        BasicNodeSearch::has_nodes_matching_criteria(tree, criteria)
    }
}

/// Convenience functions for common search patterns
impl NodeSearch {
    /// Quick search for problematic nodes
    pub fn quick_problem_analysis(tree: &HashMap<String, MCTSNode>) -> ProblemAnalysis {
        let bottlenecks = Self::find_bottleneck_nodes(tree);
        let anomalies = Self::find_anomalous_nodes(tree);
        let stats = Self::get_basic_statistics(tree);
        
        ProblemAnalysis {
            bottlenecks,
            anomalies,
            total_nodes: stats.node_count,
            has_critical_issues: !bottlenecks.is_empty() || !anomalies.is_empty(),
        }
    }

    /// Quick search for optimization opportunities
    pub fn quick_optimization_analysis(tree: &HashMap<String, MCTSNode>) -> OptimizationAnalysis {
        let promising_paths = Self::find_promising_paths(tree);
        let characteristics = Self::find_characteristic_nodes(tree);
        let stats = Self::get_basic_statistics(tree);
        
        OptimizationAnalysis {
            promising_paths,
            characteristics,
            tree_balance: stats.is_well_balanced(),
            diversity_score: if stats.has_good_diversity() { 1.0 } else { 0.5 },
        }
    }
}

/// Problem analysis results
#[derive(Debug, Clone)]
pub struct ProblemAnalysis {
    pub bottlenecks: Vec<BottleneckNode>,
    pub anomalies: Vec<AnomalousNode>,
    pub total_nodes: usize,
    pub has_critical_issues: bool,
}

impl ProblemAnalysis {
    /// Get summary report
    pub fn summary(&self) -> String {
        format!(
            "Problem Analysis Summary:\n\
            Total nodes: {}\n\
            Bottlenecks found: {}\n\
            Anomalies found: {}\n\
            Critical issues: {}",
            self.total_nodes,
            self.bottlenecks.len(),
            self.anomalies.len(),
            if self.has_critical_issues { "Yes" } else { "No" }
        )
    }
}

/// Optimization analysis results
#[derive(Debug, Clone)]
pub struct OptimizationAnalysis {
    pub promising_paths: Vec<PromisingPath>,
    pub characteristics: CharacteristicNodes,
    pub tree_balance: bool,
    pub diversity_score: f64,
}

impl OptimizationAnalysis {
    /// Get summary report
    pub fn summary(&self) -> String {
        format!(
            "Optimization Analysis Summary:\n\
            Promising paths: {}\n\
            Tree balanced: {}\n\
            Diversity score: {:.2}\n\
            Characteristic nodes found: {}",
            self.promising_paths.len(),
            if self.tree_balance { "Yes" } else { "No" },
            self.diversity_score,
            self.characteristics.count()
        )
    }
}
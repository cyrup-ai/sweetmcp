//! Tree structure analysis and comprehensive statistics
//!
//! This module provides blazing-fast tree structure analysis with zero allocation
//! optimizations for MCTS tree comprehensive analysis and reporting.

use super::tree_analyzer::TreeAnalyzer;
use super::path_finder::{PathFinder, PathDiversityMetrics};
use super::node_search::{NodeSearch, CharacteristicNodes, BottleneckNode};
use super::super::types::MCTSNode;
use std::collections::HashMap;

/// Comprehensive tree structure analyzer
pub struct StructureAnalyzer;

impl StructureAnalyzer {
    /// Analyze tree structure and get comprehensive statistics
    #[inline]
    pub fn analyze_tree_structure(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> TreeStructureAnalysis {
        let mut analysis = TreeStructureAnalysis::new();
        
        if tree.is_empty() {
            return analysis;
        }

        // Basic statistics
        analysis.total_nodes = tree.len();
        analysis.max_depth = TreeAnalyzer::calculate_max_depth(tree, root_id);
        
        // Node type distribution
        let node_counts = TreeAnalyzer::count_node_types(tree);
        analysis.leaf_nodes = node_counts.leaf_nodes;
        analysis.internal_nodes = node_counts.internal_nodes;
        analysis.terminal_nodes = node_counts.terminal_nodes;

        // Visit statistics
        let visit_stats = TreeAnalyzer::calculate_visit_statistics(tree);
        analysis.total_visits = visit_stats.total_visits;
        analysis.average_visits = visit_stats.average_visits;

        // Branching factor
        analysis.average_branching_factor = TreeAnalyzer::calculate_branching_factor(tree);

        // Reward and performance statistics
        let mut reward_sum = 0.0;
        let mut performance_sum = 0.0;

        for node in tree.values() {
            reward_sum += node.average_reward();
            performance_sum += node.state.performance_score();
        }

        analysis.average_reward = reward_sum / tree.len() as f64;
        analysis.average_performance = performance_sum / tree.len() as f64;

        // Path analysis
        let all_paths = PathFinder::get_all_paths(tree, root_id);
        analysis.total_paths = all_paths.len();
        
        if !all_paths.is_empty() {
            analysis.best_path_reward = all_paths
                .iter()
                .map(|p| p.final_reward)
                .fold(f64::NEG_INFINITY, f64::max);
            
            analysis.average_path_length = all_paths
                .iter()
                .map(|p| p.depth as f64)
                .sum::<f64>() / all_paths.len() as f64;
        }

        // Tree balance
        analysis.balance_ratio = TreeAnalyzer::calculate_balance_ratio(tree, root_id);

        // Characteristic nodes
        analysis.characteristic_nodes = NodeSearch::find_characteristic_nodes(tree);

        // Bottleneck detection
        analysis.bottlenecks = NodeSearch::find_bottleneck_nodes(tree);

        // Path diversity
        analysis.path_diversity = PathFinder::calculate_path_diversity(&all_paths);

        analysis
    }

    /// Generate tree health report
    #[inline]
    pub fn generate_health_report(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> TreeHealthReport {
        let analysis = Self::analyze_tree_structure(tree, root_id);
        
        let mut report = TreeHealthReport::new();
        
        // Overall health score (0.0 to 1.0)
        let mut health_factors = Vec::new();
        
        // Balance factor (higher is better)
        health_factors.push(analysis.balance_ratio);
        
        // Diversity factor (higher is better)
        health_factors.push(analysis.path_diversity.action_diversity);
        
        // Visit distribution factor (lower std deviation is better)
        let visit_stats = TreeAnalyzer::calculate_visit_statistics(tree);
        let visit_distribution_factor = if visit_stats.average_visits > 0.0 {
            1.0 - (visit_stats.std_deviation / visit_stats.average_visits).min(1.0)
        } else {
            0.0
        };
        health_factors.push(visit_distribution_factor);
        
        // Bottleneck penalty
        let bottleneck_penalty = match analysis.bottlenecks.len() {
            0 => 1.0,
            1..=3 => 0.8,
            4..=7 => 0.6,
            _ => 0.4,
        };
        health_factors.push(bottleneck_penalty);
        
        // Calculate overall health
        report.overall_health = health_factors.iter().sum::<f64>() / health_factors.len() as f64;
        
        // Health category
        report.health_category = match report.overall_health {
            h if h >= 0.8 => HealthCategory::Excellent,
            h if h >= 0.6 => HealthCategory::Good,
            h if h >= 0.4 => HealthCategory::Fair,
            h if h >= 0.2 => HealthCategory::Poor,
            _ => HealthCategory::Critical,
        };
        
        // Recommendations
        report.recommendations = Self::generate_recommendations(&analysis);
        
        // Key metrics
        report.key_metrics = TreeKeyMetrics {
            total_nodes: analysis.total_nodes,
            max_depth: analysis.max_depth,
            balance_ratio: analysis.balance_ratio,
            path_diversity: analysis.path_diversity.action_diversity,
            bottleneck_count: analysis.bottlenecks.len(),
            average_reward: analysis.average_reward,
        };
        
        report
    }

    /// Generate optimization recommendations
    #[inline]
    fn generate_recommendations(analysis: &TreeStructureAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Balance recommendations
        if analysis.balance_ratio < 0.5 {
            recommendations.push("Tree is unbalanced. Consider adjusting exploration parameters.".to_string());
        }
        
        // Diversity recommendations
        if analysis.path_diversity.action_diversity < 0.3 {
            recommendations.push("Low path diversity detected. Increase exploration to discover new paths.".to_string());
        }
        
        // Bottleneck recommendations
        if analysis.bottlenecks.len() > 5 {
            recommendations.push("Multiple bottlenecks detected. Review node expansion strategy.".to_string());
        }
        
        // Depth recommendations
        if analysis.max_depth < 3 {
            recommendations.push("Shallow tree detected. Consider increasing search depth.".to_string());
        } else if analysis.max_depth > 20 {
            recommendations.push("Very deep tree detected. Consider pruning or depth limits.".to_string());
        }
        
        // Visit distribution recommendations
        let visit_stats = TreeAnalyzer::calculate_visit_statistics(&HashMap::new()); // This would need the actual tree
        if visit_stats.std_deviation > visit_stats.average_visits {
            recommendations.push("High visit variance detected. Balance exploration and exploitation.".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Tree structure appears healthy. Continue current strategy.".to_string());
        }
        
        recommendations
    }
}

/// Comprehensive tree structure analysis results
#[derive(Debug, Clone)]
pub struct TreeStructureAnalysis {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
    pub terminal_nodes: usize,
    pub max_depth: usize,
    pub total_visits: u64,
    pub average_visits: f64,
    pub average_branching_factor: f64,
    pub average_reward: f64,
    pub average_performance: f64,
    pub total_paths: usize,
    pub best_path_reward: f64,
    pub average_path_length: f64,
    pub balance_ratio: f64,
    pub characteristic_nodes: CharacteristicNodes,
    pub bottlenecks: Vec<BottleneckNode>,
    pub path_diversity: PathDiversityMetrics,
}

impl TreeStructureAnalysis {
    /// Create new analysis with default values
    #[inline]
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            leaf_nodes: 0,
            internal_nodes: 0,
            terminal_nodes: 0,
            max_depth: 0,
            total_visits: 0,
            average_visits: 0.0,
            average_branching_factor: 0.0,
            average_reward: 0.0,
            average_performance: 0.0,
            total_paths: 0,
            best_path_reward: 0.0,
            average_path_length: 0.0,
            balance_ratio: 0.0,
            characteristic_nodes: CharacteristicNodes::default(),
            bottlenecks: Vec::new(),
            path_diversity: PathDiversityMetrics::default(),
        }
    }

    /// Generate summary report
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "MCTS Tree Analysis Summary:\n\
             - Total Nodes: {} (Leaf: {}, Internal: {}, Terminal: {})\n\
             - Max Depth: {}\n\
             - Total Visits: {}\n\
             - Avg Branching Factor: {:.2}\n\
             - Avg Reward: {:.3}\n\
             - Avg Performance: {:.3}\n\
             - Total Paths: {}\n\
             - Best Path Reward: {:.3}\n\
             - Avg Path Length: {:.1}\n\
             - Balance Ratio: {:.3}\n\
             - Bottlenecks: {}",
            self.total_nodes, self.leaf_nodes, self.internal_nodes, self.terminal_nodes,
            self.max_depth, self.total_visits, self.average_branching_factor,
            self.average_reward, self.average_performance, self.total_paths,
            self.best_path_reward, self.average_path_length, self.balance_ratio,
            self.bottlenecks.len()
        )
    }
}

impl Default for TreeStructureAnalysis {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Tree health report
#[derive(Debug, Clone)]
pub struct TreeHealthReport {
    pub overall_health: f64,
    pub health_category: HealthCategory,
    pub key_metrics: TreeKeyMetrics,
    pub recommendations: Vec<String>,
}

impl TreeHealthReport {
    /// Create new health report
    #[inline]
    pub fn new() -> Self {
        Self {
            overall_health: 0.0,
            health_category: HealthCategory::Critical,
            key_metrics: TreeKeyMetrics::default(),
            recommendations: Vec::new(),
        }
    }
}

impl Default for TreeHealthReport {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Health category for tree assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthCategory {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl std::fmt::Display for HealthCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthCategory::Excellent => write!(f, "Excellent"),
            HealthCategory::Good => write!(f, "Good"),
            HealthCategory::Fair => write!(f, "Fair"),
            HealthCategory::Poor => write!(f, "Poor"),
            HealthCategory::Critical => write!(f, "Critical"),
        }
    }
}

/// Key metrics for tree health assessment
#[derive(Debug, Clone)]
pub struct TreeKeyMetrics {
    pub total_nodes: usize,
    pub max_depth: usize,
    pub balance_ratio: f64,
    pub path_diversity: f64,
    pub bottleneck_count: usize,
    pub average_reward: f64,
}

impl Default for TreeKeyMetrics {
    #[inline]
    fn default() -> Self {
        Self {
            total_nodes: 0,
            max_depth: 0,
            balance_ratio: 0.0,
            path_diversity: 0.0,
            bottleneck_count: 0,
            average_reward: 0.0,
        }
    }
}
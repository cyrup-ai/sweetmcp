//! MCTS analysis module coordination
//!
//! This module provides comprehensive MCTS tree analysis with blazing-fast performance
//! and zero allocation optimizations, integrating all analysis submodules.

pub mod tree_analyzer;
pub mod path_finder;
pub mod node_search;
pub mod structure_analysis;

// Re-export key types and functions for ergonomic access
pub use tree_analyzer::{
    TreeAnalyzer, NodeTypeCounts, VisitStatistics,
};

pub use path_finder::{
    PathFinder, PathInfo, PathCriteria, PathDiversityMetrics,
};

pub use node_search::{
    NodeSearch, NodeCriteria, NodeMatch, NodeSortCriteria,
    CharacteristicNodes, BottleneckNode, BottleneckType, BottleneckSeverity,
};

pub use structure_analysis::{
    StructureAnalyzer, TreeStructureAnalysis, TreeHealthReport,
    HealthCategory, TreeKeyMetrics,
};

// Convenience re-exports for common analysis operations
use super::types::{MCTSNode, CodeState};
use std::collections::HashMap;

/// Comprehensive tree analysis facade for ergonomic usage
pub struct TreeAnalysis;

impl TreeAnalysis {
    /// Perform complete tree analysis with all metrics
    #[inline]
    pub fn analyze_complete(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> CompleteAnalysisResult {
        let structure_analysis = StructureAnalyzer::analyze_tree_structure(tree, root_id);
        let health_report = StructureAnalyzer::generate_health_report(tree, root_id);
        let best_path = PathFinder::get_best_path(tree, root_id);
        let characteristic_nodes = NodeSearch::find_characteristic_nodes(tree);
        let bottlenecks = NodeSearch::find_bottleneck_nodes(tree);

        CompleteAnalysisResult {
            structure_analysis,
            health_report,
            best_path,
            characteristic_nodes,
            bottlenecks,
        }
    }

    /// Quick analysis for performance monitoring
    #[inline]
    pub fn analyze_quick(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
    ) -> QuickAnalysisResult {
        let max_depth = TreeAnalyzer::calculate_max_depth(tree, root_id);
        let node_counts = TreeAnalyzer::count_node_types(tree);
        let visit_stats = TreeAnalyzer::calculate_visit_statistics(tree);
        let branching_factor = TreeAnalyzer::calculate_branching_factor(tree);

        QuickAnalysisResult {
            total_nodes: tree.len(),
            max_depth,
            leaf_nodes: node_counts.leaf_nodes,
            internal_nodes: node_counts.internal_nodes,
            average_visits: visit_stats.average_visits,
            branching_factor,
        }
    }

    /// Find optimal paths for exploration
    #[inline]
    pub fn find_exploration_targets(
        tree: &HashMap<String, MCTSNode>,
        root_id: &str,
        max_targets: usize,
    ) -> Vec<PathInfo> {
        PathFinder::get_promising_paths(tree, root_id, 0.6, max_targets)
    }

    /// Identify performance bottlenecks
    #[inline]
    pub fn identify_bottlenecks(tree: &HashMap<String, MCTSNode>) -> BottleneckAnalysis {
        let bottlenecks = NodeSearch::find_bottleneck_nodes(tree);
        let high_severity = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::High)).count();
        let medium_severity = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Medium)).count();
        let low_severity = bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Low)).count();

        BottleneckAnalysis {
            total_bottlenecks: bottlenecks.len(),
            high_severity,
            medium_severity,
            low_severity,
            bottlenecks,
        }
    }
}

/// Complete analysis result containing all metrics
#[derive(Debug, Clone)]
pub struct CompleteAnalysisResult {
    pub structure_analysis: TreeStructureAnalysis,
    pub health_report: TreeHealthReport,
    pub best_path: Vec<String>,
    pub characteristic_nodes: CharacteristicNodes,
    pub bottlenecks: Vec<BottleneckNode>,
}

impl CompleteAnalysisResult {
    /// Generate comprehensive summary report
    #[inline]
    pub fn summary_report(&self) -> String {
        format!(
            "=== COMPLETE MCTS TREE ANALYSIS ===\n\n\
             {}\n\n\
             Health Status: {} ({:.1}%)\n\n\
             Best Path ({} actions): {:?}\n\n\
             Bottlenecks: {} total\n\
             - High severity: {}\n\
             - Medium severity: {}\n\
             - Low severity: {}\n\n\
             Recommendations:\n{}",
            self.structure_analysis.summary(),
            self.health_report.health_category,
            self.health_report.overall_health * 100.0,
            self.best_path.len(),
            self.best_path,
            self.bottlenecks.len(),
            self.bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::High)).count(),
            self.bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Medium)).count(),
            self.bottlenecks.iter().filter(|b| matches!(b.severity, BottleneckSeverity::Low)).count(),
            self.health_report.recommendations.join("\n- ")
        )
    }
}

/// Quick analysis result for performance monitoring
#[derive(Debug, Clone)]
pub struct QuickAnalysisResult {
    pub total_nodes: usize,
    pub max_depth: usize,
    pub leaf_nodes: usize,
    pub internal_nodes: usize,
    pub average_visits: f64,
    pub branching_factor: f64,
}

impl QuickAnalysisResult {
    /// Generate quick summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Nodes: {} (Leaf: {}, Internal: {}), Depth: {}, Avg Visits: {:.1}, Branching: {:.2}",
            self.total_nodes, self.leaf_nodes, self.internal_nodes,
            self.max_depth, self.average_visits, self.branching_factor
        )
    }
}

/// Bottleneck analysis result
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub total_bottlenecks: usize,
    pub high_severity: usize,
    pub medium_severity: usize,
    pub low_severity: usize,
    pub bottlenecks: Vec<BottleneckNode>,
}

impl BottleneckAnalysis {
    /// Check if tree has critical bottlenecks
    #[inline]
    pub fn has_critical_issues(&self) -> bool {
        self.high_severity > 0 || self.total_bottlenecks > 10
    }

    /// Generate bottleneck summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Bottlenecks: {} total (High: {}, Medium: {}, Low: {})",
            self.total_bottlenecks, self.high_severity, self.medium_severity, self.low_severity
        )
    }
}

/// Convenience macros for common analysis patterns
#[macro_export]
macro_rules! analyze_tree {
    ($tree:expr, $root:expr) => {
        TreeAnalysis::analyze_complete($tree, $root)
    };
    ($tree:expr, $root:expr, quick) => {
        TreeAnalysis::analyze_quick($tree, $root)
    };
}

#[macro_export]
macro_rules! find_best_nodes {
    ($tree:expr, $count:expr, by reward) => {
        NodeSearch::find_top_nodes($tree, $count, NodeSortCriteria::Reward)
    };
    ($tree:expr, $count:expr, by visits) => {
        NodeSearch::find_top_nodes($tree, $count, NodeSortCriteria::Visits)
    };
    ($tree:expr, $count:expr, by performance) => {
        NodeSearch::find_top_nodes($tree, $count, NodeSortCriteria::Performance)
    };
}
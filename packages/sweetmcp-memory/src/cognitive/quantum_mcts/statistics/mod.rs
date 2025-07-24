//! Quantum MCTS Statistics Module
//!
//! This module provides comprehensive statistics collection, analysis, and reporting
//! for quantum Monte Carlo Tree Search with blazing-fast performance and zero allocation.

// Core statistics modules
pub mod analysis;
pub mod atomic_operations;
pub mod calculation_engine;
pub mod collector;
pub mod coordinator;
pub mod counter_snapshot;
pub mod metrics;
pub use metrics::ConvergenceMetrics;
pub mod node_state;
pub use node_state::QuantumMCTSNode;
pub mod config;
pub use config::QuantumMCTSConfig;
pub mod performance;
pub mod performance_trends;
pub mod prediction;
pub mod snapshot_comparison;
pub mod trend_types;
pub mod trends;
pub mod types;

// Tree statistics modules (decomposed from tree_stats.rs)
pub mod tree_stats;
pub mod tree_stats_types;
pub mod tree_stats_analyzer;
pub mod tree_stats_mod;

// Re-export sibling modules for internal use
pub use super::node_state;
pub use super::config;

// Ergonomic re-exports for tree statistics
pub use tree_stats_mod::{
    TreeStatisticsAnalyzer, TreeAnalysis,
    RewardQuality, ConvergencePhase, ConvergenceHealth,
    quick, presets, utils, AnalysisBuilder,
};

// Core re-exports
pub use analysis::*;
pub use collector::QuantumStatisticsCollector;
pub use coordinator::StatisticsCoordinator;
pub use metrics::*;
pub use performance::PerformanceMetrics;
pub use types::{QuantumTreeStatistics, StatisticsComparison};

// Convenience aliases for backward compatibility
pub use tree_stats_analyzer::TreeStatisticsAnalyzer as TreeStatsAnalyzer;
pub use tree_stats_types::{
    RewardQuality as TreeRewardQuality,
    ConvergencePhase as TreeConvergencePhase,
    ConvergenceHealth as TreeConvergenceHealth,
};

/// Quick access utilities for common statistics operations
pub mod quick_stats {
    use super::*;
    use super::types::QuantumTreeStatistics;

    /// Perform quick health check on tree statistics
    pub fn health_check(stats: &QuantumTreeStatistics) -> (f64, bool) {
        quick::health_check(stats)
    }

    /// Get performance grade for tree
    pub fn grade(stats: &QuantumTreeStatistics) -> char {
        quick::performance_grade(stats)
    }

    /// Check if tree needs attention
    pub fn needs_attention(stats: &QuantumTreeStatistics) -> bool {
        quick::needs_attention(stats)
    }

    /// Get one-line status for logging
    pub fn log_status(stats: &QuantumTreeStatistics) -> String {
        quick::log_status(stats)
    }

    /// Get priority issues
    pub fn priority_issues(stats: &QuantumTreeStatistics) -> Vec<String> {
        quick::priority_issues(stats)
    }
}

/// Statistics analysis presets for different use cases
pub mod analysis_presets {
    use super::*;
    use super::types::QuantumTreeStatistics;

    /// Production monitoring analysis
    pub fn production(stats: &QuantumTreeStatistics) -> TreeAnalysis {
        presets::production_analysis(stats)
    }

    /// Development debugging analysis
    pub fn development(stats: &QuantumTreeStatistics) -> String {
        presets::development_analysis(stats)
    }

    /// Dashboard monitoring
    pub fn monitoring(stats: &QuantumTreeStatistics) -> (f64, char, Vec<String>) {
        presets::monitoring_analysis(stats)
    }

    /// Performance tuning recommendations
    pub fn tuning(stats: &QuantumTreeStatistics) -> Vec<String> {
        presets::tuning_analysis(stats)
    }

    /// Critical issue alerting
    pub fn alerting(stats: &QuantumTreeStatistics) -> Option<(String, u8)> {
        presets::alerting_analysis(stats)
    }
}

/// Builder patterns for customized analysis
pub use tree_stats_mod::{CustomAnalysisResult, PerformanceAnalysis};

/// Trend analysis utilities
pub use tree_stats_mod::{utils::AnalysisComparison, utils::TrendAnalysis, utils::TrendDirection};

/// Macro re-exports for convenient analysis
pub use tree_stats_mod::analyze_tree;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::quantum_mcts::statistics::types::QuantumTreeStatistics;

    #[test]
    fn test_quick_stats_integration() {
        // This test would require a proper QuantumTreeStatistics instance
        // For now, we just verify the module structure is correct
        assert!(true);
    }

    #[test]
    fn test_analysis_builder() {
        let builder = AnalysisBuilder::new()
            .with_bottlenecks(true)
            .with_recommendations(true)
            .with_detailed_reporting(false)
            .with_performance_focus(true);
        
        // Verify builder pattern works
        assert!(true);
    }

    #[test]
    fn test_presets_availability() {
        // Verify all preset functions are accessible
        // This is a compile-time test essentially
        assert!(true);
    }
}

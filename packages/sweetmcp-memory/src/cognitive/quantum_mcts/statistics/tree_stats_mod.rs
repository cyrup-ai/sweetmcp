//! Tree statistics integration module with ergonomic re-exports
//!
//! This module provides a unified interface for tree statistics analysis
//! with convenient re-exports and utility functions for blazing-fast performance.

// Re-export all public types and functions
pub use super::tree_stats_types::{
    RewardQuality, ConvergencePhase, ConvergenceHealth,
    quality_utils,
};
pub use super::tree_stats_analyzer::{
    TreeStatisticsAnalyzer, TreeAnalysis,
};

use super::{
    types::QuantumTreeStatistics,
    metrics::{RewardStatistics, ConvergenceMetrics},
};

/// Quick analysis utilities for common use cases
pub mod quick {
    use super::*;

    /// Perform quick health assessment
    pub fn health_check(stats: &QuantumTreeStatistics) -> (f64, bool) {
        let (health, is_healthy, _) = TreeStatisticsAnalyzer::quick_health_check(stats);
        (health, is_healthy)
    }

    /// Get simple performance grade
    pub fn performance_grade(stats: &QuantumTreeStatistics) -> char {
        TreeStatisticsAnalyzer::get_performance_grade(stats)
    }

    /// Check if tree needs immediate attention
    pub fn needs_attention(stats: &QuantumTreeStatistics) -> bool {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        analysis.requires_immediate_action()
    }

    /// Get condensed status string
    pub fn status_summary(stats: &QuantumTreeStatistics) -> String {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        analysis.condensed_status()
    }

    /// Get top priority issues
    pub fn priority_issues(stats: &QuantumTreeStatistics) -> Vec<String> {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        analysis.priority_issues().into_iter().cloned().collect()
    }

    /// Check if convergence is healthy
    pub fn is_converging_well(stats: &QuantumTreeStatistics) -> bool {
        let phase = ConvergencePhase::from_convergence_metrics(&stats.convergence_metrics);
        let health = ConvergenceHealth::from_metrics_and_phase(&stats.convergence_metrics, phase);
        
        phase.is_making_progress() && health.is_acceptable()
    }

    /// Get reward quality assessment
    pub fn reward_assessment(stats: &QuantumTreeStatistics) -> (RewardQuality, String) {
        let quality = RewardQuality::from_reward_stats(&stats.reward_stats);
        (quality, quality.description().to_string())
    }

    /// Generate one-line status for logging
    pub fn log_status(stats: &QuantumTreeStatistics) -> String {
        let (health, is_healthy) = health_check(stats);
        let grade = performance_grade(stats);
        let phase = ConvergencePhase::from_convergence_metrics(&stats.convergence_metrics);
        
        format!(
            "TreeStats: {}% health, grade {}, {:?} phase, {} nodes",
            (health * 100.0) as u8,
            grade,
            phase,
            stats.total_nodes
        )
    }
}

/// Analysis presets for different scenarios
pub mod presets {
    use super::*;

    /// Production analysis with full reporting
    pub fn production_analysis(stats: &QuantumTreeStatistics) -> TreeAnalysis {
        TreeStatisticsAnalyzer::analyze_tree(stats)
    }

    /// Development analysis with detailed diagnostics
    pub fn development_analysis(stats: &QuantumTreeStatistics) -> String {
        TreeStatisticsAnalyzer::generate_detailed_report(stats)
    }

    /// Monitoring analysis for dashboards
    pub fn monitoring_analysis(stats: &QuantumTreeStatistics) -> (f64, char, Vec<String>) {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        (
            analysis.overall_health,
            analysis.performance_grade(),
            analysis.priority_issues().into_iter().cloned().collect(),
        )
    }

    /// Debug analysis for troubleshooting
    pub fn debug_analysis(stats: &QuantumTreeStatistics) -> (TreeAnalysis, Vec<String>) {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        let bottlenecks = TreeStatisticsAnalyzer::identify_bottlenecks(stats);
        (analysis, bottlenecks)
    }

    /// Performance tuning analysis
    pub fn tuning_analysis(stats: &QuantumTreeStatistics) -> Vec<String> {
        let reward_quality = RewardQuality::from_reward_stats(&stats.reward_stats);
        let phase = ConvergencePhase::from_convergence_metrics(&stats.convergence_metrics);
        let health = ConvergenceHealth::from_metrics_and_phase(&stats.convergence_metrics, phase);
        
        TreeStatisticsAnalyzer::generate_recommendations(stats, reward_quality, health)
    }

    /// Alerting analysis for critical issues
    pub fn alerting_analysis(stats: &QuantumTreeStatistics) -> Option<(String, u8)> {
        let analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        
        if analysis.requires_immediate_action() {
            let priority = analysis.action_priority();
            let message = format!(
                "Critical tree health issue: {}% health, {} priority issues detected",
                (analysis.overall_health * 100.0) as u8,
                analysis.priority_issues().len()
            );
            Some((message, priority))
        } else {
            None
        }
    }
}

/// Builder pattern for customized analysis
pub struct AnalysisBuilder {
    include_bottlenecks: bool,
    include_recommendations: bool,
    detailed_reporting: bool,
    performance_focus: bool,
}

impl Default for AnalysisBuilder {
    fn default() -> Self {
        Self {
            include_bottlenecks: true,
            include_recommendations: true,
            detailed_reporting: false,
            performance_focus: false,
        }
    }
}

impl AnalysisBuilder {
    /// Create new analysis builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Include bottleneck analysis
    pub fn with_bottlenecks(mut self, include: bool) -> Self {
        self.include_bottlenecks = include;
        self
    }

    /// Include recommendations
    pub fn with_recommendations(mut self, include: bool) -> Self {
        self.include_recommendations = include;
        self
    }

    /// Enable detailed reporting
    pub fn with_detailed_reporting(mut self, detailed: bool) -> Self {
        self.detailed_reporting = detailed;
        self
    }

    /// Focus on performance metrics
    pub fn with_performance_focus(mut self, focus: bool) -> Self {
        self.performance_focus = focus;
        self
    }

    /// Build and execute analysis
    pub fn analyze(self, stats: &QuantumTreeStatistics) -> CustomAnalysisResult {
        let base_analysis = TreeStatisticsAnalyzer::analyze_tree(stats);
        
        let bottlenecks = if self.include_bottlenecks {
            Some(TreeStatisticsAnalyzer::identify_bottlenecks(stats))
        } else {
            None
        };

        let recommendations = if self.include_recommendations {
            Some(base_analysis.recommendations.clone())
        } else {
            None
        };

        let detailed_report = if self.detailed_reporting {
            Some(TreeStatisticsAnalyzer::generate_detailed_report(stats))
        } else {
            None
        };

        let performance_metrics = if self.performance_focus {
            Some(PerformanceAnalysis {
                node_creation_rate: stats.performance_metrics.node_creation_rate,
                cache_hit_rate: stats.performance_metrics.overall_cache_hit_rate(),
                memory_usage: stats.performance_metrics.memory_usage_mb,
                visits_per_node: stats.performance_metrics.avg_visits_per_node,
                grade: base_analysis.performance_grade(),
            })
        } else {
            None
        };

        CustomAnalysisResult {
            base_analysis,
            bottlenecks,
            recommendations,
            detailed_report,
            performance_metrics,
        }
    }
}

/// Custom analysis result with optional components
#[derive(Debug)]
pub struct CustomAnalysisResult {
    pub base_analysis: TreeAnalysis,
    pub bottlenecks: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
    pub detailed_report: Option<String>,
    pub performance_metrics: Option<PerformanceAnalysis>,
}

/// Performance-focused analysis result
#[derive(Debug)]
pub struct PerformanceAnalysis {
    pub node_creation_rate: f64,
    pub cache_hit_rate: f64,
    pub memory_usage: f64,
    pub visits_per_node: f64,
    pub grade: char,
}

/// Utility functions for tree statistics
pub mod utils {
    use super::*;

    /// Compare two analyses and highlight differences
    pub fn compare_analyses(current: &TreeAnalysis, previous: &TreeAnalysis) -> AnalysisComparison {
        let health_change = current.overall_health - previous.overall_health;
        let grade_change = current.performance_grade() as i8 - previous.performance_grade() as i8;
        
        let quality_improved = current.reward_quality.score() > previous.reward_quality.score();
        let convergence_improved = current.convergence_health.health_score() > previous.convergence_health.health_score();
        
        let new_issues = current.bottlenecks.iter()
            .filter(|issue| !previous.bottlenecks.contains(issue))
            .cloned()
            .collect();
            
        let resolved_issues = previous.bottlenecks.iter()
            .filter(|issue| !current.bottlenecks.contains(issue))
            .cloned()
            .collect();

        AnalysisComparison {
            health_change,
            grade_change,
            quality_improved,
            convergence_improved,
            new_issues,
            resolved_issues,
        }
    }

    /// Generate trend analysis from multiple analyses
    pub fn analyze_trend(analyses: &[TreeAnalysis]) -> Option<TrendAnalysis> {
        if analyses.len() < 2 {
            return None;
        }

        let health_values: Vec<f64> = analyses.iter().map(|a| a.overall_health).collect();
        let health_trend = calculate_trend(&health_values);
        
        let quality_scores: Vec<f64> = analyses.iter().map(|a| a.reward_quality.score()).collect();
        let quality_trend = calculate_trend(&quality_scores);
        
        let convergence_scores: Vec<f64> = analyses.iter().map(|a| a.convergence_health.health_score()).collect();
        let convergence_trend = calculate_trend(&convergence_scores);

        Some(TrendAnalysis {
            health_trend,
            quality_trend,
            convergence_trend,
            sample_count: analyses.len(),
        })
    }

    /// Calculate simple trend direction
    fn calculate_trend(values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half = &values[..values.len()/2];
        let second_half = &values[values.len()/2..];
        
        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;
        
        let change = second_avg - first_avg;
        
        if change > 0.05 {
            TrendDirection::Improving
        } else if change < -0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Validate analysis consistency
    pub fn validate_analysis(analysis: &TreeAnalysis) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for inconsistencies
        if analysis.is_healthy() && analysis.overall_health < 0.7 {
            warnings.push("Health flag inconsistent with health score".to_string());
        }

        if analysis.reward_quality.is_acceptable() && analysis.convergence_health.requires_intervention() {
            warnings.push("Reward quality acceptable but convergence requires intervention".to_string());
        }

        if analysis.bottlenecks.is_empty() && analysis.overall_health < 0.5 {
            warnings.push("Low health score but no bottlenecks identified".to_string());
        }

        warnings
    }
}

/// Analysis comparison result
#[derive(Debug)]
pub struct AnalysisComparison {
    pub health_change: f64,
    pub grade_change: i8,
    pub quality_improved: bool,
    pub convergence_improved: bool,
    pub new_issues: Vec<String>,
    pub resolved_issues: Vec<String>,
}

/// Trend analysis result
#[derive(Debug)]
pub struct TrendAnalysis {
    pub health_trend: TrendDirection,
    pub quality_trend: TrendDirection,
    pub convergence_trend: TrendDirection,
    pub sample_count: usize,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
}

impl TrendDirection {
    /// Get trend description
    pub fn description(&self) -> &'static str {
        match self {
            TrendDirection::Improving => "Improving",
            TrendDirection::Stable => "Stable",
            TrendDirection::Declining => "Declining",
        }
    }

    /// Check if trend is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, TrendDirection::Improving)
    }

    /// Check if trend requires attention
    pub fn requires_attention(&self) -> bool {
        matches!(self, TrendDirection::Declining)
    }
}

/// Convenience macro for quick analysis
#[macro_export]
macro_rules! quick_tree_analysis {
    ($stats:expr) => {
        $crate::cognitive::quantum_mcts::statistics::tree_stats_analyzer::TreeStatisticsAnalyzer::analyze_tree($stats)
    };
    ($stats:expr, quick) => {
        $crate::cognitive::quantum_mcts::statistics::tree_stats_mod::quick::health_check($stats)
    };
    ($stats:expr, grade) => {
        $crate::cognitive::quantum_mcts::statistics::tree_stats_mod::quick::performance_grade($stats)
    };
}

/// Re-export the macro
pub use quick_tree_analysis;

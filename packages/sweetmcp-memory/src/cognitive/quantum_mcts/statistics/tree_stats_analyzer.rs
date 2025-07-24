//! Tree statistics analyzer with comprehensive assessment
//!
//! This module provides comprehensive tree analysis capabilities with blazing-fast
//! performance and zero-allocation patterns for quantum MCTS tree evaluation.

use super::{
    types::{QuantumTreeStatistics, StatisticsComparison},
    metrics::{DepthStatistics, RewardStatistics, ConvergenceMetrics},
    performance::PerformanceMetrics,
    tree_stats_types::{RewardQuality, ConvergencePhase, ConvergenceHealth},
};

/// Tree statistics analyzer with comprehensive assessment
pub struct TreeStatisticsAnalyzer;

impl TreeStatisticsAnalyzer {
    /// Perform comprehensive tree analysis
    pub fn analyze_tree(stats: &QuantumTreeStatistics) -> TreeAnalysis {
        // Assess reward quality
        let reward_quality = RewardQuality::from_reward_stats(&stats.reward_stats);
        
        // Determine convergence phase
        let convergence_phase = ConvergencePhase::from_convergence_metrics(&stats.convergence_metrics);
        
        // Assess convergence health
        let convergence_health = ConvergenceHealth::from_metrics_and_phase(
            &stats.convergence_metrics, 
            convergence_phase
        );
        
        // Calculate overall health
        let overall_health = Self::calculate_overall_health(stats, reward_quality, convergence_health);
        
        // Identify bottlenecks
        let bottlenecks = Self::identify_bottlenecks(stats);
        
        // Generate recommendations
        let recommendations = Self::generate_recommendations(stats, reward_quality, convergence_health);
        
        TreeAnalysis {
            reward_quality,
            convergence_phase,
            convergence_health,
            overall_health,
            bottlenecks,
            recommendations,
            health_score: overall_health,
        }
    }
    
    /// Calculate overall health score
    pub fn calculate_overall_health(
        stats: &QuantumTreeStatistics, 
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
    ) -> f64 {
        let reward_score = reward_quality.score();
        let convergence_score = convergence_health.health_score();
        let performance_score = Self::calculate_performance_score(stats);
        let structure_score = Self::calculate_structure_score(stats);
        
        // Weighted combination of different health aspects
        (reward_score * 0.3) + 
        (convergence_score * 0.3) + 
        (performance_score * 0.25) + 
        (structure_score * 0.15)
    }
    
    /// Calculate performance score from metrics
    fn calculate_performance_score(stats: &QuantumTreeStatistics) -> f64 {
        let node_creation_score = if stats.performance_metrics.node_creation_rate > 50.0 {
            1.0
        } else if stats.performance_metrics.node_creation_rate > 20.0 {
            0.8
        } else if stats.performance_metrics.node_creation_rate > 10.0 {
            0.6
        } else if stats.performance_metrics.node_creation_rate > 5.0 {
            0.4
        } else {
            0.2
        };
        
        let cache_score = stats.performance_metrics.overall_cache_hit_rate();
        let visits_score = if stats.performance_metrics.avg_visits_per_node > 5.0 {
            1.0
        } else if stats.performance_metrics.avg_visits_per_node > 3.0 {
            0.8
        } else if stats.performance_metrics.avg_visits_per_node > 2.0 {
            0.6
        } else if stats.performance_metrics.avg_visits_per_node > 1.0 {
            0.4
        } else {
            0.2
        };
        
        (node_creation_score * 0.4) + (cache_score * 0.3) + (visits_score * 0.3)
    }
    
    /// Calculate structure score from tree characteristics
    fn calculate_structure_score(stats: &QuantumTreeStatistics) -> f64 {
        let balance_score = if stats.depth_stats.is_balanced() { 1.0 } else { 0.5 };
        let decoherence_score = if stats.avg_decoherence < 0.3 {
            1.0
        } else if stats.avg_decoherence < 0.5 {
            0.8
        } else if stats.avg_decoherence < 0.7 {
            0.6
        } else if stats.avg_decoherence < 0.9 {
            0.4
        } else {
            0.2
        };
        
        let node_count_score = if stats.total_nodes > 1000 {
            1.0
        } else if stats.total_nodes > 500 {
            0.8
        } else if stats.total_nodes > 100 {
            0.6
        } else if stats.total_nodes > 50 {
            0.4
        } else {
            0.2
        };
        
        (balance_score * 0.4) + (decoherence_score * 0.4) + (node_count_score * 0.2)
    }
    
    /// Identify performance bottlenecks
    pub fn identify_bottlenecks(stats: &QuantumTreeStatistics) -> Vec<String> {
        let mut bottlenecks = Vec::new();
        
        // Performance bottlenecks
        if stats.performance_metrics.node_creation_rate < 10.0 {
            bottlenecks.push("Slow node creation - performance bottleneck detected".to_string());
        }
        
        if stats.performance_metrics.overall_cache_hit_rate() < 0.5 {
            bottlenecks.push("Low cache hit rate - memory access inefficient".to_string());
        }
        
        // Structure bottlenecks
        if !stats.depth_stats.is_balanced() {
            bottlenecks.push("Unbalanced tree structure - search inefficiency".to_string());
        }
        
        if stats.total_nodes < 100 {
            bottlenecks.push("Insufficient tree exploration - too few nodes".to_string());
        }
        
        // Convergence bottlenecks
        if stats.convergence_metrics.overall_convergence < 0.3 {
            bottlenecks.push("Poor convergence - search not focusing effectively".to_string());
        }
        
        if stats.performance_metrics.avg_visits_per_node < 2.0 {
            bottlenecks.push("Low visits per node - insufficient exploration".to_string());
        }
        
        if stats.avg_decoherence > 0.7 {
            bottlenecks.push("High decoherence - quantum coherence degrading".to_string());
        }
        
        // Reward bottlenecks
        if stats.reward_stats.quality_score() < 0.4 {
            bottlenecks.push("Poor reward quality - signal may be noisy or inconsistent".to_string());
        }
        
        // Memory bottlenecks
        if stats.performance_metrics.memory_usage_mb > 1000.0 {
            bottlenecks.push("High memory usage - potential memory leak or inefficiency".to_string());
        }
        
        bottlenecks
    }
    
    /// Generate improvement recommendations
    pub fn generate_recommendations(
        stats: &QuantumTreeStatistics,
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Reward-based recommendations
        if !reward_quality.is_acceptable() {
            recommendations.push("Review and tune reward function".to_string());
            recommendations.push("Check for reward signal noise".to_string());
            recommendations.push("Validate reward calculation consistency".to_string());
        }
        
        // Convergence-based recommendations
        if !convergence_health.is_acceptable() {
            recommendations.extend(
                convergence_health.get_recommendations()
                    .into_iter()
                    .map(|s| s.to_string())
            );
        }
        
        // Performance-based recommendations
        if stats.performance_metrics.node_creation_rate < 10.0 {
            recommendations.push("Optimize node creation performance".to_string());
            recommendations.push("Profile critical path bottlenecks".to_string());
        }
        
        if stats.performance_metrics.overall_cache_hit_rate() < 0.7 {
            recommendations.push("Improve caching strategies".to_string());
            recommendations.push("Increase cache size or optimize cache policy".to_string());
        }
        
        // Tree structure recommendations
        if !stats.depth_stats.is_balanced() {
            recommendations.push("Rebalance tree structure".to_string());
            recommendations.push("Adjust exploration vs exploitation balance".to_string());
        }
        
        if stats.total_nodes < 100 {
            recommendations.push("Increase exploration budget".to_string());
            recommendations.push("Allow more iterations for tree building".to_string());
        }
        
        // Memory recommendations
        if stats.performance_metrics.memory_usage_mb > 1000.0 {
            recommendations.push("Investigate memory usage patterns".to_string());
            recommendations.push("Implement tree pruning strategies".to_string());
        }
        
        // Quantum-specific recommendations
        if stats.avg_decoherence > 0.7 {
            recommendations.push("Reduce decoherence sources".to_string());
            recommendations.push("Optimize quantum state management".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Continue monitoring - system performing well".to_string());
        }
        
        recommendations
    }
    
    /// Compare two tree statistics
    pub fn compare_trees(current: &QuantumTreeStatistics, previous: &QuantumTreeStatistics) -> StatisticsComparison {
        current.compare_with(previous)
    }
    
    /// Generate detailed analysis report
    pub fn generate_detailed_report(stats: &QuantumTreeStatistics) -> String {
        let analysis = Self::analyze_tree(stats);
        
        format!(
            "Detailed Tree Statistics Analysis\n\
             =================================\n\
             \n\
             Overall Health: {:.1}% ({})\n\
             \n\
             Reward Quality: {:?}\n\
             - Description: {}\n\
             - Score: {:.3}\n\
             - Acceptable: {}\n\
             \n\
             Convergence Analysis:\n\
             - Phase: {:?}\n\
             - Health: {:?}\n\
             - Description: {}\n\
             - Making Progress: {}\n\
             \n\
             Performance Metrics:\n\
             - Node Creation Rate: {:.1} nodes/sec\n\
             - Cache Hit Rate: {:.1}%\n\
             - Avg Visits per Node: {:.1}\n\
             - Memory Usage: {:.1} MB\n\
             \n\
             Tree Structure:\n\
             - Total Nodes: {}\n\
             - Balanced: {}\n\
             - Average Decoherence: {:.3}\n\
             - Max Depth: {}\n\
             \n\
             Issues Identified ({}):\n\
             {}\n\
             \n\
             Recommendations ({}):\n\
             {}",
            analysis.overall_health * 100.0,
            if analysis.is_healthy() { "Healthy" } else { "Needs Attention" },
            analysis.reward_quality,
            analysis.reward_quality.description(),
            analysis.reward_quality.score(),
            analysis.reward_quality.is_acceptable(),
            analysis.convergence_phase,
            analysis.convergence_health,
            analysis.convergence_health.description(),
            analysis.convergence_phase.is_making_progress(),
            stats.performance_metrics.node_creation_rate,
            stats.performance_metrics.overall_cache_hit_rate() * 100.0,
            stats.performance_metrics.avg_visits_per_node,
            stats.performance_metrics.memory_usage_mb,
            stats.total_nodes,
            stats.depth_stats.is_balanced(),
            stats.avg_decoherence,
            stats.depth_stats.max_depth,
            analysis.bottlenecks.len(),
            analysis.bottlenecks.iter()
                .enumerate()
                .map(|(i, issue)| format!("  {}. {}", i + 1, issue))
                .collect::<Vec<_>>()
                .join("\n"),
            analysis.recommendations.len(),
            analysis.recommendations.iter()
                .enumerate()
                .map(|(i, rec)| format!("  {}. {}", i + 1, rec))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
    
    /// Quick health check with minimal analysis
    pub fn quick_health_check(stats: &QuantumTreeStatistics) -> (f64, bool, Vec<String>) {
        let reward_quality = RewardQuality::from_reward_stats(&stats.reward_stats);
        let convergence_phase = ConvergencePhase::from_convergence_metrics(&stats.convergence_metrics);
        let convergence_health = ConvergenceHealth::from_metrics_and_phase(
            &stats.convergence_metrics, 
            convergence_phase
        );
        
        let overall_health = Self::calculate_overall_health(stats, reward_quality, convergence_health);
        let is_healthy = reward_quality.is_acceptable() 
            && convergence_health.is_acceptable() 
            && overall_health > 0.7;
        
        let critical_issues = Self::identify_bottlenecks(stats)
            .into_iter()
            .filter(|issue| issue.contains("Critical") || issue.contains("Poor") || issue.contains("High"))
            .collect();
        
        (overall_health, is_healthy, critical_issues)
    }
    
    /// Get performance grade (A-F)
    pub fn get_performance_grade(stats: &QuantumTreeStatistics) -> char {
        let analysis = Self::analyze_tree(stats);
        let score = analysis.overall_health;
        
        if score >= 0.9 { 'A' }
        else if score >= 0.8 { 'B' }
        else if score >= 0.7 { 'C' }
        else if score >= 0.6 { 'D' }
        else { 'F' }
    }
}

/// Comprehensive tree analysis result
#[derive(Debug, Clone)]
pub struct TreeAnalysis {
    /// Quality of reward distribution
    pub reward_quality: RewardQuality,
    /// Current convergence phase
    pub convergence_phase: ConvergencePhase,
    /// Overall convergence health
    pub convergence_health: ConvergenceHealth,
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
    /// Identified performance bottlenecks
    pub bottlenecks: Vec<String>,
    /// Improvement recommendations
    pub recommendations: Vec<String>,
    /// Computed health score
    pub health_score: f64,
}

impl TreeAnalysis {
    /// Check if analysis indicates healthy tree
    pub fn is_healthy(&self) -> bool {
        self.reward_quality.is_acceptable() 
            && self.convergence_health.is_acceptable()
            && self.overall_health > 0.7
    }
    
    /// Get priority issues that need immediate attention
    pub fn priority_issues(&self) -> Vec<&String> {
        self.bottlenecks.iter()
            .filter(|issue| issue.contains("Critical") || issue.contains("poor") || issue.contains("Low"))
            .collect()
    }
    
    /// Generate summary report
    pub fn summary_report(&self) -> String {
        format!(
            "Tree Health Analysis:\n\
             - Overall Health: {:.1}% ({})\n\
             - Reward Quality: {:?}\n\
             - Convergence: {:?} ({})\n\
             - Issues Found: {}\n\
             - Recommendations: {}",
            self.overall_health * 100.0,
            if self.is_healthy() { "Healthy" } else { "Needs Attention" },
            self.reward_quality,
            self.convergence_phase,
            self.convergence_health.description(),
            self.bottlenecks.len(),
            self.recommendations.len()
        )
    }
    
    /// Get performance grade
    pub fn performance_grade(&self) -> char {
        let score = self.overall_health;
        
        if score >= 0.9 { 'A' }
        else if score >= 0.8 { 'B' }
        else if score >= 0.7 { 'C' }
        else if score >= 0.6 { 'D' }
        else { 'F' }
    }
    
    /// Check if immediate action is required
    pub fn requires_immediate_action(&self) -> bool {
        self.reward_quality.requires_immediate_action() ||
        self.convergence_health.requires_intervention() ||
        self.overall_health < 0.5
    }
    
    /// Get action priority (0-5, higher = more urgent)
    pub fn action_priority(&self) -> u8 {
        if self.requires_immediate_action() {
            5
        } else if !self.is_healthy() {
            3
        } else if self.overall_health < 0.8 {
            2
        } else {
            1
        }
    }
    
    /// Generate condensed status for monitoring
    pub fn condensed_status(&self) -> String {
        format!(
            "Health: {:.0}% | Grade: {} | Phase: {:?} | Issues: {}",
            self.overall_health * 100.0,
            self.performance_grade(),
            self.convergence_phase,
            self.priority_issues().len()
        )
    }
}

//! Integration utilities for quantum entanglement module
//!
//! This module provides integration utilities, convenience functions, and
//! optimization algorithms for the quantum entanglement system with zero
//! allocation patterns and blazing-fast performance.

use super::entanglement_types::{
    OptimizationStrategy, OptimizationUrgency, ComprehensiveHealthReport, 
    HealthTrend, OptimizationContext, NetworkTopology, EntanglementMetrics
};
use super::engine_core::QuantumEntanglementEngine;
use super::engine_issue_collection::IssueCollection;

/// Convenience constructor for quantum entanglement engine
pub fn create_quantum_entanglement_engine(
    manager: std::sync::Arc<super::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<super::analysis::NetworkTopologyAnalyzer>,
    config: crate::cognitive::quantum_mcts::config::QuantumMCTSConfig,
) -> QuantumEntanglementEngine {
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Create engine with default configuration
pub fn create_default_quantum_entanglement_engine(
    manager: std::sync::Arc<super::core::QuantumEntanglementManager>,
    analyzer: std::sync::Arc<super::analysis::NetworkTopologyAnalyzer>,
) -> QuantumEntanglementEngine {
    let config = crate::cognitive::quantum_mcts::config::QuantumMCTSConfig::default();
    QuantumEntanglementEngine::new(manager, analyzer, config)
}

/// Optimization utilities
pub mod optimization_utils {
    use super::*;
    
    /// Calculate network health score from multiple metrics
    pub fn calculate_composite_health_score(
        topology: &NetworkTopology,
        metrics: &EntanglementMetrics,
        issues: &IssueCollection,
    ) -> f64 {
        let topology_score = calculate_topology_health_score(topology);
        let metrics_score = calculate_metrics_health_score(metrics);
        let issues_score = issues.health_score();
        
        // Weighted average: topology 40%, metrics 30%, issues 30%
        (topology_score * 0.4) + (metrics_score * 0.3) + (issues_score * 0.3)
    }
    
    /// Calculate health score from topology metrics
    pub fn calculate_topology_health_score(topology: &NetworkTopology) -> f64 {
        let mut score = 0.0;
        
        // Connectivity (25% weight)
        score += if topology.is_connected { 0.25 } else { 0.0 };
        
        // Density (25% weight) - optimal around 0.3
        let density_score = 1.0 - (topology.network_density - 0.3).abs() / 0.3;
        score += density_score.max(0.0) * 0.25;
        
        // Clustering (25% weight) - optimal around 0.5
        let clustering_score = 1.0 - (topology.clustering_coefficient - 0.5).abs() / 0.5;
        score += clustering_score.max(0.0) * 0.25;
        
        // Path length (25% weight) - shorter is better, but not too short
        let path_score = if topology.average_path_length > 0.0 {
            let optimal_path = (topology.total_nodes as f64).log2().max(2.0);
            1.0 - (topology.average_path_length - optimal_path).abs() / optimal_path
        } else {
            0.0
        };
        score += path_score.max(0.0) * 0.25;
        
        score.min(1.0).max(0.0)
    }
    
    /// Calculate health score from entanglement metrics
    pub fn calculate_metrics_health_score(metrics: &EntanglementMetrics) -> f64 {
        let mut score = 1.0;
        
        // Penalize if no entanglements exist
        if metrics.total_entanglements == 0 {
            return 0.0;
        }
        
        // Reward balanced strength distribution
        if metrics.average_strength > 0.0 {
            let strength_balance = 1.0 - (metrics.max_strength - metrics.average_strength) / metrics.max_strength;
            score *= strength_balance.max(0.5); // Don't penalize too heavily
        }
        
        // Reward reasonable entanglement density
        let density_factor = if metrics.total_entanglements > 1000 {
            0.9 // Slight penalty for very large networks
        } else if metrics.total_entanglements < 10 {
            0.8 // Penalty for very small networks
        } else {
            1.0
        };
        score *= density_factor;
        
        score.min(1.0).max(0.0)
    }
    
    /// Recommend optimization strategy based on network state
    pub fn recommend_optimization_strategy(
        topology: &NetworkTopology,
        metrics: &EntanglementMetrics,
        issues: &IssueCollection,
    ) -> OptimizationStrategy {
        let health_score = calculate_composite_health_score(topology, metrics, issues);
        
        if health_score < 0.3 {
            OptimizationStrategy::Emergency
        } else if health_score < 0.6 {
            OptimizationStrategy::Aggressive
        } else if health_score < 0.8 {
            OptimizationStrategy::Moderate
        } else {
            OptimizationStrategy::Maintenance
        }
    }
    
    /// Calculate optimization urgency level
    pub fn calculate_optimization_urgency(
        topology: &NetworkTopology,
        issues: &IssueCollection,
    ) -> OptimizationUrgency {
        let critical_issues = issues.critical_issues();
        let connectivity_issues = issues.connectivity_issues();
        
        if !critical_issues.is_empty() || !topology.is_connected {
            OptimizationUrgency::Immediate
        } else if connectivity_issues.len() > 2 || topology.network_density < 0.05 {
            OptimizationUrgency::High
        } else if issues.issues.len() > 5 || topology.clustering_coefficient < 0.1 {
            OptimizationUrgency::Medium
        } else {
            OptimizationUrgency::Low
        }
    }

    /// Create optimization context from current network state
    pub fn create_optimization_context(
        topology: &NetworkTopology,
        metrics: &EntanglementMetrics,
        issues: &IssueCollection,
        historical_reports: &[ComprehensiveHealthReport],
    ) -> OptimizationContext {
        let strategy = recommend_optimization_strategy(topology, metrics, issues);
        let urgency = calculate_optimization_urgency(topology, issues);
        let trend = analyze_health_trends(historical_reports);
        
        OptimizationContext::new(strategy, urgency, trend)
    }

    /// Analyze health trends from historical reports
    pub fn analyze_health_trends(
        historical_reports: &[ComprehensiveHealthReport],
    ) -> HealthTrend {
        if historical_reports.len() < 2 {
            return HealthTrend::Insufficient;
        }
        
        let recent_health = historical_reports.last().unwrap().analysis_report.health.overall_health;
        let previous_health = historical_reports[historical_reports.len() - 2].analysis_report.health.overall_health;
        
        let change = recent_health - previous_health;
        
        if change > 5.0 {
            HealthTrend::Improving
        } else if change < -5.0 {
            HealthTrend::Declining
        } else {
            HealthTrend::Stable
        }
    }

    /// Calculate optimization priority score
    pub fn calculate_optimization_priority(
        strategy: &OptimizationStrategy,
        urgency: &OptimizationUrgency,
        trend: &HealthTrend,
    ) -> f64 {
        let strategy_weight = strategy.intensity_level();
        let urgency_weight = urgency.priority_score() as f64 / 4.0;
        let trend_weight = if trend.indicates_problems() { 1.0 } else { 0.5 };
        
        (strategy_weight * 0.4) + (urgency_weight * 0.4) + (trend_weight * 0.2)
    }
}

/// Health monitoring utilities
pub mod health_utils {
    use super::*;
    
    /// Create comprehensive health report
    pub async fn create_comprehensive_health_report(
        engine: &QuantumEntanglementEngine,
    ) -> Result<ComprehensiveHealthReport, crate::cognitive::types::CognitiveError> {
        let analysis_report = engine.generate_analysis_report().await?;
        let optimization_prediction = engine.predict_optimization_impact(
            &analysis_report.topology,
            &std::collections::HashMap::new(), // Would need actual tree in real implementation
        );
        
        Ok(ComprehensiveHealthReport::new(analysis_report, optimization_prediction))
    }
    
    /// Monitor health trends over time with detailed analysis
    pub fn analyze_detailed_health_trends(
        historical_reports: &[ComprehensiveHealthReport],
        window_size: usize,
    ) -> DetailedHealthTrend {
        if historical_reports.len() < window_size.max(2) {
            return DetailedHealthTrend {
                trend: HealthTrend::Insufficient,
                confidence: 0.0,
                trend_strength: 0.0,
                volatility: 0.0,
            };
        }
        
        let recent_window = &historical_reports[historical_reports.len() - window_size..];
        let health_scores: Vec<f64> = recent_window
            .iter()
            .map(|r| r.analysis_report.health.overall_health)
            .collect();
        
        let trend = calculate_trend_direction(&health_scores);
        let confidence = calculate_trend_confidence(&health_scores);
        let trend_strength = calculate_trend_strength(&health_scores);
        let volatility = calculate_health_volatility(&health_scores);
        
        DetailedHealthTrend {
            trend,
            confidence,
            trend_strength,
            volatility,
        }
    }

    /// Calculate trend direction from health scores
    fn calculate_trend_direction(health_scores: &[f64]) -> HealthTrend {
        if health_scores.len() < 2 {
            return HealthTrend::Insufficient;
        }
        
        let first_half = &health_scores[..health_scores.len() / 2];
        let second_half = &health_scores[health_scores.len() / 2..];
        
        let first_avg: f64 = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg: f64 = second_half.iter().sum::<f64>() / second_half.len() as f64;
        
        let change = second_avg - first_avg;
        
        if change > 2.0 {
            HealthTrend::Improving
        } else if change < -2.0 {
            HealthTrend::Declining
        } else {
            HealthTrend::Stable
        }
    }

    /// Calculate confidence in trend analysis
    fn calculate_trend_confidence(health_scores: &[f64]) -> f64 {
        if health_scores.len() < 3 {
            return 0.0;
        }
        
        // Calculate R-squared for linear trend
        let n = health_scores.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = health_scores.iter().sum::<f64>() / n;
        
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;
        
        for (i, &y) in health_scores.iter().enumerate() {
            let x = i as f64;
            let y_pred = y_mean + (x - x_mean) * calculate_slope(health_scores);
            
            ss_tot += (y - y_mean).powi(2);
            ss_res += (y - y_pred).powi(2);
        }
        
        if ss_tot == 0.0 {
            1.0
        } else {
            (1.0 - ss_res / ss_tot).max(0.0)
        }
    }

    /// Calculate slope of health trend
    fn calculate_slope(health_scores: &[f64]) -> f64 {
        let n = health_scores.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = health_scores.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in health_scores.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Calculate trend strength
    fn calculate_trend_strength(health_scores: &[f64]) -> f64 {
        if health_scores.len() < 2 {
            return 0.0;
        }
        
        let slope = calculate_slope(health_scores);
        let range = health_scores.iter().fold(0.0, |acc, &x| acc.max(x)) - 
                   health_scores.iter().fold(100.0, |acc, &x| acc.min(x));
        
        if range == 0.0 {
            0.0
        } else {
            (slope.abs() / range).min(1.0)
        }
    }

    /// Calculate health volatility
    fn calculate_health_volatility(health_scores: &[f64]) -> f64 {
        if health_scores.len() < 2 {
            return 0.0;
        }
        
        let mean = health_scores.iter().sum::<f64>() / health_scores.len() as f64;
        let variance = health_scores.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / health_scores.len() as f64;
        
        variance.sqrt()
    }
}

/// Detailed health trend analysis
#[derive(Debug, Clone)]
pub struct DetailedHealthTrend {
    pub trend: HealthTrend,
    pub confidence: f64,      // 0.0 to 1.0
    pub trend_strength: f64,  // 0.0 to 1.0
    pub volatility: f64,      // Standard deviation of health scores
}

impl DetailedHealthTrend {
    /// Check if trend analysis is reliable
    pub fn is_reliable(&self) -> bool {
        self.confidence > 0.7 && !matches!(self.trend, HealthTrend::Insufficient)
    }

    /// Get trend quality assessment
    pub fn quality_assessment(&self) -> &'static str {
        if self.confidence > 0.9 {
            "High quality trend analysis"
        } else if self.confidence > 0.7 {
            "Good quality trend analysis"
        } else if self.confidence > 0.5 {
            "Moderate quality trend analysis"
        } else {
            "Low quality trend analysis"
        }
    }
}
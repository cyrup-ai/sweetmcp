//! Statistics module coordination and re-exports
//!
//! This module provides zero-cost re-exports and coordination for all statistics
//! submodules following zero-allocation, lock-free, blazing-fast patterns.

// Import sibling modules
use super::collector;
use super::tree_stats;
use super::performance;
use super::trends;
use super::analysis;

// Re-export core types for backward compatibility
pub use collector::{QuantumStatisticsCollector, CounterSnapshot};
pub use tree_stats::{
    QuantumTreeStatistics, DepthStatistics, RewardStatistics, ConvergenceMetrics,
    RewardQuality, ConvergencePhase, ConvergenceHealth,
};
pub use performance::{
    PerformanceMetrics, ThroughputMetrics, PerformanceBottleneck, PerformanceTrend,
    ThroughputAnalysis, Priority,
};
pub use trends::{
    StatisticsSnapshot, PerformanceTrends, PerformancePrediction, PredictionReliability,
    TrendRecommendation, TrendMomentum, SnapshotComparison,
};
pub use analysis::StatisticsUtils;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::cognitive::{
    quantum::{EntanglementGraph, QuantumMetrics},
    types::CognitiveError,
};
use super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// Statistics coordinator for managing all statistics functionality
#[derive(Debug)]
pub struct StatisticsCoordinator {
    /// Core statistics collector with atomic operations
    pub collector: QuantumStatisticsCollector,
    /// Configuration for statistical analysis
    config: QuantumMCTSConfig,
    /// Start time for coordinator lifecycle tracking
    start_time: Instant,
}

impl StatisticsCoordinator {
    /// Create new statistics coordinator with comprehensive initialization
    pub fn new(config: QuantumMCTSConfig) -> Self {
        info!("Initializing StatisticsCoordinator with zero-allocation patterns");
        
        let collector = QuantumStatisticsCollector::new(config.clone());
        
        Self {
            collector,
            config,
            start_time: Instant::now(),
        }
    }
    
    /// Collect comprehensive statistics with all analysis layers
    pub async fn collect_comprehensive_statistics(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        entanglement_graph: &RwLock<EntanglementGraph>,
        quantum_metrics: &RwLock<QuantumMetrics>,
    ) -> Result<QuantumTreeStatistics, CognitiveError> {
        debug!("Collecting comprehensive quantum tree statistics");
        
        let statistics = self.collector
            .collect_statistics(tree, entanglement_graph, quantum_metrics)
            .await?;
        
        debug!(
            "Statistics collected: {} nodes, {} visits, convergence: {:.3}",
            statistics.total_nodes,
            statistics.total_visits,
            statistics.convergence_metrics.overall_convergence
        );
        
        Ok(statistics)
    }
    
    /// Take snapshot and update trends with comprehensive analysis
    pub async fn take_comprehensive_snapshot(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        entanglement_graph: &RwLock<EntanglementGraph>,
        quantum_metrics: &RwLock<QuantumMetrics>,
    ) -> Result<StatisticsSnapshot, CognitiveError> {
        debug!("Taking comprehensive statistics snapshot");
        
        self.collector
            .take_snapshot(tree, entanglement_graph, quantum_metrics)
            .await?;
        
        let statistics = self.collector
            .collect_statistics(tree, entanglement_graph, quantum_metrics)
            .await?;
        
        let snapshot = StatisticsSnapshot::new(statistics);
        
        debug!(
            "Snapshot taken at {:.3}s with {} nodes",
            snapshot.age_seconds(),
            snapshot.statistics.total_nodes
        );
        
        Ok(snapshot)
    }
    
    /// Analyze current performance trends with comprehensive assessment
    pub async fn analyze_performance_trends(&self) -> Result<PerformanceTrends, CognitiveError> {
        debug!("Analyzing performance trends from historical data");
        
        let history = self.collector.get_history().await;
        let trends = if history.len() >= 2 {
            PerformanceTrends::from_snapshots(&history)
        } else {
            PerformanceTrends::default()
        };
        
        debug!(
            "Trends analyzed: grade={}, trending_up={}, stable={}",
            trends.performance_grade(),
            trends.trending_up,
            trends.is_stable
        );
        
        Ok(trends)
    }
    
    /// Get comprehensive performance assessment with recommendations
    pub async fn get_performance_assessment(&self) -> Result<PerformanceAssessment, CognitiveError> {
        debug!("Generating comprehensive performance assessment");
        
        let history = self.collector.get_history().await;
        if history.is_empty() {
            return Ok(PerformanceAssessment::empty());
        }
        
        let latest_statistics = &history.last().unwrap().statistics;
        let trends = self.analyze_performance_trends().await?;
        let counter_snapshot = self.collector.get_counter_values();
        
        // Analyze operation ratios
        let operation_ratios = counter_snapshot.operation_ratios();
        let bottlenecks = latest_statistics.performance_metrics.identify_bottlenecks();
        let recommendations = trends.get_recommendations();
        
        let assessment = PerformanceAssessment {
            overall_grade: latest_statistics.performance_grade(),
            tree_health_score: latest_statistics.tree_health_score(),
            convergence_health: latest_statistics.convergence_metrics.convergence_health(),
            performance_trend: latest_statistics.performance_metrics.performance_trend(),
            operation_balance: operation_ratios.is_balanced(),
            bottlenecks,
            recommendations,
            trends,
            counter_snapshot,
            assessment_time: Instant::now(),
        };
        
        info!(
            "Performance assessment: grade={}, health={:.3}, trend={:?}",
            assessment.overall_grade,
            assessment.tree_health_score,
            assessment.performance_trend
        );
        
        Ok(assessment)
    }
    
    /// Predict future performance based on current trends
    pub async fn predict_future_performance(
        &self,
        hours_ahead: f64,
    ) -> Result<PerformancePrediction, CognitiveError> {
        debug!("Predicting performance {} hours ahead", hours_ahead);
        
        let trends = self.analyze_performance_trends().await?;
        let prediction = trends.predict_future_performance(hours_ahead);
        
        debug!(
            "Prediction: {} nodes, {} visits, confidence={:.3}",
            prediction.predicted_nodes,
            prediction.predicted_visits,
            prediction.confidence
        );
        
        Ok(prediction)
    }
    
    /// Get detailed analysis report with all metrics
    pub async fn generate_analysis_report(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        entanglement_graph: &RwLock<EntanglementGraph>,
        quantum_metrics: &RwLock<QuantumMetrics>,
    ) -> Result<AnalysisReport, CognitiveError> {
        debug!("Generating comprehensive analysis report");
        
        let statistics = self.collect_comprehensive_statistics(
            tree, entanglement_graph, quantum_metrics
        ).await?;
        
        let trends = self.analyze_performance_trends().await?;
        let assessment = self.get_performance_assessment().await?;
        let prediction = self.predict_future_performance(24.0).await?; // 24-hour prediction
        
        let report = AnalysisReport {
            statistics,
            trends,
            assessment,
            prediction,
            generation_time: Instant::now(),
            coordinator_uptime: self.start_time.elapsed(),
        };
        
        info!(
            "Analysis report generated: {} nodes analyzed, uptime={:.1}s",
            report.statistics.total_nodes,
            report.coordinator_uptime.as_secs_f64()
        );
        
        Ok(report)
    }
    
    /// Reset all statistics and counters
    pub async fn reset_statistics(&self) -> Result<(), CognitiveError> {
        info!("Resetting all statistics and counters");
        
        self.collector.reset_counters();
        
        // Clear history
        let mut history = self.collector.history.write().await;
        history.clear();
        
        debug!("Statistics reset completed");
        Ok(())
    }
    
    /// Get current counter values for real-time monitoring
    pub fn get_real_time_counters(&self) -> CounterSnapshot {
        self.collector.get_counter_values()
    }
    
    /// Record operation for real-time tracking
    pub fn record_operation(&self, operation: OperationType) {
        match operation {
            OperationType::Selection => self.collector.record_selection(),
            OperationType::Expansion => self.collector.record_expansion(),
            OperationType::Backpropagation => self.collector.record_backpropagation(),
            OperationType::Simulation => self.collector.record_simulation(),
        }
    }
    
    /// Get coordinator uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    
    /// Get configuration snapshot
    pub fn get_config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
}

/// Operation types for recording
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Selection,
    Expansion,
    Backpropagation,
    Simulation,
}

/// Comprehensive performance assessment
#[derive(Debug, Clone)]
pub struct PerformanceAssessment {
    /// Overall performance grade
    pub overall_grade: char,
    /// Tree health score (0.0 to 1.0)
    pub tree_health_score: f64,
    /// Convergence health status
    pub convergence_health: ConvergenceHealth,
    /// Performance trend indicator
    pub performance_trend: PerformanceTrend,
    /// Whether operations are balanced
    pub operation_balance: bool,
    /// Identified performance bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Trend-based recommendations
    pub recommendations: Vec<TrendRecommendation>,
    /// Detailed performance trends
    pub trends: PerformanceTrends,
    /// Current counter snapshot
    pub counter_snapshot: CounterSnapshot,
    /// When assessment was generated
    pub assessment_time: Instant,
}

impl PerformanceAssessment {
    /// Create empty assessment for initialization
    pub fn empty() -> Self {
        Self {
            overall_grade: 'F',
            tree_health_score: 0.0,
            convergence_health: ConvergenceHealth::Poor,
            performance_trend: PerformanceTrend::Poor,
            operation_balance: false,
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
            trends: PerformanceTrends::default(),
            counter_snapshot: CounterSnapshot {
                nodes: 0,
                visits: 0,
                selections: 0,
                expansions: 0,
                backpropagations: 0,
                simulations: 0,
            },
            assessment_time: Instant::now(),
        }
    }
    
    /// Check if performance is acceptable
    pub fn is_acceptable(&self) -> bool {
        self.overall_grade >= 'C' && 
        self.tree_health_score > 0.5 &&
        self.convergence_health.is_acceptable()
    }
    
    /// Get summary description of current state
    pub fn summary(&self) -> String {
        format!(
            "Grade: {}, Health: {:.1}%, Trend: {:?}, Balance: {}",
            self.overall_grade,
            self.tree_health_score * 100.0,
            self.performance_trend,
            if self.operation_balance { "Good" } else { "Poor" }
        )
    }
    
    /// Get priority recommendations
    pub fn priority_recommendations(&self) -> Vec<&TrendRecommendation> {
        self.recommendations.iter()
            .filter(|rec| rec.priority() >= trends::Priority::Medium)
            .collect()
    }
}

/// Comprehensive analysis report
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// Current statistics snapshot
    pub statistics: QuantumTreeStatistics,
    /// Performance trends analysis
    pub trends: PerformanceTrends,
    /// Performance assessment
    pub assessment: PerformanceAssessment,
    /// Future performance prediction
    pub prediction: PerformancePrediction,
    /// When report was generated
    pub generation_time: Instant,
    /// Coordinator uptime when generated
    pub coordinator_uptime: std::time::Duration,
}

impl AnalysisReport {
    /// Get executive summary of the report
    pub fn executive_summary(&self) -> String {
        format!(
            "QUANTUM MCTS ANALYSIS REPORT\n\
             ===========================\n\
             Nodes: {} | Visits: {} | Convergence: {:.1}%\n\
             Grade: {} | Health: {:.1}% | Trend: {:?}\n\
             Uptime: {:.1}s | Generated: {:.3}s ago",
            self.statistics.total_nodes,
            self.statistics.total_visits,
            self.statistics.convergence_metrics.overall_convergence * 100.0,
            self.assessment.overall_grade,
            self.assessment.tree_health_score * 100.0,
            self.assessment.performance_trend,
            self.coordinator_uptime.as_secs_f64(),
            self.generation_time.elapsed().as_secs_f64()
        )
    }
    
    /// Check if report indicates healthy system
    pub fn indicates_healthy_system(&self) -> bool {
        self.assessment.is_acceptable() && 
        self.trends.is_performing_well() &&
        self.prediction.is_positive()
    }
}
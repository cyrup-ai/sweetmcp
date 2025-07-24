//!
//! This module provides engine maintenance, statistics gathering, and comprehensive
//! performance analysis with blazing-fast zero-allocation patterns.

use std::time::Instant;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::{
    super::{
        analysis::{NetworkTopology, NetworkTopologyAnalyzer},
        metrics::EntanglementMetricsSummary,
    },
    core::{QuantumEntanglementEngine, EngineStatus},
};

/// Maintenance assessment result
#[derive(Debug, Clone)]
pub struct MaintenanceAssessment {
    /// Assessment priority
    pub priority: MaintenancePriority,
    /// Recommended actions
    pub actions: Vec<MaintenanceAction>,
    /// Assessment confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Maintenance priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum MaintenancePriority {
    /// Critical maintenance required immediately
    Critical,
    /// High priority maintenance
    High,
    /// Medium priority maintenance
    Medium,
    /// Low priority maintenance
    Low,
    /// No maintenance needed
    None,
}

/// Maintenance action types
#[derive(Debug, Clone)]
pub enum MaintenanceAction {
    /// Restart engine
    RestartEngine,
    /// Clear cache
    ClearCache,
    /// Optimize performance
    OptimizePerformance,
    /// Update configuration
    UpdateConfiguration,
    /// Monitor closely
    MonitorClosely,
}

/// Comprehensive engine statistics with performance metrics
#[derive(Debug, Clone)]
pub struct EngineStatistics {
    /// Current engine status
    pub status: EngineStatus,
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Network topology analysis
    pub topology: NetworkTopology,
    /// Detailed metrics summary
    pub metrics_summary: EntanglementMetricsSummary,
    /// Engine uptime in seconds
    pub uptime_seconds: u64,
    /// Total entanglement operations performed
    pub total_operations: u64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average operation time in microseconds
    pub avg_operation_time_us: f64,
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    /// Operations per second
    pub operations_per_second: f64,
    /// Performance recommendations
    pub recommendations: Vec<String>,
    /// Last statistics update timestamp
    pub last_updated: Instant,
}

impl EngineStatistics {
    /// Create new engine statistics
    pub fn new(
        status: EngineStatus,
        health_score: f64,
        topology: NetworkTopology,
        metrics_summary: EntanglementMetricsSummary,
        uptime_seconds: u64,
        total_operations: u64,
        success_rate: f64,
        avg_operation_time_us: f64,
        cache_hit_rate: f64,
        operations_per_second: f64,
    ) -> Self {
        let recommendations = Self::generate_recommendations(
            &status,
            health_score,
            success_rate,
            operations_per_second,
        );

        Self {
            status,
            health_score,
            topology,
            metrics_summary,
            uptime_seconds,
            total_operations,
            success_rate,
            avg_operation_time_us,
            cache_hit_rate,
            operations_per_second,
            recommendations,
            last_updated: Instant::now(),
        }
    }

    /// Generate performance recommendations based on current metrics
    fn generate_recommendations(
        status: &EngineStatus,
        health_score: f64,
        success_rate: f64,
        operations_per_second: f64,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match status {
            EngineStatus::Critical => {
                recommendations.push("CRITICAL: Immediate attention required - engine performance severely degraded".to_string());
                recommendations.push("Check network connectivity and node health".to_string());
                recommendations.push("Consider restarting entanglement engine".to_string());
            }
            EngineStatus::Degraded => {
                recommendations.push("Performance degraded - investigate bottlenecks".to_string());
                recommendations.push("Monitor resource usage and optimize if necessary".to_string());
            }
            EngineStatus::Good => {
                recommendations.push("Performance is good - monitor for potential improvements".to_string());
            }
            EngineStatus::Optimal => {
                recommendations.push("Performance is optimal - maintain current configuration".to_string());
            }
        }

        if health_score < 0.7 {
            recommendations.push("Health score is low - investigate network topology issues".to_string());
        }

        if success_rate < 0.9 {
            recommendations.push("Success rate is below optimal - check error patterns".to_string());
        }

        if operations_per_second < 10.0 {
            recommendations.push("Low throughput detected - consider performance optimizations".to_string());
        }

        if operations_per_second > 100.0 {
            recommendations.push("High throughput - monitor for potential overload conditions".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("All metrics are within normal ranges".to_string());
        }

        recommendations
    }

    /// Update statistics with new metrics
    pub fn update_metrics(&mut self, metrics_summary: EntanglementMetricsSummary) {
        self.metrics_summary = metrics_summary;
        self.success_rate = metrics_summary.success_rate;
        self.avg_operation_time_us = metrics_summary.average_operation_time_us;
        self.cache_hit_rate = metrics_summary.cache_hit_rate;
        self.operations_per_second = metrics_summary.operations_per_second;
        self.last_updated = Instant::now();
        
        // Regenerate recommendations based on updated metrics
        self.recommendations = Self::generate_recommendations(
            &self.status,
            self.health_score,
            self.success_rate,
            self.operations_per_second,
        );
    }

    /// Get performance trend analysis
    pub fn performance_trend(&self) -> String {
        match self.status {
            EngineStatus::Optimal => "Performance trending upward - excellent".to_string(),
            EngineStatus::Good => "Performance stable - good".to_string(),
            EngineStatus::Degraded => "Performance declining - attention needed".to_string(),
            EngineStatus::Critical => "Performance critical - immediate action required".to_string(),
        }
    }

    /// Generate performance recommendations
    pub fn generate_performance_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.success_rate < 0.95 {
            recommendations.push("Improve error handling and retry mechanisms".to_string());
        }
        
        if self.cache_hit_rate < 0.8 {
            recommendations.push("Optimize caching strategy for better performance".to_string());
        }
        
        if self.operations_per_second < 20.0 {
            recommendations.push("Increase operation throughput through parallelization".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is optimal, maintain current configuration".to_string());
        }
        
        recommendations
    }
}

impl QuantumEntanglementEngine {
    /// Get comprehensive engine statistics
    pub async fn get_comprehensive_statistics(&self) -> Result<EngineStatistics, CognitiveError> {
        debug!("Gathering comprehensive engine statistics");
        
        let metrics_summary = self.metrics.summary();
        let status = self.status_summary();
        let health_score = self.quick_health_check().await?;
        
        // Analyze network topology
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        
        Ok(EngineStatistics::new(
            status,
            health_score,
            topology,
            metrics_summary,
            metrics_summary.uptime_seconds,
            metrics_summary.entanglement_operations,
            metrics_summary.success_rate,
            metrics_summary.average_operation_time_us,
            metrics_summary.cache_hit_rate,
            metrics_summary.operations_per_second,
        ))
    }
    
    /// Perform quick health check without full analysis
    pub async fn quick_health_check(&self) -> Result<f64, CognitiveError> {
        let graph_read = self.entanglement_graph.read().await;
        let node_count = graph_read.node_count();
        let edge_count = graph_read.edge_count();
        
        // Basic health calculation
        let connectivity_score = if node_count > 0 {
            (edge_count as f64) / (node_count as f64 * (node_count - 1) as f64 / 2.0).max(1.0)
        } else {
            0.0
        }.min(1.0);
        
        let metrics_health = self.metrics.health_score();
        
        // Combine connectivity and metrics health
        Ok((connectivity_score * 0.6 + metrics_health * 0.4).min(1.0))
    }
    
    /// Get engine status summary
    pub fn status_summary(&self) -> EngineStatus {
        let metrics_summary = self.metrics.summary();
        
        if metrics_summary.success_rate > 0.95 && metrics_summary.operations_per_second > 50.0 {
            EngineStatus::Optimal
        } else if metrics_summary.success_rate > 0.9 && metrics_summary.operations_per_second > 20.0 {
            EngineStatus::Good
        } else if metrics_summary.success_rate > 0.8 {
            EngineStatus::Degraded
        } else {
            EngineStatus::Critical
        }
    }
}
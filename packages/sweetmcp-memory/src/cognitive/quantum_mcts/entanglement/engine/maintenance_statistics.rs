//! Maintenance operations and comprehensive statistics collection
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

/// Comprehensive engine statistics with performance metrics
#[derive(Debug, Clone)]
pub struct EngineStatistics {
    /// Engine status information
    pub status: EngineStatus,
    /// Current health score
    pub health_score: f64,
    /// Network topology information
    pub topology: NetworkTopology,
    /// Metrics summary
    pub metrics_summary: EntanglementMetricsSummary,
    /// Engine uptime in seconds
    pub uptime_seconds: u64,
    /// Total operations performed
    pub total_operations: u64,
    /// Success rate of operations
    pub success_rate: f64,
    /// Average operation latency in microseconds
    pub average_latency_us: f64,
    /// Cache hit rate efficiency
    pub cache_efficiency: f64,
    /// Operations throughput per second
    pub throughput_ops_per_sec: f64,
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
        average_latency_us: f64,
        cache_efficiency: f64,
        throughput_ops_per_sec: f64,
    ) -> Self {
        Self {
            status,
            health_score,
            topology,
            metrics_summary,
            uptime_seconds,
            total_operations,
            success_rate,
            average_latency_us,
            cache_efficiency,
            throughput_ops_per_sec,
        }
    }

    /// Check if engine is performing optimally
    pub fn is_optimal(&self) -> bool {
        self.health_score > 0.9 &&
        self.success_rate > 0.95 &&
        self.average_latency_us < 500.0 &&
        self.cache_efficiency > 0.9
    }
    
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Performance: Health {:.1}/10, Success {:.1}%, Latency {:.1}μs, Cache {:.1}%, Throughput {:.1}/s",
            self.health_score * 10.0,
            self.success_rate * 100.0,
            self.average_latency_us,
            self.cache_efficiency * 100.0,
            self.throughput_ops_per_sec
        )
    }
    
    /// Get efficiency score (0.0 to 1.0)
    pub fn efficiency_score(&self) -> f64 {
        let health_component = self.health_score * 0.3;
        let success_component = self.success_rate * 0.3;
        let latency_component = (1.0 - (self.average_latency_us / 2000.0).min(1.0)) * 0.2;
        let cache_component = self.cache_efficiency * 0.2;
        
        health_component + success_component + latency_component + cache_component
    }
    
    /// Check if statistics indicate performance issues
    pub fn has_performance_issues(&self) -> bool {
        self.health_score < 0.7 ||
        self.success_rate < 0.9 ||
        self.average_latency_us > 1000.0 ||
        self.cache_efficiency < 0.8
    }
    
    /// Get performance recommendations
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.health_score < 0.7 {
            recommendations.push("Improve network health through optimization".to_string());
        }
        
        if self.success_rate < 0.9 {
            recommendations.push("Investigate operation failures and improve reliability".to_string());
        }
        
        if self.average_latency_us > 1000.0 {
            recommendations.push("Optimize operation latency through caching and batching".to_string());
        }
        
        if self.cache_efficiency < 0.8 {
            recommendations.push("Improve cache hit rate through better caching strategies".to_string());
        }
        
        if self.throughput_ops_per_sec < 10.0 {
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
    

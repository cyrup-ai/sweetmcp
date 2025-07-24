//! Comprehensive analysis reporting for quantum entanglement networks
//!
//! This module provides detailed analysis reports combining topology, quality,
//! health metrics, and bottleneck analysis with zero-allocation patterns and
//! blazing-fast performance.

use std::collections::HashMap;
use std::time::Instant;

use crate::cognitive::types::CognitiveError;
use super::{
    node_state::QuantumMCTSNode,
    analysis::{NetworkTopology, EntanglementQuality, NetworkTopologyAnalyzer, NetworkBottleneck},
    engine::{QuantumEntanglementEngine, NetworkHealthReport},
    metrics::EntanglementMetricsSummary,
};

/// Comprehensive analysis report combining all network metrics
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisReport {
    /// Network topology analysis
    pub topology: NetworkTopology,
    /// Entanglement quality assessment
    pub quality: EntanglementQuality,
    /// Network health report
    pub health_report: NetworkHealthReport,
    /// Identified bottlenecks
    pub bottlenecks: Vec<NetworkBottleneck>,
    /// Metrics summary
    pub metrics_summary: EntanglementMetricsSummary,
    /// Analysis execution time in milliseconds
    pub analysis_time_ms: u64,
    /// Number of nodes analyzed
    pub node_count: usize,
    /// Analysis timestamp
    pub timestamp: Instant,
}

impl ComprehensiveAnalysisReport {
    /// Generate comprehensive analysis report
    pub async fn generate(
        engine: &QuantumEntanglementEngine,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<Self, CognitiveError> {
        let start_time = Instant::now();
        
        // Perform all analysis components
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(
            &engine.manager().entanglement_graph
        ).await?;
        
        let quality = NetworkTopologyAnalyzer::analyze_entanglement_quality(
            &engine.manager().entanglement_graph, 
            0.7, // quality threshold
            0.4  // strength threshold
        ).await?;
        
        let health_report = engine.health_check().await?;
        
        let bottlenecks = NetworkTopologyAnalyzer::find_network_bottlenecks(
            &engine.manager().entanglement_graph, 
            tree
        ).await?;
        
        let metrics_summary = engine.metrics().summary();
        
        let analysis_time = start_time.elapsed();
        
        Ok(Self {
            topology,
            quality,
            health_report,
            bottlenecks,
            metrics_summary,
            analysis_time_ms: analysis_time.as_millis() as u64,
            node_count: tree.len(),
            timestamp: Instant::now(),
        })
    }
    
    /// Get overall network score (0.0 to 1.0)
    pub fn overall_score(&self) -> f64 {
        let topology_score = self.topology.efficiency_score();
        let quality_score = self.quality.overall_quality;
        let health_score = self.health_report.health_score;
        let bottleneck_score = if self.bottlenecks.is_empty() { 1.0 } else { 0.8 };
        
        (topology_score * 0.3 + quality_score * 0.25 + health_score * 0.25 + bottleneck_score * 0.2)
            .max(0.0).min(1.0)
    }
    
    /// Get overall grade (A-F)
    pub fn overall_grade(&self) -> char {
        let score = self.overall_score();
        if score >= 0.9 { 'A' }
        else if score >= 0.8 { 'B' }
        else if score >= 0.7 { 'C' }
        else if score >= 0.6 { 'D' }
        else { 'F' }
    }
    
    /// Check if network is performing well
    pub fn is_performing_well(&self) -> bool {
        self.overall_score() >= 0.7 && 
        self.topology.has_good_connectivity() &&
        self.health_report.is_healthy() &&
        self.bottlenecks.len() <= 2
    }
    
    /// Generate detailed analysis summary
    pub fn detailed_summary(&self) -> String {
        format!(
            "Comprehensive Entanglement Network Analysis\n\
             ==========================================\n\
             \n\
             Overall Score: {:.2} (Grade: {})\n\
             Analysis Time: {}ms\n\
             Nodes Analyzed: {}\n\
             \n\
             Topology Analysis:\n\
             - Efficiency Score: {:.2}\n\
             - Connectivity: {}\n\
             - Average Path Length: {:.2}\n\
             - Clustering Coefficient: {:.2}\n\
             \n\
             Quality Assessment:\n\
             - Overall Quality: {:.2}\n\
             - Average Strength: {:.2}\n\
             - Strong Connections: {}\n\
             - Weak Connections: {}\n\
             \n\
             Health Report:\n\
             - Health Score: {:.2}\n\
             - Status: {}\n\
             - Active Issues: {}\n\
             \n\
             Bottlenecks:\n\
             - Count: {}\n\
             - Critical: {}\n\
             \n\
             Performance Metrics:\n\
             - Entanglements Created: {}\n\
             - Operations Attempted: {}\n\
             - Success Rate: {:.1}%\n\
             - Average Latency: {:.2}ms",
            self.overall_score(),
            self.overall_grade(),
            self.analysis_time_ms,
            self.node_count,
            self.topology.efficiency_score(),
            if self.topology.has_good_connectivity() { "Good" } else { "Poor" },
            self.topology.average_path_length(),
            self.topology.clustering_coefficient(),
            self.quality.overall_quality,
            self.quality.average_strength,
            self.quality.strong_connections,
            self.quality.weak_connections,
            self.health_report.health_score,
            if self.health_report.is_healthy() { "Healthy" } else { "Unhealthy" },
            self.health_report.active_issues.len(),
            self.bottlenecks.len(),
            self.bottlenecks.iter().filter(|b| b.is_critical()).count(),
            self.metrics_summary.entanglements_created(),
            self.metrics_summary.operations_attempted(),
            if self.metrics_summary.operations_attempted() > 0 {
                (self.metrics_summary.operations_successful() as f64 / 
                 self.metrics_summary.operations_attempted() as f64) * 100.0
            } else {
                100.0
            },
            self.metrics_summary.average_operation_latency().as_secs_f64() * 1000.0
        )
    }
    
    /// Get critical issues that need immediate attention
    pub fn critical_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        if self.overall_score() < 0.5 {
            issues.push("Overall network performance is critically low".to_string());
        }
        
        if !self.topology.has_good_connectivity() {
            issues.push("Network topology has poor connectivity".to_string());
        }
        
        if !self.health_report.is_healthy() {
            issues.push("Network health check failed".to_string());
        }
        
        let critical_bottlenecks = self.bottlenecks.iter().filter(|b| b.is_critical()).count();
        if critical_bottlenecks > 0 {
            issues.push(format!("{} critical bottlenecks detected", critical_bottlenecks));
        }
        
        if self.quality.overall_quality < 0.4 {
            issues.push("Entanglement quality is critically low".to_string());
        }
        
        let success_rate = if self.metrics_summary.operations_attempted() > 0 {
            self.metrics_summary.operations_successful() as f64 / 
            self.metrics_summary.operations_attempted() as f64
        } else {
            1.0
        };
        
        if success_rate < 0.8 {
            issues.push(format!("Operation success rate is low: {:.1}%", success_rate * 100.0));
        }
        
        issues
    }
    
    /// Get recommendations for improvement
    pub fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.topology.efficiency_score() < 0.7 {
            recommendations.push("Consider optimizing network topology for better efficiency".to_string());
        }
        
        if self.quality.weak_connections > self.quality.strong_connections {
            recommendations.push("Prune weak connections and strengthen remaining entanglements".to_string());
        }
        
        if self.bottlenecks.len() > 3 {
            recommendations.push("Address network bottlenecks to improve performance".to_string());
        }
        
        if self.analysis_time_ms > 1000 {
            recommendations.push("Analysis time is high - consider optimizing analysis algorithms".to_string());
        }
        
        let avg_latency_ms = self.metrics_summary.average_operation_latency().as_secs_f64() * 1000.0;
        if avg_latency_ms > 100.0 {
            recommendations.push("Operation latency is high - investigate performance bottlenecks".to_string());
        }
        
        if self.node_count > 1000 && self.topology.clustering_coefficient() < 0.3 {
            recommendations.push("Large network with low clustering - consider hierarchical organization".to_string());
        }
        
        recommendations
    }
    
    /// Check if analysis is stale (older than specified duration)
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.timestamp.elapsed() > max_age
    }
    
    /// Get analysis age
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
    
    /// Create a condensed version for quick overview
    pub fn condensed_report(&self) -> String {
        format!(
            "Network Analysis: {} ({:.2}) | {} nodes | {}ms | {} issues",
            self.overall_grade(),
            self.overall_score(),
            self.node_count,
            self.analysis_time_ms,
            self.critical_issues().len()
        )
    }
    
    /// Export analysis data for external processing
    pub fn export_data(&self) -> AnalysisExportData {
        AnalysisExportData {
            overall_score: self.overall_score(),
            overall_grade: self.overall_grade(),
            topology_efficiency: self.topology.efficiency_score(),
            quality_score: self.quality.overall_quality,
            health_score: self.health_report.health_score,
            bottleneck_count: self.bottlenecks.len(),
            critical_bottlenecks: self.bottlenecks.iter().filter(|b| b.is_critical()).count(),
            node_count: self.node_count,
            analysis_time_ms: self.analysis_time_ms,
            entanglements_created: self.metrics_summary.entanglements_created(),
            operations_attempted: self.metrics_summary.operations_attempted(),
            operations_successful: self.metrics_summary.operations_successful(),
            average_latency_ms: self.metrics_summary.average_operation_latency().as_secs_f64() * 1000.0,
            timestamp: self.timestamp,
        }
    }
}

/// Exported analysis data for external processing
#[derive(Debug, Clone)]
pub struct AnalysisExportData {
    pub overall_score: f64,
    pub overall_grade: char,
    pub topology_efficiency: f64,
    pub quality_score: f64,
    pub health_score: f64,
    pub bottleneck_count: usize,
    pub critical_bottlenecks: usize,
    pub node_count: usize,
    pub analysis_time_ms: u64,
    pub entanglements_created: u64,
    pub operations_attempted: u64,
    pub operations_successful: u64,
    pub average_latency_ms: f64,
    pub timestamp: Instant,
}

impl AnalysisExportData {
    /// Convert to JSON-serializable format (without Instant)
    pub fn to_serializable(&self) -> SerializableAnalysisData {
        SerializableAnalysisData {
            overall_score: self.overall_score,
            overall_grade: self.overall_grade,
            topology_efficiency: self.topology_efficiency,
            quality_score: self.quality_score,
            health_score: self.health_score,
            bottleneck_count: self.bottleneck_count,
            critical_bottlenecks: self.critical_bottlenecks,
            node_count: self.node_count,
            analysis_time_ms: self.analysis_time_ms,
            entanglements_created: self.entanglements_created,
            operations_attempted: self.operations_attempted,
            operations_successful: self.operations_successful,
            average_latency_ms: self.average_latency_ms,
            timestamp_secs: self.timestamp.elapsed().as_secs(),
        }
    }
}

/// Serializable version of analysis data
#[derive(Debug, Clone)]
pub struct SerializableAnalysisData {
    pub overall_score: f64,
    pub overall_grade: char,
    pub topology_efficiency: f64,
    pub quality_score: f64,
    pub health_score: f64,
    pub bottleneck_count: usize,
    pub critical_bottlenecks: usize,
    pub node_count: usize,
    pub analysis_time_ms: u64,
    pub entanglements_created: u64,
    pub operations_attempted: u64,
    pub operations_successful: u64,
    pub average_latency_ms: f64,
    pub timestamp_secs: u64,
}

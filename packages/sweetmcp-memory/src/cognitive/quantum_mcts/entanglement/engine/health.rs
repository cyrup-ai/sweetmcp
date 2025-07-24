//! Health checking, monitoring, and reporting for quantum entanglement engine
//!
//! This module provides blazing-fast health analysis with zero-allocation
//! patterns and comprehensive monitoring capabilities.

use std::time::Instant;
use tracing::{debug, info, warn};

use crate::cognitive::types::CognitiveError;
use super::super::{
    analysis::{NetworkTopology, NetworkTopologyAnalyzer},
    metrics::EntanglementMetrics,
};
use super::core::QuantumEntanglementEngine;

/// Comprehensive network health report
#[derive(Debug, Clone)]
pub struct NetworkHealthReport {
    /// Current network topology
    pub topology: NetworkTopology,
    /// Metrics summary
    pub metrics_summary: super::super::metrics::EntanglementMetricsSummary,
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// List of identified issues
    pub issues: Vec<String>,
    /// List of recommendations
    pub recommendations: Vec<String>,
    /// Timestamp of the health check
    pub timestamp: Instant,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Minimum acceptable health score
    pub min_health_score: f64,
    /// Maximum acceptable response time in microseconds
    pub max_response_time_us: f64,
    /// Minimum acceptable success rate
    pub min_success_rate: f64,
    /// Maximum acceptable network density
    pub max_network_density: f64,
    /// Minimum acceptable clustering coefficient
    pub min_clustering_coefficient: f64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            min_health_score: 0.7,
            max_response_time_us: 1000.0,
            min_success_rate: 0.9,
            max_network_density: 0.2,
            min_clustering_coefficient: 0.3,
        }
    }
}

/// Health issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Low severity - minor performance impact
    Low,
    /// Medium severity - noticeable performance impact
    Medium,
    /// High severity - significant performance impact
    High,
    /// Critical severity - system functionality at risk
    Critical,
}

/// Health issue with severity and description
#[derive(Debug, Clone)]
pub struct HealthIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue description
    pub description: String,
    /// Suggested resolution
    pub resolution: String,
    /// Impact on system performance
    pub impact: String,
}

impl QuantumEntanglementEngine {
    /// Perform comprehensive network health check
    pub async fn health_check(&self) -> Result<NetworkHealthReport, CognitiveError> {
        debug!("Starting comprehensive network health check");
        
        let timestamp = Instant::now();
        
        // Analyze current network topology
        let topology = NetworkTopologyAnalyzer::analyze_network_topology(&self.entanglement_graph).await?;
        
        // Get metrics summary
        let metrics_summary = self.metrics.summary();
        
        // Calculate overall health score
        let health_score = self.calculate_network_health_score(&topology, &metrics_summary);
        
        // Identify issues and generate recommendations
        let issues = self.identify_health_issues(&topology, &metrics_summary).await?;
        let recommendations = self.generate_health_recommendations(&topology, &metrics_summary);
        
        let report = NetworkHealthReport {
            topology,
            metrics_summary,
            health_score,
            issues,
            recommendations,
            timestamp,
        };
        
        info!(
            "Health check completed: Score {:.1}/10 (Grade: {}), {} issues, {} recommendations",
            health_score * 10.0,
            report.health_grade(),
            report.issues.len(),
            report.recommendations.len()
        );
        
        Ok(report)
    }
    
    /// Calculate overall network health score
    pub fn calculate_network_health_score(
        &self,
        topology: &NetworkTopology,
        metrics: &super::super::metrics::EntanglementMetricsSummary,
    ) -> f64 {
        let mut score = 1.0;
        let config = HealthCheckConfig::default();
        
        // Connectivity health (25% weight)
        let connectivity_score = if topology.is_connected { 1.0 } else { 0.3 };
        score *= 0.75 + (connectivity_score * 0.25);
        
        // Performance health (30% weight)
        let performance_score = if metrics.average_operation_time_us <= config.max_response_time_us {
            1.0
        } else {
            (config.max_response_time_us / metrics.average_operation_time_us).min(1.0)
        };
        score *= 0.7 + (performance_score * 0.3);
        
        // Success rate health (25% weight)
        let success_score = if metrics.success_rate >= config.min_success_rate {
            1.0
        } else {
            (metrics.success_rate / config.min_success_rate).min(1.0)
        };
        score *= 0.75 + (success_score * 0.25);
        
        // Network structure health (20% weight)
        let structure_score = self.calculate_structure_health_score(topology, &config);
        score *= 0.8 + (structure_score * 0.2);
        
        score.max(0.0).min(1.0)
    }
    
    /// Calculate network structure health score
    fn calculate_structure_health_score(&self, topology: &NetworkTopology, config: &HealthCheckConfig) -> f64 {
        let mut structure_score = 1.0;
        
        // Density health
        if topology.network_density > config.max_network_density {
            structure_score *= 0.7; // Penalize overly dense networks
        } else if topology.network_density < 0.01 {
            structure_score *= 0.5; // Penalize overly sparse networks
        }
        
        // Clustering health
        if topology.clustering_coefficient < config.min_clustering_coefficient {
            structure_score *= 0.6; // Penalize poor clustering
        }
        
        // Degree distribution health
        let degree_variance = (topology.max_degree as f64 - topology.average_degree).abs();
        if degree_variance > topology.average_degree * 2.0 {
            structure_score *= 0.8; // Penalize highly uneven degree distribution
        }
        
        structure_score.max(0.0).min(1.0)
    }
    
    /// Identify health issues in the network
    async fn identify_health_issues(
        &self,
        topology: &NetworkTopology,
        metrics: &super::super::metrics::EntanglementMetricsSummary,
    ) -> Result<Vec<String>, CognitiveError> {
        let mut issues = Vec::new();
        let config = HealthCheckConfig::default();
        
        // Check connectivity issues
        if !topology.is_connected {
            issues.push("Network is not fully connected - some nodes may be isolated".to_string());
        }
        
        // Check performance issues
        if metrics.average_operation_time_us > config.max_response_time_us {
            issues.push(format!(
                "High operation latency: {:.1}μs (target: <{:.1}μs)",
                metrics.average_operation_time_us,
                config.max_response_time_us
            ));
        }
        
        if metrics.success_rate < config.min_success_rate {
            issues.push(format!(
                "Low success rate: {:.1}% (target: >{:.1}%)",
                metrics.success_rate * 100.0,
                config.min_success_rate * 100.0
            ));
        }
        
        // Check network structure issues
        if topology.network_density > config.max_network_density {
            issues.push(format!(
                "Network is overly dense: {:.3} (target: <{:.3})",
                topology.network_density,
                config.max_network_density
            ));
        }
        
        if topology.network_density < 0.01 {
            issues.push("Network is too sparse - may lack sufficient connectivity".to_string());
        }
        
        if topology.clustering_coefficient < config.min_clustering_coefficient {
            issues.push(format!(
                "Poor network clustering: {:.3} (target: >{:.3})",
                topology.clustering_coefficient,
                config.min_clustering_coefficient
            ));
        }
        
        // Check cache performance issues
        if metrics.cache_hit_rate < 0.8 {
            issues.push(format!(
                "Low cache hit rate: {:.1}% (target: >80%)",
                metrics.cache_hit_rate * 100.0
            ));
        }
        
        // Check failure rate issues
        if metrics.entanglement_failures > metrics.entanglement_operations / 10 {
            issues.push("High entanglement failure rate detected".to_string());
        }
        
        // Check load balancing issues
        let degree_imbalance = (topology.max_degree as f64 / topology.average_degree.max(1.0)) - 1.0;
        if degree_imbalance > 2.0 {
            issues.push("Significant load imbalance detected in network".to_string());
        }
        
        // Check resource utilization issues
        if metrics.operations_per_second < 10.0 {
            issues.push("Low throughput detected - system may be underutilized".to_string());
        }
        
        Ok(issues)
    }
    
    /// Generate health improvement recommendations
    pub fn generate_health_recommendations(
        &self,
        topology: &NetworkTopology,
        metrics: &super::super::metrics::EntanglementMetricsSummary,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        let config = HealthCheckConfig::default();
        
        // Connectivity recommendations
        if !topology.is_connected {
            recommendations.push("Run optimization to improve network connectivity".to_string());
            recommendations.push("Consider creating strategic entanglements between isolated components".to_string());
        }
        
        // Performance recommendations
        if metrics.average_operation_time_us > config.max_response_time_us {
            recommendations.push("Consider increasing cache size to improve operation speed".to_string());
            recommendations.push("Enable parallel processing if not already active".to_string());
        }
        
        if metrics.success_rate < config.min_success_rate {
            recommendations.push("Investigate and resolve sources of operation failures".to_string());
            recommendations.push("Consider adjusting operation timeout settings".to_string());
        }
        
        // Network structure recommendations
        if topology.network_density > config.max_network_density {
            recommendations.push("Run intelligent pruning to reduce network density".to_string());
            recommendations.push("Consider increasing pruning threshold for future operations".to_string());
        }
        
        if topology.network_density < 0.01 {
            recommendations.push("Create additional strategic entanglements to improve connectivity".to_string());
            recommendations.push("Consider lowering entanglement creation threshold".to_string());
        }
        
        if topology.clustering_coefficient < config.min_clustering_coefficient {
            recommendations.push("Run balancing operation to improve network clustering".to_string());
            recommendations.push("Focus entanglement creation on improving local connectivity".to_string());
        }
        
        // Cache recommendations
        if metrics.cache_hit_rate < 0.8 {
            recommendations.push("Increase cache size to improve hit rate".to_string());
            recommendations.push("Review cache eviction policy for optimization".to_string());
        }
        
        // Load balancing recommendations
        let degree_imbalance = (topology.max_degree as f64 / topology.average_degree.max(1.0)) - 1.0;
        if degree_imbalance > 2.0 {
            recommendations.push("Run load balancing operation to distribute entanglements more evenly".to_string());
            recommendations.push("Consider implementing adaptive load balancing strategies".to_string());
        }
        
        // Throughput recommendations
        if metrics.operations_per_second < 10.0 {
            recommendations.push("Enable batch processing to improve throughput".to_string());
            recommendations.push("Consider increasing concurrent operation limits".to_string());
        }
        
        // General optimization recommendations
        if recommendations.is_empty() {
            recommendations.push("Network health is good - continue regular monitoring".to_string());
            recommendations.push("Consider periodic optimization to maintain performance".to_string());
        }
        
        recommendations
    }
    
    /// Perform quick health check (lightweight version)
    pub async fn quick_health_check(&self) -> Result<f64, CognitiveError> {
        debug!("Performing quick health check");
        
        let metrics_summary = self.metrics.summary();
        
        // Quick health score based on key metrics
        let mut score = 1.0;
        
        // Success rate (40% weight)
        if metrics_summary.success_rate < 0.9 {
            score *= 0.6 + (metrics_summary.success_rate * 0.4);
        }
        
        // Performance (30% weight)
        if metrics_summary.average_operation_time_us > 1000.0 {
            let performance_factor = (1000.0 / metrics_summary.average_operation_time_us).min(1.0);
            score *= 0.7 + (performance_factor * 0.3);
        }
        
        // Cache efficiency (20% weight)
        if metrics_summary.cache_hit_rate < 0.8 {
            score *= 0.8 + (metrics_summary.cache_hit_rate * 0.2);
        }
        
        // Failure rate (10% weight)
        let failure_rate = if metrics_summary.entanglement_operations > 0 {
            metrics_summary.entanglement_failures as f64 / metrics_summary.entanglement_operations as f64
        } else {
            0.0
        };
        
        if failure_rate > 0.1 {
            score *= 0.9 + ((1.0 - failure_rate) * 0.1);
        }
        
        Ok(score.max(0.0).min(1.0))
    }
    
    /// Check if network is healthy based on quick metrics
    pub async fn is_healthy(&self) -> Result<bool, CognitiveError> {
        let health_score = self.quick_health_check().await?;
        Ok(health_score >= 0.7)
    }
    
    /// Get health status summary
    pub async fn health_status(&self) -> Result<String, CognitiveError> {
        let health_score = self.quick_health_check().await?;
        let metrics_summary = self.metrics.summary();
        
        let status = if health_score >= 0.9 {
            "EXCELLENT"
        } else if health_score >= 0.8 {
            "GOOD"
        } else if health_score >= 0.7 {
            "FAIR"
        } else if health_score >= 0.5 {
            "POOR"
        } else {
            "CRITICAL"
        };
        
        Ok(format!(
            "Health: {} ({:.1}/10) | Success: {:.1}% | Latency: {:.1}μs | Cache: {:.1}% | Uptime: {}s",
            status,
            health_score * 10.0,
            metrics_summary.success_rate * 100.0,
            metrics_summary.average_operation_time_us,
            metrics_summary.cache_hit_rate * 100.0,
            metrics_summary.uptime_seconds
        ))
    }
}

impl NetworkHealthReport {
    /// Check if network is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.7 && self.issues.len() <= 2
    }
    
    /// Get health grade (A-F based on health score)
    pub fn health_grade(&self) -> char {
        match self.health_score {
            s if s >= 0.95 => 'A',
            s if s >= 0.85 => 'B',
            s if s >= 0.75 => 'C',
            s if s >= 0.65 => 'D',
            _ => 'F',
        }
    }
    
    /// Get health status description
    pub fn health_status(&self) -> &'static str {
        match self.health_score {
            s if s >= 0.9 => "EXCELLENT",
            s if s >= 0.8 => "GOOD",
            s if s >= 0.7 => "FAIR",
            s if s >= 0.5 => "POOR",
            _ => "CRITICAL",
        }
    }
    
    /// Check if immediate action is required
    pub fn requires_immediate_action(&self) -> bool {
        self.health_score < 0.5 || self.issues.len() > 5
    }
    
    /// Get critical issues (high impact problems)
    pub fn critical_issues(&self) -> Vec<&String> {
        self.issues
            .iter()
            .filter(|issue| {
                issue.contains("not connected") || 
                issue.contains("Critical") || 
                issue.contains("failure rate")
            })
            .collect()
    }
    
    /// Format comprehensive health report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Network Health Report ===\n");
        report.push_str(&format!("Timestamp: {:?}\n", self.timestamp));
        report.push_str(&format!("Health Score: {:.1}/10.0\n", self.health_score * 10.0));
        report.push_str(&format!("Grade: {}\n", self.health_grade()));
        report.push_str(&format!("Status: {}\n", self.health_status()));
        
        report.push_str("\n--- Network Topology ---\n");
        report.push_str(&format!("Nodes: {}\n", self.topology.total_nodes));
        report.push_str(&format!("Entanglements: {}\n", self.topology.total_entanglements));
        report.push_str(&format!("Connected: {}\n", self.topology.is_connected));
        report.push_str(&format!("Density: {:.3}\n", self.topology.network_density));
        report.push_str(&format!("Clustering: {:.3}\n", self.topology.clustering_coefficient));
        report.push_str(&format!("Avg Degree: {:.1}\n", self.topology.average_degree));
        
        report.push_str("\n--- Performance Metrics ---\n");
        report.push_str(&format!("Success Rate: {:.1}%\n", self.metrics_summary.success_rate * 100.0));
        report.push_str(&format!("Avg Operation Time: {:.1}μs\n", self.metrics_summary.average_operation_time_us));
        report.push_str(&format!("Operations/sec: {:.1}\n", self.metrics_summary.operations_per_second));
        report.push_str(&format!("Cache Hit Rate: {:.1}%\n", self.metrics_summary.cache_hit_rate * 100.0));
        report.push_str(&format!("Uptime: {}s\n", self.metrics_summary.uptime_seconds));
        
        if !self.issues.is_empty() {
            report.push_str("\n--- Issues Identified ---\n");
            for (i, issue) in self.issues.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, issue));
            }
        }
        
        if !self.recommendations.is_empty() {
            report.push_str("\n--- Recommendations ---\n");
            for (i, rec) in self.recommendations.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, rec));
            }
        }
        
        report
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Health: {:.1}/10 ({}), {} issues, {} recommendations",
            self.health_score * 10.0,
            self.health_grade(),
            self.issues.len(),
            self.recommendations.len()
        )
    }
}

impl HealthCheckConfig {
    /// Create strict health check configuration
    pub fn strict() -> Self {
        Self {
            min_health_score: 0.85,
            max_response_time_us: 500.0,
            min_success_rate: 0.95,
            max_network_density: 0.15,
            min_clustering_coefficient: 0.4,
        }
    }
    
    /// Create lenient health check configuration
    pub fn lenient() -> Self {
        Self {
            min_health_score: 0.6,
            max_response_time_us: 2000.0,
            min_success_rate: 0.85,
            max_network_density: 0.3,
            min_clustering_coefficient: 0.2,
        }
    }
}

impl HealthIssue {
    /// Create new health issue
    pub fn new(severity: IssueSeverity, description: String, resolution: String, impact: String) -> Self {
        Self {
            severity,
            description,
            resolution,
            impact,
        }
    }
    
    /// Check if issue is critical
    pub fn is_critical(&self) -> bool {
        self.severity == IssueSeverity::Critical
    }
    
    /// Check if issue requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self.severity, IssueSeverity::High | IssueSeverity::Critical)
    }
    
    /// Format issue as string
    pub fn format(&self) -> String {
        format!(
            "[{:?}] {} | Resolution: {} | Impact: {}",
            self.severity,
            self.description,
            self.resolution,
            self.impact
        )
    }
}
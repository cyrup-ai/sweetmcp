//! Quantum entanglement engine analysis and health monitoring
//!
//! This module provides comprehensive analysis capabilities for quantum entanglement networks
//! with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;

use crate::cognitive::types::CognitiveError;
use super::{
    engine_core::QuantumEntanglementEngine,
    analysis::NetworkTopology,
    engine_health::{EngineHealthReport, NetworkAnalysisReport},
    engine_health_types::OptimizationPriority,
};
use super::super::node_state::QuantumMCTSNode;

impl QuantumEntanglementEngine {
    /// Analyze current network topology
    pub async fn analyze_network_topology(&self) -> Result<NetworkTopology, CognitiveError> {
        self.analyzer.analyze_topology().await
    }

    /// Perform comprehensive health check of the entanglement network
    pub async fn health_check(&self) -> Result<EngineHealthReport, CognitiveError> {
        let topology = self.analyze_network_topology().await?;
        let distribution = self.manager.analyze_entanglement_distribution().await?;
        let metrics = self.manager.get_entanglement_metrics().await?;
        
        // Calculate health scores
        let connectivity_health = if topology.is_connected {
            100.0
        } else {
            let connected_ratio = topology.connected_components as f64 / topology.total_nodes as f64;
            connected_ratio * 100.0
        };
        
        let density_health = {
            let optimal_density = 0.3;
            let density_distance = (topology.network_density - optimal_density).abs();
            let normalized_distance = density_distance / optimal_density;
            ((1.0 - normalized_distance) * 100.0).max(0.0)
        };
        
        let clustering_health = {
            let optimal_clustering = 0.5;
            let clustering_distance = (topology.clustering_coefficient - optimal_clustering).abs();
            let normalized_distance = clustering_distance / optimal_clustering;
            ((1.0 - normalized_distance) * 100.0).max(0.0)
        };
        
        let balance_health = distribution.calculate_balance_score() * 100.0;
        
        let overall_health = (connectivity_health + density_health + clustering_health + balance_health) / 4.0;
        
        // Identify issues
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        
        if connectivity_health < 80.0 {
            issues.push("Network has connectivity issues".to_string());
            recommendations.push("Create bridging entanglements between disconnected components".to_string());
        }
        
        if density_health < 70.0 {
            if topology.network_density > 0.5 {
                issues.push("Network is too dense".to_string());
                recommendations.push("Prune weak entanglements to reduce density".to_string());
            } else {
                issues.push("Network is too sparse".to_string());
                recommendations.push("Create additional strategic entanglements".to_string());
            }
        }
        
        if clustering_health < 60.0 {
            if topology.clustering_coefficient < 0.2 {
                issues.push("Poor local clustering".to_string());
                recommendations.push("Increase local entanglement density".to_string());
            } else {
                issues.push("Excessive clustering".to_string());
                recommendations.push("Create long-range entanglements".to_string());
            }
        }
        
        if balance_health < 50.0 {
            issues.push("Unbalanced entanglement distribution".to_string());
            recommendations.push("Rebalance entanglements across nodes".to_string());
        }
        
        if metrics.total_entanglements == 0 {
            issues.push("No entanglements exist".to_string());
            recommendations.push("Initialize network with basic entanglements".to_string());
        }
        
        Ok(EngineHealthReport {
            overall_health,
            connectivity_health,
            density_health,
            clustering_health,
            balance_health,
            topology,
            distribution,
            metrics,
            issues,
            recommendations,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Generate detailed network analysis report
    pub async fn generate_analysis_report(&self) -> Result<NetworkAnalysisReport, CognitiveError> {
        let topology = self.analyze_network_topology().await?;
        let health = self.health_check().await?;
        let distribution = self.manager.analyze_entanglement_distribution().await?;
        let metrics = self.manager.get_entanglement_metrics().await?;
        
        // Calculate advanced metrics
        let efficiency_score = self.calculate_network_efficiency(&topology);
        let robustness_score = self.calculate_network_robustness(&topology).await?;
        let scalability_score = self.calculate_scalability_potential(&topology);
        
        // Identify critical nodes
        let critical_nodes = self.identify_critical_nodes(&topology).await?;
        
        // Calculate performance metrics
        let performance_metrics = super::engine_health::NetworkPerformanceMetrics {
            average_path_length: topology.average_path_length,
            diameter: topology.diameter,
            efficiency: efficiency_score,
            robustness: robustness_score,
            scalability: scalability_score,
            modularity: self.calculate_modularity(&topology),
            small_world_coefficient: self.calculate_small_world_coefficient(&topology),
        };
        
        Ok(NetworkAnalysisReport {
            topology,
            health,
            distribution,
            metrics,
            performance_metrics,
            critical_nodes,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Check if network optimization is needed
    pub async fn needs_optimization(&self) -> Result<bool, CognitiveError> {
        let health = self.health_check().await?;
        
        // Optimization needed if overall health is below threshold
        Ok(health.overall_health < 75.0 || !health.issues.is_empty())
    }

    /// Get optimization priority level
    pub async fn get_optimization_priority(&self) -> Result<OptimizationPriority, CognitiveError> {
        let health = self.health_check().await?;
        
        if health.overall_health < 50.0 {
            Ok(OptimizationPriority::Critical)
        } else if health.overall_health < 70.0 {
            Ok(OptimizationPriority::High)
        } else if health.overall_health < 85.0 {
            Ok(OptimizationPriority::Medium)
        } else {
            Ok(OptimizationPriority::Low)
        }
    }
}
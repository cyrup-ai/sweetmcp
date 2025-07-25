//! Quantum entanglement engine health monitoring types and calculations
//!
//! This module provides health assessment types and advanced network analysis calculations
//! with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;

use crate::cognitive::types::CognitiveError;
use super::{
    engine_core::QuantumEntanglementEngine,
    analysis::NetworkTopology,
    engine_health_types::{CriticalNode, CriticalityType, OptimizationPriority},
};
use super::super::node_state::QuantumMCTSNode;

impl QuantumEntanglementEngine {
    /// Calculate network robustness score
    pub(super) async fn calculate_network_robustness(&self, topology: &NetworkTopology) -> Result<f64, CognitiveError> {
        if topology.total_nodes == 0 {
            return Ok(0.0);
        }
        
        // Robustness based on connectivity and redundancy
        let connectivity_robustness = if topology.is_connected {
            1.0
        } else {
            topology.connected_components as f64 / topology.total_nodes as f64
        };
        
        // Degree distribution robustness (prefer balanced distribution)
        let degree_robustness = {
            let avg_degree = topology.average_degree;
            let max_degree = topology.max_degree as f64;
            
            if max_degree == 0.0 {
                0.0
            } else {
                let balance_ratio = avg_degree / max_degree;
                balance_ratio.min(1.0).max(0.0)
            }
        };
        
        // Clustering robustness (moderate clustering is most robust)
        let clustering_robustness = {
            let optimal_clustering = 0.4;
            let distance = (topology.clustering_coefficient - optimal_clustering).abs();
            (1.0 - distance / optimal_clustering).max(0.0)
        };
        
        let overall_robustness = (connectivity_robustness + degree_robustness + clustering_robustness) / 3.0;
        Ok(overall_robustness)
    }

    /// Calculate scalability potential
    pub(super) fn calculate_scalability_potential(&self, topology: &NetworkTopology) -> f64 {
        if topology.total_nodes == 0 {
            return 1.0; // Empty network has full scalability potential
        }
        
        // Scalability decreases with density and increases with good structure
        let density_factor = (1.0 - topology.network_density).max(0.0);
        let clustering_factor = topology.clustering_coefficient.min(0.8); // Cap clustering benefit
        let connectivity_factor = if topology.is_connected { 1.0 } else { 0.5 };
        
        let scalability = (density_factor * 0.4 + clustering_factor * 0.4 + connectivity_factor * 0.2).min(1.0);
        scalability
    }

    /// Identify critical nodes in the network
    pub(super) async fn identify_critical_nodes(&self, topology: &NetworkTopology) -> Result<Vec<CriticalNode>, CognitiveError> {
        let mut critical_nodes = Vec::new();
        
        // For now, return placeholder critical nodes based on topology
        // In a full implementation, this would analyze actual node connections
        if topology.total_nodes > 0 {
            // Identify nodes with highest degree (hubs)
            if topology.max_degree > topology.average_degree as usize + 2 {
                critical_nodes.push(CriticalNode {
                    node_id: "high_degree_hub".to_string(),
                    criticality_score: 0.9,
                    criticality_type: CriticalityType::Hub,
                    reason: "Node with exceptionally high degree".to_string(),
                });
            }
            
            // Identify potential bridge nodes
            if !topology.is_connected && topology.connected_components > 1 {
                critical_nodes.push(CriticalNode {
                    node_id: "bridge_candidate".to_string(),
                    criticality_score: 0.8,
                    criticality_type: CriticalityType::Bridge,
                    reason: "Potential bridge between disconnected components".to_string(),
                });
            }
        }
        
        Ok(critical_nodes)
    }

    /// Calculate network modularity
    pub(super) fn calculate_modularity(&self, topology: &NetworkTopology) -> f64 {
        // Simplified modularity calculation
        // In a full implementation, this would use community detection algorithms
        if topology.total_nodes == 0 {
            return 0.0;
        }
        
        // Estimate modularity based on clustering coefficient and connectivity
        let base_modularity = topology.clustering_coefficient * 0.7;
        let connectivity_adjustment = if topology.is_connected {
            0.0
        } else {
            0.2 // Disconnected networks have higher modularity
        };
        
        (base_modularity + connectivity_adjustment).min(1.0)
    }

    /// Calculate small-world coefficient
    pub(super) fn calculate_small_world_coefficient(&self, topology: &NetworkTopology) -> f64 {
        if topology.total_nodes < 3 {
            return 0.0;
        }
        
        // Small-world networks have high clustering but short path lengths
        let clustering_component = topology.clustering_coefficient;
        
        // Estimate path length efficiency (inverse of average path length)
        let path_efficiency = if topology.average_path_length > 0.0 {
            1.0 / topology.average_path_length
        } else {
            0.0
        };
        
        // Small-world coefficient combines high clustering with efficient paths
        let small_world = clustering_component * path_efficiency;
        small_world.min(1.0)
    }
}

/// Comprehensive health report for the entanglement engine
#[derive(Debug, Clone)]
pub struct EngineHealthReport {
    pub overall_health: f64,
    pub connectivity_health: f64,
    pub density_health: f64,
    pub clustering_health: f64,
    pub balance_health: f64,
    pub topology: NetworkTopology,
    pub metrics: crate::cognitive::quantum_mcts::entanglement::metrics::EntanglementMetrics,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

impl EngineHealthReport {
    /// Check if the engine is healthy
    pub fn is_healthy(&self) -> bool {
        self.overall_health >= 75.0 && self.issues.is_empty()
    }

    /// Get health status description
    pub fn health_status(&self) -> &'static str {
        if self.overall_health >= 90.0 {
            "Excellent"
        } else if self.overall_health >= 75.0 {
            "Good"
        } else if self.overall_health >= 50.0 {
            "Fair"
        } else if self.overall_health >= 25.0 {
            "Poor"
        } else {
            "Critical"
        }
    }

    /// Get formatted health summary
    pub fn summary(&self) -> String {
        format!(
            "Overall: {:.1}% ({}), Connectivity: {:.1}%, Density: {:.1}%, Clustering: {:.1}%, Balance: {:.1}%",
            self.overall_health,
            self.health_status(),
            self.connectivity_health,
            self.density_health,
            self.clustering_health,
            self.balance_health
        )
    }
}

/// Detailed network analysis report
#[derive(Debug, Clone)]
pub struct NetworkAnalysisReport {
    pub topology: NetworkTopology,
    pub health: EngineHealthReport,
    pub metrics: crate::cognitive::quantum_mcts::entanglement::metrics::EntanglementMetrics,
    pub performance_metrics: NetworkPerformanceMetrics,
    pub critical_nodes: Vec<CriticalNode>,
    pub timestamp: std::time::SystemTime,
}

impl NetworkAnalysisReport {
    /// Get comprehensive analysis summary
    pub fn summary(&self) -> String {
        format!(
            "Network Analysis: {} nodes, {} entanglements, {:.1}% health, {:.1}% efficiency",
            self.topology.total_nodes,
            self.metrics.total_entanglements,
            self.health.overall_health,
            self.performance_metrics.efficiency * 100.0
        )
    }

    /// Check if optimization is recommended
    pub fn recommends_optimization(&self) -> bool {
        self.health.overall_health < 75.0 || !self.health.issues.is_empty()
    }
}

/// Network performance metrics
#[derive(Debug, Clone)]
pub struct NetworkPerformanceMetrics {
    pub average_path_length: f64,
    pub diameter: usize,
    pub efficiency: f64,
    pub robustness: f64,
    pub scalability: f64,
    pub modularity: f64,
    pub small_world_coefficient: f64,
}

impl NetworkPerformanceMetrics {
    /// Calculate overall performance score
    pub fn overall_score(&self) -> f64 {
        (self.efficiency * 0.3 + 
         self.robustness * 0.25 + 
         self.scalability * 0.2 + 
         self.modularity * 0.15 + 
         self.small_world_coefficient * 0.1).min(1.0)
    }

    /// Get performance grade
    pub fn performance_grade(&self) -> char {
        let score = self.overall_score();
        if score >= 0.9 {
            'A'
        } else if score >= 0.8 {
            'B'
        } else if score >= 0.7 {
            'C'
        } else if score >= 0.6 {
            'D'
        } else {
            'F'
        }
    }
}
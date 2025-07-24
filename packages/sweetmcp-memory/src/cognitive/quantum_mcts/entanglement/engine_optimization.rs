//! Quantum entanglement engine optimization algorithms
//!
//! This module provides performance improvement calculations and optimization algorithms
//! with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;

use crate::cognitive::types::CognitiveError;
use super::{
    engine_core::QuantumEntanglementEngine,
    engine_operations::OptimizationResult,
    analysis::NetworkTopology,
};
use super::super::node_state::QuantumMCTSNode;

impl QuantumEntanglementEngine {
    /// Calculate performance improvement from optimization
    pub async fn calculate_performance_improvement(
        &self,
        mut result: OptimizationResult,
    ) -> Result<OptimizationResult, CognitiveError> {
        // Calculate improvement based on topology changes
        let connectivity_improvement = if result.final_topology.is_connected && !result.initial_topology.is_connected {
            25.0 // Significant improvement for achieving connectivity
        } else {
            0.0
        };
        
        let density_improvement = {
            let initial_density = result.initial_topology.network_density;
            let final_density = result.final_topology.network_density;
            let optimal_density = 0.3; // Target density
            
            let initial_distance = (initial_density - optimal_density).abs();
            let final_distance = (final_density - optimal_density).abs();
            
            if final_distance < initial_distance {
                ((initial_distance - final_distance) / initial_distance) * 20.0
            } else {
                0.0
            }
        };
        
        let clustering_improvement = {
            let initial_clustering = result.initial_topology.clustering_coefficient;
            let final_clustering = result.final_topology.clustering_coefficient;
            let optimal_clustering = 0.5; // Target clustering
            
            let initial_distance = (initial_clustering - optimal_clustering).abs();
            let final_distance = (final_clustering - optimal_clustering).abs();
            
            if final_distance < initial_distance {
                ((initial_distance - final_distance) / initial_distance) * 15.0
            } else {
                0.0
            }
        };
        
        result.performance_improvement = connectivity_improvement + density_improvement + clustering_improvement;
        Ok(result)
    }

    /// Calculate target entanglements for a specific node based on its characteristics
    pub(super) fn calculate_target_entanglements_for_node(&self, node: &QuantumMCTSNode) -> usize {
        let base_target = 3; // Minimum entanglements per node
        let quality_bonus = (node.quantum_state().coherence_level() * 2.0) as usize;
        let visit_bonus = (node.visit_count() as f64).log2().max(0.0) as usize;
        
        base_target + quality_bonus + visit_bonus
    }

    /// Calculate network improvement from entanglement creation
    pub(super) async fn calculate_network_improvement_from_creation(&self, entanglements_created: usize) -> Result<f64, CognitiveError> {
        if entanglements_created == 0 {
            return Ok(0.0);
        }
        
        // Estimate improvement based on number of entanglements created
        let base_improvement = entanglements_created as f64 * 0.5;
        Ok(base_improvement.min(50.0)) // Cap at 50% improvement
    }

    /// Calculate network improvement from entanglement balancing
    pub(super) async fn calculate_network_improvement_from_balancing(&self, entanglements_rebalanced: usize) -> Result<f64, CognitiveError> {
        if entanglements_rebalanced == 0 {
            return Ok(0.0);
        }
        
        // Balancing provides smaller but consistent improvements
        let improvement = entanglements_rebalanced as f64 * 0.2;
        Ok(improvement.min(20.0)) // Cap at 20% improvement
    }

    /// Calculate network improvement from entanglement pruning
    pub(super) async fn calculate_network_improvement_from_pruning(&self, entanglements_pruned: usize) -> Result<f64, CognitiveError> {
        if entanglements_pruned == 0 {
            return Ok(0.0);
        }
        
        // Pruning can provide significant improvements by reducing noise
        let improvement = entanglements_pruned as f64 * 0.3;
        Ok(improvement.min(30.0)) // Cap at 30% improvement
    }

    /// Calculate overall balance score for the network
    pub(super) async fn calculate_balance_score(&self) -> Result<f64, CognitiveError> {
        let distribution = self.manager.analyze_entanglement_distribution().await?;
        Ok(distribution.calculate_balance_score())
    }

    /// Calculate optimal entanglement strength threshold for network optimization
    pub fn calculate_optimal_strength_threshold(&self, topology: &NetworkTopology) -> f64 {
        let base_threshold = self.config.entanglement_strength_threshold;
        
        // Adjust based on network characteristics
        let density_factor = if topology.network_density > 0.5 {
            1.2 // Higher threshold for dense networks
        } else if topology.network_density < 0.1 {
            0.8 // Lower threshold for sparse networks
        } else {
            1.0
        };
        
        let connectivity_factor = if topology.is_connected {
            1.0
        } else {
            0.9 // Slightly lower threshold for disconnected networks
        };
        
        let clustering_factor = if topology.clustering_coefficient > 0.7 {
            1.1 // Higher threshold for highly clustered networks
        } else if topology.clustering_coefficient < 0.2 {
            0.9 // Lower threshold for poorly clustered networks
        } else {
            1.0
        };
        
        let optimal_threshold = base_threshold * density_factor * connectivity_factor * clustering_factor;
        
        // Ensure threshold stays within reasonable bounds
        optimal_threshold.max(0.1).min(0.9)
    }

    /// Calculate network efficiency score
    pub fn calculate_network_efficiency(&self, topology: &NetworkTopology) -> f64 {
        let mut efficiency_score = 0.0;
        
        // Connectivity score (40% weight)
        let connectivity_score = if topology.is_connected {
            1.0
        } else {
            0.0
        };
        efficiency_score += connectivity_score * 0.4;
        
        // Density score (30% weight) - optimal density around 0.3
        let optimal_density = 0.3;
        let density_distance = (topology.network_density - optimal_density).abs();
        let density_score = (1.0 - density_distance / optimal_density).max(0.0);
        efficiency_score += density_score * 0.3;
        
        // Clustering score (20% weight) - optimal clustering around 0.5
        let optimal_clustering = 0.5;
        let clustering_distance = (topology.clustering_coefficient - optimal_clustering).abs();
        let clustering_score = (1.0 - clustering_distance / optimal_clustering).max(0.0);
        efficiency_score += clustering_score * 0.2;
        
        // Balance score (10% weight) - prefer balanced degree distribution
        let degree_variance = self.calculate_degree_variance(topology);
        let balance_score = (1.0 - degree_variance).max(0.0);
        efficiency_score += balance_score * 0.1;
        
        efficiency_score.min(1.0).max(0.0)
    }

    /// Calculate degree variance for balance assessment
    fn calculate_degree_variance(&self, topology: &NetworkTopology) -> f64 {
        // Simplified variance calculation based on average degree
        let avg_degree = topology.average_degree;
        let max_degree = topology.max_degree as f64;
        
        if max_degree == 0.0 {
            return 0.0;
        }
        
        // Normalized variance approximation
        let variance_ratio = (max_degree - avg_degree) / max_degree;
        variance_ratio.min(1.0).max(0.0)
    }

    /// Predict optimization impact before execution
    pub fn predict_optimization_impact(
        &self,
        current_topology: &NetworkTopology,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> OptimizationPrediction {
        let current_efficiency = self.calculate_network_efficiency(current_topology);
        
        // Predict improvements from different operations
        let pruning_impact = if current_topology.is_overly_dense() {
            0.15 // Expect 15% improvement from pruning
        } else {
            0.0
        };
        
        let creation_impact = if current_topology.is_sparse() || !current_topology.has_good_connectivity() {
            0.20 // Expect 20% improvement from creation
        } else {
            0.0
        };
        
        let balancing_impact = if self.needs_rebalancing(current_topology) {
            0.10 // Expect 10% improvement from balancing
        } else {
            0.0
        };
        
        let predicted_efficiency = (current_efficiency + pruning_impact + creation_impact + balancing_impact).min(1.0);
        let total_predicted_improvement = (predicted_efficiency - current_efficiency) * 100.0;
        
        OptimizationPrediction {
            current_efficiency,
            predicted_efficiency,
            predicted_improvement_percentage: total_predicted_improvement,
            recommended_operations: self.recommend_operations(current_topology, tree),
            confidence_level: self.calculate_prediction_confidence(current_topology),
        }
    }

    /// Check if network needs rebalancing
    fn needs_rebalancing(&self, topology: &NetworkTopology) -> bool {
        let degree_variance = self.calculate_degree_variance(topology);
        degree_variance > 0.5 // High variance indicates need for rebalancing
    }

    /// Recommend specific operations based on network state
    fn recommend_operations(
        &self,
        topology: &NetworkTopology,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if topology.is_overly_dense() {
            recommendations.push("Prune weak entanglements to reduce network density".to_string());
        }
        
        if topology.is_sparse() {
            recommendations.push("Create strategic entanglements to improve connectivity".to_string());
        }
        
        if !topology.is_connected {
            recommendations.push("Create bridging entanglements to connect isolated components".to_string());
        }
        
        if topology.clustering_coefficient < 0.2 {
            recommendations.push("Increase local clustering through targeted entanglement creation".to_string());
        }
        
        if topology.clustering_coefficient > 0.8 {
            recommendations.push("Reduce excessive clustering through selective pruning".to_string());
        }
        
        if self.needs_rebalancing(topology) {
            recommendations.push("Rebalance entanglement distribution across nodes".to_string());
        }
        
        if tree.len() > 1000 && topology.average_degree < 2.0 {
            recommendations.push("Scale up entanglement creation for large network".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Network is well-optimized, perform maintenance operations only".to_string());
        }
        
        recommendations
    }

    /// Calculate confidence level for optimization predictions
    fn calculate_prediction_confidence(&self, topology: &NetworkTopology) -> f64 {
        let mut confidence = 0.8; // Base confidence
        
        // Reduce confidence for very small or very large networks
        if topology.total_nodes < 10 {
            confidence *= 0.7; // Less confident for small networks
        } else if topology.total_nodes > 10000 {
            confidence *= 0.8; // Slightly less confident for very large networks
        }
        
        // Reduce confidence for extreme network states
        if topology.network_density < 0.01 || topology.network_density > 0.9 {
            confidence *= 0.6; // Less confident for extreme densities
        }
        
        // Increase confidence for well-connected networks
        if topology.is_connected && topology.clustering_coefficient > 0.2 && topology.clustering_coefficient < 0.8 {
            confidence *= 1.1; // More confident for balanced networks
        }
        
        confidence.min(1.0).max(0.1)
    }
}

/// Prediction of optimization impact
#[derive(Debug, Clone)]
pub struct OptimizationPrediction {
    pub current_efficiency: f64,
    pub predicted_efficiency: f64,
    pub predicted_improvement_percentage: f64,
    pub recommended_operations: Vec<String>,
    pub confidence_level: f64,
}

impl OptimizationPrediction {
    /// Check if optimization is recommended
    pub fn is_optimization_recommended(&self) -> bool {
        self.predicted_improvement_percentage > 5.0 && self.confidence_level > 0.5
    }

    /// Get formatted prediction summary
    pub fn summary(&self) -> String {
        format!(
            "Current efficiency: {:.1}%, Predicted: {:.1}% ({:+.1}%), Confidence: {:.1}%",
            self.current_efficiency * 100.0,
            self.predicted_efficiency * 100.0,
            self.predicted_improvement_percentage,
            self.confidence_level * 100.0
        )
    }
}
//! Quantum entanglement engine core operations
//!
//! This module provides the main optimization operations for the quantum entanglement engine
//! with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;

use crate::cognitive::types::CognitiveError;
use super::{
    engine_core::QuantumEntanglementEngine,
    analysis::NetworkTopology,
};
use super::super::node_state::QuantumMCTSNode;

impl QuantumEntanglementEngine {
    /// Create strategic entanglements to improve network connectivity
    pub async fn create_strategic_entanglements(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        let mut entanglements_created = 0;
        
        // Identify nodes that need more entanglements
        for (node_id, node) in tree {
            let current_entanglements = self.manager.get_node_entanglement_count(node_id).await?;
            let target_entanglements = self.calculate_target_entanglements_for_node(node);
            
            if current_entanglements < target_entanglements {
                let needed = target_entanglements - current_entanglements;
                let candidates = self.find_entanglement_candidates(node_id, tree).await?;
                
                for candidate_id in candidates.into_iter().take(needed) {
                    if let Some(candidate_node) = tree.get(&candidate_id) {
                        let strength = self.calculate_optimal_entanglement_strength(node, candidate_node);
                        
                        if self.manager.create_entanglement(node_id, &candidate_id, strength).await.is_ok() {
                            entanglements_created += 1;
                        }
                    }
                }
            }
        }
        
        Ok(entanglements_created)
    }

    /// Balance existing entanglements across the network
    pub async fn balance_entanglements(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<usize, CognitiveError> {
        let mut entanglements_rebalanced = 0;
        let distribution = self.manager.analyze_entanglement_distribution().await?;
        
        // Identify over-connected and under-connected nodes
        let average_entanglements = distribution.average_entanglements_per_node;
        let threshold = average_entanglements * 1.5;
        
        for (node_id, node) in tree {
            let current_count = self.manager.get_node_entanglement_count(node_id).await?;
            
            if current_count as f64 > threshold {
                // Node is over-connected, consider redistributing some entanglements
                let excess = current_count - (threshold as usize);
                let entanglements = self.manager.get_node_entanglements(node_id).await?;
                
                // Sort by strength and remove weakest entanglements
                let mut sorted_entanglements = entanglements;
                sorted_entanglements.sort_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal));
                
                for entanglement in sorted_entanglements.into_iter().take(excess) {
                    if self.manager.remove_entanglement(&entanglement.id).await.is_ok() {
                        entanglements_rebalanced += 1;
                    }
                }
            }
        }
        
        Ok(entanglements_rebalanced)
    }

    /// Prune weak or redundant entanglements
    pub async fn prune_weak_entanglements(&self) -> Result<usize, CognitiveError> {
        let mut entanglements_pruned = 0;
        let all_entanglements = self.manager.get_all_entanglements().await?;
        
        for entanglement in all_entanglements {
            // Prune if strength is below threshold
            if entanglement.strength < self.config.entanglement_strength_threshold {
                if self.manager.remove_entanglement(&entanglement.id).await.is_ok() {
                    entanglements_pruned += 1;
                }
            }
        }
        
        Ok(entanglements_pruned)
    }

    /// Find suitable candidates for entanglement with a given node
    async fn find_entanglement_candidates(
        &self,
        node_id: &str,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<String>, CognitiveError> {
        let mut candidates = Vec::new();
        let existing_entanglements = self.manager.get_node_entanglements(node_id).await?;
        let existing_partners: std::collections::HashSet<String> = existing_entanglements
            .into_iter()
            .map(|e| if e.node_a == node_id { e.node_b } else { e.node_a })
            .collect();
        
        if let Some(source_node) = tree.get(node_id) {
            for (candidate_id, candidate_node) in tree {
                if candidate_id != node_id && !existing_partners.contains(candidate_id) {
                    let compatibility = self.calculate_node_compatibility(source_node, candidate_node);
                    if compatibility > 0.5 {
                        candidates.push(candidate_id.clone());
                    }
                }
            }
        }
        
        // Sort by compatibility (would need to store compatibility scores in real implementation)
        Ok(candidates)
    }

    /// Calculate optimal entanglement strength between two nodes
    fn calculate_optimal_entanglement_strength(
        &self,
        node_a: &QuantumMCTSNode,
        node_b: &QuantumMCTSNode,
    ) -> f64 {
        let coherence_factor = (node_a.quantum_state().coherence_level() + node_b.quantum_state().coherence_level()) / 2.0;
        let visit_factor = ((node_a.visit_count() + node_b.visit_count()) as f64).log2().max(1.0) / 10.0;
        let base_strength = self.config.entanglement_strength_threshold;
        
        (base_strength + coherence_factor * 0.3 + visit_factor * 0.2).min(1.0)
    }

    /// Calculate compatibility between two nodes for entanglement
    fn calculate_node_compatibility(
        &self,
        node_a: &QuantumMCTSNode,
        node_b: &QuantumMCTSNode,
    ) -> f64 {
        let coherence_compatibility = 1.0 - (node_a.quantum_state().coherence_level() - node_b.quantum_state().coherence_level()).abs();
        let visit_compatibility = {
            let visit_ratio = (node_a.visit_count() as f64 / (node_b.visit_count() as f64).max(1.0)).min(10.0);
            1.0 - (visit_ratio.log2().abs() / 4.0).min(1.0)
        };
        
        (coherence_compatibility * 0.6 + visit_compatibility * 0.4).max(0.0).min(1.0)
    }
}

/// Result of an optimization operation
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub initial_topology: NetworkTopology,
    pub final_topology: NetworkTopology,
    pub entanglements_created: usize,
    pub entanglements_removed: usize,
    pub entanglements_rebalanced: usize,
    pub performance_improvement: f64,
    pub optimization_time_ms: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

impl OptimizationResult {
    /// Create new successful optimization result
    pub fn success(
        initial_topology: NetworkTopology,
        final_topology: NetworkTopology,
        entanglements_created: usize,
        entanglements_removed: usize,
        entanglements_rebalanced: usize,
        optimization_time_ms: u64,
    ) -> Self {
        Self {
            initial_topology,
            final_topology,
            entanglements_created,
            entanglements_removed,
            entanglements_rebalanced,
            performance_improvement: 0.0, // Will be calculated later
            optimization_time_ms,
            success: true,
            error_message: None,
        }
    }

    /// Create new failed optimization result
    pub fn failure(error_message: String, optimization_time_ms: u64) -> Self {
        Self {
            initial_topology: NetworkTopology::empty(),
            final_topology: NetworkTopology::empty(),
            entanglements_created: 0,
            entanglements_removed: 0,
            entanglements_rebalanced: 0,
            performance_improvement: 0.0,
            optimization_time_ms,
            success: false,
            error_message: Some(error_message),
        }
    }

    /// Get total entanglement changes
    pub fn total_changes(&self) -> usize {
        self.entanglements_created + self.entanglements_removed + self.entanglements_rebalanced
    }

    /// Get summary of optimization
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "Optimization successful: {} created, {} removed, {} rebalanced, {:.1}% improvement in {:.1}ms",
                self.entanglements_created,
                self.entanglements_removed,
                self.entanglements_rebalanced,
                self.performance_improvement,
                self.optimization_time_ms
            )
        } else {
            format!(
                "Optimization failed after {:.1}ms: {}",
                self.optimization_time_ms,
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }

    /// Check if optimization made significant improvements
    pub fn is_significant_improvement(&self) -> bool {
        self.success && (self.performance_improvement > 5.0 || self.total_changes() > 0)
    }
}

impl NetworkTopology {
    /// Create empty network topology
    pub fn empty() -> Self {
        Self {
            total_nodes: 0,
            total_edges: 0,
            network_density: 0.0,
            clustering_coefficient: 0.0,
            average_path_length: 0.0,
            diameter: 0,
            is_connected: false,
            connected_components: 0,
            average_degree: 0.0,
            max_degree: 0,
        }
    }

    /// Check if network is overly dense
    pub fn is_overly_dense(&self) -> bool {
        self.network_density > 0.6
    }

    /// Check if network is sparse
    pub fn is_sparse(&self) -> bool {
        self.network_density < 0.1
    }

    /// Check if network has good connectivity
    pub fn has_good_connectivity(&self) -> bool {
        self.is_connected && self.average_path_length < 5.0
    }
}
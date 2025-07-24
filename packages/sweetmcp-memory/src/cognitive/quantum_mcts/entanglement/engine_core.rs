//! Core quantum entanglement engine implementation
//!
//! This module provides the core QuantumEntanglementEngine struct and basic operations
//! with zero-allocation patterns and blazing-fast performance.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cognitive::{
    quantum::EntanglementGraph,
    types::CognitiveError,
};
use super::{
    core::QuantumEntanglementManager,
    analysis::NetworkTopology,
    metrics::EntanglementMetrics,
};
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// High-level entanglement management interface with optimization capabilities
pub struct QuantumEntanglementEngine {
    /// Core entanglement manager
    pub(super) manager: QuantumEntanglementManager,
    /// Performance metrics tracker
    pub(super) metrics: Arc<EntanglementMetrics>,
    /// Configuration parameters
    pub(super) config: QuantumMCTSConfig,
    /// Reference to the entanglement graph
    pub(super) entanglement_graph: Arc<RwLock<EntanglementGraph>>,
}

impl QuantumEntanglementEngine {
    /// Create new entanglement engine with comprehensive initialization
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        let metrics = Arc::new(EntanglementMetrics::new());
        let manager = QuantumEntanglementManager::new(
            config.clone(),
            entanglement_graph.clone(),
            metrics.clone(),
        );
        
        Self {
            manager,
            metrics,
            config,
            entanglement_graph,
        }
    }
    
    /// Create entanglement engine with custom metrics
    pub fn with_metrics(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
        metrics: Arc<EntanglementMetrics>,
    ) -> Self {
        let manager = QuantumEntanglementManager::new(
            config.clone(),
            entanglement_graph.clone(),
            metrics.clone(),
        );
        
        Self {
            manager,
            metrics,
            config,
            entanglement_graph,
        }
    }

    /// Get manager reference for direct operations
    pub fn manager(&mut self) -> &mut QuantumEntanglementManager {
        &mut self.manager
    }

    /// Get metrics reference
    pub fn metrics(&self) -> &EntanglementMetrics {
        &self.metrics
    }

    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        &self.config
    }

    /// Update configuration and clear caches
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config.clone();
        self.manager.update_config(new_config);
    }

    /// Calculate dynamic pruning threshold based on network state
    pub(super) fn calculate_dynamic_pruning_threshold(&self, topology: &NetworkTopology) -> f64 {
        let base_threshold = self.config.entanglement_strength_threshold;
        let density_factor = topology.network_density.min(1.0);
        let connectivity_factor = if topology.is_connected { 0.9 } else { 1.1 };
        
        // Adjust threshold based on network characteristics
        let adjusted_threshold = base_threshold * density_factor * connectivity_factor;
        
        // Ensure threshold stays within reasonable bounds
        adjusted_threshold.max(0.1).min(0.9)
    }

    /// Calculate optimal batch size for entanglement creation
    pub(super) fn calculate_optimal_batch_size(&self, total_nodes: usize) -> usize {
        // Use square root scaling for optimal batch size
        let base_size = (total_nodes as f64).sqrt() as usize;
        base_size.max(10).min(100) // Clamp between 10 and 100
    }

    /// Calculate adaptive pruning threshold based on network analysis
    pub(super) async fn calculate_adaptive_pruning_threshold(
        &self,
        topology: &NetworkTopology,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<f64, CognitiveError> {
        let base_threshold = self.config.entanglement_strength_threshold;
        
        // Factor in network density
        let density_adjustment = if topology.network_density > 0.5 {
            1.2 // Higher threshold for dense networks
        } else if topology.network_density < 0.1 {
            0.8 // Lower threshold for sparse networks
        } else {
            1.0
        };
        
        // Factor in connectivity
        let connectivity_adjustment = if topology.is_connected {
            1.0
        } else {
            0.9 // Slightly lower threshold for disconnected networks
        };
        
        // Factor in average node quality
        let total_quality: f64 = tree.values()
            .map(|node| node.quantum_state().coherence_level())
            .sum();
        let avg_quality = if tree.is_empty() {
            0.5
        } else {
            total_quality / tree.len() as f64
        };
        
        let quality_adjustment = if avg_quality > 0.7 {
            1.1 // Higher threshold for high-quality nodes
        } else if avg_quality < 0.3 {
            0.9 // Lower threshold for low-quality nodes
        } else {
            1.0
        };
        
        let adaptive_threshold = base_threshold * density_adjustment * connectivity_adjustment * quality_adjustment;
        
        // Ensure threshold is within reasonable bounds
        Ok(adaptive_threshold.max(0.05).min(0.95))
    }

    /// Prune weak entanglements for a specific node
    pub async fn prune_node_weak_entanglements(
        &mut self,
        node_id: &str,
        strength_threshold: f64,
    ) -> Result<usize, CognitiveError> {
        let pruned_count = self.manager.prune_node_entanglements(node_id, strength_threshold).await?;
        
        if pruned_count > 0 {
            self.metrics.record_entanglements_pruned(pruned_count);
        }
        
        Ok(pruned_count)
    }
}
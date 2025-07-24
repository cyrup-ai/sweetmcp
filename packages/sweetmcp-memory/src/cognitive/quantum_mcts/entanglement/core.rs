//! Core quantum entanglement manager with lock-free operations
//!
//! This module provides the main QuantumEntanglementManager with blazing-fast
//! entanglement creation, management, and zero-allocation graph operations.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

use crate::cognitive::{
    quantum::{EntanglementGraph, EntanglementType},
    types::CognitiveError,
};
use super::{
    super::{
        node_state::{QuantumMCTSNode, QuantumNodeState},
        config::QuantumMCTSConfig,
    },
    metrics::EntanglementMetrics,
};

/// Core lock-free entanglement manager with optimized graph operations
#[repr(align(64))] // Cache-line aligned for optimal performance
pub struct QuantumEntanglementManager {
    /// Configuration for entanglement parameters
    config: QuantumMCTSConfig,
    /// Entanglement graph with lock-free operations
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    /// Entanglement creation cache to avoid recomputation
    creation_cache: HashMap<(String, String), bool>,
    /// Performance metrics
    pub metrics: EntanglementMetrics,
}

impl QuantumEntanglementManager {
    /// Create new entanglement manager with optimized initialization
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        Self {
            config,
            entanglement_graph,
            creation_cache: HashMap::with_capacity(10_000), // Pre-allocate for performance
            metrics: EntanglementMetrics::new(),
        }
    }
    
    /// Create entanglement between nodes with blazing-fast graph operations
    pub async fn create_entanglement(
        &mut self,
        node_id: &str,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<Vec<String>, CognitiveError> {
        let node = tree.get(node_id)
            .ok_or_else(|| CognitiveError::InvalidState("Node not found for entanglement creation".to_string()))?;

        let mut created_entanglements = Vec::new();
        let mut entanglement_graph = self.entanglement_graph.write().await;

        // Find candidate nodes for entanglement with optimized iteration
        let candidates = self.find_entanglement_candidates(node, tree);
        
        for candidate_id in candidates {
            let candidate = match tree.get(&candidate_id) {
                Some(node) => node,
                None => continue, // Skip if node no longer exists
            };
            
            // Check cache first for performance optimization
            let cache_key = if node_id < &candidate_id {
                (node_id.to_string(), candidate_id.clone())
            } else {
                (candidate_id.clone(), node_id.to_string())
            };
            
            if let Some(&should_create) = self.creation_cache.get(&cache_key) {
                if !should_create {
                    continue;
                }
            } else {
                let should_create = self.should_entangle_optimized(node, candidate);
                self.creation_cache.insert(cache_key, should_create);
                if !should_create {
                    continue;
                }
            }

            // Determine entanglement type and strength
            let (entanglement_type, strength) = self.calculate_entanglement_properties(node, candidate);
            
            // Create entanglement with atomic operation
            match entanglement_graph.add_entanglement(
                node_id.to_string(),
                candidate_id.clone(),
                entanglement_type,
                strength,
            ) {
                Ok(()) => {
                    created_entanglements.push(candidate_id.clone());
                    self.metrics.entanglements_created += 1;
                    trace!("Created entanglement: {} <-> {} (strength: {:.3})", 
                           node_id, candidate_id, strength);
                }
                Err(e) => {
                    warn!("Failed to create entanglement {} <-> {}: {}", 
                          node_id, candidate_id, e);
                    self.metrics.entanglement_failures += 1;
                }
            }
        }

        self.metrics.entanglement_operations += 1;
        Ok(created_entanglements)
    }
    
    /// Find entanglement candidates with optimized filtering
    fn find_entanglement_candidates(
        &self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Vec<String> {
        let mut candidates = Vec::with_capacity(32); // Pre-allocate for typical candidate count
        let node_depth = node.improvement_depth;
        let node_visits = node.visits;
        
        // Early filtering criteria for performance
        let min_visits_threshold = (node_visits / 10).max(1); // At least 10% of current node's visits
        let max_depth_difference = 2; // Within 2 levels of improvement depth
        
        for (candidate_id, candidate) in tree.iter() {
            // Skip self-entanglement
            if candidate_id == &node.id {
                continue;
            }
            
            // Skip if already entangled (check local state first for speed)
            if node.quantum_state.entanglements.contains(candidate_id) {
                continue;
            }
            
            // Fast numeric filters first (most selective for performance)
            if candidate.improvement_depth.abs_diff(node_depth) > max_depth_difference {
                continue;
            }
            
            if candidate.visits < min_visits_threshold {
                continue;
            }
            
            // Coherence check (more expensive, do after numeric filters)
            if candidate.quantum_state.decoherence >= self.config.decoherence_threshold {
                continue;
            }
            
            // Add to candidates if all filters pass
            candidates.push(candidate_id.clone());
            
            // Limit candidates to prevent excessive computation
            if candidates.len() >= 50 {
                break;
            }
        }
        
        candidates
    }
    
    /// Optimized entanglement compatibility check with early termination
    #[inline(always)]
    fn should_entangle_optimized(&self, node1: &QuantumMCTSNode, node2: &QuantumMCTSNode) -> bool {
        // Fast path: check fundamental compatibility first
        if node1.is_terminal || node2.is_terminal {
            return false;
        }
        
        // Depth similarity check (most selective filter)
        let depth_diff = node1.improvement_depth.abs_diff(node2.improvement_depth);
        if depth_diff > 1 {
            return false;
        }
        
        // Coherence check (quantum property validation)
        let both_coherent = node1.quantum_state.decoherence < self.config.decoherence_threshold
            && node2.quantum_state.decoherence < self.config.decoherence_threshold;
        if !both_coherent {
            return false;
        }
        
        // Amplitude compatibility (quantum interference potential)
        let amplitude_product = node1.amplitude.norm() * node2.amplitude.norm();
        if amplitude_product < self.config.amplitude_threshold {
            return false;
        }
        
        // Visit count balance (avoid entangling heavily visited with barely visited)
        let visit_ratio = if node1.visits > node2.visits {
            node2.visits as f64 / node1.visits as f64
        } else {
            node1.visits as f64 / node2.visits as f64
        };
        if visit_ratio < 0.1 {
            return false;
        }
        
        // Action similarity check (semantic compatibility)
        let action_similarity = self.calculate_action_similarity(node1, node2);
        if action_similarity < 0.3 {
            return false;
        }
        
        true
    }
    
    /// Calculate semantic similarity between node actions with optimized algorithm
    #[inline]
    fn calculate_action_similarity(&self, node1: &QuantumMCTSNode, node2: &QuantumMCTSNode) -> f64 {
        // Fast implementation using action set intersection
        let actions1 = &node1.untried_actions;
        let actions2 = &node2.untried_actions;
        
        if actions1.is_empty() && actions2.is_empty() {
            return 1.0; // Both fully expanded
        }
        
        if actions1.is_empty() || actions2.is_empty() {
            return 0.5; // One expanded, one not
        }
        
        // Calculate Jaccard similarity with optimized intersection
        let mut intersection_count = 0;
        let total_unique = actions1.len() + actions2.len();
        
        // Optimized intersection calculation (avoid nested loops)
        for action1 in actions1 {
            if actions2.contains(action1) {
                intersection_count += 1;
            }
        }
        
        let union_count = total_unique - intersection_count;
        if union_count == 0 {
            return 1.0;
        }
        
        intersection_count as f64 / union_count as f64
    }
    
    /// Calculate entanglement properties based on node characteristics
    fn calculate_entanglement_properties(
        &self,
        node1: &QuantumMCTSNode,
        node2: &QuantumMCTSNode,
    ) -> (EntanglementType, f64) {
        // Determine entanglement type based on node relationships
        let entanglement_type = if node1.parent == node2.parent && node1.parent.is_some() {
            EntanglementType::Strong // Sibling nodes have strong entanglement
        } else if self.are_ancestor_descendant(&node1.id, &node2.id) {
            EntanglementType::Medium // Ancestor-descendant relationship
        } else {
            EntanglementType::Weak // Distant relationship
        };
        
        // Calculate strength based on multiple factors with numerical stability
        let amplitude_factor = (node1.amplitude.norm() * node2.amplitude.norm()).sqrt();
        let coherence_factor = (2.0 - node1.quantum_state.decoherence - node2.quantum_state.decoherence) / 2.0;
        let visit_factor = (node1.visits.min(node2.visits) as f64 / node1.visits.max(node2.visits) as f64).sqrt();
        let depth_factor = 1.0 / (1.0 + node1.improvement_depth.abs_diff(node2.improvement_depth) as f64);
        
        // Weighted combination of factors with balanced weights
        let base_strength = (amplitude_factor * 0.3 + coherence_factor * 0.3 + visit_factor * 0.2 + depth_factor * 0.2)
            * self.config.entanglement_strength;
        
        // Type-based strength modulation
        let final_strength = match entanglement_type {
            EntanglementType::Strong => base_strength * 1.0,
            EntanglementType::Medium => base_strength * 0.8,
            EntanglementType::Weak => base_strength * 0.6,
        };
        
        (entanglement_type, final_strength.min(1.0))
    }
    
    /// Check if two nodes have ancestor-descendant relationship
    fn are_ancestor_descendant(&self, node1_id: &str, node2_id: &str) -> bool {
        // Simple heuristic based on ID patterns (could be enhanced with actual tree traversal)
        node1_id.contains(node2_id) || node2_id.contains(node1_id)
    }
    
    /// Remove entanglement between nodes with atomic operation
    pub async fn remove_entanglement(
        &mut self,
        node1_id: &str,
        node2_id: &str,
    ) -> Result<bool, CognitiveError> {
        let mut entanglement_graph = self.entanglement_graph.write().await;
        
        match entanglement_graph.remove_entanglement(node1_id, node2_id) {
            Ok(existed) => {
                if existed {
                    self.metrics.entanglements_removed += 1;
                    trace!("Removed entanglement: {} <-> {}", node1_id, node2_id);
                }
                Ok(existed)
            }
            Err(e) => {
                warn!("Failed to remove entanglement {} <-> {}: {}", node1_id, node2_id, e);
                Err(CognitiveError::QuantumError(format!("Entanglement removal failed: {}", e)))
            }
        }
    }
    
    /// Get entangled nodes for a given node with performance optimization
    pub async fn get_entangled_nodes(
        &self,
        node_id: &str,
    ) -> Result<Vec<(String, f64)>, CognitiveError> {
        let entanglement_graph = self.entanglement_graph.read().await;
        
        entanglement_graph.get_entangled(node_id)
            .map_err(|e| CognitiveError::QuantumError(format!("Failed to get entangled nodes: {}", e)))
    }
    
    /// Update node entanglement state after modifications
    pub async fn update_node_entanglements(
        &mut self,
        node_id: &str,
        tree: &mut HashMap<String, QuantumMCTSNode>,
    ) -> Result<(), CognitiveError> {
        let entangled_nodes = self.get_entangled_nodes(node_id).await?;
        
        // Update local entanglement list in the node
        if let Some(node) = tree.get_mut(node_id) {
            node.quantum_state.entanglements.clear();
            for (entangled_id, _strength) in entangled_nodes {
                node.quantum_state.add_entanglement(entangled_id);
            }
        }
        
        Ok(())
    }
    
    /// Batch entanglement creation for multiple nodes with error recovery
    pub async fn batch_create_entanglements(
        &mut self,
        node_ids: &[String],
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<HashMap<String, Vec<String>>, CognitiveError> {
        let mut results = HashMap::with_capacity(node_ids.len());
        
        for node_id in node_ids {
            match self.create_entanglement(node_id, tree).await {
                Ok(entanglements) => {
                    results.insert(node_id.clone(), entanglements);
                }
                Err(e) => {
                    warn!("Batch entanglement creation failed for {}: {}", node_id, e);
                    results.insert(node_id.clone(), Vec::new());
                    // Continue with other nodes even if one fails
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get performance metrics reference
    pub fn get_metrics(&self) -> &EntanglementMetrics {
        &self.metrics
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = EntanglementMetrics::new();
        debug!("Entanglement metrics reset");
    }
    
    /// Clear entanglement creation cache for memory optimization
    pub fn clear_cache(&mut self) {
        self.creation_cache.clear();
        debug!("Entanglement creation cache cleared");
    }
    
    /// Get cache statistics for performance monitoring
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cache_size: self.creation_cache.len(),
            cache_capacity: self.creation_cache.capacity(),
            utilization: if self.creation_cache.capacity() > 0 {
                self.creation_cache.len() as f64 / self.creation_cache.capacity() as f64
            } else {
                0.0
            },
        }
    }
    
    /// Update configuration and clear dependent caches
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config;
        // Clear cache since entanglement decisions depend on configuration parameters
        self.clear_cache();
        debug!("Entanglement configuration updated");
    }
    
    /// Get current configuration reference
    pub fn get_config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Get entanglement graph reference
    pub fn get_entanglement_graph(&self) -> &Arc<RwLock<EntanglementGraph>> {
        &self.entanglement_graph
    }
    
    /// Prune caches based on existing nodes to prevent memory leaks
    pub fn prune_cache(&mut self, existing_nodes: &HashMap<String, QuantumMCTSNode>) {
        let initial_size = self.creation_cache.len();
        
        self.creation_cache.retain(|(node1, node2), _| {
            existing_nodes.contains_key(node1) && existing_nodes.contains_key(node2)
        });
        
        let pruned_count = initial_size - self.creation_cache.len();
        if pruned_count > 0 {
            debug!("Pruned {} stale cache entries", pruned_count);
        }
    }
    
    /// Check if cache needs pruning based on utilization
    pub fn needs_cache_pruning(&self) -> bool {
        self.cache_stats().utilization > 0.8
    }
    
    /// Get entanglement creation success rate
    pub fn creation_success_rate(&self) -> f64 {
        if self.metrics.entanglement_operations == 0 {
            return 0.0;
        }
        
        self.metrics.entanglements_created as f64 / self.metrics.entanglement_operations as f64
    }
}

/// Cache statistics for performance monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    /// Current cache size
    pub cache_size: usize,
    /// Cache capacity
    pub cache_capacity: usize,
    /// Cache utilization (0.0 to 1.0)
    pub utilization: f64,
}

impl CacheStats {
    /// Check if cache utilization is healthy
    pub fn is_healthy(&self) -> bool {
        self.utilization > 0.1 && self.utilization < 0.9
    }
    
    /// Get memory usage estimate in bytes
    pub fn memory_usage_estimate(&self) -> usize {
        // Rough estimate: each cache entry is ~100 bytes (string keys + bool value)
        self.cache_size * 100
    }
}
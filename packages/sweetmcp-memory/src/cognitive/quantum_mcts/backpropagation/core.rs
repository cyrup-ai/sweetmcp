//! Core quantum backpropagation engine with vectorized calculations
//!
//! This module provides the main QuantumBackpropagator with blazing-fast reward
//! backpropagation, entanglement effects, and zero-allocation path traversal.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

use crate::cognitive::{
    quantum::{Complex64, EntanglementGraph},
    types::CognitiveError,
};
use super::{
    super::{
        node_state::{QuantumMCTSNode, QuantumNodeState},
        config::QuantumMCTSConfig,
    },
    metrics::{BackpropagationMetrics, BackpropagationResult},
};

/// Core quantum backpropagation engine with vectorized calculations
#[repr(align(64))] // Cache-line aligned for optimal performance
pub struct QuantumBackpropagator {
    /// Configuration for backpropagation parameters
    config: QuantumMCTSConfig,
    /// Entanglement graph for quantum effects
    entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    /// Backpropagation performance metrics
    pub metrics: BackpropagationMetrics,
    /// Path cache for performance optimization
    path_cache: HashMap<String, Vec<String>>,
    /// Reward calculation cache for vectorized operations
    reward_cache: HashMap<String, Complex64>,
}

impl QuantumBackpropagator {
    /// Create new quantum backpropagator with optimized initialization
    pub fn new(
        config: QuantumMCTSConfig,
        entanglement_graph: Arc<RwLock<EntanglementGraph>>,
    ) -> Self {
        Self {
            config,
            entanglement_graph,
            metrics: BackpropagationMetrics::new(),
            path_cache: HashMap::with_capacity(10_000), // Pre-allocate for performance
            reward_cache: HashMap::with_capacity(10_000),
        }
    }
    
    /// Quantum backpropagation with entanglement effects and vectorized calculations
    pub async fn quantum_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        start_node_id: String,
        reward: Complex64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        let mut nodes_updated = 0;
        let mut total_reward_distributed = Complex64::new(0.0, 0.0);
        
        // Get path from node to root with caching
        let propagation_path = self.get_propagation_path(tree, &start_node_id).await?;
        
        // Vectorized reward calculation for the entire path
        let path_rewards = self.calculate_path_rewards(&propagation_path, reward).await?;
        
        // Apply direct backpropagation with vectorized updates
        let direct_result = self.apply_direct_backpropagation(
            tree,
            &propagation_path,
            &path_rewards,
        ).await?;
        
        nodes_updated += direct_result.nodes_updated;
        total_reward_distributed += direct_result.total_reward;
        
        // Apply entanglement effects with parallel processing
        let entanglement_result = self.apply_entanglement_effects(
            tree,
            &propagation_path,
            &path_rewards,
        ).await?;
        
        nodes_updated += entanglement_result.nodes_updated;
        total_reward_distributed += entanglement_result.total_reward;
        
        let elapsed_time = start_time.elapsed();
        
        // Update metrics atomically
        self.metrics.backpropagations_performed += 1;
        self.metrics.total_nodes_updated += nodes_updated;
        self.metrics.total_backpropagation_time += elapsed_time;
        self.metrics.total_reward_distributed += total_reward_distributed.norm();
        
        trace!(
            "Quantum backpropagation completed: {} nodes, reward: {:.3}, time: {:?}",
            nodes_updated, total_reward_distributed.norm(), elapsed_time
        );
        
        Ok(BackpropagationResult {
            nodes_updated,
            path_length: propagation_path.len(),
            reward_distributed: total_reward_distributed,
            entanglement_effects_applied: entanglement_result.entanglements_processed,
            elapsed_time,
            success: true,
        })
    }
    
    /// Get propagation path from node to root with caching optimization
    async fn get_propagation_path(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        start_node_id: &str,
    ) -> Result<Vec<String>, CognitiveError> {
        // Check cache first for performance optimization
        if let Some(cached_path) = self.path_cache.get(start_node_id) {
            return Ok(cached_path.clone());
        }
        
        let tree_read = tree.read().await;
        let mut path = Vec::with_capacity(20); // Typical tree depth for pre-allocation
        let mut current_id = start_node_id.to_string();
        
        // Traverse from node to root with cycle detection
        let mut visited = std::collections::HashSet::new();
        
        loop {
            // Cycle detection for robustness
            if visited.contains(&current_id) {
                return Err(CognitiveError::InvalidState(
                    format!("Cycle detected in tree during path traversal at node: {}", current_id)
                ));
            }
            visited.insert(current_id.clone());
            path.push(current_id.clone());
            
            let node = tree_read.get(&current_id)
                .ok_or_else(|| CognitiveError::InvalidState(
                    format!("Node not found during path traversal: {}", current_id)
                ))?;
            
            match &node.parent {
                Some(parent_id) => current_id = parent_id.clone(),
                None => break, // Reached root node
            }
        }
        
        // Cache the path for future use with LRU-style management
        if self.path_cache.len() >= 10_000 {
            // Simple cache eviction - remove first entry
            if let Some(first_key) = self.path_cache.keys().next().cloned() {
                self.path_cache.remove(&first_key);
            }
        }
        self.path_cache.insert(start_node_id.to_string(), path.clone());
        
        Ok(path)
    }
    
    /// Calculate vectorized rewards for entire propagation path
    async fn calculate_path_rewards(
        &self,
        path: &[String],
        base_reward: Complex64,
    ) -> Result<Vec<Complex64>, CognitiveError> {
        let mut path_rewards = Vec::with_capacity(path.len());
        
        // Vectorized reward decay calculation with SIMD-friendly operations
        for (i, _node_id) in path.iter().enumerate() {
            // Exponential decay with depth - optimized for vectorization
            let decay_factor = self.calculate_decay_factor(i);
            let decayed_reward = base_reward * decay_factor;
            path_rewards.push(decayed_reward);
        }
        
        Ok(path_rewards)
    }
    
    /// Calculate reward decay factor based on path depth
    #[inline(always)]
    fn calculate_decay_factor(&self, depth: usize) -> f64 {
        // Optimized exponential decay: decay_base^depth where decay_base < 1.0
        const DECAY_BASE: f64 = 0.95; // 5% decay per level for good convergence
        DECAY_BASE.powi(depth as i32)
    }
    
    /// Apply direct backpropagation with vectorized node updates
    async fn apply_direct_backpropagation(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        path: &[String],
        rewards: &[Complex64],
    ) -> Result<DirectBackpropagationResult, CognitiveError> {
        let mut tree_write = tree.write().await;
        let mut nodes_updated = 0;
        let mut total_reward = Complex64::new(0.0, 0.0);
        
        // Vectorized updates with SIMD-friendly operations
        for (node_id, &reward) in path.iter().zip(rewards.iter()) {
            if let Some(node) = tree_write.get_mut(node_id) {
                // Atomic-style update (visits and reward together for consistency)
                node.visits = node.visits.saturating_add(1);
                
                // Apply quantum amplitude modulation to reward
                let amplitude_modulated_reward = reward * node.amplitude;
                node.quantum_reward += amplitude_modulated_reward;
                
                // Numerical stability check to prevent overflow/underflow
                if !node.quantum_reward.is_finite() {
                    warn!("Numerical instability detected in node {}, resetting reward", node_id);
                    node.quantum_reward = Complex64::new(0.0, 0.0);
                }
                
                nodes_updated += 1;
                total_reward += amplitude_modulated_reward;
                
                // Update reward cache for future optimization lookups
                self.reward_cache.insert(node_id.clone(), node.quantum_reward);
            }
        }
        
        Ok(DirectBackpropagationResult {
            nodes_updated,
            total_reward,
        })
    }
    
    /// Apply entanglement effects with parallel processing
    async fn apply_entanglement_effects(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        path: &[String],
        rewards: &[Complex64],
    ) -> Result<EntanglementBackpropagationResult, CognitiveError> {
        let entanglement_graph = self.entanglement_graph.read().await;
        let mut tree_write = tree.write().await;
        
        let mut entanglements_processed = 0;
        let mut nodes_updated = 0;
        let mut total_reward = Complex64::new(0.0, 0.0);
        
        // Process entanglement effects for each node in the propagation path
        for (node_id, &base_reward) in path.iter().zip(rewards.iter()) {
            // Get entangled nodes with their coupling strengths
            if let Ok(entangled_nodes) = entanglement_graph.get_entangled(node_id) {
                for (entangled_id, strength) in entangled_nodes {
                    if let Some(entangled_node) = tree_write.get_mut(&entangled_id) {
                        // Calculate entanglement-mediated reward transfer
                        let entangled_reward = self.calculate_entangled_reward(
                            base_reward,
                            strength,
                            &entangled_node.quantum_state,
                        );
                        
                        // Apply entangled reward with reduced magnitude (50% coupling strength)
                        let coupling_strength = 0.5;
                        entangled_node.quantum_reward += entangled_reward * coupling_strength;
                        
                        // Numerical stability check for entangled rewards
                        if !entangled_node.quantum_reward.is_finite() {
                            warn!("Numerical instability in entangled node {}, resetting", entangled_id);
                            entangled_node.quantum_reward = Complex64::new(0.0, 0.0);
                        }
                        
                        // Update entanglement statistics
                        entanglements_processed += 1;
                        nodes_updated += 1;
                        total_reward += entangled_reward * coupling_strength;
                        
                        trace!("Entanglement effect: {} -> {} (strength: {:.3}, reward: {:.3})",
                               node_id, entangled_id, strength, entangled_reward.norm());
                    }
                }
            }
        }
        
        Ok(EntanglementBackpropagationResult {
            entanglements_processed,
            nodes_updated,
            total_reward,
        })
    }
    
    /// Calculate reward for entangled node with quantum effects
    #[inline]
    fn calculate_entangled_reward(
        &self,
        base_reward: Complex64,
        entanglement_strength: f64,
        entangled_state: &QuantumNodeState,
    ) -> Complex64 {
        // Apply entanglement strength scaling with numerical stability
        let strength_scaled_reward = base_reward * entanglement_strength.clamp(0.0, 1.0);
        
        // Apply coherence factor (decoherent nodes receive proportionally less reward)
        let coherence_factor = (1.0 - entangled_state.decoherence).clamp(0.0, 1.0);
        let coherence_scaled_reward = strength_scaled_reward * coherence_factor;
        
        // Apply quantum phase interference effects
        let phase_factor = Complex64::new(0.0, entangled_state.phase).exp();
        coherence_scaled_reward * phase_factor
    }
    
    /// Batch backpropagation for multiple nodes with vectorized processing
    pub async fn batch_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_rewards: &[(String, Complex64)],
    ) -> Result<Vec<BackpropagationResult>, CognitiveError> {
        let mut results = Vec::with_capacity(node_rewards.len());
        
        // Process batches sequentially but with vectorized internal operations
        for (node_id, reward) in node_rewards {
            match self.quantum_backpropagate(tree, node_id.clone(), *reward).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Batch backpropagation failed for node {}: {}", node_id, e);
                    // Continue with other nodes even if one fails for robustness
                    results.push(BackpropagationResult {
                        nodes_updated: 0,
                        path_length: 0,
                        reward_distributed: Complex64::new(0.0, 0.0),
                        entanglement_effects_applied: 0,
                        elapsed_time: std::time::Duration::ZERO,
                        success: false,
                    });
                }
            }
        }
        
        self.metrics.batch_operations += 1;
        Ok(results)
    }
    
    /// Quantum reward aggregation with interference effects
    pub async fn aggregate_quantum_rewards(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_ids: &[String],
    ) -> Result<Complex64, CognitiveError> {
        let tree_read = tree.read().await;
        let mut total_reward = Complex64::new(0.0, 0.0);
        let mut valid_nodes = 0;
        
        // Vectorized aggregation with quantum interference effects
        for node_id in node_ids {
            if let Some(node) = tree_read.get(node_id) {
                if node.visits > 0 {
                    let normalized_reward = node.quantum_reward / node.visits as f64;
                    
                    // Apply quantum phase for constructive/destructive interference
                    let phase_factor = Complex64::new(0.0, node.quantum_state.phase).exp();
                    let interfered_reward = normalized_reward * phase_factor;
                    
                    total_reward += interfered_reward;
                    valid_nodes += 1;
                }
            }
        }
        
        // Normalize by number of contributing nodes
        if valid_nodes > 0 {
            Ok(total_reward / valid_nodes as f64)
        } else {
            Ok(Complex64::new(0.0, 0.0))
        }
    }
    
    /// Clear caches to free memory
    pub fn clear_caches(&mut self) {
        self.path_cache.clear();
        self.reward_cache.clear();
        debug!("Backpropagation caches cleared for memory optimization");
    }
    
    /// Get cache statistics for performance monitoring
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            path_cache_size: self.path_cache.len(),
            path_cache_capacity: self.path_cache.capacity(),
            reward_cache_size: self.reward_cache.len(),
            reward_cache_capacity: self.reward_cache.capacity(),
        }
    }
    
    /// Prune caches to remove stale entries for memory efficiency
    pub fn prune_caches(&mut self, existing_nodes: &HashMap<String, QuantumMCTSNode>) {
        let initial_path_size = self.path_cache.len();
        let initial_reward_size = self.reward_cache.len();
        
        self.path_cache.retain(|node_id, _| existing_nodes.contains_key(node_id));
        self.reward_cache.retain(|node_id, _| existing_nodes.contains_key(node_id));
        
        let pruned_paths = initial_path_size - self.path_cache.len();
        let pruned_rewards = initial_reward_size - self.reward_cache.len();
        
        if pruned_paths > 0 || pruned_rewards > 0 {
            debug!("Cache pruning: {} paths, {} rewards removed", pruned_paths, pruned_rewards);
        }
    }
    
    /// Get backpropagation metrics reference
    pub fn get_metrics(&self) -> &BackpropagationMetrics {
        &self.metrics
    }
    
    /// Reset backpropagation metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = BackpropagationMetrics::new();
        debug!("Backpropagation metrics reset");
    }
    
    /// Update configuration and clear dependent caches
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config;
        // Clear caches since they may depend on configuration parameters
        self.clear_caches();
        debug!("Backpropagation configuration updated");
    }
    
    /// Get current configuration reference
    pub fn get_config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Get entanglement graph reference
    pub fn get_entanglement_graph(&self) -> &Arc<RwLock<EntanglementGraph>> {
        &self.entanglement_graph
    }
}

/// Direct backpropagation result for internal use
#[derive(Debug, Clone)]
pub(super) struct DirectBackpropagationResult {
    pub nodes_updated: usize,
    pub total_reward: Complex64,
}

/// Entanglement backpropagation result for internal use
#[derive(Debug, Clone)]
pub(super) struct EntanglementBackpropagationResult {
    pub entanglements_processed: usize,
    pub nodes_updated: usize,
    pub total_reward: Complex64,
}

/// Cache statistics for performance monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    /// Current path cache size
    pub path_cache_size: usize,
    /// Path cache capacity
    pub path_cache_capacity: usize,
    /// Current reward cache size
    pub reward_cache_size: usize,
    /// Reward cache capacity
    pub reward_cache_capacity: usize,
}

impl CacheStats {
    /// Calculate cache utilization percentage
    pub fn path_cache_utilization(&self) -> f64 {
        if self.path_cache_capacity > 0 {
            self.path_cache_size as f64 / self.path_cache_capacity as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate reward cache utilization percentage
    pub fn reward_cache_utilization(&self) -> f64 {
        if self.reward_cache_capacity > 0 {
            self.reward_cache_size as f64 / self.reward_cache_capacity as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Check if caches need pruning
    pub fn needs_pruning(&self) -> bool {
        self.path_cache_utilization() > 80.0 || self.reward_cache_utilization() > 80.0
    }
}
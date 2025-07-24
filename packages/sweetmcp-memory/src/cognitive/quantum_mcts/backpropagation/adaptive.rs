//! Adaptive backpropagation with dynamic learning rates
//!
//! This module provides adaptive backpropagation algorithms with dynamic learning
//! rates, reward normalization, and context-aware adaptation strategies.

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::{
    super::{
        node_state::{QuantumMCTSNode, QuantumNodeState},
        config::QuantumMCTSConfig,
    },
    core::QuantumBackpropagator,
    metrics::{BackpropagationResult, NormalizationResult},
};

/// Adaptive backpropagation methods for QuantumBackpropagator
impl QuantumBackpropagator {
    /// Advanced backpropagation with adaptive learning rates
    pub async fn adaptive_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        start_node_id: String,
        reward: Complex64,
        base_learning_rate: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        let path = self.get_propagation_path(tree, &start_node_id).await?;
        
        let mut tree_write = tree.write().await;
        let mut nodes_updated = 0;
        let mut total_reward_distributed = Complex64::new(0.0, 0.0);
        
        // Adaptive backpropagation with dynamic learning rate modulation
        for (depth, node_id) in path.iter().enumerate() {
            if let Some(node) = tree_write.get_mut(node_id) {
                // Calculate adaptive learning rate based on node characteristics
                let node_learning_rate = self.calculate_adaptive_learning_rate(
                    base_learning_rate,
                    node,
                    depth,
                );
                
                // Apply reward with adaptive rate and decay
                let decay_factor = self.calculate_decay_factor(depth);
                let adapted_reward = reward * decay_factor * node_learning_rate * node.amplitude;
                
                // Update node with careful numerical handling
                node.visits = node.visits.saturating_add(1);
                node.quantum_reward += adapted_reward;
                
                // Numerical stability check
                if !node.quantum_reward.is_finite() {
                    warn!("Numerical instability in adaptive backprop for node {}, resetting", node_id);
                    node.quantum_reward = Complex64::new(0.0, 0.0);
                }
                
                nodes_updated += 1;
                total_reward_distributed += adapted_reward;
                
                trace!("Adaptive backprop node {}: rate={:.3}, reward={:.3}",
                       node_id, node_learning_rate, adapted_reward.norm());
            }
        }
        
        let elapsed_time = start_time.elapsed();
        self.metrics.adaptive_backpropagations += 1;
        
        Ok(BackpropagationResult {
            nodes_updated,
            path_length: path.len(),
            reward_distributed: total_reward_distributed,
            entanglement_effects_applied: 0, // Not applied in adaptive mode
            elapsed_time,
            success: true,
        })
    }
    
    /// Calculate adaptive learning rate based on node characteristics
    #[inline]
    pub fn calculate_adaptive_learning_rate(
        &self,
        base_rate: f64,
        node: &QuantumMCTSNode,
        depth: usize,
    ) -> f64 {
        // Visit-based adaptation (less visited nodes learn faster)
        let visit_factor = self.calculate_visit_factor(node.visits);
        
        // Coherence-based adaptation (coherent nodes learn more effectively)
        let coherence_factor = self.calculate_coherence_factor(&node.quantum_state);
        
        // Depth-based adaptation (deeper nodes have reduced learning capacity)
        let depth_factor = self.calculate_depth_factor_adaptive(depth);
        
        // Amplitude-based adaptation (low amplitude nodes need enhanced learning)
        let amplitude_factor = self.calculate_amplitude_factor(node.amplitude);
        
        // Reward history adaptation (avoid over-adaptation to outliers)
        let reward_factor = self.calculate_reward_history_factor(node);
        
        // Combine all factors with numerical stability
        let combined_rate = base_rate * visit_factor * coherence_factor * 
                           depth_factor * amplitude_factor * reward_factor;
        
        // Ensure learning rate stays within reasonable bounds
        combined_rate.clamp(base_rate * 0.1, base_rate * 3.0)
    }
    
    /// Calculate visit-based learning factor
    #[inline]
    fn calculate_visit_factor(&self, visits: u64) -> f64 {
        // Inverse square root scaling for diminishing learning with more visits
        1.0 / (1.0 + (visits as f64).sqrt() * 0.1)
    }
    
    /// Calculate coherence-based learning factor
    #[inline]
    fn calculate_coherence_factor(&self, quantum_state: &QuantumNodeState) -> f64 {
        // Coherent states can learn more effectively
        let coherence = 1.0 - quantum_state.decoherence.clamp(0.0, 1.0);
        0.5 + coherence * 0.5 // Range: [0.5, 1.0]
    }
    
    /// Calculate depth-based learning factor for adaptive backpropagation
    #[inline]
    fn calculate_depth_factor_adaptive(&self, depth: usize) -> f64 {
        // More gradual decay than standard backpropagation
        1.0 / (1.0 + depth as f64 * 0.05) // 5% reduction per level
    }
    
    /// Calculate amplitude-based learning factor
    #[inline]
    fn calculate_amplitude_factor(&self, amplitude: Complex64) -> f64 {
        // Low amplitude nodes need more aggressive learning
        let amplitude_norm = amplitude.norm().clamp(0.01, 1.0);
        1.5 - amplitude_norm * 0.5 // Range: [1.0, 1.5]
    }
    
    /// Calculate reward history factor to prevent over-adaptation
    #[inline]
    fn calculate_reward_history_factor(&self, node: &QuantumMCTSNode) -> f64 {
        if node.visits == 0 {
            return 1.0;
        }
        
        // Calculate average reward magnitude to detect outliers
        let avg_reward_magnitude = node.quantum_reward.norm() / node.visits as f64;
        
        // Moderate extreme values to prevent over-adaptation
        if avg_reward_magnitude > 2.0 {
            0.7 // Reduce learning for nodes with very high rewards
        } else if avg_reward_magnitude < 0.1 {
            1.2 // Increase learning for nodes with very low rewards
        } else {
            1.0 // Normal learning rate
        }
    }
    
    /// Multi-objective adaptive backpropagation with multiple reward components
    pub async fn multi_objective_adaptive_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        start_node_id: String,
        rewards: &[Complex64], // Multiple objective rewards
        weights: &[f64],       // Weights for each objective
        base_learning_rate: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        if rewards.len() != weights.len() {
            return Err(CognitiveError::InvalidParameter(
                "Rewards and weights must have the same length".to_string()
            ));
        }
        
        let start_time = std::time::Instant::now();
        let path = self.get_propagation_path(tree, &start_node_id).await?;
        
        // Calculate weighted combined reward
        let combined_reward = rewards.iter()
            .zip(weights.iter())
            .map(|(reward, weight)| *reward * *weight)
            .fold(Complex64::new(0.0, 0.0), |acc, r| acc + r);
        
        let mut tree_write = tree.write().await;
        let mut nodes_updated = 0;
        let mut total_reward_distributed = Complex64::new(0.0, 0.0);
        
        // Apply multi-objective adaptive backpropagation
        for (depth, node_id) in path.iter().enumerate() {
            if let Some(node) = tree_write.get_mut(node_id) {
                // Calculate objective-specific learning rates
                let base_adaptive_rate = self.calculate_adaptive_learning_rate(
                    base_learning_rate,
                    node,
                    depth,
                );
                
                // Apply multi-objective reward with individual adaptations
                let mut node_reward_update = Complex64::new(0.0, 0.0);
                
                for (i, (&objective_reward, &weight)) in rewards.iter().zip(weights.iter()).enumerate() {
                    // Objective-specific adaptation
                    let objective_factor = self.calculate_objective_adaptation_factor(i, node);
                    let objective_learning_rate = base_adaptive_rate * objective_factor;
                    
                    let decay_factor = self.calculate_decay_factor(depth);
                    let objective_contribution = objective_reward * weight * 
                                               objective_learning_rate * decay_factor * node.amplitude;
                    
                    node_reward_update += objective_contribution;
                }
                
                // Update node
                node.visits = node.visits.saturating_add(1);
                node.quantum_reward += node_reward_update;
                
                // Numerical stability
                if !node.quantum_reward.is_finite() {
                    warn!("Numerical instability in multi-objective backprop for node {}", node_id);
                    node.quantum_reward = Complex64::new(0.0, 0.0);
                }
                
                nodes_updated += 1;
                total_reward_distributed += node_reward_update;
                
                trace!("Multi-objective backprop node {}: reward={:.3}",
                       node_id, node_reward_update.norm());
            }
        }
        
        let elapsed_time = start_time.elapsed();
        self.metrics.adaptive_backpropagations += 1;
        
        Ok(BackpropagationResult {
            nodes_updated,
            path_length: path.len(),
            reward_distributed: total_reward_distributed,
            entanglement_effects_applied: 0,
            elapsed_time,
            success: true,
        })
    }
    
    /// Calculate objective-specific adaptation factor
    #[inline]
    fn calculate_objective_adaptation_factor(&self, objective_index: usize, node: &QuantumMCTSNode) -> f64 {
        // Different objectives might need different adaptation strategies
        match objective_index {
            0 => 1.0,                                    // Primary objective - normal rate
            1 => 0.7,                                    // Secondary objective - reduced rate
            2 => 0.5,                                    // Tertiary objective - further reduced
            _ => 1.0 / (1.0 + objective_index as f64),   // Diminishing rates for additional objectives
        }
    }
    
    /// Vectorized reward normalization for numerical stability
    pub async fn normalize_tree_rewards(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        max_magnitude: f64,
    ) -> Result<NormalizationResult, CognitiveError> {
        let mut tree_write = tree.write().await;
        let mut nodes_normalized = 0;
        let mut total_scaling_applied = 0.0;
        let max_magnitude = max_magnitude.max(0.01); // Ensure positive max magnitude
        
        // Vectorized normalization pass
        for node in tree_write.values_mut() {
            let current_magnitude = node.quantum_reward.norm();
            
            if current_magnitude > max_magnitude {
                let scaling_factor = max_magnitude / current_magnitude;
                node.quantum_reward *= scaling_factor;
                
                nodes_normalized += 1;
                total_scaling_applied += scaling_factor;
                
                trace!("Normalized node {} reward: {:.3} -> {:.3}",
                       node.id, current_magnitude, node.quantum_reward.norm());
            }
        }
        
        let average_scaling = if nodes_normalized > 0 {
            total_scaling_applied / nodes_normalized as f64
        } else {
            1.0
        };
        
        self.metrics.normalization_operations += 1;
        
        debug!("Reward normalization: {} nodes normalized, avg scaling: {:.3}",
               nodes_normalized, average_scaling);
        
        Ok(NormalizationResult {
            nodes_normalized,
            average_scaling_factor: average_scaling,
            max_magnitude_enforced: max_magnitude,
        })
    }
    
    /// Adaptive batch normalization with statistical analysis
    pub async fn adaptive_normalize_tree_rewards(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        target_percentile: f64, // e.g., 95.0 for 95th percentile
    ) -> Result<NormalizationResult, CognitiveError> {
        let tree_read = tree.read().await;
        
        // Collect all reward magnitudes for statistical analysis
        let reward_magnitudes: Vec<f64> = tree_read.values()
            .filter(|node| node.visits > 0)
            .map(|node| (node.quantum_reward / node.visits as f64).norm())
            .collect();
        
        if reward_magnitudes.is_empty() {
            return Ok(NormalizationResult {
                nodes_normalized: 0,
                average_scaling_factor: 1.0,
                max_magnitude_enforced: 0.0,
            });
        }
        
        // Calculate adaptive threshold based on percentile
        let mut sorted_magnitudes = reward_magnitudes.clone();
        sorted_magnitudes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let percentile_index = ((target_percentile / 100.0) * (sorted_magnitudes.len() - 1) as f64) as usize;
        let adaptive_threshold = sorted_magnitudes[percentile_index.min(sorted_magnitudes.len() - 1)];
        
        drop(tree_read);
        
        // Apply normalization with adaptive threshold
        self.normalize_tree_rewards(tree, adaptive_threshold).await
    }
    
    /// Temperature-based adaptive backpropagation for exploration control
    pub async fn temperature_adaptive_backpropagate(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        start_node_id: String,
        reward: Complex64,
        temperature: f64, // Higher temperature = more exploration
        base_learning_rate: f64,
    ) -> Result<BackpropagationResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        let path = self.get_propagation_path(tree, &start_node_id).await?;
        
        let mut tree_write = tree.write().await;
        let mut nodes_updated = 0;
        let mut total_reward_distributed = Complex64::new(0.0, 0.0);
        
        // Temperature-modulated adaptive backpropagation
        for (depth, node_id) in path.iter().enumerate() {
            if let Some(node) = tree_write.get_mut(node_id) {
                // Calculate base adaptive learning rate
                let base_adaptive_rate = self.calculate_adaptive_learning_rate(
                    base_learning_rate,
                    node,
                    depth,
                );
                
                // Apply temperature modulation
                let temperature_factor = self.calculate_temperature_factor(temperature, node, depth);
                let temperature_modulated_rate = base_adaptive_rate * temperature_factor;
                
                // Apply reward with temperature adaptation
                let decay_factor = self.calculate_decay_factor(depth);
                let adapted_reward = reward * decay_factor * temperature_modulated_rate * node.amplitude;
                
                // Add temperature-based noise for exploration
                let exploration_noise = self.calculate_exploration_noise(temperature, depth);
                let final_reward = adapted_reward + exploration_noise;
                
                // Update node
                node.visits = node.visits.saturating_add(1);
                node.quantum_reward += final_reward;
                
                // Numerical stability
                if !node.quantum_reward.is_finite() {
                    warn!("Numerical instability in temperature adaptive backprop for node {}", node_id);
                    node.quantum_reward = Complex64::new(0.0, 0.0);
                }
                
                nodes_updated += 1;
                total_reward_distributed += final_reward;
                
                trace!("Temperature adaptive backprop node {}: temp={:.3}, reward={:.3}",
                       node_id, temperature, final_reward.norm());
            }
        }
        
        let elapsed_time = start_time.elapsed();
        self.metrics.adaptive_backpropagations += 1;
        
        Ok(BackpropagationResult {
            nodes_updated,
            path_length: path.len(),
            reward_distributed: total_reward_distributed,
            entanglement_effects_applied: 0,
            elapsed_time,
            success: true,
        })
    }
    
    /// Calculate temperature factor for exploration control
    #[inline]
    fn calculate_temperature_factor(&self, temperature: f64, node: &QuantumMCTSNode, depth: usize) -> f64 {
        // Higher temperature increases learning rate variation
        let base_factor = 1.0;
        let temperature_effect = temperature * 0.1; // Scale temperature impact
        let depth_modulation = 1.0 / (1.0 + depth as f64 * 0.05); // Reduce with depth
        
        base_factor + temperature_effect * depth_modulation
    }
    
    /// Calculate exploration noise based on temperature
    #[inline]
    fn calculate_exploration_noise(&self, temperature: f64, depth: usize) -> Complex64 {
        if temperature <= 0.01 {
            return Complex64::new(0.0, 0.0); // No noise at very low temperature
        }
        
        // Generate temperature-scaled noise that decreases with depth
        let noise_magnitude = temperature * 0.01 / (1.0 + depth as f64 * 0.1);
        let noise_phase = fastrand::f64() * 2.0 * std::f64::consts::PI;
        
        Complex64::new(0.0, noise_phase).exp() * noise_magnitude
    }
}
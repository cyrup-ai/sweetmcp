//! Quantum amplitude amplification engine with blazing-fast optimization
//!
//! This module provides specialized amplitude amplification for promising quantum
//! paths with zero-allocation confidence calculation and adaptive amplification factors.

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::{
    super::{
        node_state::QuantumMCTSNode,
        config::QuantumMCTSConfig,
    },
    metrics::AmplificationResult,
};

/// Quantum amplitude amplification engine
pub struct AmplificationEngine {
    config: QuantumMCTSConfig,
}

impl AmplificationEngine {
    /// Create new amplification engine with optimized configuration
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self { config }
    }
    
    /// Apply quantum amplitude amplification to promising paths
    pub async fn amplify_promising_paths(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<AmplificationResult, CognitiveError> {
        let mut tree_write = tree.write().await;
        let root = tree_write.get(root_id)
            .ok_or_else(|| CognitiveError::InvalidState("Root node not found for amplification".to_string()))?;

        if root.children.is_empty() {
            return Ok(AmplificationResult {
                nodes_processed: 0,
                nodes_amplified: 0,
                average_amplification: 1.0,
                total_amplification: 0.0,
            });
        }

        let mut nodes_processed = 0;
        let mut nodes_amplified = 0;
        let mut total_amplification = 0.0;

        // Pre-allocate for zero-allocation pattern
        let child_count = root.children.len();
        let mut child_rewards = Vec::with_capacity(child_count);
        let mut child_ids = Vec::with_capacity(child_count);

        // Collect child information for blazing-fast processing
        for child_id in root.children.values() {
            if let Some(child) = tree_write.get(child_id) {
                let avg_reward = if child.visits > 0 {
                    child.quantum_reward.norm() / child.visits as f64
                } else {
                    0.0
                };
                child_rewards.push(avg_reward);
                child_ids.push(child_id.clone());
            }
        }

        // Calculate amplification threshold with optimized statistics
        let mean_reward = if !child_rewards.is_empty() {
            child_rewards.iter().sum::<f64>() / child_rewards.len() as f64
        } else {
            0.0
        };

        // Apply amplification to promising nodes using zero-allocation iteration
        for (i, child_id) in child_ids.iter().enumerate() {
            if let Some(child) = tree_write.get_mut(child_id) {
                nodes_processed += 1;
                
                let avg_reward = child_rewards[i];
                if avg_reward > mean_reward && avg_reward > self.config.amplitude_threshold {
                    let confidence = self.calculate_node_confidence(child);
                    let amplification_factor = self.calculate_amplification_factor(avg_reward, confidence);
                    
                    // Apply quantum amplitude amplification with blazing-fast complex multiplication
                    child.amplitude *= Complex64::new(amplification_factor, 0.0);
                    
                    nodes_amplified += 1;
                    total_amplification += amplification_factor;
                    
                    trace!("Amplified node {} by factor {:.3}", child_id, amplification_factor);
                }
            }
        }

        let average_amplification = if nodes_amplified > 0 {
            total_amplification / nodes_amplified as f64
        } else {
            1.0
        };

        debug!("Amplitude amplification: {}/{} nodes amplified, avg factor: {:.3}",
               nodes_amplified, nodes_processed, average_amplification);

        Ok(AmplificationResult {
            nodes_processed,
            nodes_amplified,
            average_amplification,
            total_amplification,
        })
    }
    
    /// Calculate node confidence based on multiple factors with blazing-fast computation
    #[inline]
    fn calculate_node_confidence(&self, node: &QuantumMCTSNode) -> f64 {
        let visit_confidence = (node.visits as f64).sqrt() / (node.visits as f64 + 10.0);
        let amplitude_confidence = node.amplitude.norm().min(1.0);
        let coherence_confidence = 1.0 - node.quantum_state.decoherence;
        let depth_confidence = 1.0 / (1.0 + node.improvement_depth as f64 * 0.1);
        
        // Weighted combination of confidence factors with zero allocation
        visit_confidence * 0.3 + amplitude_confidence * 0.3 + coherence_confidence * 0.3 + depth_confidence * 0.1
    }
    
    /// Calculate adaptive amplification factor with optimized computation
    #[inline]
    fn calculate_amplification_factor(&self, avg_reward: f64, confidence: f64) -> f64 {
        let base_factor = 1.1; // Minimum amplification
        let performance_factor = (avg_reward / self.config.amplitude_threshold).min(3.0); // Cap at 3x
        let confidence_factor = confidence.sqrt(); // Square root for moderation
        
        base_factor + (performance_factor - 1.0) * confidence_factor * 0.5
    }
    
    /// Update amplification configuration
    pub fn update_config(&mut self, config: QuantumMCTSConfig) {
        self.config = config;
    }
    
    /// Get current amplification threshold
    pub fn get_amplitude_threshold(&self) -> f64 {
        self.config.amplitude_threshold
    }
}
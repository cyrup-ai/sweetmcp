//! Quantum convergence analysis with blazing-fast multi-metric evaluation
//!
//! This module provides advanced convergence detection using amplitude concentration,
//! visit distribution, reward stability, and entropy analysis with zero-allocation patterns.

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::trace;

use crate::cognitive::types::CognitiveError;
use super::{
    super::{
        node_state::QuantumMCTSNode,
        config::QuantumMCTSConfig,
    },
};

/// Quantum convergence analysis engine
pub struct ConvergenceAnalyzer {
    config: QuantumMCTSConfig,
}

impl ConvergenceAnalyzer {
    /// Create new convergence analyzer with optimized configuration
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self { config }
    }
    
    /// Advanced quantum convergence check with multiple metrics
    pub async fn check_quantum_convergence_advanced(
        &self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        root_id: &str,
    ) -> Result<f64, CognitiveError> {
        let tree_read = tree.read().await;
        let root = tree_read.get(root_id)
            .ok_or_else(|| CognitiveError::InvalidState("Root node not found for convergence check".to_string()))?;
        
        if root.children.is_empty() {
            return Ok(0.0);
        }

        // Calculate multiple convergence metrics with blazing-fast computation
        let amplitude_convergence = self.calculate_amplitude_convergence(&tree_read, root)?;
        let visit_convergence = self.calculate_visit_convergence(&tree_read, root)?;
        let reward_convergence = self.calculate_reward_convergence(&tree_read, root)?;
        let entropy_convergence = self.calculate_entropy_convergence(&tree_read, root)?;

        // Weighted combination of convergence metrics with zero allocation
        let overall_convergence = amplitude_convergence * 0.3 
            + visit_convergence * 0.25 
            + reward_convergence * 0.25 
            + entropy_convergence * 0.2;

        trace!("Convergence analysis - Amplitude: {:.3}, Visits: {:.3}, Reward: {:.3}, Entropy: {:.3}, Overall: {:.3}",
               amplitude_convergence, visit_convergence, reward_convergence, entropy_convergence, overall_convergence);

        Ok(overall_convergence.min(1.0))
    }
    
    /// Calculate amplitude-based convergence (concentration of quantum amplitude)
    fn calculate_amplitude_convergence(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        root: &QuantumMCTSNode,
    ) -> Result<f64, CognitiveError> {
        // Pre-allocate for zero-allocation pattern
        let mut amplitudes = Vec::with_capacity(root.children.len());
        let mut total_amplitude = 0.0;

        // Single-pass collection for blazing-fast processing
        for child_id in root.children.values() {
            if let Some(child) = tree.get(child_id) {
                let amp = child.amplitude.norm();
                amplitudes.push(amp);
                total_amplitude += amp;
            }
        }

        if amplitudes.is_empty() || total_amplitude <= 1e-10 {
            return Ok(0.0);
        }

        // Calculate concentration using normalized amplitudes with optimized max finding
        let max_amplitude = amplitudes.iter().copied().fold(0.0f64, f64::max);
        let concentration = max_amplitude / total_amplitude;

        // Convert to convergence score (higher concentration = higher convergence)
        Ok(concentration.powi(2)) // Quadratic to emphasize strong concentration
    }
    
    /// Calculate visit-based convergence (concentration of MCTS visits)
    fn calculate_visit_convergence(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        root: &QuantumMCTSNode,
    ) -> Result<f64, CognitiveError> {
        // Pre-allocate for blazing-fast collection
        let mut visits = Vec::with_capacity(root.children.len());
        let mut total_visits = 0u64;

        // Single-pass collection with zero allocation
        for child_id in root.children.values() {
            if let Some(child) = tree.get(child_id) {
                visits.push(child.visits);
                total_visits += child.visits;
            }
        }

        if visits.is_empty() || total_visits == 0 {
            return Ok(0.0);
        }

        // Calculate visit concentration with optimized max finding
        let max_visits = visits.iter().copied().max().unwrap_or(0);
        let concentration = max_visits as f64 / total_visits as f64;

        // Apply square root to moderate the effect (visits grow more slowly than amplitudes)
        Ok(concentration.sqrt())
    }
    
    /// Calculate reward-based convergence (stability of reward estimates)
    fn calculate_reward_convergence(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        root: &QuantumMCTSNode,
    ) -> Result<f64, CognitiveError> {
        // Pre-allocate for blazing-fast processing
        let mut rewards = Vec::with_capacity(root.children.len());

        // Collect average rewards with zero-allocation iteration
        for child_id in root.children.values() {
            if let Some(child) = tree.get(child_id) {
                if child.visits > 0 {
                    let avg_reward = child.quantum_reward.norm() / child.visits as f64;
                    rewards.push(avg_reward);
                }
            }
        }

        if rewards.len() < 2 {
            return Ok(0.0);
        }

        // Calculate coefficient of variation (std dev / mean) with optimized statistics
        let rewards_len = rewards.len() as f64;
        let mean = rewards.iter().sum::<f64>() / rewards_len;
        let variance = rewards.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / rewards_len;
        let std_dev = variance.sqrt();

        if mean <= 1e-10 {
            return Ok(0.0);
        }

        let cv = std_dev / mean;
        
        // Convert to convergence score (lower CV = higher convergence)
        // Use exponential decay for smooth convergence curve
        Ok((-cv * 2.0).exp())
    }
    
    /// Calculate entropy-based convergence (information distribution)
    fn calculate_entropy_convergence(
        &self,
        tree: &HashMap<String, QuantumMCTSNode>,
        root: &QuantumMCTSNode,
    ) -> Result<f64, CognitiveError> {
        // Collect probability distribution with zero allocation
        let mut probabilities = Vec::with_capacity(root.children.len());
        let mut total_amplitude = 0.0;

        // First pass: collect amplitudes
        for child_id in root.children.values() {
            if let Some(child) = tree.get(child_id) {
                let amp_norm = child.amplitude.norm();
                probabilities.push(amp_norm);
                total_amplitude += amp_norm;
            }
        }

        if probabilities.is_empty() || total_amplitude <= 1e-10 {
            return Ok(0.0);
        }

        // Normalize to probabilities and calculate entropy in single pass
        let mut entropy = 0.0;
        let total_inv = 1.0 / total_amplitude; // Precompute for blazing-fast division
        
        for amp in probabilities.iter_mut() {
            *amp *= total_inv; // Normalize
            if *amp > 1e-10 {
                entropy -= *amp * amp.ln();
            }
        }

        // Calculate maximum possible entropy for normalization
        let max_entropy = (probabilities.len() as f64).ln();
        
        if max_entropy <= 1e-10 {
            return Ok(1.0);
        }

        // Convert to convergence score (lower entropy = higher convergence)
        let normalized_entropy = entropy / max_entropy;
        Ok(1.0 - normalized_entropy)
    }
    
    /// Update convergence configuration
    pub fn update_config(&mut self, config: QuantumMCTSConfig) {
        self.config = config;
    }
}
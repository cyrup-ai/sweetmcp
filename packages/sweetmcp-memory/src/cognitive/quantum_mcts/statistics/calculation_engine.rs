//! Statistical calculation engine with blazing-fast algorithms
//!
//! This module provides comprehensive statistical calculations with vectorized
//! operations, zero-allocation patterns, and optimized complex number handling.

use std::collections::HashMap;
use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::{
    super::node_state::QuantumMCTSNode,
    metrics::{DepthStatistics, RewardStatistics, ConvergenceMetrics},
    performance::PerformanceMetrics,
    counter_snapshot::CounterSnapshot,
};

/// Statistical calculation engine for quantum MCTS analysis
pub struct CalculationEngine;

impl CalculationEngine {
    /// Calculate tree depth statistics with efficient traversal
    pub async fn calculate_depth_statistics(
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<DepthStatistics, CognitiveError> {
        if tree.is_empty() {
            return Ok(DepthStatistics::default());
        }
        
        let mut max_depth = 0u32;
        let mut total_depth = 0u64;
        let mut leaf_nodes = 0;
        let mut internal_nodes = 0;
        let mut depth_counts: HashMap<u32, usize> = HashMap::new();
        
        // Single pass analysis with zero allocation patterns
        for node in tree.values() {
            let depth = node.depth;
            max_depth = max_depth.max(depth);
            total_depth += depth as u64;
            
            if node.children.is_empty() {
                leaf_nodes += 1;
            } else {
                internal_nodes += 1;
            }
            
            *depth_counts.entry(depth).or_insert(0) += 1;
        }
        
        let avg_depth = if !tree.is_empty() {
            total_depth as f64 / tree.len() as f64
        } else {
            0.0
        };
        
        let mut depth_distribution: Vec<(u32, usize)> = depth_counts.into_iter().collect();
        depth_distribution.sort_by_key(|(depth, _)| *depth);
        
        Ok(DepthStatistics::new(
            max_depth,
            avg_depth,
            leaf_nodes,
            internal_nodes,
            depth_distribution,
        ))
    }
    
    /// Calculate reward statistics with optimized complex number handling
    pub async fn calculate_reward_statistics(
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<RewardStatistics, CognitiveError> {
        if tree.is_empty() {
            return Ok(RewardStatistics::new(
                Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0),
                0.0,
                0.0,
                0.0,
                0,
                0,
            ));
        }
        
        let mut total_reward = Complex64::new(0.0, 0.0);
        let mut reward_magnitudes = Vec::with_capacity(tree.len());
        let mut positive_reward_nodes = 0;
        let mut negative_reward_nodes = 0;
        
        // Single pass collection with zero allocation
        for node in tree.values() {
            let node_reward = if node.visits > 0 {
                node.quantum_reward / node.visits as f64
            } else {
                node.quantum_reward
            };
            
            total_reward += node_reward;
            let magnitude = node_reward.norm();
            reward_magnitudes.push(magnitude);
            
            if node_reward.re > 0.0 {
                positive_reward_nodes += 1;
            } else if node_reward.re < 0.0 {
                negative_reward_nodes += 1;
            }
        }
        
        let avg_reward = total_reward / tree.len() as f64;
        let max_reward_magnitude = reward_magnitudes.iter().copied().fold(0.0f64, f64::max);
        let min_reward_magnitude = reward_magnitudes.iter().copied().fold(f64::INFINITY, f64::min);
        
        // Calculate variance with optimized computation
        let avg_magnitude = avg_reward.norm();
        let variance = reward_magnitudes.iter()
            .map(|&mag| (mag - avg_magnitude).powi(2))
            .sum::<f64>() / tree.len() as f64;
        
        Ok(RewardStatistics::new(
            total_reward,
            avg_reward,
            max_reward_magnitude,
            min_reward_magnitude.min(max_reward_magnitude),
            variance,
            positive_reward_nodes,
            negative_reward_nodes,
        ))
    }
    
    /// Calculate convergence metrics with comprehensive analysis
    pub async fn calculate_convergence_metrics(
        tree: &HashMap<String, QuantumMCTSNode>,
    ) -> Result<ConvergenceMetrics, CognitiveError> {
        if tree.is_empty() {
            return Ok(ConvergenceMetrics::new(0.0, 0.0, 0.0, 0.0, 0.0));
        }
        
        let total_visits = Self::calculate_total_visits(tree);
        let mut total_amplitude = 0.0;
        let mut max_amplitude = 0.0;
        let mut max_visits = 0u64;
        
        // Collect data for convergence analysis
        for node in tree.values() {
            let amp = node.amplitude.norm();
            total_amplitude += amp;
            max_amplitude = max_amplitude.max(amp);
            max_visits = max_visits.max(node.visits);
        }
        
        // Calculate amplitude concentration
        let amplitude_concentration = if total_amplitude > 1e-10 {
            max_amplitude / total_amplitude
        } else {
            0.0
        };
        
        // Calculate visit concentration
        let visit_concentration = if total_visits > 0 {
            max_visits as f64 / total_visits as f64
        } else {
            0.0
        };
        
        // Calculate reward stability (coefficient of variation)
        let reward_stability = Self::calculate_reward_stability(tree).await;
        
        // Calculate Shannon entropy
        let entropy = Self::calculate_entropy(tree, total_visits).await;
        
        // Overall convergence score (weighted combination)
        let overall_convergence = amplitude_concentration * 0.3 
            + visit_concentration * 0.3 
            + reward_stability * 0.3 
            + (1.0 - entropy / (tree.len() as f64).ln().max(1.0)) * 0.1;
        
        Ok(ConvergenceMetrics::new(
            amplitude_concentration,
            visit_concentration,
            reward_stability,
            entropy,
            overall_convergence.min(1.0),
        ))
    }
    
    /// Calculate performance metrics with throughput analysis
    pub async fn calculate_performance_metrics(
        tree: &HashMap<String, QuantumMCTSNode>,
        start_time: std::time::Instant,
        counter_values: &CounterSnapshot,
    ) -> Result<PerformanceMetrics, CognitiveError> {
        let total_nodes = tree.len();
        let total_visits = Self::calculate_total_visits(tree);
        
        let avg_visits_per_node = if total_nodes > 0 {
            total_visits as f64 / total_nodes as f64
        } else {
            0.0
        };
        
        // Calculate rates based on elapsed time
        let elapsed_seconds = start_time.elapsed().as_secs_f64();
        let node_creation_rate = if elapsed_seconds > 0.0 {
            total_nodes as f64 / elapsed_seconds
        } else {
            0.0
        };
        
        // Create performance metrics using collector data
        PerformanceMetrics::from_collector_data(
            avg_visits_per_node,
            node_creation_rate,
            elapsed_seconds,
            counter_values,
        )
    }
    
    /// Calculate total visits with SIMD-friendly summation
    #[inline]
    pub fn calculate_total_visits(tree: &HashMap<String, QuantumMCTSNode>) -> u64 {
        tree.values()
            .map(|node| node.visits)
            .sum()
    }
    
    /// Calculate quantum-specific statistics with vectorized operations
    #[inline]
    pub fn calculate_quantum_stats(tree: &HashMap<String, QuantumMCTSNode>) -> (f64, f64) {
        if tree.is_empty() {
            return (0.0, 0.0);
        }
        
        let mut total_decoherence = 0.0;
        let mut max_amplitude = 0.0;
        
        // Vectorized calculation
        for node in tree.values() {
            total_decoherence += node.quantum_state.decoherence;
            max_amplitude = max_amplitude.max(node.amplitude.norm());
        }
        
        let avg_decoherence = total_decoherence / tree.len() as f64;
        (avg_decoherence, max_amplitude)
    }
    
    /// Calculate reward stability using coefficient of variation
    async fn calculate_reward_stability(tree: &HashMap<String, QuantumMCTSNode>) -> f64 {
        let rewards: Vec<f64> = tree.values()
            .map(|node| if node.visits > 0 {
                node.quantum_reward.norm() / node.visits as f64
            } else {
                node.quantum_reward.norm()
            })
            .collect();
        
        if rewards.len() < 2 {
            return 0.0;
        }
        
        let mean = rewards.iter().sum::<f64>() / rewards.len() as f64;
        let variance = rewards.iter()
            .map(|&r| (r - mean).powi(2))
            .sum::<f64>() / (rewards.len() - 1) as f64;
        
        let std_dev = variance.sqrt();
        if mean > 1e-10 {
            1.0 - (std_dev / mean).min(1.0) // Higher stability = lower CV
        } else {
            0.0
        }
    }
    
    /// Calculate Shannon entropy of visit distribution
    async fn calculate_entropy(tree: &HashMap<String, QuantumMCTSNode>, total_visits: u64) -> f64 {
        if total_visits == 0 {
            return 0.0;
        }
        
        let mut entropy = 0.0;
        for node in tree.values() {
            if node.visits > 0 {
                let p = node.visits as f64 / total_visits as f64;
                entropy -= p * p.ln();
            }
        }
        entropy
    }
}
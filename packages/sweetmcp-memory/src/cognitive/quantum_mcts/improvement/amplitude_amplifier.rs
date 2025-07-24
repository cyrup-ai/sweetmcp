//! Quantum amplitude amplifier with adaptive parameters and blazing-fast optimization
//!
//! This module provides advanced quantum amplitude amplification with machine learning
//! adaptation, confidence-based scaling, and zero-allocation performance patterns.

use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, trace};

use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::super::node_state::QuantumMCTSNode;

/// Quantum amplitude amplifier with adaptive configuration
pub struct QuantumAmplitudeAmplifier {
    /// Current amplifier configuration
    config: AmplifierConfig,
    /// Adaptation statistics
    adaptation_stats: AdaptationStats,
    /// Performance history for learning
    performance_history: Vec<AmplificationPerformance>,
}

impl QuantumAmplitudeAmplifier {
    /// Create new quantum amplitude amplifier
    pub fn new() -> Self {
        Self {
            config: AmplifierConfig::default(),
            adaptation_stats: AdaptationStats::new(),
            performance_history: Vec::with_capacity(100),
        }
    }
    
    /// Create amplifier with custom configuration
    pub fn with_config(config: AmplifierConfig) -> Self {
        Self {
            config,
            adaptation_stats: AdaptationStats::new(),
            performance_history: Vec::with_capacity(100),
        }
    }
    
    /// Amplify promising nodes with adaptive parameters
    pub async fn amplify_promising_nodes(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        convergence_score: f64,
        target_nodes: Option<&[String]>,
    ) -> Result<AmplificationResult, CognitiveError> {
        let start_time = std::time::Instant::now();
        let mut tree_write = tree.write().await;
        
        // Determine nodes to process
        let node_ids: Vec<String> = if let Some(targets) = target_nodes {
            targets.to_vec()
        } else {
            tree_write.keys().cloned().collect()
        };
        
        if node_ids.is_empty() {
            return Ok(AmplificationResult::empty());
        }
        
        let mut nodes_processed = 0;
        let mut nodes_amplified = 0;
        let mut total_amplification = 0.0;
        let mut amplification_operations = Vec::new();
        
        // Pre-allocate for zero-allocation pattern
        let mut node_scores = Vec::with_capacity(node_ids.len());
        
        // Calculate amplification scores for all nodes
        for node_id in &node_ids {
            if let Some(node) = tree_write.get(node_id) {
                let score = self.calculate_amplification_score(node, convergence_score);
                node_scores.push((node_id.clone(), score));
                nodes_processed += 1;
            }
        }
        
        // Sort by score for prioritized amplification
        node_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Apply amplification to top nodes
        let amplification_threshold = self.config.base_threshold * (1.0 + convergence_score * 0.5);
        
        for (node_id, score) in node_scores {
            if score > amplification_threshold {
                if let Some(node) = tree_write.get_mut(&node_id) {
                    let amplification_factor = self.calculate_amplification_factor(score, convergence_score);
                    let original_amplitude = node.amplitude.norm();
                    
                    // Apply quantum amplitude amplification
                    node.amplitude *= Complex64::new(amplification_factor, 0.0);
                    
                    let new_amplitude = node.amplitude.norm();
                    let actual_amplification = new_amplitude / original_amplitude.max(1e-10);
                    
                    total_amplification += actual_amplification;
                    nodes_amplified += 1;
                    
                    amplification_operations.push(AmplificationOperation {
                        node_id: node_id.clone(),
                        original_amplitude,
                        new_amplitude,
                        amplification_factor,
                        score,
                    });
                    
                    trace!("Amplified node {} by factor {:.3} (score: {:.3})", 
                           node_id, amplification_factor, score);
                }
            }
        }
        
        let processing_time = start_time.elapsed();
        let average_amplification = if nodes_amplified > 0 {
            total_amplification / nodes_amplified as f64
        } else {
            1.0
        };
        
        let result = AmplificationResult {
            nodes_processed,
            nodes_amplified,
            average_amplification,
            total_amplification,
            processing_time,
            amplification_operations,
            convergence_score,
            threshold_used: amplification_threshold,
        };
        
        // Record performance for adaptation
        self.record_performance(&result);
        
        debug!("Amplitude amplification: {}/{} nodes amplified, avg factor: {:.3}, time: {:?}",
               nodes_amplified, nodes_processed, average_amplification, processing_time);
        
        Ok(result)
    }
    
    /// Calculate amplification score for a node
    fn calculate_amplification_score(&self, node: &QuantumMCTSNode, convergence_score: f64) -> f64 {
        let visit_score = (node.visits as f64).sqrt() / (node.visits as f64 + 10.0);
        let amplitude_score = node.amplitude.norm().min(1.0);
        let coherence_score = 1.0 - node.quantum_state.decoherence;
        let reward_score = if node.visits > 0 {
            node.quantum_reward.norm() / node.visits as f64
        } else {
            0.0
        };
        
        // Adaptive weighting based on convergence
        let convergence_weight = 0.5 + convergence_score * 0.3;
        let base_weight = 1.0 - convergence_weight;
        
        (visit_score * 0.25 + amplitude_score * 0.25 + coherence_score * 0.25 + reward_score * 0.25) 
            * base_weight + reward_score * convergence_weight
    }
    
    /// Calculate adaptive amplification factor
    fn calculate_amplification_factor(&self, score: f64, convergence_score: f64) -> f64 {
        let base_factor = self.config.base_amplification;
        let score_multiplier = (score / self.config.base_threshold).min(self.config.max_amplification);
        let convergence_boost = 1.0 + convergence_score * self.config.convergence_boost;
        let adaptation_factor = self.adaptation_stats.get_adaptation_factor();
        
        (base_factor * score_multiplier * convergence_boost * adaptation_factor)
            .clamp(1.0, self.config.max_amplification)
    }
    
    /// Record performance for adaptive learning
    fn record_performance(&mut self, result: &AmplificationResult) {
        let performance = AmplificationPerformance {
            amplification_ratio: result.amplification_ratio(),
            effectiveness: result.effectiveness(),
            processing_time: result.processing_time,
            convergence_score: result.convergence_score,
        };
        
        // Maintain rolling window of performance history
        if self.performance_history.len() >= 100 {
            self.performance_history.remove(0);
        }
        self.performance_history.push(performance);
        
        // Update adaptation statistics
        self.adaptation_stats.update(&performance);
    }
    
    /// Adapt parameters based on performance history
    pub fn adapt_parameters(&mut self, result: &AmplificationResult) {
        if self.performance_history.len() < 5 {
            return; // Need more history for adaptation
        }
        
        let recent_performance: Vec<_> = self.performance_history.iter().rev().take(5).collect();
        let avg_effectiveness: f64 = recent_performance.iter().map(|p| p.effectiveness).sum::<f64>() / 5.0;
        
        // Adapt threshold based on effectiveness
        if avg_effectiveness > 0.8 {
            // High effectiveness - can be more aggressive
            self.config.base_threshold *= 0.95;
            self.config.base_amplification *= 1.05;
        } else if avg_effectiveness < 0.4 {
            // Low effectiveness - be more conservative
            self.config.base_threshold *= 1.05;
            self.config.base_amplification *= 0.95;
        }
        
        // Clamp values to reasonable ranges
        self.config.base_threshold = self.config.base_threshold.clamp(0.1, 0.9);
        self.config.base_amplification = self.config.base_amplification.clamp(1.1, 3.0);
        
        trace!("Adapted amplifier parameters: threshold={:.3}, base_amp={:.3}", 
               self.config.base_threshold, self.config.base_amplification);
    }
    
    /// Reset parameters to default
    pub fn reset_parameters(&mut self) {
        self.config = AmplifierConfig::default();
        self.adaptation_stats.reset();
        self.performance_history.clear();
        debug!("Reset amplifier parameters to defaults");
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> AmplifierConfig {
        self.config.clone()
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: AmplifierConfig) {
        self.config = config;
    }
    
    /// Get adaptation statistics
    pub fn get_adaptation_stats(&self) -> &AdaptationStats {
        &self.adaptation_stats
    }
}

impl Default for QuantumAmplitudeAmplifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Amplifier configuration with adaptive parameters
#[derive(Debug, Clone)]
pub struct AmplifierConfig {
    /// Base amplification threshold
    pub base_threshold: f64,
    /// Base amplification factor
    pub base_amplification: f64,
    /// Maximum amplification allowed
    pub max_amplification: f64,
    /// Convergence boost factor
    pub convergence_boost: f64,
    /// Adaptation learning rate
    pub learning_rate: f64,
    /// Enable adaptive thresholding
    pub adaptive_threshold: bool,
}

impl Default for AmplifierConfig {
    fn default() -> Self {
        Self {
            base_threshold: 0.5,
            base_amplification: 1.2,
            max_amplification: 3.0,
            convergence_boost: 0.5,
            learning_rate: 0.1,
            adaptive_threshold: true,
        }
    }
}

/// Amplification result with comprehensive metrics
#[derive(Debug, Clone)]
pub struct AmplificationResult {
    pub nodes_processed: usize,
    pub nodes_amplified: usize,
    pub average_amplification: f64,
    pub total_amplification: f64,
    pub processing_time: std::time::Duration,
    pub amplification_operations: Vec<AmplificationOperation>,
    pub convergence_score: f64,
    pub threshold_used: f64,
}

impl AmplificationResult {
    /// Create empty result
    pub fn empty() -> Self {
        Self {
            nodes_processed: 0,
            nodes_amplified: 0,
            average_amplification: 1.0,
            total_amplification: 0.0,
            processing_time: std::time::Duration::from_millis(0),
            amplification_operations: Vec::new(),
            convergence_score: 0.0,
            threshold_used: 0.0,
        }
    }
    
    /// Get amplification ratio (amplified / processed)
    pub fn amplification_ratio(&self) -> f64 {
        if self.nodes_processed > 0 {
            self.nodes_amplified as f64 / self.nodes_processed as f64
        } else {
            0.0
        }
    }
    
    /// Get effectiveness score
    pub fn effectiveness(&self) -> f64 {
        self.amplification_ratio() * self.average_amplification * self.convergence_score
    }
    
    /// Get processing speed (nodes per second)
    pub fn processing_speed(&self) -> f64 {
        if self.processing_time.as_secs_f64() > 0.0 {
            self.nodes_processed as f64 / self.processing_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Individual amplification operation details
#[derive(Debug, Clone)]
pub struct AmplificationOperation {
    pub node_id: String,
    pub original_amplitude: f64,
    pub new_amplitude: f64,
    pub amplification_factor: f64,
    pub score: f64,
}

/// Adaptation statistics for learning
#[derive(Debug, Clone)]
pub struct AdaptationStats {
    /// Total adaptations performed
    pub total_adaptations: u64,
    /// Success rate of adaptations
    pub success_rate: f64,
    /// Average effectiveness improvement
    pub avg_improvement: f64,
    /// Current adaptation factor
    adaptation_factor: f64,
}

impl AdaptationStats {
    /// Create new adaptation statistics
    pub fn new() -> Self {
        Self {
            total_adaptations: 0,
            success_rate: 0.0,
            avg_improvement: 0.0,
            adaptation_factor: 1.0,
        }
    }
    
    /// Update statistics with performance data
    pub fn update(&mut self, performance: &AmplificationPerformance) {
        self.total_adaptations += 1;
        
        // Update adaptation factor based on effectiveness
        if performance.effectiveness > 0.7 {
            self.adaptation_factor = (self.adaptation_factor * 1.05).min(1.5);
        } else if performance.effectiveness < 0.4 {
            self.adaptation_factor = (self.adaptation_factor * 0.95).max(0.5);
        }
        
        // Update running averages (simplified)
        self.success_rate = (self.success_rate * 0.9) + (if performance.effectiveness > 0.5 { 0.1 } else { 0.0 });
    }
    
    /// Get current adaptation factor
    pub fn get_adaptation_factor(&self) -> f64 {
        self.adaptation_factor
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Performance tracking for adaptation
#[derive(Debug, Clone)]
pub struct AmplificationPerformance {
    pub amplification_ratio: f64,
    pub effectiveness: f64,
    pub processing_time: std::time::Duration,
    pub convergence_score: f64,
}
//! Quantum scoring and measurement logic for selection algorithms
//!
//! This module provides blazing-fast quantum UCT scoring with SIMD optimization,
//! entanglement bonuses, and optimized probability calculation for selection.

use std::collections::HashMap;
use rand::Rng;

use crate::cognitive::types::CognitiveError;
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// Quantum scorer for UCT calculations with optimization
pub struct QuantumScorer {
    /// Configuration for scoring parameters
    config: QuantumMCTSConfig,
}

impl QuantumScorer {
    /// Create new quantum scorer with configuration
    pub fn new(config: QuantumMCTSConfig) -> Self {
        Self { config }
    }
    
    /// Calculate quantum bonus for UCT scoring with amplitude and coherence
    #[inline(always)]
    pub fn calculate_quantum_bonus(&self, node: &QuantumMCTSNode) -> f64 {
        let amplitude_factor = node.amplitude.norm();
        let coherence_factor = 1.0 - node.quantum_state.decoherence;
        let entanglement_factor = self.calculate_entanglement_bonus(node);
        
        // Weighted combination of quantum factors
        amplitude_factor * coherence_factor + entanglement_factor
    }
    
    /// Calculate entanglement bonus for quantum UCT scoring
    #[inline(always)]
    fn calculate_entanglement_bonus(&self, node: &QuantumMCTSNode) -> f64 {
        let entanglement_count = node.quantum_state.entanglement_count() as f64;
        let max_entanglements = 10.0; // Reasonable upper bound
        
        // Logarithmic scaling to prevent domination by highly entangled nodes
        (entanglement_count / max_entanglements).ln_1p() * 0.1
    }
    
    /// Quantum measurement for selection with optimized probability computation
    pub async fn quantum_measure_selection_optimized(
        &self,
        scores: Vec<(String, f64)>,
    ) -> Result<String, CognitiveError> {
        if scores.is_empty() {
            return Err(CognitiveError::InvalidState(
                "No children to select during quantum measurement".to_string(),
            ));
        }

        // Handle infinite scores (unvisited nodes) with priority
        for (id, score) in &scores {
            if score.is_infinite() {
                return Ok(id.clone());
            }
        }

        // Vectorized probability computation for SIMD optimization
        let (ids, raw_scores): (Vec<String>, Vec<f64>) = scores.into_iter().unzip();
        
        // Find max score for numerical stability (prevents overflow in exp)
        let max_score = raw_scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        
        // Stable softmax computation with SIMD-friendly operations
        let exp_scores: Vec<f64> = raw_scores
            .iter()
            .map(|&score| (score - max_score).exp())
            .collect();
        
        let total_exp: f64 = exp_scores.iter().sum();
        
        if total_exp <= 0.0 || !total_exp.is_finite() {
            // Fallback to uniform selection if numerical issues occur
            let mut rng = rand::rng();
            let index = rng.random_range(0..ids.len());
            return Ok(ids[index].clone());
        }
        
        // Compute normalized probabilities
        let probabilities: Vec<f64> = exp_scores
            .iter()
            .map(|&exp_score| exp_score / total_exp)
            .collect();

        // Fast quantum measurement using cumulative distribution
        let mut rng = rand::rng();
        let measurement = rng.random_range(0.0..1.0);
        let mut cumulative = 0.0;

        for (i, &p) in probabilities.iter().enumerate() {
            cumulative += p;
            if measurement < cumulative {
                return Ok(ids[i].clone());
            }
        }

        // Fallback to last element (should rarely occur due to floating point precision)
        Ok(ids.last().unwrap().clone())
    }
    
    /// Quantum UCT selection enhanced with entanglement network effects
    pub async fn quantum_uct_select_with_entanglement(
        &self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
        entanglement_influence: f64,
    ) -> Result<String, CognitiveError> {
        let parent_visits = node.visits as f64;
        let parent_visits_ln = parent_visits.ln();
        
        let mut quantum_scores: Vec<(String, f64)> = Vec::with_capacity(node.children.len());

        for (action, child_id) in &node.children {
            let child = tree
                .get(child_id)
                .ok_or_else(|| CognitiveError::InvalidState("Child not found during entangled UCT".to_string()))?;

            let score = if child.visits == 0 {
                f64::INFINITY
            } else {
                // Standard UCT components
                let exploitation = child.quantum_reward.norm() / child.visits as f64;
                let exploration = self.config.quantum_exploration
                    * (parent_visits_ln / child.visits as f64).sqrt();
                
                // Quantum components
                let amplitude_bonus = child.amplitude.norm() * (1.0 - child.quantum_state.decoherence);
                
                // Entanglement network effects
                let entanglement_bonus = self.calculate_entanglement_network_bonus(
                    child, 
                    tree, 
                    entanglement_influence
                );
                
                exploitation + exploration + amplitude_bonus + entanglement_bonus
            };

            quantum_scores.push((child_id.clone(), score));
        }

        self.quantum_measure_selection_optimized(quantum_scores).await
    }
    
    /// Calculate entanglement network bonus considering all entangled nodes
    fn calculate_entanglement_network_bonus(
        &self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
        influence: f64,
    ) -> f64 {
        let mut total_bonus = 0.0;
        
        // Consider effects from all entangled nodes
        for entangled_id in &node.quantum_state.entanglements {
            if let Some(entangled_node) = tree.get(entangled_id) {
                if entangled_node.visits > 0 {
                    let entangled_reward = entangled_node.quantum_reward.norm() / entangled_node.visits as f64;
                    let entangled_coherence = 1.0 - entangled_node.quantum_state.decoherence;
                    
                    // Weighted contribution based on entanglement strength and coherence
                    total_bonus += entangled_reward * entangled_coherence * influence * 0.1;
                }
            }
        }
        
        // Normalize bonus to prevent domination
        total_bonus.tanh() * 0.2 // Bounded between -0.2 and 0.2
    }
    
    /// Multi-objective UCT selection with weighted components
    pub async fn multi_objective_uct_select(
        &self,
        node: &QuantumMCTSNode,
        tree: &HashMap<String, QuantumMCTSNode>,
        exploration_weight: f64,
        exploitation_weight: f64,
        quantum_weight: f64,
    ) -> Result<String, CognitiveError> {
        let parent_visits = node.visits as f64;
        let parent_visits_ln = parent_visits.ln();
        
        let mut quantum_scores: Vec<(String, f64)> = Vec::with_capacity(node.children.len());

        for (action, child_id) in &node.children {
            let child = tree
                .get(child_id)
                .ok_or_else(|| CognitiveError::InvalidState("Child not found during multi-objective UCT".to_string()))?;

            let score = if child.visits == 0 {
                f64::INFINITY // Always prioritize unvisited nodes
            } else {
                // Weighted multi-objective scoring
                let exploitation = (child.quantum_reward.norm() / child.visits as f64) * exploitation_weight;
                let exploration = (self.config.quantum_exploration * (parent_visits_ln / child.visits as f64).sqrt()) * exploration_weight;
                let quantum_bonus = (child.amplitude.norm() * (1.0 - child.quantum_state.decoherence)) * quantum_weight;
                
                exploitation + exploration + quantum_bonus
            };

            quantum_scores.push((child_id.clone(), score));
        }

        self.quantum_measure_selection_optimized(quantum_scores).await
    }
    
    /// Calculate selection score with all quantum factors
    pub fn calculate_selection_score(
        &self,
        child: &QuantumMCTSNode,
        parent_visits: f64,
        use_entanglement: bool,
        tree: Option<&HashMap<String, QuantumMCTSNode>>,
    ) -> f64 {
        if child.visits == 0 {
            return f64::INFINITY;
        }
        
        // Standard UCT components
        let exploitation = child.quantum_reward.norm() / child.visits as f64;
        let exploration = self.config.quantum_exploration
            * (parent_visits.ln() / child.visits as f64).sqrt();
        
        // Quantum components
        let quantum_bonus = self.calculate_quantum_bonus(child);
        
        // Optional entanglement network effects
        let entanglement_bonus = if use_entanglement && tree.is_some() {
            self.calculate_entanglement_network_bonus(child, tree.unwrap(), 0.5)
        } else {
            0.0
        };
        
        exploitation + exploration + quantum_bonus + entanglement_bonus
    }
    
    /// Fast scoring without entanglement effects for performance-critical scenarios
    #[inline]
    pub fn calculate_fast_score(&self, child: &QuantumMCTSNode, parent_visits: f64) -> f64 {
        if child.visits == 0 {
            return f64::INFINITY;
        }
        
        let exploitation = child.quantum_reward.norm() / child.visits as f64;
        let exploration = self.config.quantum_exploration
            * (parent_visits.ln() / child.visits as f64).sqrt();
        
        exploitation + exploration
    }
    
    /// Update configuration for dynamic parameter adjustment
    pub fn update_config(&mut self, new_config: QuantumMCTSConfig) {
        self.config = new_config;
    }
    
    /// Get current configuration
    pub fn config(&self) -> &QuantumMCTSConfig {
        &self.config
    }
    
    /// Calculate confidence in a selection score
    pub fn calculate_score_confidence(&self, score: f64, visits: u32) -> f64 {
        if score.is_infinite() {
            return 1.0; // Infinite score means high confidence (unvisited node)
        }
        
        // Confidence based on visits and score magnitude
        let visit_confidence = (visits as f64).sqrt() / (visits as f64 + 10.0);
        let score_confidence = score.tanh().abs(); // Normalize score to [0,1]
        
        (visit_confidence + score_confidence) / 2.0
    }
    
    /// Calculate selection entropy for diversity analysis
    pub fn calculate_selection_entropy(&self, scores: &[(String, f64)]) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }
        
        // Convert scores to probabilities using softmax
        let max_score = scores.iter().map(|(_, s)| *s).fold(f64::NEG_INFINITY, f64::max);
        let exp_scores: Vec<f64> = scores
            .iter()
            .map(|(_, s)| (s - max_score).exp())
            .collect();
        
        let total_exp: f64 = exp_scores.iter().sum();
        
        if total_exp <= 0.0 || !total_exp.is_finite() {
            return 0.0;
        }
        
        // Calculate entropy: -Σ(p * log(p))
        let mut entropy = 0.0;
        for exp_score in exp_scores {
            let p = exp_score / total_exp;
            if p > 0.0 {
                entropy -= p * p.ln();
            }
        }
        
        entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::mcts::CodeState;
    use super::super::node_state::{QuantumNodeState, QuantumMCTSNode};
    
    #[test]
    fn test_quantum_scorer_creation() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config.clone());
        assert_eq!(scorer.config().quantum_exploration, config.quantum_exploration);
    }
    
    #[test]
    fn test_quantum_bonus_calculation() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        let classical_state = CodeState {
            code: "test".to_string(),
            latency: 1.0,
            memory: 1.0,
            relevance: 1.0,
        };
        
        let quantum_state = QuantumNodeState::new(classical_state, 2);
        let node = QuantumMCTSNode::new(
            "test_node".to_string(),
            quantum_state,
            vec!["action1".to_string()],
            None,
            0,
        );
        
        let bonus = scorer.calculate_quantum_bonus(&node);
        assert!(bonus >= 0.0);
        assert!(bonus.is_finite());
    }
    
    #[tokio::test]
    async fn test_quantum_measurement_with_empty_scores() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        let empty_scores = Vec::new();
        let result = scorer.quantum_measure_selection_optimized(empty_scores).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_quantum_measurement_with_infinite_scores() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        let scores = vec![
            ("node1".to_string(), 1.0),
            ("node2".to_string(), f64::INFINITY),
            ("node3".to_string(), 2.0),
        ];
        
        let result = scorer.quantum_measure_selection_optimized(scores).await.unwrap();
        assert_eq!(result, "node2");
    }
    
    #[test]
    fn test_fast_score_calculation() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        let classical_state = CodeState {
            code: "test".to_string(),
            latency: 1.0,
            memory: 1.0,
            relevance: 1.0,
        };
        
        let quantum_state = QuantumNodeState::new(classical_state, 2);
        let mut node = QuantumMCTSNode::new(
            "test_node".to_string(),
            quantum_state,
            vec!["action1".to_string()],
            None,
            0,
        );
        
        // Test with zero visits (should return infinity)
        let score = scorer.calculate_fast_score(&node, 10.0);
        assert_eq!(score, f64::INFINITY);
        
        // Set visits and test normal scoring
        node.visits = 5;
        let score = scorer.calculate_fast_score(&node, 10.0);
        assert!(score.is_finite());
        assert!(score > 0.0);
    }
    
    #[test]
    fn test_score_confidence_calculation() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        // Test with infinite score
        let confidence = scorer.calculate_score_confidence(f64::INFINITY, 0);
        assert_eq!(confidence, 1.0);
        
        // Test with finite score
        let confidence = scorer.calculate_score_confidence(2.0, 10);
        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
    }
    
    #[test]
    fn test_selection_entropy_calculation() {
        let config = QuantumMCTSConfig::default();
        let scorer = QuantumScorer::new(config);
        
        // Test with empty scores
        let entropy = scorer.calculate_selection_entropy(&[]);
        assert_eq!(entropy, 0.0);
        
        // Test with uniform scores (maximum entropy)
        let uniform_scores = vec![
            ("node1".to_string(), 1.0),
            ("node2".to_string(), 1.0),
            ("node3".to_string(), 1.0),
        ];
        let entropy = scorer.calculate_selection_entropy(&uniform_scores);
        assert!(entropy > 0.0);
        
        // Test with single dominant score (minimum entropy)
        let skewed_scores = vec![
            ("node1".to_string(), 10.0),
            ("node2".to_string(), 0.1),
            ("node3".to_string(), 0.1),
        ];
        let entropy_skewed = scorer.calculate_selection_entropy(&skewed_scores);
        assert!(entropy_skewed < entropy); // Should be lower than uniform
    }
}
//! High-level quantum expansion engine
//!
//! This module provides the QuantumExpansionEngine interface with
//! metadata tracking and comprehensive expansion results.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cognitive::{
    committee::EvaluationCommittee,
    quantum::{Complex64, PhaseEvolution},
    types::{CognitiveError, OptimizationSpec},
};
use super::{
    core::QuantumExpander,
    metadata::{ExpansionResult, ExpansionStats},
};
use super::super::{
    node_state::QuantumMCTSNode,
    config::QuantumMCTSConfig,
};

/// High-level expansion interface
pub struct QuantumExpansionEngine {
    expander: QuantumExpander,
}

impl QuantumExpansionEngine {
    /// Create new expansion engine
    pub fn new(
        config: QuantumMCTSConfig,
        committee: Arc<EvaluationCommittee>,
        spec: Arc<OptimizationSpec>,
        user_objective: String,
        phase_evolution: Arc<PhaseEvolution>,
    ) -> Self {
        Self {
            expander: QuantumExpander::new(config, committee, spec, user_objective, phase_evolution),
        }
    }
    
    /// Expand node with full metadata
    pub async fn expand_with_metadata(
        &mut self,
        tree: &RwLock<HashMap<String, QuantumMCTSNode>>,
        node_id: &str,
    ) -> Result<ExpansionResult, CognitiveError> {
        // Get action before expansion
        let action = {
            let tree_read = tree.read().await;
            let node = tree_read.get(node_id)
                .ok_or_else(|| CognitiveError::InvalidState("Node not found for expansion".to_string()))?;
            
            if node.untried_actions.is_empty() {
                return Ok(ExpansionResult {
                    child_id: None,
                    action: "none".to_string(),
                    child_amplitude: Complex64::new(0.0, 0.0),
                    success: false,
                    error: Some("No untried actions available".to_string()),
                });
            }
            
            // Predict which action will be selected (simplified)
            node.untried_actions[0].clone()
        };
        
        match self.expander.quantum_expand(tree, node_id).await {
            Ok(Some(child_id)) => {
                // Get child amplitude
                let child_amplitude = {
                    let tree_read = tree.read().await;
                    tree_read.get(&child_id)
                        .map(|child| child.amplitude)
                        .unwrap_or_else(|| Complex64::new(0.0, 0.0))
                };
                
                Ok(ExpansionResult {
                    child_id: Some(child_id),
                    action,
                    child_amplitude,
                    success: true,
                    error: None,
                })
            }
            Ok(None) => Ok(ExpansionResult {
                child_id: None,
                action,
                child_amplitude: Complex64::new(0.0, 0.0),
                success: false,
                error: Some("No expansion possible".to_string()),
            }),
            Err(e) => Ok(ExpansionResult {
                child_id: None,
                action,
                child_amplitude: Complex64::new(0.0, 0.0),
                success: false,
                error: Some(e.to_string()),
            }),
        }
    }
    
    /// Get expansion statistics
    pub fn stats(&self) -> ExpansionStats {
        self.expander.expansion_stats()
    }
}
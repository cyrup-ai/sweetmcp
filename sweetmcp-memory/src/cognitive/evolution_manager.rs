// src/cognitive/evolution_manager.rs
//! Manages the evolution of solutions with progress tracking and prompt enhancement

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use crate::cognitive::committee::{ConsensusDecision, EvaluationCommittee};
use crate::cognitive::mcts::CodeState;
use crate::cognitive::types::{CognitiveError, OptimizationSpec};

/// Tracks evolution progress and manages prompt enhancement
pub struct EvolutionManager {
    current_best: Arc<RwLock<CodeState>>,
    failed_attempts: Arc<RwLock<Vec<FailedAttempt>>>,
    generation_count: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
struct FailedAttempt {
    action: String,
    reasons: Vec<String>,
    suggestions: Vec<String>,
}

impl EvolutionManager {
    pub fn new(initial_state: CodeState) -> Self {
        Self {
            current_best: Arc::new(RwLock::new(initial_state)),
            failed_attempts: Arc::new(RwLock::new(Vec::new())),
            generation_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Process committee decision and update best state if progress made
    pub async fn process_decision(
        &self,
        action: &str,
        candidate_state: CodeState,
        decision: &ConsensusDecision,
    ) -> Result<bool, CognitiveError> {
        if decision.makes_progress && decision.confidence > 0.5 {
            // Update best state
            *self.current_best.write().await = candidate_state;
            
            // Clear failed attempts on success
            self.failed_attempts.write().await.clear();
            
            info!(
                "Progress made with '{}': score={:.2}, confidence={:.2}",
                action, decision.overall_score, decision.confidence
            );
            
            Ok(true)
        } else {
            // Record failure
            let mut failed = self.failed_attempts.write().await;
            failed.push(FailedAttempt {
                action: action.to_string(),
                reasons: if decision.makes_progress {
                    vec![format!("Low confidence: {:.2}", decision.confidence)]
                } else {
                    decision.dissenting_opinions.clone()
                },
                suggestions: decision.improvement_suggestions.clone(),
            });
            
            warn!(
                "No progress with '{}': makes_progress={}, score={:.2}",
                action, decision.makes_progress, decision.overall_score
            );
            
            Ok(false)
        }
    }
    
    /// Get current best state
    pub async fn get_best_state(&self) -> CodeState {
        self.current_best.read().await.clone()
    }
    
    /// Generate enhanced prompt when all attempts fail
    pub async fn generate_enhanced_prompt(
        &self,
        base_prompt: &str,
        user_objective: &str,
    ) -> String {
        let failed = self.failed_attempts.read().await;
        let generation = *self.generation_count.read().await;
        
        if failed.is_empty() {
            return base_prompt.to_string();
        }
        
        let mut enhanced = format!(
            "{}\n\n[Generation {} Feedback]\n",
            base_prompt, generation + 1
        );
        
        enhanced.push_str("The committee rejected all proposed solutions:\n\n");
        
        // Summarize failures
        for attempt in failed.iter() {
            enhanced.push_str(&format!("❌ '{}' failed because:\n", attempt.action));
            for reason in &attempt.reasons {
                enhanced.push_str(&format!("   - {}\n", reason));
            }
        }
        
        enhanced.push_str("\n📋 Committee Suggestions:\n");
        let mut all_suggestions: Vec<_> = failed.iter()
            .flat_map(|a| a.suggestions.iter())
            .cloned()
            .collect();
        all_suggestions.sort();
        all_suggestions.dedup();
        
        for suggestion in all_suggestions.iter().take(5) {
            enhanced.push_str(&format!("   - {}\n", suggestion));
        }
        
        enhanced.push_str(&format!(
            "\n🎯 Remember the objective: {}\n\n\
            Generate NEW approaches that:\n\
            1. Address the specific failure reasons above\n\
            2. Focus on incremental progress toward the objective\n\
            3. Start simple before attempting complex solutions\n\
            4. Include verification/measurement in the approach\n",
            user_objective
        ));
        
        // Increment generation counter
        *self.generation_count.write().await += 1;
        
        enhanced
    }
    
    /// Check if we should try a new generation
    pub async fn should_try_new_generation(&self, max_failures: usize) -> bool {
        self.failed_attempts.read().await.len() >= max_failures
    }
    
    /// Reset for new generation
    pub async fn reset_for_new_generation(&self) {
        self.failed_attempts.write().await.clear();
    }
}
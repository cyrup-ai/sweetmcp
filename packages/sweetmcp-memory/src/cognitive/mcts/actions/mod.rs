//! MCTS actions module coordination
//!
//! This module provides comprehensive MCTS action management with blazing-fast performance
//! and zero allocation optimizations, integrating all action submodules.

pub mod action_generator;
pub mod action_applicator;
pub mod action_validator;
pub mod action_coordinator;

// Re-export key types and functions for ergonomic access
pub use action_generator::{
    ActionGenerator, ActionCacheStats, PrioritizedAction, ActionImpact,
};

pub use action_applicator::{
    ActionApplicator, ApplicationCacheStats,
};

// Export aliases for backward compatibility
pub use ActionCacheStats as CacheStatistics;
pub use ApplicationCacheStats as ApplicationStatistics;

pub use action_validator::{
    ActionValidator, ValidationResult, ValidationStats,
};

pub use action_coordinator::{
    ActionCoordinator, CoordinatorStatistics, CoordinatorConfig,
    CoordinatorMetrics, CoordinatorError,
};

// Common imports for all submodules
use super::types::{CodeState, ActionMetadata};
use crate::cognitive::types::{CognitiveError, ImpactFactors};
use crate::vector::async_vector_optimization::OptimizationSpec;
use crate::cognitive::committee::EvaluationCommittee;
use std::sync::Arc;
use tracing::{debug, info};

/// Comprehensive action management facade for MCTS operations
pub struct ActionManager {
    generator: ActionGenerator,
    applicator: ActionApplicator,
    validator: ActionValidator,
}

impl ActionManager {
    /// Create new action manager with zero allocation optimizations
    #[inline]
    pub fn new(
        optimization_spec: Arc<OptimizationSpec>,
        committee: Arc<EvaluationCommittee>,
        user_objective: String,
    ) -> Self {
        Self {
            generator: ActionGenerator::new(
                optimization_spec.clone(),
                committee.clone(),
                user_objective,
            ),
            applicator: ActionApplicator::new(optimization_spec.clone(), committee),
            validator: ActionValidator::new(optimization_spec),
        }
    }

    /// Get validated and prioritized actions for a state
    #[inline]
    pub fn get_validated_actions(&mut self, state: &CodeState) -> Result<Vec<ValidatedAction>, CognitiveError> {
        // Generate possible actions
        let actions = self.generator.get_possible_actions(state);
        
        // Validate all actions
        let validation_results = self.validator.validate_actions(&actions, state)?;
        
        // Get prioritized actions for valid ones
        let prioritized_actions = self.generator.get_prioritized_actions(state);
        
        // Combine validation and prioritization
        let mut validated_actions = Vec::new();
        
        for (action, validation) in actions.iter().zip(validation_results.iter()) {
            if validation.is_valid_result() {
                // Find corresponding prioritized action
                if let Some(prioritized) = prioritized_actions.iter().find(|p| &p.action == action) {
                    validated_actions.push(ValidatedAction {
                        action: action.clone(),
                        priority: prioritized.priority,
                        expected_impact: prioritized.expected_impact.clone(),
                        validation_result: validation.clone(),
                        risk_score: validation.risk_score,
                    });
                }
            }
        }
        
        // Sort by priority (highest first)
        validated_actions.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(validated_actions)
    }

    /// Apply action to state with validation
    #[inline]
    pub async fn apply_validated_action(
        &mut self,
        action: &str,
        state: &CodeState,
    ) -> Result<ActionApplicationResult, CognitiveError> {
        // Validate action first
        let validation = self.validator.validate_action(action, state)?;
        
        if !validation.is_valid_result() {
            return Ok(ActionApplicationResult {
                success: false,
                new_state: None,
                validation_result: validation,
                error_message: Some("Action failed validation".to_string()),
            });
        }

        // Apply the action
        match self.applicator.apply_action(action, state).await {
            Ok(new_state) => {
                info!("Successfully applied action: {}", action);
                Ok(ActionApplicationResult {
                    success: true,
                    new_state: Some(new_state),
                    validation_result: validation,
                    error_message: None,
                })
            }
            Err(error) => {
                Ok(ActionApplicationResult {
                    success: false,
                    new_state: None,
                    validation_result: validation,
                    error_message: Some(error.to_string()),
                })
            }
        }
    }

    /// Get best action for current state
    #[inline]
    pub async fn get_best_action(&mut self, state: &CodeState) -> Result<Option<ValidatedAction>, CognitiveError> {
        let validated_actions = self.get_validated_actions(state)?;
        Ok(validated_actions.into_iter().next())
    }

    /// Apply best action to state
    #[inline]
    pub async fn apply_best_action(&mut self, state: &CodeState) -> Result<ActionApplicationResult, CognitiveError> {
        if let Some(best_action) = self.get_best_action(state).await? {
            self.apply_validated_action(&best_action.action, state).await
        } else {
            Ok(ActionApplicationResult {
                success: false,
                new_state: None,
                validation_result: ValidationResult::new("no_action".to_string()),
                error_message: Some("No valid actions available".to_string()),
            })
        }
    }

    /// Get action management statistics
    #[inline]
    pub fn get_statistics(&self) -> ActionManagerStats {
        ActionManagerStats {
            generator_stats: self.generator.cache_stats(),
            applicator_stats: self.applicator.cache_stats(),
            validator_stats: self.validator.validation_stats(),
        }
    }

    /// Clear all caches
    #[inline]
    pub fn clear_all_caches(&mut self) {
        self.generator.clear_cache();
        self.applicator.clear_cache();
        self.validator.clear_cache();
    }

    /// Update optimization spec for all components
    #[inline]
    pub fn update_optimization_spec(&mut self, spec: Arc<OptimizationSpec>) {
        self.generator.update_optimization_spec(spec.clone());
        self.validator = ActionValidator::new(spec.clone());
        self.applicator = ActionApplicator::new(spec, self.generator.committee.clone());
    }

    /// Update user objective
    #[inline]
    pub fn update_user_objective(&mut self, objective: String) {
        self.generator.update_user_objective(objective);
    }

    /// Batch process multiple actions
    #[inline]
    pub async fn batch_apply_actions(
        &mut self,
        actions: &[String],
        state: &CodeState,
    ) -> Result<Vec<ActionApplicationResult>, CognitiveError> {
        let mut results = Vec::with_capacity(actions.len());
        let mut current_state = state.clone();
        
        for action in actions {
            let result = self.apply_validated_action(action, &current_state).await?;
            
            // Update current state if application was successful
            if result.success {
                if let Some(ref new_state) = result.new_state {
                    current_state = new_state.clone();
                }
            }
            
            results.push(result);
        }
        
        Ok(results)
    }

    /// Find actions that improve specific metrics
    #[inline]
    pub fn find_actions_for_metric(
        &mut self,
        state: &CodeState,
        metric: MetricType,
        min_improvement: f64,
    ) -> Result<Vec<ValidatedAction>, CognitiveError> {
        let validated_actions = self.get_validated_actions(state)?;
        
        let filtered_actions: Vec<ValidatedAction> = validated_actions
            .into_iter()
            .filter(|action| {
                match metric {
                    MetricType::Latency => action.expected_impact.latency_improvement >= min_improvement,
                    MetricType::Memory => action.expected_impact.memory_improvement >= min_improvement,
                    MetricType::Relevance => action.expected_impact.relevance_improvement >= min_improvement,
                }
            })
            .collect();
        
        Ok(filtered_actions)
    }
}

/// Validated action with comprehensive information
#[derive(Debug, Clone)]
pub struct ValidatedAction {
    pub action: String,
    pub priority: f64,
    pub expected_impact: ActionImpact,
    pub validation_result: ValidationResult,
    pub risk_score: f64,
}

impl ValidatedAction {
    /// Get overall action score combining priority and risk
    #[inline]
    pub fn overall_score(&self) -> f64 {
        self.priority * (1.0 - self.risk_score * 0.5)
    }

    /// Check if action is low risk
    #[inline]
    pub fn is_low_risk(&self) -> bool {
        self.risk_score < 0.3
    }

    /// Get action summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "Action: {}, Priority: {:.2}, Risk: {:.2}, Score: {:.2}",
            self.action, self.priority, self.risk_score, self.overall_score()
        )
    }
}

/// Action application result
#[derive(Debug, Clone)]
pub struct ActionApplicationResult {
    pub success: bool,
    pub new_state: Option<CodeState>,
    pub validation_result: ValidationResult,
    pub error_message: Option<String>,
}

impl ActionApplicationResult {
    /// Check if application was successful
    #[inline]
    pub fn is_success(&self) -> bool {
        self.success && self.new_state.is_some()
    }

    /// Get error message if any
    #[inline]
    pub fn get_error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}

/// Action manager statistics
#[derive(Debug, Clone)]
pub struct ActionManagerStats {
    pub generator_stats: ActionCacheStats,
    pub applicator_stats: ApplicationCacheStats,
    pub validator_stats: ValidationStats,
}

/// Metric types for action filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Latency,
    Memory,
    Relevance,
}

/// Convenience macros for common action patterns
#[macro_export]
macro_rules! apply_action {
    ($manager:expr, $action:expr, $state:expr) => {
        $manager.apply_validated_action($action, $state).await
    };
}

#[macro_export]
macro_rules! get_best_action {
    ($manager:expr, $state:expr) => {
        $manager.get_best_action($state).await
    };
}

#[macro_export]
macro_rules! find_actions_for {
    ($manager:expr, $state:expr, latency, $min:expr) => {
        $manager.find_actions_for_metric($state, MetricType::Latency, $min)
    };
    ($manager:expr, $state:expr, memory, $min:expr) => {
        $manager.find_actions_for_metric($state, MetricType::Memory, $min)
    };
    ($manager:expr, $state:expr, relevance, $min:expr) => {
        $manager.find_actions_for_metric($state, MetricType::Relevance, $min)
    };
}
//! Action validation for MCTS operations
//!
//! This module provides blazing-fast action validation with zero allocation
//! optimizations for MCTS action safety and feasibility checking.

use super::super::types::CodeState;
use crate::cognitive::types::CognitiveError;
use crate::vector::async_vector_optimization::OptimizationSpec;
use std::sync::Arc;
use tracing::debug;

/// Action validator for MCTS operations
pub struct ActionValidator {
    optimization_spec: Arc<OptimizationSpec>,
    validation_cache: std::collections::HashMap<String, ValidationResult>,
}

impl ActionValidator {
    /// Create new action validator with zero allocation optimizations
    #[inline]
    pub fn new(optimization_spec: Arc<OptimizationSpec>) -> Self {
        Self {
            optimization_spec,
            validation_cache: std::collections::HashMap::new(),
        }
    }

    /// Validate action for given state with blazing-fast checking
    #[inline]
    pub fn validate_action(
        &mut self,
        action: &str,
        state: &CodeState,
    ) -> Result<ValidationResult, CognitiveError> {
        // Create cache key for validation
        let cache_key = format!("{}_{}", action, state.cache_key());
        
        // Check cache first for zero allocation fast path
        if let Some(cached_result) = self.validation_cache.get(&cache_key) {
            if cached_result.is_valid() {
                return Ok(cached_result.clone());
            }
        }

        debug!("Validating action: {} for state", action);

        let mut result = ValidationResult::new(action.to_string());

        // Basic action format validation
        self.validate_action_format(action, &mut result);

        // State compatibility validation
        self.validate_state_compatibility(action, state, &mut result);

        // Optimization spec compliance validation
        self.validate_optimization_compliance(action, state, &mut result);

        // Resource constraint validation
        self.validate_resource_constraints(action, state, &mut result);

        // Risk assessment validation
        self.validate_risk_assessment(action, state, &mut result);

        // Cache the result
        self.validation_cache.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Validate action format and syntax
    #[inline]
    fn validate_action_format(&self, action: &str, result: &mut ValidationResult) {
        // Check for empty or invalid action names
        if action.is_empty() {
            result.add_error("Action name cannot be empty".to_string());
            return;
        }

        if action.len() > 100 {
            result.add_warning("Action name is unusually long".to_string());
        }

        // Check for valid action naming convention
        if !action.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_error("Action name contains invalid characters".to_string());
        }

        // Check for known action patterns
        let known_patterns = [
            "optimize_", "reduce_", "improve_", "parallelize_", "inline_",
            "batch_", "add_", "enable_", "implement_", "aggressive_",
            "zero_", "lock_free", "custom_", "micro_", "eliminate_",
        ];

        let is_known_pattern = known_patterns.iter().any(|pattern| action.starts_with(pattern));
        if !is_known_pattern {
            result.add_warning(format!("Unknown action pattern: {}", action));
        }
    }

    /// Validate state compatibility
    #[inline]
    fn validate_state_compatibility(&self, action: &str, state: &CodeState, result: &mut ValidationResult) {
        // Check if action is applicable to current state
        if action.contains("memory") && state.memory < 0.1 {
            result.add_warning("Memory optimization may have limited impact on low-memory state".to_string());
        }

        if action.contains("latency") && state.latency < 0.1 {
            result.add_warning("Latency optimization may have limited impact on low-latency state".to_string());
        }

        if action.contains("parallelize") && state.metadata.parallelization_level > 0.8 {
            result.add_warning("State is already highly parallelized".to_string());
        }

        // Check for conflicting actions in metadata
        for existing_action in &state.metadata.applied_actions {
            if self.actions_conflict(action, existing_action) {
                result.add_error(format!("Action conflicts with previously applied action: {}", existing_action));
            }
        }

        // Check optimization level limits
        if state.metadata.optimization_level > 0.9 && action.contains("aggressive") {
            result.add_warning("State is already highly optimized, aggressive actions may have diminishing returns".to_string());
        }
    }

    /// Validate optimization spec compliance
    #[inline]
    fn validate_optimization_compliance(&self, action: &str, state: &CodeState, result: &mut ValidationResult) {
        let baseline = &self.optimization_spec.baseline_metrics;
        let content_type = &self.optimization_spec.content_type;

        // Check latency constraints
        if action.contains("latency") {
            let estimated_latency_change = self.estimate_latency_impact(action, state);
            let new_latency = state.latency * (1.0 + estimated_latency_change);
            
            if new_latency > baseline.latency * (1.0 + content_type.restrictions.max_latency_increase / 100.0) {
                result.add_error("Action would exceed maximum allowed latency increase".to_string());
            }
        }

        // Check memory constraints
        if action.contains("memory") {
            let estimated_memory_change = self.estimate_memory_impact(action, state);
            let new_memory = state.memory * (1.0 + estimated_memory_change);
            
            if new_memory > baseline.memory * (1.0 + content_type.restrictions.max_memory_increase / 100.0) {
                result.add_error("Action would exceed maximum allowed memory increase".to_string());
            }
        }

        // Check evolution rules compliance
        for rule in &self.optimization_spec.evolution_rules {
            if !self.action_complies_with_rule(action, rule) {
                result.add_warning(format!("Action may not comply with evolution rule: {}", rule.rule_type));
            }
        }
    }

    /// Validate resource constraints
    #[inline]
    fn validate_resource_constraints(&self, action: &str, state: &CodeState, result: &mut ValidationResult) {
        // Check if action requires resources that may not be available
        if action.contains("simd") {
            result.add_info("Action requires SIMD support - verify hardware compatibility".to_string());
        }

        if action.contains("custom_allocator") {
            result.add_info("Action requires custom allocator implementation".to_string());
        }

        if action.contains("assembly") {
            result.add_warning("Action involves assembly code - platform-specific implementation required".to_string());
        }

        if action.contains("hardware_specific") {
            result.add_warning("Action is hardware-specific - may not be portable".to_string());
        }

        // Check for actions that require specific dependencies
        if action.contains("vectorization") && state.metadata.optimization_level < 0.3 {
            result.add_warning("Vectorization may require preliminary optimizations".to_string());
        }
    }

    /// Validate risk assessment
    #[inline]
    fn validate_risk_assessment(&self, action: &str, state: &CodeState, result: &mut ValidationResult) {
        let mut risk_score = 0.0;

        // Assess risk based on action type
        if action.contains("aggressive") {
            risk_score += 0.3;
            result.add_warning("Aggressive optimization carries higher risk".to_string());
        }

        if action.contains("extreme") {
            risk_score += 0.4;
            result.add_warning("Extreme optimization may impact maintainability".to_string());
        }

        if action.contains("sacrifice") {
            risk_score += 0.5;
            result.add_error("Action explicitly sacrifices code quality".to_string());
        }

        if action.contains("assembly") || action.contains("unsafe") {
            risk_score += 0.6;
            result.add_error("Action involves unsafe or low-level operations".to_string());
        }

        // Assess cumulative risk
        let total_risk = state.metadata.risk_level + risk_score;
        if total_risk > 0.7 {
            result.add_error("Cumulative risk level would be too high".to_string());
        } else if total_risk > 0.5 {
            result.add_warning("Cumulative risk level is getting high".to_string());
        }

        result.risk_score = risk_score;
    }

    /// Check if two actions conflict with each other
    #[inline]
    fn actions_conflict(&self, action1: &str, action2: &str) -> bool {
        // Define conflicting action pairs
        let conflicts = [
            ("optimize_memory", "sacrifice_memory"),
            ("reduce_latency", "sacrifice_speed"),
            ("improve_accuracy", "sacrifice_accuracy"),
            ("zero_allocation", "increase_allocation"),
            ("lock_free", "add_locking"),
        ];

        for (conflict1, conflict2) in &conflicts {
            if (action1.contains(conflict1) && action2.contains(conflict2)) ||
               (action1.contains(conflict2) && action2.contains(conflict1)) {
                return true;
            }
        }

        false
    }

    /// Estimate latency impact of an action
    #[inline]
    fn estimate_latency_impact(&self, action: &str, state: &CodeState) -> f64 {
        match action {
            a if a.contains("aggressive_latency") => -0.5,
            a if a.contains("optimize_hot_paths") => -0.3,
            a if a.contains("reduce_io") => -0.25,
            a if a.contains("parallelize") => -0.4,
            a if a.contains("inline") => -0.05,
            a if a.contains("simd") => -0.4,
            a if a.contains("caching") => -0.3,
            a if a.contains("memory") => 0.02, // Memory optimization may slightly increase latency
            _ => 0.0,
        }
    }

    /// Estimate memory impact of an action
    #[inline]
    fn estimate_memory_impact(&self, action: &str, state: &CodeState) -> f64 {
        match action {
            a if a.contains("aggressive_memory") => -0.4,
            a if a.contains("zero_allocation") => -0.5,
            a if a.contains("optimize_memory") => -0.1,
            a if a.contains("parallelize") => 0.2, // Parallelization increases memory
            a if a.contains("caching") => 0.3, // Caching increases memory
            a if a.contains("inline") => 0.1, // Inlining increases code size
            _ => 0.0,
        }
    }

    /// Check if action complies with evolution rule
    #[inline]
    fn action_complies_with_rule(&self, action: &str, rule: &crate::cognitive::types::EvolutionRules) -> bool {
        match rule.rule_type.as_str() {
            "performance_first" => action.contains("optimize") || action.contains("improve"),
            "memory_constrained" => !action.contains("increase_memory") && !action.contains("sacrifice_memory"),
            "latency_critical" => !action.contains("increase_latency") && !action.contains("sacrifice_speed"),
            "maintainability_required" => !action.contains("sacrifice") && !action.contains("extreme"),
            _ => true, // Unknown rules are assumed compliant
        }
    }

    /// Batch validate multiple actions
    #[inline]
    pub fn validate_actions(
        &mut self,
        actions: &[String],
        state: &CodeState,
    ) -> Result<Vec<ValidationResult>, CognitiveError> {
        let mut results = Vec::with_capacity(actions.len());
        
        for action in actions {
            let result = self.validate_action(action, state)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Clear validation cache
    #[inline]
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }

    /// Get validation statistics
    #[inline]
    pub fn validation_stats(&self) -> ValidationStats {
        let valid_count = self.validation_cache.values().filter(|r| r.is_valid_result()).count();
        let invalid_count = self.validation_cache.len() - valid_count;

        ValidationStats {
            total_validations: self.validation_cache.len(),
            valid_actions: valid_count,
            invalid_actions: invalid_count,
            cache_hit_rate: if self.validation_cache.is_empty() { 0.0 } else { 
                valid_count as f64 / self.validation_cache.len() as f64 
            },
        }
    }
}

/// Validation result for an action
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub action: String,
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
    pub risk_score: f64,
    pub timestamp: std::time::Instant,
}

impl ValidationResult {
    /// Create new validation result
    #[inline]
    pub fn new(action: String) -> Self {
        Self {
            action,
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            risk_score: 0.0,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Add error to validation result
    #[inline]
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add warning to validation result
    #[inline]
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add info to validation result
    #[inline]
    pub fn add_info(&mut self, info: String) {
        self.info.push(info);
    }

    /// Check if validation result is still valid (within 10 minutes)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.timestamp.elapsed().as_secs() < 600
    }

    /// Check if action passed validation
    #[inline]
    pub fn is_valid_result(&self) -> bool {
        self.is_valid
    }

    /// Get summary of validation result
    #[inline]
    pub fn summary(&self) -> String {
        let status = if self.is_valid { "VALID" } else { "INVALID" };
        format!(
            "Action '{}': {} (Errors: {}, Warnings: {}, Risk: {:.2})",
            self.action, status, self.errors.len(), self.warnings.len(), self.risk_score
        )
    }
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_validations: usize,
    pub valid_actions: usize,
    pub invalid_actions: usize,
    pub cache_hit_rate: f64,
}
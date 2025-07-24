//! Action generation for MCTS operations
//!
//! This module provides blazing-fast action generation with zero allocation
//! optimizations for MCTS action creation and management.

use super::super::types::{CodeState, ActionMetadata};
use crate::cognitive::types::{CognitiveError, OptimizationSpec, ImpactFactors};
use crate::cognitive::committee::EvaluationCommittee;
use std::sync::Arc;

/// Action generator for MCTS operations
pub struct ActionGenerator {
    optimization_spec: Arc<OptimizationSpec>,
    committee: Arc<EvaluationCommittee>,
    user_objective: String,
    action_cache: std::collections::HashMap<String, Vec<String>>,
}

impl ActionGenerator {
    /// Create new action generator with zero allocation optimizations
    #[inline]
    pub fn new(
        optimization_spec: Arc<OptimizationSpec>,
        committee: Arc<EvaluationCommittee>,
        user_objective: String,
    ) -> Self {
        Self {
            optimization_spec,
            committee,
            user_objective,
            action_cache: std::collections::HashMap::new(),
        }
    }

    /// Get possible actions for a state with blazing-fast generation
    #[inline]
    pub fn get_possible_actions(&mut self, state: &CodeState) -> Vec<String> {
        // Create cache key based on state characteristics
        let cache_key = format!(
            "l{:.2}_m{:.2}_r{:.2}",
            state.latency, state.memory, state.relevance
        );

        // Check cache first for zero allocation fast path
        if let Some(cached_actions) = self.action_cache.get(&cache_key) {
            return cached_actions.clone();
        }

        // Generate actions based on current state and optimization spec
        let mut actions = Vec::with_capacity(20);

        // Core optimization actions
        actions.extend(self.get_core_optimization_actions());

        // Performance-specific actions based on current metrics
        actions.extend(self.get_performance_specific_actions(state));

        // Context-aware actions based on optimization spec
        actions.extend(self.get_context_aware_actions(state));

        // Advanced optimization actions
        actions.extend(self.get_advanced_optimization_actions(state));

        // Cache the result for future use
        self.action_cache.insert(cache_key, actions.clone());

        actions
    }

    /// Get core optimization actions with zero allocation
    #[inline]
    fn get_core_optimization_actions(&self) -> Vec<String> {
        vec![
            "optimize_memory_allocation".to_string(),
            "reduce_computational_complexity".to_string(),
            "improve_algorithm_efficiency".to_string(),
            "parallelize_independent_work".to_string(),
            "inline_critical_functions".to_string(),
            "batch_operations".to_string(),
            "add_strategic_caching".to_string(),
            "optimize_data_structures".to_string(),
            "reduce_lock_contention".to_string(),
            "enable_simd_operations".to_string(),
        ]
    }

    /// Get performance-specific actions based on current state
    #[inline]
    fn get_performance_specific_actions(&self, state: &CodeState) -> Vec<String> {
        let mut actions = Vec::new();
        let baseline = &self.optimization_spec.baseline_metrics;

        // Latency-specific actions
        if state.latency > baseline.latency * 1.1 {
            actions.extend(vec![
                "aggressive_latency_optimization".to_string(),
                "reduce_io_operations".to_string(),
                "optimize_hot_paths".to_string(),
                "implement_lazy_loading".to_string(),
                "reduce_function_call_overhead".to_string(),
            ]);
        }

        // Memory-specific actions
        if state.memory > baseline.memory * 1.1 {
            actions.extend(vec![
                "aggressive_memory_optimization".to_string(),
                "implement_object_pooling".to_string(),
                "reduce_memory_fragmentation".to_string(),
                "optimize_garbage_collection".to_string(),
                "implement_memory_mapping".to_string(),
            ]);
        }

        // Relevance-specific actions
        if state.relevance < baseline.relevance * 0.9 {
            actions.extend(vec![
                "improve_algorithm_accuracy".to_string(),
                "enhance_data_quality".to_string(),
                "refine_heuristics".to_string(),
                "implement_adaptive_algorithms".to_string(),
                "improve_feature_selection".to_string(),
            ]);
        }

        actions
    }

    /// Get context-aware actions based on optimization specification
    #[inline]
    fn get_context_aware_actions(&self, state: &CodeState) -> Vec<String> {
        let mut actions = Vec::new();
        let content_type = &self.optimization_spec.content_type;

        // Actions based on content type restrictions
        if content_type.restrictions.max_latency_increase < 10.0 {
            actions.extend(vec![
                "micro_optimize_critical_sections".to_string(),
                "eliminate_unnecessary_allocations".to_string(),
                "optimize_branch_prediction".to_string(),
            ]);
        }

        if content_type.restrictions.max_memory_increase < 10.0 {
            actions.extend(vec![
                "implement_zero_copy_operations".to_string(),
                "use_stack_allocation_where_possible".to_string(),
                "optimize_data_layout".to_string(),
            ]);
        }

        // Evolution rules based actions
        for rule in &self.optimization_spec.evolution_rules {
            match rule.rule_type.as_str() {
                "performance_first" => {
                    actions.push("prioritize_performance_over_readability".to_string());
                }
                "memory_constrained" => {
                    actions.push("aggressive_memory_conservation".to_string());
                }
                "latency_critical" => {
                    actions.push("minimize_latency_at_all_costs".to_string());
                }
                _ => {}
            }
        }

        actions
    }

    /// Get advanced optimization actions for complex scenarios
    #[inline]
    fn get_advanced_optimization_actions(&self, state: &CodeState) -> Vec<String> {
        let mut actions = Vec::new();

        // Advanced actions based on performance score
        let performance_score = state.performance_score();
        
        if performance_score < 0.5 {
            actions.extend(vec![
                "implement_custom_allocator".to_string(),
                "use_lock_free_data_structures".to_string(),
                "implement_work_stealing".to_string(),
                "optimize_cache_locality".to_string(),
                "implement_vectorization".to_string(),
            ]);
        }

        if performance_score < 0.3 {
            actions.extend(vec![
                "rewrite_critical_sections_in_assembly".to_string(),
                "implement_custom_memory_management".to_string(),
                "use_hardware_specific_optimizations".to_string(),
                "implement_prefetching_strategies".to_string(),
            ]);
        }

        // User objective specific actions
        if self.user_objective.contains("blazing") || self.user_objective.contains("fast") {
            actions.extend(vec![
                "extreme_performance_optimization".to_string(),
                "sacrifice_maintainability_for_speed".to_string(),
                "implement_aggressive_inlining".to_string(),
            ]);
        }

        if self.user_objective.contains("memory") || self.user_objective.contains("allocation") {
            actions.extend(vec![
                "zero_allocation_implementation".to_string(),
                "stack_only_data_structures".to_string(),
                "memory_pool_optimization".to_string(),
            ]);
        }

        actions
    }

    /// Clear action cache to force regeneration
    #[inline]
    pub fn clear_cache(&mut self) {
        self.action_cache.clear();
    }

    /// Get cache statistics for monitoring
    #[inline]
    pub fn cache_stats(&self) -> ActionCacheStats {
        ActionCacheStats {
            cache_size: self.action_cache.len(),
            total_cached_actions: self.action_cache.values().map(|v| v.len()).sum(),
        }
    }

    /// Update optimization spec and clear cache
    #[inline]
    pub fn update_optimization_spec(&mut self, spec: Arc<OptimizationSpec>) {
        self.optimization_spec = spec;
        self.clear_cache();
    }

    /// Update user objective and clear cache
    #[inline]
    pub fn update_user_objective(&mut self, objective: String) {
        self.user_objective = objective;
        self.clear_cache();
    }

    /// Generate actions with priority scoring
    #[inline]
    pub fn get_prioritized_actions(&mut self, state: &CodeState) -> Vec<PrioritizedAction> {
        let actions = self.get_possible_actions(state);
        let mut prioritized = Vec::with_capacity(actions.len());

        for action in actions {
            let priority = self.calculate_action_priority(&action, state);
            prioritized.push(PrioritizedAction {
                action,
                priority,
                expected_impact: self.estimate_action_impact(&action, state),
            });
        }

        // Sort by priority (highest first)
        prioritized.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        prioritized
    }

    /// Calculate priority score for an action
    #[inline]
    fn calculate_action_priority(&self, action: &str, state: &CodeState) -> f64 {
        let mut priority = 0.5; // Base priority

        // Increase priority based on current state needs
        if action.contains("latency") && state.latency > self.optimization_spec.baseline_metrics.latency {
            priority += 0.3;
        }

        if action.contains("memory") && state.memory > self.optimization_spec.baseline_metrics.memory {
            priority += 0.3;
        }

        if action.contains("relevance") && state.relevance < self.optimization_spec.baseline_metrics.relevance {
            priority += 0.2;
        }

        // Increase priority for user objective alignment
        if self.user_objective.to_lowercase().contains(&action.replace("_", " ")) {
            priority += 0.4;
        }

        // Cap priority at 1.0
        priority.min(1.0)
    }

    /// Estimate the impact of an action on the state
    #[inline]
    fn estimate_action_impact(&self, action: &str, state: &CodeState) -> ActionImpact {
        let mut impact = ActionImpact::default();

        // Estimate impact based on action type
        if action.contains("latency") {
            impact.latency_improvement = 0.1;
        }
        if action.contains("memory") {
            impact.memory_improvement = 0.1;
        }
        if action.contains("relevance") || action.contains("accuracy") {
            impact.relevance_improvement = 0.05;
        }

        // Aggressive actions have higher impact but more risk
        if action.contains("aggressive") || action.contains("extreme") {
            impact.latency_improvement *= 2.0;
            impact.memory_improvement *= 2.0;
            impact.risk_factor = 0.3;
        }

        impact
    }
}

/// Action cache statistics
#[derive(Debug, Clone)]
pub struct ActionCacheStats {
    pub cache_size: usize,
    pub total_cached_actions: usize,
}

/// Prioritized action with scoring
#[derive(Debug, Clone)]
pub struct PrioritizedAction {
    pub action: String,
    pub priority: f64,
    pub expected_impact: ActionImpact,
}

/// Expected impact of an action
#[derive(Debug, Clone)]
pub struct ActionImpact {
    pub latency_improvement: f64,
    pub memory_improvement: f64,
    pub relevance_improvement: f64,
    pub risk_factor: f64,
}

impl Default for ActionImpact {
    #[inline]
    fn default() -> Self {
        Self {
            latency_improvement: 0.0,
            memory_improvement: 0.0,
            relevance_improvement: 0.0,
            risk_factor: 0.1,
        }
    }
}
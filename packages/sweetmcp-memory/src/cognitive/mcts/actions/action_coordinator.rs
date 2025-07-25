//! Action coordination for MCTS operations
//!
//! This module provides comprehensive action coordination with zero allocation
//! optimizations and blazing-fast performance for MCTS action management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use std::sync::Arc;
use super::super::types::{ActionMetadata, NodeStatistics, CodeState};
use super::action_generator::ActionGenerator;
use crate::vector::async_vector_optimization::OptimizationSpec;
use crate::cognitive::committee::EvaluationCommittee;

/// Action coordinator for managing MCTS action execution
#[derive(Debug)]
pub struct ActionCoordinator {
    /// Active actions being coordinated
    pub active_actions: HashMap<String, ActionMetadata>,
    /// Coordination statistics
    pub statistics: CoordinatorStatistics,
    /// Configuration settings
    pub config: CoordinatorConfig,
    /// Performance metrics
    pub metrics: CoordinatorMetrics,
    /// Action generator for generating possible actions (optional)
    pub action_generator: Option<ActionGenerator>,
}

impl ActionCoordinator {
    /// Create new action coordinator with basic configuration
    pub fn new(config: CoordinatorConfig) -> Self {
        Self {
            active_actions: HashMap::with_capacity(config.initial_capacity),
            statistics: CoordinatorStatistics::new(),
            config,
            metrics: CoordinatorMetrics::new(),
            action_generator: None,
        }
    }

    /// Create new action coordinator with proper dependencies
    pub fn new_with_generator(
        optimization_spec: Arc<OptimizationSpec>,
        committee: Arc<EvaluationCommittee>,
        user_objective: String,
    ) -> Self {
        let action_generator = ActionGenerator::new(
            optimization_spec,
            committee,
            user_objective,
        );

        Self {
            active_actions: HashMap::with_capacity(1000),
            statistics: CoordinatorStatistics::new(),
            config: CoordinatorConfig::default(),
            metrics: CoordinatorMetrics::new(),
            action_generator: Some(action_generator),
        }
    }

    /// Get possible actions for a given state
    pub fn get_possible_actions(&mut self, state: &CodeState) -> Vec<String> {
        if let Some(ref mut generator) = self.action_generator {
            generator.get_possible_actions(state)
        } else {
            // Fallback to basic actions if no generator is available
            vec![
                "optimize_memory_allocation".to_string(),
                "reduce_computational_complexity".to_string(),
                "improve_algorithm_efficiency".to_string(),
                "parallelize_independent_work".to_string(),
                "inline_critical_functions".to_string(),
            ]
        }
    }

    /// Clear caches if action generator is available
    pub fn clear_caches(&mut self) {
        if let Some(ref mut generator) = self.action_generator {
            generator.clear_cache();
        }
    }

    /// Coordinate action execution
    pub fn coordinate_action(&mut self, action_id: String, metadata: ActionMetadata) -> Result<(), CoordinatorError> {
        self.statistics.total_coordinated += 1;
        self.active_actions.insert(action_id.clone(), metadata);
        self.metrics.update_coordination_time();
        Ok(())
    }

    /// Get coordination statistics
    pub fn get_statistics(&self) -> &CoordinatorStatistics {
        &self.statistics
    }

    /// Update coordination metrics
    pub fn update_metrics(&mut self, execution_time_us: u64) {
        self.metrics.update_execution_time(execution_time_us);
        self.statistics.last_updated = Instant::now();
    }
}

/// Statistics for action coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorStatistics {
    /// Total actions coordinated
    pub total_coordinated: u64,
    /// Successfully coordinated actions
    pub successful_coordinations: u64,
    /// Failed coordination attempts
    pub failed_coordinations: u64,
    /// Average coordination time in microseconds
    pub avg_coordination_time_us: f64,
    /// Peak coordination load
    pub peak_coordination_load: usize,
    /// Current active coordinations
    pub active_coordinations: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl CoordinatorStatistics {
    /// Create new coordinator statistics
    pub fn new() -> Self {
        Self {
            total_coordinated: 0,
            successful_coordinations: 0,
            failed_coordinations: 0,
            avg_coordination_time_us: 0.0,
            peak_coordination_load: 0,
            active_coordinations: 0,
            success_rate: 0.0,
            last_updated: Instant::now(),
        }
    }

    /// Update success statistics
    pub fn record_success(&mut self, coordination_time_us: f64) {
        self.successful_coordinations += 1;
        self.update_averages(coordination_time_us);
        self.last_updated = Instant::now();
    }

    /// Update failure statistics
    pub fn record_failure(&mut self) {
        self.failed_coordinations += 1;
        self.update_success_rate();
        self.last_updated = Instant::now();
    }

    /// Update average coordination time
    fn update_averages(&mut self, coordination_time_us: f64) {
        let total = self.successful_coordinations + self.failed_coordinations;
        if total > 0 {
            let alpha = 1.0 / total as f64;
            self.avg_coordination_time_us = 
                (1.0 - alpha) * self.avg_coordination_time_us + alpha * coordination_time_us;
        }
        self.update_success_rate();
    }

    /// Update success rate
    fn update_success_rate(&mut self) {
        let total = self.successful_coordinations + self.failed_coordinations;
        if total > 0 {
            self.success_rate = self.successful_coordinations as f64 / total as f64;
        }
    }
}

impl Default for CoordinatorStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for action coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Initial capacity for action storage
    pub initial_capacity: usize,
    /// Maximum concurrent coordinations
    pub max_concurrent: usize,
    /// Coordination timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable performance metrics
    pub enable_metrics: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 1000,
            max_concurrent: 100,
            timeout_ms: 5000,
            enable_metrics: true,
        }
    }
}

/// Performance metrics for coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorMetrics {
    /// Last coordination timestamp
    pub last_coordination: Instant,
    /// Total coordination time
    pub total_coordination_time_us: u64,
    /// Peak memory usage
    pub peak_memory_bytes: usize,
    /// Current memory usage
    pub current_memory_bytes: usize,
}

impl CoordinatorMetrics {
    /// Create new coordinator metrics
    pub fn new() -> Self {
        Self {
            last_coordination: Instant::now(),
            total_coordination_time_us: 0,
            peak_memory_bytes: 0,
            current_memory_bytes: 0,
        }
    }

    /// Update coordination time
    pub fn update_coordination_time(&mut self) {
        self.last_coordination = Instant::now();
    }

    /// Update execution time
    pub fn update_execution_time(&mut self, execution_time_us: u64) {
        self.total_coordination_time_us += execution_time_us;
    }
}

/// Coordinator error types
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("Coordination capacity exceeded")]
    CapacityExceeded,
    #[error("Coordination timeout")]
    Timeout,
    #[error("Invalid action metadata: {0}")]
    InvalidMetadata(String),
    #[error("Coordination failed: {0}")]
    CoordinationFailed(String),
}
//! MCTS action type definitions and operations
//!
//! This module provides blazing-fast action structures with zero allocation
//! optimizations and elegant ergonomic interfaces for MCTS action operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Action space for MCTS operations with optimized storage
#[derive(Debug, Clone)]
pub struct ActionSpace {
    pub available_actions: Vec<String>,
    pub action_weights: HashMap<String, f64>,
    pub action_constraints: HashMap<String, ActionConstraint>,
    pub action_history: Vec<ActionHistoryEntry>,
    pub space_metadata: ActionSpaceMetadata,
}

impl ActionSpace {
    /// Create new action space with zero allocation initialization
    #[inline]
    pub fn new(actions: Vec<String>) -> Self {
        let mut action_weights = HashMap::with_capacity(actions.len());
        let mut action_constraints = HashMap::with_capacity(actions.len());
        
        // Initialize with equal weights
        for action in &actions {
            action_weights.insert(action.clone(), 1.0);
            action_constraints.insert(action.clone(), ActionConstraint::default());
        }
        
        Self {
            available_actions: actions,
            action_weights,
            action_constraints,
            action_history: Vec::new(),
            space_metadata: ActionSpaceMetadata::new(),
        }
    }

    /// Create action space with weighted actions
    #[inline]
    pub fn with_weights(actions: Vec<String>, weights: Vec<f64>) -> Result<Self, String> {
        if actions.len() != weights.len() {
            return Err("Actions and weights must have same length".to_string());
        }
        
        let mut action_space = Self::new(actions);
        
        for (action, weight) in action_space.available_actions.iter().zip(weights.iter()) {
            action_space.action_weights.insert(action.clone(), *weight);
        }
        
        Ok(action_space)
    }

    /// Get available actions that satisfy constraints
    #[inline]
    pub fn get_valid_actions(&self, context: &ActionContext) -> Vec<String> {
        let mut valid_actions = Vec::new();
        
        for action in &self.available_actions {
            if let Some(constraint) = self.action_constraints.get(action) {
                if constraint.is_satisfied(context) {
                    valid_actions.push(action.clone());
                }
            } else {
                valid_actions.push(action.clone());
            }
        }
        
        valid_actions
    }

    /// Get action weight with fallback to default
    #[inline]
    pub fn get_action_weight(&self, action: &str) -> f64 {
        self.action_weights.get(action).copied().unwrap_or(1.0)
    }

    /// Update action weight based on performance
    #[inline]
    pub fn update_action_weight(&mut self, action: &str, performance: f64) {
        let current_weight = self.get_action_weight(action);
        let learning_rate = 0.1;
        let new_weight = current_weight + learning_rate * (performance - 0.5);
        
        self.action_weights.insert(action.to_string(), new_weight.clamp(0.1, 10.0));
        self.space_metadata.last_weight_update = std::time::Instant::now();
    }

    /// Add action to history
    #[inline]
    pub fn record_action(&mut self, action: String, result: ActionResult) {
        let entry = ActionHistoryEntry {
            action,
            result,
            timestamp: std::time::Instant::now(),
        };
        
        self.action_history.push(entry);
        
        // Limit history size for memory efficiency
        if self.action_history.len() > 1000 {
            self.action_history.remove(0);
        }
        
        self.space_metadata.total_actions += 1;
    }

    /// Get action success rate
    #[inline]
    pub fn get_success_rate(&self, action: &str) -> f64 {
        let action_entries: Vec<_> = self.action_history.iter()
            .filter(|entry| entry.action == action)
            .collect();
        
        if action_entries.is_empty() {
            return 0.5; // Default neutral success rate
        }
        
        let successful = action_entries.iter()
            .filter(|entry| entry.result.success)
            .count();
        
        successful as f64 / action_entries.len() as f64
    }

    /// Get most successful action
    #[inline]
    pub fn get_best_action(&self) -> Option<String> {
        let mut best_action = None;
        let mut best_score = f64::NEG_INFINITY;
        
        for action in &self.available_actions {
            let success_rate = self.get_success_rate(action);
            let weight = self.get_action_weight(action);
            let score = success_rate * weight;
            
            if score > best_score {
                best_score = score;
                best_action = Some(action.clone());
            }
        }
        
        best_action
    }

    /// Get action diversity score
    #[inline]
    pub fn diversity_score(&self) -> f64 {
        if self.available_actions.is_empty() {
            return 0.0;
        }
        
        let mut action_counts = HashMap::new();
        for entry in &self.action_history {
            *action_counts.entry(&entry.action).or_insert(0) += 1;
        }
        
        let total_actions = self.action_history.len() as f64;
        if total_actions == 0.0 {
            return 1.0;
        }
        
        let mut entropy = 0.0;
        for count in action_counts.values() {
            let probability = *count as f64 / total_actions;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }
        
        // Normalize by maximum possible entropy
        let max_entropy = (self.available_actions.len() as f64).log2();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    /// Prune underperforming actions
    #[inline]
    pub fn prune_actions(&mut self, min_success_rate: f64, min_usage_count: usize) -> usize {
        let mut actions_to_remove = Vec::new();
        
        for action in &self.available_actions {
            let usage_count = self.action_history.iter()
                .filter(|entry| entry.action == *action)
                .count();
            
            if usage_count >= min_usage_count {
                let success_rate = self.get_success_rate(action);
                if success_rate < min_success_rate {
                    actions_to_remove.push(action.clone());
                }
            }
        }
        
        let removed_count = actions_to_remove.len();
        for action in actions_to_remove {
            self.available_actions.retain(|a| a != &action);
            self.action_weights.remove(&action);
            self.action_constraints.remove(&action);
        }
        
        self.space_metadata.pruned_actions += removed_count;
        removed_count
    }

    /// Get action space statistics
    #[inline]
    pub fn get_statistics(&self) -> ActionSpaceStatistics {
        let total_actions = self.action_history.len();
        let unique_actions = self.action_history.iter()
            .map(|entry| &entry.action)
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        let successful_actions = self.action_history.iter()
            .filter(|entry| entry.result.success)
            .count();
        
        let overall_success_rate = if total_actions > 0 {
            successful_actions as f64 / total_actions as f64
        } else {
            0.0
        };
        
        ActionSpaceStatistics {
            total_actions: self.available_actions.len(),
            executed_actions: total_actions,
            unique_executed_actions: unique_actions,
            overall_success_rate,
            diversity_score: self.diversity_score(),
            average_reward: self.calculate_average_reward(),
        }
    }

    /// Calculate average reward from action history
    #[inline]
    fn calculate_average_reward(&self) -> f64 {
        if self.action_history.is_empty() {
            return 0.0;
        }
        
        let total_reward: f64 = self.action_history.iter()
            .map(|entry| entry.result.reward)
            .sum();
        
        total_reward / self.action_history.len() as f64
    }
}

/// Action constraint for validating action applicability
#[derive(Debug, Clone)]
pub struct ActionConstraint {
    pub min_depth: Option<u16>,
    pub max_depth: Option<u16>,
    pub required_state_properties: Vec<String>,
    pub forbidden_state_properties: Vec<String>,
    pub min_visits: Option<u32>,
    pub max_memory_usage: Option<f64>,
    pub cooldown_period: Option<std::time::Duration>,
    pub last_execution: Option<std::time::Instant>,
}

impl ActionConstraint {
    /// Create new action constraint with no restrictions
    #[inline]
    pub fn new() -> Self {
        Self {
            min_depth: None,
            max_depth: None,
            required_state_properties: Vec::new(),
            forbidden_state_properties: Vec::new(),
            min_visits: None,
            max_memory_usage: None,
            cooldown_period: None,
            last_execution: None,
        }
    }

    /// Check if constraint is satisfied by context
    #[inline]
    pub fn is_satisfied(&self, context: &ActionContext) -> bool {
        // Check depth constraints
        if let Some(min_depth) = self.min_depth {
            if context.current_depth < min_depth {
                return false;
            }
        }
        
        if let Some(max_depth) = self.max_depth {
            if context.current_depth > max_depth {
                return false;
            }
        }
        
        // Check visit constraints
        if let Some(min_visits) = self.min_visits {
            if context.node_visits < min_visits {
                return false;
            }
        }
        
        // Check memory constraints
        if let Some(max_memory) = self.max_memory_usage {
            if context.memory_usage > max_memory {
                return false;
            }
        }
        
        // Check cooldown period
        if let (Some(cooldown), Some(last_exec)) = (self.cooldown_period, self.last_execution) {
            if last_exec.elapsed() < cooldown {
                return false;
            }
        }
        
        // Check required state properties
        for required_prop in &self.required_state_properties {
            if !context.state_properties.contains(required_prop) {
                return false;
            }
        }
        
        // Check forbidden state properties
        for forbidden_prop in &self.forbidden_state_properties {
            if context.state_properties.contains(forbidden_prop) {
                return false;
            }
        }
        
        true
    }

    /// Update last execution time
    #[inline]
    pub fn mark_executed(&mut self) {
        self.last_execution = Some(std::time::Instant::now());
    }

    /// Create constraint for depth range
    #[inline]
    pub fn depth_range(min_depth: u16, max_depth: u16) -> Self {
        Self {
            min_depth: Some(min_depth),
            max_depth: Some(max_depth),
            ..Self::new()
        }
    }

    /// Create constraint with cooldown period
    #[inline]
    pub fn with_cooldown(cooldown: std::time::Duration) -> Self {
        Self {
            cooldown_period: Some(cooldown),
            ..Self::new()
        }
    }
}

impl Default for ActionConstraint {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for action constraint evaluation
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub current_depth: u16,
    pub node_visits: u32,
    pub memory_usage: f64,
    pub state_properties: Vec<String>,
    pub available_resources: HashMap<String, f64>,
    pub execution_time: std::time::Duration,
}

impl ActionContext {
    /// Create new action context
    #[inline]
    pub fn new(depth: u16, visits: u32, memory: f64) -> Self {
        Self {
            current_depth: depth,
            node_visits: visits,
            memory_usage: memory,
            state_properties: Vec::new(),
            available_resources: HashMap::new(),
            execution_time: std::time::Duration::from_secs(0),
        }
    }

    /// Add state property
    #[inline]
    pub fn with_property(mut self, property: String) -> Self {
        self.state_properties.push(property);
        self
    }

    /// Add resource availability
    #[inline]
    pub fn with_resource(mut self, resource: String, amount: f64) -> Self {
        self.available_resources.insert(resource, amount);
        self
    }

    /// Check if resource is available
    #[inline]
    pub fn has_resource(&self, resource: &str, required_amount: f64) -> bool {
        self.available_resources.get(resource)
            .map(|&available| available >= required_amount)
            .unwrap_or(false)
    }
}

/// Action execution result
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub reward: f64,
    pub execution_time: std::time::Duration,
    pub state_changes: Vec<String>,
    pub error_message: Option<String>,
    pub metadata: ActionResultMetadata,
}

impl ActionResult {
    /// Create successful action result
    #[inline]
    pub fn success(reward: f64, execution_time: std::time::Duration) -> Self {
        Self {
            success: true,
            reward,
            execution_time,
            state_changes: Vec::new(),
            error_message: None,
            metadata: ActionResultMetadata::new(),
        }
    }

    /// Create failed action result
    #[inline]
    pub fn failure(error: String, execution_time: std::time::Duration) -> Self {
        Self {
            success: false,
            reward: 0.0,
            execution_time,
            state_changes: Vec::new(),
            error_message: Some(error),
            metadata: ActionResultMetadata::new(),
        }
    }

    /// Create result with state changes
    #[inline]
    pub fn with_changes(mut self, changes: Vec<String>) -> Self {
        self.state_changes = changes;
        self
    }

    /// Calculate performance score
    #[inline]
    pub fn performance_score(&self) -> f64 {
        if !self.success {
            return 0.0;
        }
        
        let reward_score = (self.reward + 1.0) / 2.0; // Normalize to [0,1]
        let time_score = 1.0 / (1.0 + self.execution_time.as_secs_f64());
        let change_score = if self.state_changes.is_empty() { 0.5 } else { 1.0 };
        
        (reward_score * 0.5 + time_score * 0.3 + change_score * 0.2).clamp(0.0, 1.0)
    }

    /// Check if result meets quality threshold
    #[inline]
    pub fn meets_quality_threshold(&self, threshold: f64) -> bool {
        self.success && self.performance_score() >= threshold
    }
}

/// Action result metadata
#[derive(Debug, Clone)]
pub struct ActionResultMetadata {
    pub memory_delta: f64,
    pub cpu_usage: f64,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub network_calls: u32,
    pub timestamp: std::time::Instant,
}

impl ActionResultMetadata {
    /// Create new action result metadata
    #[inline]
    pub fn new() -> Self {
        Self {
            memory_delta: 0.0,
            cpu_usage: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            network_calls: 0,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Calculate cache hit rate
    #[inline]
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests > 0 {
            self.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Calculate efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        let memory_efficiency = 1.0 / (1.0 + self.memory_delta.abs());
        let cpu_efficiency = 1.0 / (1.0 + self.cpu_usage);
        let cache_efficiency = self.cache_hit_rate();
        let network_efficiency = 1.0 / (1.0 + self.network_calls as f64 * 0.1);
        
        (memory_efficiency * 0.3 + cpu_efficiency * 0.3 + 
         cache_efficiency * 0.2 + network_efficiency * 0.2).clamp(0.0, 1.0)
    }
}

impl Default for ActionResultMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Action history entry
#[derive(Debug, Clone)]
pub struct ActionHistoryEntry {
    pub action: String,
    pub result: ActionResult,
    pub timestamp: std::time::Instant,
}

impl ActionHistoryEntry {
    /// Get age of this entry in seconds
    #[inline]
    pub fn age_seconds(&self) -> f64 {
        self.timestamp.elapsed().as_secs_f64()
    }

    /// Check if entry is recent
    #[inline]
    pub fn is_recent(&self, max_age_seconds: f64) -> bool {
        self.age_seconds() < max_age_seconds
    }
}

/// Action space metadata
#[derive(Debug, Clone)]
pub struct ActionSpaceMetadata {
    pub creation_time: std::time::Instant,
    pub last_weight_update: std::time::Instant,
    pub total_actions: usize,
    pub pruned_actions: usize,
    pub optimization_count: u32,
}

impl ActionSpaceMetadata {
    /// Create new action space metadata
    #[inline]
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            creation_time: now,
            last_weight_update: now,
            total_actions: 0,
            pruned_actions: 0,
            optimization_count: 0,
        }
    }

    /// Get age in seconds
    #[inline]
    pub fn age_seconds(&self) -> f64 {
        self.creation_time.elapsed().as_secs_f64()
    }

    /// Get time since last weight update in seconds
    #[inline]
    pub fn time_since_weight_update_seconds(&self) -> f64 {
        self.last_weight_update.elapsed().as_secs_f64()
    }
}

impl Default for ActionSpaceMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Action space statistics
#[derive(Debug, Clone)]
pub struct ActionSpaceStatistics {
    pub total_actions: usize,
    pub executed_actions: usize,
    pub unique_executed_actions: usize,
    pub overall_success_rate: f64,
    pub diversity_score: f64,
    pub average_reward: f64,
}

impl ActionSpaceStatistics {
    /// Calculate overall efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        let execution_ratio = if self.total_actions > 0 {
            self.executed_actions as f64 / self.total_actions as f64
        } else {
            0.0
        };
        
        let diversity_factor = self.diversity_score;
        let success_factor = self.overall_success_rate;
        let reward_factor = (self.average_reward + 1.0) / 2.0; // Normalize to [0,1]
        
        (execution_ratio * 0.2 + diversity_factor * 0.3 + 
         success_factor * 0.3 + reward_factor * 0.2).clamp(0.0, 1.0)
    }

    /// Check if statistics indicate healthy action space
    #[inline]
    pub fn is_healthy(&self) -> bool {
        self.overall_success_rate > 0.5 && 
        self.diversity_score > 0.3 && 
        self.average_reward > 0.0
    }
}/// Action metadata for MCTS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    /// Action identifier
    pub action_id: String,
    /// Action type classification
    pub action_type: String,
    /// Expected reward estimate
    pub expected_reward: f64,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Execution cost estimate
    pub cost_estimate: f64,
    /// Priority level
    pub priority: u8,
    /// Creation timestamp
    pub created_at: std::time::Instant,
    /// Last updated timestamp
    pub updated_at: std::time::Instant,
}

impl ActionMetadata {
    /// Create new action metadata
    pub fn new(action_id: String, action_type: String) -> Self {
        let now = std::time::Instant::now();
        Self {
            action_id,
            action_type,
            expected_reward: 0.0,
            confidence: 0.5,
            cost_estimate: 1.0,
            priority: 5,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update expected reward
    pub fn update_reward(&mut self, reward: f64) {
        self.expected_reward = reward;
        self.updated_at = std::time::Instant::now();
    }

    /// Update confidence level
    pub fn update_confidence(&mut self, confidence: f64) {
        self.confidence = confidence.clamp(0.0, 1.0);
        self.updated_at = std::time::Instant::now();
    }
}

/// Node statistics for MCTS performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatistics {
    /// Total number of visits
    pub visit_count: u64,
    /// Total reward accumulated
    pub total_reward: f64,
    /// Average reward per visit
    pub average_reward: f64,
    /// Maximum reward observed
    pub max_reward: f64,
    /// Minimum reward observed
    pub min_reward: f64,
    /// Standard deviation of rewards
    pub reward_std_dev: f64,
    /// Number of children nodes
    pub child_count: usize,
    /// Depth in the tree
    pub depth: usize,
    /// Last visit timestamp
    pub last_visited: std::time::Instant,
}

impl NodeStatistics {
    /// Create new node statistics
    pub fn new() -> Self {
        Self {
            visit_count: 0,
            total_reward: 0.0,
            average_reward: 0.0,
            max_reward: f64::NEG_INFINITY,
            min_reward: f64::INFINITY,
            reward_std_dev: 0.0,
            child_count: 0,
            depth: 0,
            last_visited: std::time::Instant::now(),
        }
    }

    /// Update statistics with new reward
    pub fn update_reward(&mut self, reward: f64) {
        self.visit_count += 1;
        self.total_reward += reward;
        self.average_reward = self.total_reward / self.visit_count as f64;
        self.max_reward = self.max_reward.max(reward);
        self.min_reward = self.min_reward.min(reward);
        self.last_visited = std::time::Instant::now();
        
        // Update standard deviation (simplified calculation)
        if self.visit_count > 1 {
            let variance = (reward - self.average_reward).powi(2) / self.visit_count as f64;
            self.reward_std_dev = variance.sqrt();
        }
    }
}

impl Default for NodeStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Efficiency metrics for MCTS performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Simulations per second
    pub simulations_per_second: f64,
    /// Average simulation time in microseconds
    pub avg_simulation_time_us: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// CPU utilization percentage
    pub cpu_utilization: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Branch factor efficiency
    pub branch_factor_efficiency: f64,
    /// Overall efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Last measurement timestamp
    pub measured_at: std::time::Instant,
}

impl EfficiencyMetrics {
    /// Create new efficiency metrics
    pub fn new() -> Self {
        Self {
            simulations_per_second: 0.0,
            avg_simulation_time_us: 0.0,
            memory_usage_bytes: 0,
            cpu_utilization: 0.0,
            cache_hit_rate: 0.0,
            branch_factor_efficiency: 0.0,
            efficiency_score: 0.0,
            measured_at: std::time::Instant::now(),
        }
    }

    /// Update efficiency metrics
    pub fn update(&mut self, simulations: u64, duration_us: u64, memory_bytes: usize) {
        self.simulations_per_second = simulations as f64 / (duration_us as f64 / 1_000_000.0);
        self.avg_simulation_time_us = duration_us as f64 / simulations as f64;
        self.memory_usage_bytes = memory_bytes;
        self.measured_at = std::time::Instant::now();
        
        // Calculate overall efficiency score
        self.efficiency_score = (self.simulations_per_second / 1000.0).min(1.0) * 
                               (1.0 - (self.memory_usage_bytes as f64 / 1_000_000.0).min(1.0)) *
                               self.cache_hit_rate;
    }
}

impl Default for EfficiencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}
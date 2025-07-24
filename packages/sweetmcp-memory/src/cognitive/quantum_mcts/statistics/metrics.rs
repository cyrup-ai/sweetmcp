//! Core metrics for quantum MCTS analysis
//!
//! This module provides fundamental metrics for tree depth and reward analysis
//! with blazing-fast computation and zero-allocation statistical methods.

use serde::Serialize;
use crate::cognitive::quantum::Complex64;

/// Convergence analysis metrics
#[derive(Debug, Clone, Serialize)]
pub struct ConvergenceMetrics {
    /// Number of iterations to convergence
    pub iterations_to_convergence: u32,
    /// Final convergence rate
    pub convergence_rate: f64,
    /// Convergence stability score
    pub stability_score: f64,
    /// Whether convergence was achieved
    pub converged: bool,
}

impl ConvergenceMetrics {
    /// Create new convergence metrics
    pub fn new(iterations: u32, rate: f64, stability: f64, converged: bool) -> Self {
        Self {
            iterations_to_convergence: iterations,
            convergence_rate: rate,
            stability_score: stability,
            converged,
        }
    }
}

/// Performance metrics for quantum MCTS
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    /// Average execution time per iteration
    pub avg_execution_time_ms: f64,
    /// Memory usage statistics
    pub memory_usage_mb: f64,
    /// Quantum circuit depth
    pub circuit_depth: u32,
    /// Gate count
    pub gate_count: usize,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new(exec_time: f64, memory: f64, depth: u32, gates: usize) -> Self {
        Self {
            avg_execution_time_ms: exec_time,
            memory_usage_mb: memory,
            circuit_depth: depth,
            gate_count: gates,
        }
    }
}

/// Tree depth analysis with detailed distribution
#[derive(Debug, Clone, Serialize)]
pub struct DepthStatistics {
    /// Maximum tree depth
    pub max_depth: u32,
    /// Average depth of all nodes
    pub avg_depth: f64,
    /// Number of leaf nodes
    pub leaf_nodes: usize,
    /// Number of internal nodes
    pub internal_nodes: usize,
    /// Depth distribution (depth -> node count)
    pub depth_distribution: Vec<(u32, usize)>,
}

impl DepthStatistics {
    /// Create new depth statistics
    pub fn new(
        max_depth: u32,
        avg_depth: f64,
        leaf_nodes: usize,
        internal_nodes: usize,
        depth_distribution: Vec<(u32, usize)>,
    ) -> Self {
        Self {
            max_depth,
            avg_depth,
            leaf_nodes,
            internal_nodes,
            depth_distribution,
        }
    }
    
    /// Get tree balance ratio (leaves to internal nodes)
    pub fn balance_ratio(&self) -> f64 {
        if self.internal_nodes > 0 {
            self.leaf_nodes as f64 / self.internal_nodes as f64
        } else {
            0.0
        }
    }
    
    /// Check if tree is well-balanced
    pub fn is_balanced(&self) -> bool {
        let ratio = self.balance_ratio();
        ratio >= 0.5 && ratio <= 2.0 // Reasonable balance range
    }
    
    /// Get depth efficiency score
    pub fn depth_efficiency(&self) -> f64 {
        if self.max_depth > 0 {
            self.avg_depth / self.max_depth as f64
        } else {
            0.0
        }
    }
    
    /// Get total nodes count
    pub fn total_nodes(&self) -> usize {
        self.leaf_nodes + self.internal_nodes
    }
    
    /// Get depth distribution summary
    pub fn depth_summary(&self) -> String {
        if self.depth_distribution.is_empty() {
            return "No depth data".to_string();
        }
        
        let most_common = self.depth_distribution.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(depth, count)| (*depth, *count))
            .unwrap_or((0, 0));
        
        format!(
            "Max: {}, Avg: {:.1}, Most common: depth {} ({} nodes)",
            self.max_depth, self.avg_depth, most_common.0, most_common.1
        )
    }
}

impl Default for DepthStatistics {
    fn default() -> Self {
        Self {
            max_depth: 0,
            avg_depth: 0.0,
            leaf_nodes: 0,
            internal_nodes: 0,
            depth_distribution: Vec::new(),
        }
    }
}

/// Reward distribution analysis with comprehensive metrics
#[derive(Debug, Clone, Serialize)]
pub struct RewardStatistics {
    /// Total reward across all nodes
    pub total_reward: Complex64,
    /// Average reward per node
    pub avg_reward: Complex64,
    /// Maximum reward magnitude
    pub max_reward_magnitude: f64,
    /// Minimum reward magnitude
    pub min_reward_magnitude: f64,
    /// Reward variance
    pub reward_variance: f64,
    /// Number of nodes with positive real reward
    pub positive_reward_nodes: usize,
    /// Number of nodes with negative real reward
    pub negative_reward_nodes: usize,
}

impl RewardStatistics {
    /// Create new reward statistics
    pub fn new(
        total_reward: Complex64,
        avg_reward: Complex64,
        max_reward_magnitude: f64,
        min_reward_magnitude: f64,
        reward_variance: f64,
        positive_reward_nodes: usize,
        negative_reward_nodes: usize,
    ) -> Self {
        Self {
            total_reward,
            avg_reward,
            max_reward_magnitude,
            min_reward_magnitude,
            reward_variance,
            positive_reward_nodes,
            negative_reward_nodes,
        }
    }
    
    /// Get reward stability (lower variance = higher stability)
    pub fn reward_stability(&self) -> f64 {
        let avg_magnitude = self.avg_reward.norm();
        if avg_magnitude > 1e-10 {
            1.0 - (self.reward_variance.sqrt() / avg_magnitude).min(1.0)
        } else {
            0.0
        }
    }
    
    /// Get positive reward ratio
    pub fn positive_ratio(&self) -> f64 {
        let total_nodes = self.positive_reward_nodes + self.negative_reward_nodes;
        if total_nodes > 0 {
            self.positive_reward_nodes as f64 / total_nodes as f64
        } else {
            0.0
        }
    }
    
    /// Get reward range (max - min)
    pub fn reward_range(&self) -> f64 {
        self.max_reward_magnitude - self.min_reward_magnitude
    }
    
    /// Get reward quality score
    pub fn quality_score(&self) -> f64 {
        let stability_weight = 0.4;
        let positive_ratio_weight = 0.3;
        let magnitude_weight = 0.3;
        
        let stability = self.reward_stability();
        let positive_ratio = self.positive_ratio();
        let magnitude = self.avg_reward.norm().min(1.0);
        
        stability * stability_weight + positive_ratio * positive_ratio_weight + magnitude * magnitude_weight
    }
    
    /// Get reward distribution summary
    pub fn reward_summary(&self) -> String {
        format!(
            "Avg: {:.3}, Range: [{:.3}, {:.3}], Positive: {:.1}%, Stability: {:.3}",
            self.avg_reward.norm(),
            self.min_reward_magnitude,
            self.max_reward_magnitude,
            self.positive_ratio() * 100.0,
            self.reward_stability()
        )
    }
    
    /// Check if rewards show good distribution
    pub fn has_good_distribution(&self) -> bool {
        self.positive_ratio() > 0.6 
            && self.reward_stability() > 0.7 
            && self.reward_range() > 0.1
    }
}
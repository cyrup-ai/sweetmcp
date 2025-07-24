//! Core statistics data structures and type definitions
//!
//! This module provides fundamental statistics types with blazing-fast serialization
//! and zero-allocation patterns for quantum MCTS performance monitoring.

use std::time::Instant;
use serde::Serialize;

use crate::cognitive::quantum::{Complex64, QuantumMetrics};
use super::metrics::{DepthStatistics, RewardStatistics, ConvergenceMetrics, PerformanceMetrics};

/// Lock-free quantum tree statistics with atomic counters
#[derive(Debug, Serialize)]
pub struct QuantumTreeStatistics {
    /// Total number of nodes in the tree
    pub total_nodes: usize,
    /// Total visits across all nodes
    pub total_visits: u64,
    /// Total number of entanglements
    pub total_entanglements: usize,
    /// Average decoherence across all nodes
    pub avg_decoherence: f64,
    /// Maximum amplitude magnitude
    pub max_amplitude: f64,
    /// Quantum metrics snapshot
    pub quantum_metrics: QuantumMetrics,
    /// Tree depth statistics
    pub depth_stats: DepthStatistics,
    /// Reward distribution statistics
    pub reward_stats: RewardStatistics,
    /// Convergence metrics
    pub convergence_metrics: ConvergenceMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

impl QuantumTreeStatistics {
    /// Create new quantum tree statistics
    pub fn new(
        total_nodes: usize,
        total_visits: u64,
        total_entanglements: usize,
        avg_decoherence: f64,
        max_amplitude: f64,
        quantum_metrics: QuantumMetrics,
        depth_stats: DepthStatistics,
        reward_stats: RewardStatistics,
        convergence_metrics: ConvergenceMetrics,
        performance_metrics: PerformanceMetrics,
    ) -> Self {
        Self {
            total_nodes,
            total_visits,
            total_entanglements,
            avg_decoherence,
            max_amplitude,
            quantum_metrics,
            depth_stats,
            reward_stats,
            convergence_metrics,
            performance_metrics,
        }
    }
    
    /// Get overall health score with weighted analysis
    pub fn health_score(&self) -> f64 {
        let convergence_weight = 0.3;
        let performance_weight = 0.3;
        let quantum_weight = 0.2;
        let stability_weight = 0.2;
        
        let convergence_score = self.convergence_metrics.overall_convergence;
        let performance_score = self.performance_metrics.throughput_metrics.overall_throughput();
        let quantum_score = 1.0 - self.avg_decoherence;
        let stability_score = self.reward_stats.reward_stability();
        
        convergence_score * convergence_weight
            + performance_score * performance_weight
            + quantum_score * quantum_weight
            + stability_score * stability_weight
    }
    
    /// Check if statistics indicate healthy operation
    pub fn is_healthy(&self) -> bool {
        self.health_score() > 0.7
            && self.convergence_metrics.overall_convergence > 0.6
            && self.performance_metrics.avg_visits_per_node > 1.0
    }
    
    /// Get summary report as formatted string
    pub fn summary_report(&self) -> String {
        format!(
            "Quantum MCTS Statistics Summary:\n\
             - Nodes: {} (visits: {})\n\
             - Convergence: {:.3} (grade: {})\n\
             - Health Score: {:.3}\n\
             - Max Depth: {}\n\
             - Avg Decoherence: {:.3}\n\
             - Performance: {:.3} ops/sec",
            self.total_nodes,
            self.total_visits,
            self.convergence_metrics.overall_convergence,
            self.convergence_metrics.convergence_grade(),
            self.health_score(),
            self.depth_stats.max_depth,
            self.avg_decoherence,
            self.performance_metrics.throughput_metrics.overall_throughput() * 1000.0
        )
    }
    
    /// Compare with another statistics snapshot
    pub fn compare_with(&self, other: &QuantumTreeStatistics) -> StatisticsComparison {
        StatisticsComparison {
            node_growth: self.total_nodes as i64 - other.total_nodes as i64,
            visit_growth: self.total_visits as i64 - other.total_visits as i64,
            convergence_delta: self.convergence_metrics.overall_convergence - other.convergence_metrics.overall_convergence,
            health_delta: self.health_score() - other.health_score(),
            performance_delta: self.performance_metrics.throughput_metrics.overall_throughput() 
                - other.performance_metrics.throughput_metrics.overall_throughput(),
        }
    }
}

/// Statistics comparison result
#[derive(Debug, Clone)]
pub struct StatisticsComparison {
    /// Change in node count
    pub node_growth: i64,
    /// Change in visit count
    pub visit_growth: i64,
    /// Change in convergence score
    pub convergence_delta: f64,
    /// Change in health score
    pub health_delta: f64,
    /// Change in performance score
    pub performance_delta: f64,
}

impl StatisticsComparison {
    /// Check if comparison shows improvement
    pub fn shows_improvement(&self) -> bool {
        self.convergence_delta > 0.01 
            && self.health_delta > 0.0 
            && self.performance_delta > 0.0
    }
    
    /// Check if comparison shows degradation
    pub fn shows_degradation(&self) -> bool {
        self.convergence_delta < -0.01 
            || (self.health_delta < -0.05 && self.performance_delta < -0.05)
    }
    
    /// Get improvement summary
    pub fn improvement_summary(&self) -> String {
        let trend = if self.shows_improvement() {
            "📈 Improving"
        } else if self.shows_degradation() {
            "📉 Degrading"
        } else {
            "➡️ Stable"
        };
        
        format!(
            "{} - Nodes: {:+}, Visits: {:+}, Convergence: {:+.3}, Health: {:+.3}",
            trend, self.node_growth, self.visit_growth, self.convergence_delta, self.health_delta
        )
    }
}

/// Statistics snapshot for temporal analysis
#[derive(Debug, Clone)]
pub struct StatisticsSnapshot {
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
    /// Statistics at this point in time
    pub statistics: QuantumTreeStatistics,
}

impl StatisticsSnapshot {
    /// Create new statistics snapshot
    pub fn new(statistics: QuantumTreeStatistics) -> Self {
        Self {
            timestamp: Instant::now(),
            statistics,
        }
    }
    
    /// Get age of snapshot
    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
    
    /// Check if snapshot is recent
    pub fn is_recent(&self, threshold: std::time::Duration) -> bool {
        self.age() < threshold
    }
    
    /// Compare with another snapshot
    pub fn compare_with(&self, other: &StatisticsSnapshot) -> StatisticsComparison {
        self.statistics.compare_with(&other.statistics)
    }
}

/// Counter snapshot for atomic values
#[derive(Debug, Clone, Default, Serialize)]
pub struct CounterSnapshot {
    /// Total nodes
    pub nodes: usize,
    /// Total visits
    pub visits: u64,
    /// Total selections
    pub selections: u64,
    /// Total expansions
    pub expansions: u64,
    /// Total backpropagations
    pub backpropagations: u64,
    /// Total simulations
    pub simulations: u64,
}

impl CounterSnapshot {
    /// Create new counter snapshot
    pub fn new(
        nodes: usize,
        visits: u64,
        selections: u64,
        expansions: u64,
        backpropagations: u64,
        simulations: u64,
    ) -> Self {
        Self {
            nodes,
            visits,
            selections,
            expansions,
            backpropagations,
            simulations,
        }
    }
    
    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.selections + self.expansions + self.backpropagations + self.simulations
    }
    
    /// Get operation ratios for balance analysis
    pub fn operation_ratios(&self) -> OperationRatios {
        let total_ops = self.total_operations();
        if total_ops == 0 {
            return OperationRatios::default();
        }
        
        OperationRatios {
            selection_ratio: self.selections as f64 / total_ops as f64,
            expansion_ratio: self.expansions as f64 / total_ops as f64,
            backpropagation_ratio: self.backpropagations as f64 / total_ops as f64,
            simulation_ratio: self.simulations as f64 / total_ops as f64,
        }
    }
}

/// Operation distribution ratios
#[derive(Debug, Clone, Default, Serialize)]
pub struct OperationRatios {
    pub selection_ratio: f64,
    pub expansion_ratio: f64,
    pub backpropagation_ratio: f64,
    pub simulation_ratio: f64,
}

impl OperationRatios {
    /// Check if ratios are balanced for good MCTS performance
    pub fn is_balanced(&self) -> bool {
        // Good MCTS should have roughly equal selections, expansions, and backpropagations
        // Simulations might be higher due to batching
        let expected_ratio = 0.25; // 25% each for balanced operation
        let tolerance = 0.15; // 15% tolerance
        
        (self.selection_ratio - expected_ratio).abs() < tolerance &&
        (self.expansion_ratio - expected_ratio).abs() < tolerance &&
        (self.backpropagation_ratio - expected_ratio).abs() < tolerance
    }
    
    /// Get balance score (1.0 = perfectly balanced)
    pub fn balance_score(&self) -> f64 {
        let expected = 0.25;
        let deviations = [
            (self.selection_ratio - expected).abs(),
            (self.expansion_ratio - expected).abs(),
            (self.backpropagation_ratio - expected).abs(),
            (self.simulation_ratio - expected).abs(),
        ];
        
        let max_deviation = deviations.iter().fold(0.0f64, |a, &b| a.max(b));
        (1.0 - max_deviation * 4.0).max(0.0) // Scale and clamp
    }
}
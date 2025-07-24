//! Metrics and tracking structures with blazing-fast performance monitoring
//!
//! This module provides comprehensive metrics collection, memory tracking, and
//! result structures with zero-allocation patterns and optimized statistical computation.

use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::cognitive::{
    quantum::Complex64,
    types::CognitiveError,
};
use super::super::node_state::QuantumMCTSNode;

/// Memory usage tracker with bounds checking
#[derive(Debug)]
pub struct MemoryTracker {
    max_nodes: usize,
    peak_usage: usize,
    pressure_threshold: f64,
}

impl MemoryTracker {
    /// Create new memory tracker with optimized thresholds
    pub fn new(max_nodes: usize) -> Self {
        Self {
            max_nodes,
            peak_usage: 0,
            pressure_threshold: 0.8, // 80% of max capacity
        }
    }
    
    /// Get current usage and update peak with blazing-fast tracking
    pub async fn current_usage(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) -> usize {
        let tree_read = tree.read().await;
        let usage = tree_read.len();
        self.peak_usage = self.peak_usage.max(usage);
        usage
    }
    
    /// Check memory bounds with zero-allocation validation
    pub async fn check_bounds(&mut self, tree: &RwLock<HashMap<String, QuantumMCTSNode>>) -> Result<(), CognitiveError> {
        let usage = self.current_usage(tree).await;
        
        if usage > self.max_nodes {
            return Err(CognitiveError::ResourceExhaustion(
                format!("Tree size {} exceeds maximum {}", usage, self.max_nodes)
            ));
        }
        
        Ok(())
    }
    
    /// Check if memory is under pressure with blazing-fast comparison
    pub fn is_under_pressure(&self) -> bool {
        self.peak_usage as f64 / self.max_nodes as f64 > self.pressure_threshold
    }
    
    /// Get peak usage
    pub fn peak_usage(&self) -> usize {
        self.peak_usage
    }
    
    /// Reset peak usage counter
    pub fn reset(&mut self) {
        self.peak_usage = 0;
    }
    
    /// Update memory limit
    pub fn update_limit(&mut self, new_limit: usize) {
        self.max_nodes = new_limit;
    }
}

/// Improvement performance metrics with zero-allocation updates
#[derive(Debug, Clone, Default)]
pub struct ImprovementMetrics {
    /// Number of recursive depths completed
    pub depths_completed: u32,
    /// Total improvement time across all depths
    pub total_improvement_time: Duration,
    /// Total number of simulations performed
    pub total_simulations: u64,
    /// Total reward accumulated from simulations
    pub total_reward: f64,
    /// Number of amplitude amplification operations
    pub amplification_operations: u64,
    /// Total nodes amplified across all operations
    pub total_nodes_amplified: usize,
}

impl ImprovementMetrics {
    /// Create new metrics with optimized initialization
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Calculate average reward with blazing-fast division
    pub fn average_reward(&self) -> f64 {
        if self.total_simulations > 0 {
            self.total_reward / self.total_simulations as f64
        } else {
            0.0
        }
    }
    
    /// Calculate simulations per second with optimized computation
    pub fn simulations_per_second(&self) -> f64 {
        if self.total_improvement_time.as_secs_f64() > 0.0 {
            self.total_simulations as f64 / self.total_improvement_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Single simulation result with quality metrics
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub node_id: String,
    pub reward: Complex64,
    pub simulation_quality: f64,
}

/// Single iteration result with performance tracking
#[derive(Debug, Clone)]
pub struct IterationResult {
    pub completed_iterations: u32,
    pub successful_simulations: u32,
    pub failed_simulations: u32,
    pub total_time: Duration,
    pub average_simulation_time: Duration,
}

/// Single depth result with comprehensive metrics
#[derive(Debug, Clone, Default)]
pub struct DepthResult {
    pub depth: u32,
    pub iterations_completed: u32,
    pub convergence_score: f64,
    pub amplification_factor: f64,
    pub nodes_amplified: usize,
    pub elapsed_time: Duration,
    pub memory_usage: usize,
}

/// Amplitude amplification result with detailed statistics
#[derive(Debug, Clone)]
pub struct AmplificationResult {
    pub nodes_processed: usize,
    pub nodes_amplified: usize,
    pub average_amplification: f64,
    pub total_amplification: f64,
}

/// Complete improvement result with comprehensive analysis
#[derive(Debug, Clone)]
pub struct ImprovementResult {
    pub total_depths: u32,
    pub final_convergence_score: f64,
    pub best_convergence_score: f64,
    pub improvement_history: Vec<DepthResult>,
    pub total_time: Duration,
    pub memory_peak: usize,
    pub success: bool,
    pub termination_reason: TerminationReason,
}

impl ImprovementResult {
    /// Analyze convergence trend with blazing-fast comparison
    pub fn convergence_trend(&self) -> ConvergenceTrend {
        if self.improvement_history.len() < 2 {
            return ConvergenceTrend::Insufficient;
        }
        
        let first_score = self.improvement_history[0].convergence_score;
        let last_score = self.improvement_history.last().unwrap().convergence_score;
        
        if last_score > first_score + 0.1 {
            ConvergenceTrend::Improving
        } else if last_score < first_score - 0.1 {
            ConvergenceTrend::Degrading
        } else {
            ConvergenceTrend::Stable
        }
    }
    
    /// Calculate average convergence with zero-allocation computation
    pub fn average_convergence(&self) -> f64 {
        if self.improvement_history.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.improvement_history.iter()
            .map(|depth| depth.convergence_score)
            .sum();
        
        sum / self.improvement_history.len() as f64
    }
    
    /// Get improvement efficiency (convergence per time)
    pub fn improvement_efficiency(&self) -> f64 {
        if self.total_time.as_secs_f64() > 0.0 {
            self.final_convergence_score / self.total_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get memory efficiency (convergence per peak memory)
    pub fn memory_efficiency(&self) -> f64 {
        if self.memory_peak > 0 {
            self.final_convergence_score / self.memory_peak as f64
        } else {
            0.0
        }
    }
}

/// Comprehensive metrics summary for improvement analysis
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    /// Total number of operations performed
    pub total_operations: u64,
    /// Average operation duration
    pub average_duration: Duration,
    /// Success rate of operations
    pub success_rate: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStatsSummary,
    /// Performance trend over time
    pub performance_trend: PerformanceTrend,
    /// Quality metrics
    pub quality_score: f64,
    /// Efficiency metrics
    pub efficiency_score: f64,
}

impl MetricsSummary {
    /// Create new metrics summary
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update summary with new metrics
    pub fn update(&mut self, metrics: &ImprovementMetrics) {
        self.total_operations = metrics.total_simulations;
        self.average_duration = if metrics.total_simulations > 0 {
            metrics.total_improvement_time / metrics.total_simulations as u32
        } else {
            Duration::from_millis(0)
        };
        self.quality_score = metrics.average_reward();
        self.efficiency_score = metrics.simulations_per_second() / 100.0; // Normalized
    }
    
    /// Get overall health score
    pub fn health_score(&self) -> f64 {
        (self.success_rate * 0.4 + self.quality_score * 0.3 + self.efficiency_score * 0.3).clamp(0.0, 1.0)
    }
    
    /// Check if metrics indicate good performance
    pub fn is_healthy(&self) -> bool {
        self.health_score() > 0.7 && self.success_rate > 0.8
    }
}

/// Memory statistics summary
#[derive(Debug, Clone, Default)]
pub struct MemoryStatsSummary {
    /// Current memory usage
    pub current_usage: usize,
    /// Peak memory usage
    pub peak_usage: usize,
    /// Average memory usage
    pub average_usage: f64,
    /// Memory growth rate
    pub growth_rate: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PerformanceTrend {
    #[default]
    Stable,
    Improving,
    Degrading,
    Volatile,
    Insufficient,
}

impl PerformanceTrend {
    /// Check if trend is positive
    pub fn is_positive(self) -> bool {
        matches!(self, PerformanceTrend::Improving | PerformanceTrend::Stable)
    }
    
    /// Check if trend requires attention
    pub fn needs_attention(self) -> bool {
        matches!(self, PerformanceTrend::Degrading | PerformanceTrend::Volatile)
    }
    
    /// Get trend description
    pub fn description(self) -> &'static str {
        match self {
            PerformanceTrend::Stable => "Performance is stable",
            PerformanceTrend::Improving => "Performance is improving",
            PerformanceTrend::Degrading => "Performance is degrading",
            PerformanceTrend::Volatile => "Performance is volatile",
            PerformanceTrend::Insufficient => "Insufficient data for analysis",
        }
    }
}

/// Termination reason enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReason {
    HighConvergence,
    NoImprovement,
    MemoryPressure,
    MaxDepthReached,
    Timeout,
    Error,
}

/// Convergence trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceTrend {
    Improving,
    Stable,
    Degrading,
    Insufficient,
}

/// Calculate quantum state quality with blazing-fast computation
#[inline]
pub fn calculate_quantum_state_quality(quantum_state: &super::super::node_state::QuantumNodeState) -> f64 {
    let coherence = 1.0 - quantum_state.decoherence;
    let entanglement_factor = (quantum_state.entanglement_count() as f64 / 10.0).min(1.0);
    let phase_stability = (quantum_state.phase.cos().abs() + quantum_state.phase.sin().abs()) / 2.0;
    
    (coherence * 0.5 + entanglement_factor * 0.3 + phase_stability * 0.2).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new(1000);
        assert!(!tracker.is_under_pressure());
        
        tracker.peak_usage = 850;
        assert!(tracker.is_under_pressure()); // 85% > 80% threshold
        
        tracker.reset();
        assert_eq!(tracker.peak_usage(), 0);
    }
    
    #[test]
    fn test_improvement_metrics() {
        let mut metrics = ImprovementMetrics::new();
        metrics.total_simulations = 100;
        metrics.total_reward = 250.0;
        metrics.total_improvement_time = Duration::from_secs(10);
        
        assert!((metrics.average_reward() - 2.5).abs() < f64::EPSILON);
        assert!((metrics.simulations_per_second() - 10.0).abs() < f64::EPSILON);
    }
    
    #[test]
    fn test_improvement_result_analysis() {
        let result = ImprovementResult {
            total_depths: 3,
            final_convergence_score: 0.85,
            best_convergence_score: 0.85,
            improvement_history: vec![
                DepthResult {
                    depth: 0,
                    convergence_score: 0.5,
                    ..Default::default()
                },
                DepthResult {
                    depth: 1,
                    convergence_score: 0.7,
                    ..Default::default()
                },
                DepthResult {
                    depth: 2,
                    convergence_score: 0.85,
                    ..Default::default()
                },
            ],
            total_time: Duration::from_secs(30),
            memory_peak: 500,
            success: true,
            termination_reason: TerminationReason::HighConvergence,
        };
        
        assert_eq!(result.convergence_trend(), ConvergenceTrend::Improving);
        assert!((result.average_convergence() - 0.683).abs() < 0.01); // (0.5+0.7+0.85)/3
    }
}
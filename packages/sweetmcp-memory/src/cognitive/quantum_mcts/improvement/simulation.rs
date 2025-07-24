//! Simulation result types and performance tracking
//!
//! This module provides comprehensive simulation result structures with
//! blazing-fast performance metrics and zero-allocation tracking patterns.

use std::time::Duration;
use crate::cognitive::quantum::Complex64;

/// Single simulation result with quality metrics
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Node ID that was simulated
    pub node_id: String,
    /// Quantum reward obtained from simulation
    pub reward: Complex64,
    /// Quality score of the simulation (0.0 to 1.0)
    pub simulation_quality: f64,
    /// Time taken for simulation
    pub simulation_time: Duration,
    /// Number of steps in simulation
    pub steps_taken: u32,
    /// Convergence achieved during simulation
    pub convergence_score: f64,
    /// Errors encountered during simulation
    pub error_count: u32,
}

impl SimulationResult {
    /// Create new simulation result with basic metrics
    pub fn new(node_id: String, reward: Complex64, simulation_quality: f64) -> Self {
        Self {
            node_id,
            reward,
            simulation_quality,
            simulation_time: Duration::from_millis(0),
            steps_taken: 0,
            convergence_score: 0.0,
            error_count: 0,
        }
    }
    
    /// Create new simulation result with comprehensive metrics
    pub fn new_comprehensive(
        node_id: String,
        reward: Complex64,
        simulation_quality: f64,
        simulation_time: Duration,
        steps_taken: u32,
        convergence_score: f64,
        error_count: u32,
    ) -> Self {
        Self {
            node_id,
            reward,
            simulation_quality,
            simulation_time,
            steps_taken,
            convergence_score,
            error_count,
        }
    }
    
    /// Get simulation efficiency (quality per time)
    pub fn efficiency(&self) -> f64 {
        if self.simulation_time.as_secs_f64() > 0.0 {
            self.simulation_quality / self.simulation_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get steps per second
    pub fn steps_per_second(&self) -> f64 {
        if self.simulation_time.as_secs_f64() > 0.0 {
            self.steps_taken as f64 / self.simulation_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Check if simulation was successful
    pub fn is_successful(&self) -> bool {
        self.simulation_quality > 0.5 && self.error_count == 0
    }
    
    /// Get reward magnitude
    pub fn reward_magnitude(&self) -> f64 {
        self.reward.norm()
    }
}

/// Single iteration result with performance tracking
#[derive(Debug, Clone)]
pub struct IterationResult {
    /// Number of iterations completed
    pub completed_iterations: u32,
    /// Number of successful simulations
    pub successful_simulations: u32,
    /// Number of failed simulations
    pub failed_simulations: u32,
    /// Total time for all iterations
    pub total_time: Duration,
    /// Average time per simulation
    pub average_simulation_time: Duration,
    /// Peak memory usage during iteration
    pub peak_memory_usage: usize,
    /// Total reward accumulated
    pub total_reward: Complex64,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
}

impl IterationResult {
    /// Create new iteration result
    pub fn new(
        completed_iterations: u32,
        successful_simulations: u32,
        failed_simulations: u32,
        total_time: Duration,
    ) -> Self {
        let average_simulation_time = if completed_iterations > 0 {
            total_time / completed_iterations
        } else {
            Duration::from_millis(0)
        };
        
        Self {
            completed_iterations,
            successful_simulations,
            failed_simulations,
            total_time,
            average_simulation_time,
            peak_memory_usage: 0,
            total_reward: Complex64::new(0.0, 0.0),
            quality_metrics: QualityMetrics::default(),
        }
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.completed_iterations > 0 {
            self.successful_simulations as f64 / self.completed_iterations as f64
        } else {
            0.0
        }
    }
    
    /// Get iterations per second
    pub fn iterations_per_second(&self) -> f64 {
        if self.total_time.as_secs_f64() > 0.0 {
            self.completed_iterations as f64 / self.total_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get average reward magnitude
    pub fn average_reward_magnitude(&self) -> f64 {
        if self.successful_simulations > 0 {
            self.total_reward.norm() / self.successful_simulations as f64
        } else {
            0.0
        }
    }
    
    /// Check if iteration was efficient
    pub fn is_efficient(&self) -> bool {
        self.success_rate() > 0.8 && self.iterations_per_second() > 1.0
    }
}

/// Single depth result with comprehensive metrics
#[derive(Debug, Clone, Default)]
pub struct DepthResult {
    /// Depth level in recursive improvement
    pub depth: u32,
    /// Number of iterations completed at this depth
    pub iterations_completed: u32,
    /// Convergence score achieved
    pub convergence_score: f64,
    /// Amplification factor applied
    pub amplification_factor: f64,
    /// Number of nodes amplified
    pub nodes_amplified: usize,
    /// Time elapsed for this depth
    pub elapsed_time: Duration,
    /// Memory usage at this depth
    pub memory_usage: usize,
    /// Quality score for this depth
    pub depth_quality: f64,
    /// Improvement over previous depth
    pub improvement_delta: f64,
    /// Number of nodes evaluated
    pub nodes_evaluated: usize,
}

impl DepthResult {
    /// Create new depth result
    pub fn new(depth: u32, iterations_completed: u32, convergence_score: f64) -> Self {
        Self {
            depth,
            iterations_completed,
            convergence_score,
            ..Default::default()
        }
    }
    
    /// Get depth efficiency (convergence per time)
    pub fn efficiency(&self) -> f64 {
        if self.elapsed_time.as_secs_f64() > 0.0 {
            self.convergence_score / self.elapsed_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get iterations per second at this depth
    pub fn iterations_per_second(&self) -> f64 {
        if self.elapsed_time.as_secs_f64() > 0.0 {
            self.iterations_completed as f64 / self.elapsed_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get memory efficiency (convergence per memory used)
    pub fn memory_efficiency(&self) -> f64 {
        if self.memory_usage > 0 {
            self.convergence_score / self.memory_usage as f64
        } else {
            0.0
        }
    }
    
    /// Check if depth result shows good progress
    pub fn shows_good_progress(&self) -> bool {
        self.convergence_score > 0.6 && self.improvement_delta > 0.0
    }
    
    /// Get amplification effectiveness
    pub fn amplification_effectiveness(&self) -> f64 {
        if self.nodes_amplified > 0 {
            self.improvement_delta * self.amplification_factor
        } else {
            0.0
        }
    }
}

/// Quality metrics for simulation analysis
#[derive(Debug, Clone, Default)]
pub struct QualityMetrics {
    /// Average quality score
    pub average_quality: f64,
    /// Quality standard deviation
    pub quality_std_dev: f64,
    /// Minimum quality observed
    pub min_quality: f64,
    /// Maximum quality observed
    pub max_quality: f64,
    /// Quality trend (improving/degrading)
    pub quality_trend: QualityTrend,
}

impl QualityMetrics {
    /// Create new quality metrics
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Update metrics with new quality sample
    pub fn update_with_sample(&mut self, quality: f64, sample_count: usize) {
        if sample_count == 1 {
            self.average_quality = quality;
            self.min_quality = quality;
            self.max_quality = quality;
            self.quality_std_dev = 0.0;
        } else {
            // Update running statistics
            let old_avg = self.average_quality;
            self.average_quality = (self.average_quality * (sample_count - 1) as f64 + quality) / sample_count as f64;
            
            // Update min/max
            self.min_quality = self.min_quality.min(quality);
            self.max_quality = self.max_quality.max(quality);
            
            // Simple trend analysis
            if quality > old_avg {
                self.quality_trend = QualityTrend::Improving;
            } else if quality < old_avg {
                self.quality_trend = QualityTrend::Degrading;
            } else {
                self.quality_trend = QualityTrend::Stable;
            }
        }
    }
    
    /// Get quality consistency (1.0 - coefficient of variation)
    pub fn consistency(&self) -> f64 {
        if self.average_quality > 0.0 {
            1.0 - (self.quality_std_dev / self.average_quality)
        } else {
            0.0
        }
    }
    
    /// Check if quality metrics are good
    pub fn is_good(&self) -> bool {
        self.average_quality > 0.7 && self.consistency() > 0.8
    }
}

/// Quality trend enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QualityTrend {
    #[default]
    Stable,
    Improving,
    Degrading,
    Insufficient,
}

/// Batch simulation results for efficient processing
#[derive(Debug, Clone)]
pub struct BatchSimulationResult {
    /// Individual simulation results
    pub results: Vec<SimulationResult>,
    /// Batch statistics
    pub batch_stats: BatchStats,
    /// Total batch time
    pub total_batch_time: Duration,
}

impl BatchSimulationResult {
    /// Create new batch result
    pub fn new(results: Vec<SimulationResult>) -> Self {
        let total_batch_time = results.iter()
            .map(|r| r.simulation_time)
            .sum();
        
        let batch_stats = BatchStats::from_results(&results);
        
        Self {
            results,
            batch_stats,
            total_batch_time,
        }
    }
    
    /// Get batch success rate
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        
        let successful = self.results.iter().filter(|r| r.is_successful()).count();
        successful as f64 / self.results.len() as f64
    }
    
    /// Get average batch quality
    pub fn average_quality(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        
        let total_quality: f64 = self.results.iter().map(|r| r.simulation_quality).sum();
        total_quality / self.results.len() as f64
    }
    
    /// Get batch throughput (simulations per second)
    pub fn throughput(&self) -> f64 {
        if self.total_batch_time.as_secs_f64() > 0.0 {
            self.results.len() as f64 / self.total_batch_time.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Batch statistics for comprehensive analysis
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Total simulations in batch
    pub total_simulations: usize,
    /// Successful simulations
    pub successful_simulations: usize,
    /// Average reward magnitude
    pub average_reward: f64,
    /// Total processing time
    pub total_time: Duration,
    /// Quality distribution
    pub quality_distribution: QualityDistribution,
}

impl BatchStats {
    /// Create batch stats from simulation results
    pub fn from_results(results: &[SimulationResult]) -> Self {
        if results.is_empty() {
            return Self::default();
        }
        
        let total_simulations = results.len();
        let successful_simulations = results.iter().filter(|r| r.is_successful()).count();
        
        let total_reward: f64 = results.iter().map(|r| r.reward_magnitude()).sum();
        let average_reward = total_reward / total_simulations as f64;
        
        let total_time: Duration = results.iter().map(|r| r.simulation_time).sum();
        
        let quality_distribution = QualityDistribution::from_results(results);
        
        Self {
            total_simulations,
            successful_simulations,
            average_reward,
            total_time,
            quality_distribution,
        }
    }
}

/// Quality distribution analysis
#[derive(Debug, Clone, Default)]
pub struct QualityDistribution {
    /// Samples in each quality range
    pub low_quality: usize,      // 0.0 - 0.3
    pub medium_quality: usize,   // 0.3 - 0.7
    pub high_quality: usize,     // 0.7 - 1.0
}

impl QualityDistribution {
    /// Create quality distribution from results
    pub fn from_results(results: &[SimulationResult]) -> Self {
        let mut distribution = Self::default();
        
        for result in results {
            if result.simulation_quality < 0.3 {
                distribution.low_quality += 1;
            } else if result.simulation_quality < 0.7 {
                distribution.medium_quality += 1;
            } else {
                distribution.high_quality += 1;
            }
        }
        
        distribution
    }
    
    /// Get percentage of high-quality simulations
    pub fn high_quality_percentage(&self) -> f64 {
        let total = self.low_quality + self.medium_quality + self.high_quality;
        if total > 0 {
            self.high_quality as f64 / total as f64
        } else {
            0.0
        }
    }
    
    /// Check if distribution is healthy
    pub fn is_healthy(&self) -> bool {
        self.high_quality_percentage() > 0.6
    }
}
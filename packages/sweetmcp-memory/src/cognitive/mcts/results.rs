//! MCTS results and performance metrics
//!
//! This module provides blazing-fast MCTS result types with zero allocation
//! optimizations and elegant ergonomic interfaces for result analysis and reporting.

use super::{
    types::CodeState,
    execution::{ExecutionResult, ExecutionSummary},
    analysis::{TreeStructureAnalysis, Bottleneck},
    actions::CoordinatorStatistics,
};

/// Comprehensive MCTS result
#[derive(Debug, Clone)]
pub struct MCTSResult {
    pub execution_result: ExecutionResult,
    pub tree_analysis: TreeStructureAnalysis,
    pub best_modification: Option<CodeState>,
    pub best_path: Vec<String>,
    pub bottlenecks: Vec<Bottleneck>,
    pub coordinator_stats: CoordinatorStatistics,
}

impl MCTSResult {
    /// Create new MCTS result
    #[inline]
    pub fn new(
        execution_result: ExecutionResult,
        tree_analysis: TreeStructureAnalysis,
        best_modification: Option<CodeState>,
        best_path: Vec<String>,
        bottlenecks: Vec<Bottleneck>,
        coordinator_stats: CoordinatorStatistics,
    ) -> Self {
        Self {
            execution_result,
            tree_analysis,
            best_modification,
            best_path,
            bottlenecks,
            coordinator_stats,
        }
    }

    /// Check if MCTS run was successful
    #[inline]
    pub fn is_successful(&self) -> bool {
        self.execution_result.is_successful() &&
        self.best_modification.is_some() &&
        !self.best_path.is_empty()
    }

    /// Get overall quality score
    #[inline]
    pub fn quality_score(&self) -> f64 {
        let execution_quality = self.execution_result.quality_score();
        let tree_quality = if self.tree_analysis.total_nodes > 0 {
            self.tree_analysis.best_path_reward / self.tree_analysis.max_depth as f64
        } else {
            0.0
        };
        let coordinator_quality = self.coordinator_stats.overall_efficiency();

        (execution_quality * 0.5 + tree_quality * 0.3 + coordinator_quality * 0.2).clamp(0.0, 1.0)
    }

    /// Generate comprehensive report
    #[inline]
    pub fn generate_report(&self) -> String {
        format!(
            "MCTS Result Report:\n\
             \n\
             Execution:\n\
             - Iterations: {}\n\
             - Time: {:?}\n\
             - Efficiency: {:.2}\n\
             - Converged: {}\n\
             \n\
             Tree Analysis:\n\
             - Total Nodes: {}\n\
             - Max Depth: {}\n\
             - Best Reward: {:.3}\n\
             - Avg Branching: {:.2}\n\
             \n\
             Best Solution:\n\
             - Actions: {:?}\n\
             - Performance: {:.3}\n\
             \n\
             Issues:\n\
             - Bottlenecks: {}\n\
             - Quality Score: {:.2}",
            self.execution_result.iterations_completed,
            self.execution_result.execution_time,
            self.execution_result.efficiency.overall_efficiency,
            self.execution_result.converged,
            self.tree_analysis.total_nodes,
            self.tree_analysis.max_depth,
            self.tree_analysis.best_path_reward,
            self.tree_analysis.average_branching_factor,
            self.best_path,
            self.best_modification.as_ref().map(|s| s.performance_score()).unwrap_or(0.0),
            self.bottlenecks.len(),
            self.quality_score()
        )
    }

    /// Get performance improvement percentage
    #[inline]
    pub fn performance_improvement(&self) -> f64 {
        self.best_modification
            .as_ref()
            .map(|state| {
                // Calculate improvement based on performance score
                let baseline_score = 1.0; // Assume baseline performance of 1.0
                let current_score = state.performance_score();
                ((current_score - baseline_score) / baseline_score * 100.0).max(0.0)
            })
            .unwrap_or(0.0)
    }

    /// Check if result meets quality threshold
    #[inline]
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.quality_score() >= threshold
    }

    /// Get execution efficiency
    #[inline]
    pub fn execution_efficiency(&self) -> f64 {
        self.execution_result.efficiency.overall_efficiency
    }

    /// Get tree exploration efficiency
    #[inline]
    pub fn exploration_efficiency(&self) -> f64 {
        if self.tree_analysis.total_nodes == 0 {
            0.0
        } else {
            self.tree_analysis.best_path_reward / self.tree_analysis.total_nodes as f64
        }
    }

    /// Get coordinator efficiency
    #[inline]
    pub fn coordinator_efficiency(&self) -> f64 {
        self.coordinator_stats.overall_efficiency()
    }

    /// Check if convergence was achieved
    #[inline]
    pub fn converged(&self) -> bool {
        self.execution_result.converged
    }

    /// Get total iterations completed
    #[inline]
    pub fn iterations_completed(&self) -> u64 {
        self.execution_result.iterations_completed
    }

    /// Get execution time
    #[inline]
    pub fn execution_time(&self) -> std::time::Duration {
        self.execution_result.execution_time
    }

    /// Get number of bottlenecks found
    #[inline]
    pub fn bottleneck_count(&self) -> usize {
        self.bottlenecks.len()
    }

    /// Check if result has critical bottlenecks
    #[inline]
    pub fn has_critical_bottlenecks(&self) -> bool {
        self.bottlenecks.iter().any(|b| b.is_critical())
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub tree_nodes: usize,
    pub tree_memory_bytes: usize,
    pub coordinator_memory_bytes: usize,
    pub total_memory_bytes: usize,
}

impl MemoryUsage {
    /// Create new memory usage statistics
    #[inline]
    pub fn new(
        tree_nodes: usize,
        tree_memory_bytes: usize,
        coordinator_memory_bytes: usize,
    ) -> Self {
        Self {
            tree_nodes,
            tree_memory_bytes,
            coordinator_memory_bytes,
            total_memory_bytes: tree_memory_bytes + coordinator_memory_bytes,
        }
    }

    /// Get memory usage in megabytes
    #[inline]
    pub fn total_mb(&self) -> f64 {
        self.total_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get tree memory usage in megabytes
    #[inline]
    pub fn tree_mb(&self) -> f64 {
        self.tree_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get coordinator memory usage in megabytes
    #[inline]
    pub fn coordinator_mb(&self) -> f64 {
        self.coordinator_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Check if memory usage is acceptable
    #[inline]
    pub fn is_acceptable(&self, max_mb: f64) -> bool {
        self.total_mb() <= max_mb
    }

    /// Get memory efficiency score
    #[inline]
    pub fn efficiency_score(&self) -> f64 {
        if self.tree_nodes == 0 {
            0.0
        } else {
            // Lower bytes per node is better
            let bytes_per_node = self.tree_memory_bytes as f64 / self.tree_nodes as f64;
            (1000.0 / (bytes_per_node + 1.0)).min(1.0)
        }
    }

    /// Get memory distribution breakdown
    #[inline]
    pub fn distribution(&self) -> MemoryDistribution {
        let tree_percentage = if self.total_memory_bytes > 0 {
            (self.tree_memory_bytes as f64 / self.total_memory_bytes as f64) * 100.0
        } else {
            0.0
        };

        let coordinator_percentage = if self.total_memory_bytes > 0 {
            (self.coordinator_memory_bytes as f64 / self.total_memory_bytes as f64) * 100.0
        } else {
            0.0
        };

        MemoryDistribution {
            tree_percentage,
            coordinator_percentage,
        }
    }

    /// Generate memory usage report
    #[inline]
    pub fn generate_report(&self) -> String {
        let distribution = self.distribution();
        format!(
            "Memory Usage Report:\n\
             - Total: {:.1} MB\n\
             - Tree: {:.1} MB ({:.1}%)\n\
             - Coordinator: {:.1} MB ({:.1}%)\n\
             - Nodes: {}\n\
             - Efficiency: {:.2}",
            self.total_mb(),
            self.tree_mb(),
            distribution.tree_percentage,
            self.coordinator_mb(),
            distribution.coordinator_percentage,
            self.tree_nodes,
            self.efficiency_score()
        )
    }
}

/// Memory distribution breakdown
#[derive(Debug, Clone)]
pub struct MemoryDistribution {
    pub tree_percentage: f64,
    pub coordinator_percentage: f64,
}

/// Performance summary for MCTS operations
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_nodes: usize,
    pub total_visits: u64,
    pub best_performance_score: f64,
    pub average_branching_factor: f64,
    pub max_depth: usize,
    pub memory_usage_mb: f64,
    pub coordinator_efficiency: f64,
    pub convergence_rate: f64,
}

impl PerformanceSummary {
    /// Create new performance summary
    #[inline]
    pub fn new(
        total_nodes: usize,
        total_visits: u64,
        best_performance_score: f64,
        average_branching_factor: f64,
        max_depth: usize,
        memory_usage_mb: f64,
        coordinator_efficiency: f64,
        convergence_rate: f64,
    ) -> Self {
        Self {
            total_nodes,
            total_visits,
            best_performance_score,
            average_branching_factor,
            max_depth,
            memory_usage_mb,
            coordinator_efficiency,
            convergence_rate,
        }
    }

    /// Calculate overall performance score
    #[inline]
    pub fn overall_performance(&self) -> f64 {
        let exploration_score = (self.total_nodes as f64 / 1000.0).min(1.0);
        let efficiency_score = self.coordinator_efficiency;
        let convergence_score = self.convergence_rate;
        let memory_score = (10.0 / (self.memory_usage_mb + 1.0)).min(1.0);

        (exploration_score * 0.3 + 
         efficiency_score * 0.3 + 
         convergence_score * 0.2 + 
         memory_score * 0.2).clamp(0.0, 1.0)
    }

    /// Generate performance report
    #[inline]
    pub fn generate_report(&self) -> String {
        format!(
            "MCTS Performance Summary:\n\
             - Nodes Explored: {}\n\
             - Total Visits: {}\n\
             - Best Score: {:.3}\n\
             - Branching Factor: {:.2}\n\
             - Max Depth: {}\n\
             - Memory Usage: {:.1} MB\n\
             - Coordinator Efficiency: {:.2}\n\
             - Convergence Rate: {:.2}\n\
             - Overall Performance: {:.2}",
            self.total_nodes,
            self.total_visits,
            self.best_performance_score,
            self.average_branching_factor,
            self.max_depth,
            self.memory_usage_mb,
            self.coordinator_efficiency,
            self.convergence_rate,
            self.overall_performance()
        )
    }

    /// Check if performance is satisfactory
    #[inline]
    pub fn is_satisfactory(&self, threshold: f64) -> bool {
        self.overall_performance() >= threshold
    }

    /// Get exploration efficiency
    #[inline]
    pub fn exploration_efficiency(&self) -> f64 {
        if self.total_visits == 0 {
            0.0
        } else {
            self.total_nodes as f64 / self.total_visits as f64
        }
    }

    /// Get depth efficiency
    #[inline]
    pub fn depth_efficiency(&self) -> f64 {
        if self.max_depth == 0 {
            0.0
        } else {
            self.best_performance_score / self.max_depth as f64
        }
    }

    /// Get memory efficiency
    #[inline]
    pub fn memory_efficiency(&self) -> f64 {
        if self.memory_usage_mb == 0.0 {
            1.0
        } else {
            (self.total_nodes as f64 / self.memory_usage_mb).min(1000.0) / 1000.0
        }
    }

    /// Check if convergence was achieved
    #[inline]
    pub fn converged(&self) -> bool {
        self.convergence_rate > 0.8
    }

    /// Get performance grade
    #[inline]
    pub fn performance_grade(&self) -> char {
        let score = self.overall_performance();
        match score {
            s if s >= 0.9 => 'A',
            s if s >= 0.8 => 'B',
            s if s >= 0.7 => 'C',
            s if s >= 0.6 => 'D',
            _ => 'F',
        }
    }
}
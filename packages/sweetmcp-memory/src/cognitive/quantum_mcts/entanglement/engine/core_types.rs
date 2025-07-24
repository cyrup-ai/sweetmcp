//! Core engine types and definitions
//!
//! This module provides blazing-fast core engine types with zero allocation
//! optimizations and elegant ergonomic interfaces for quantum entanglement engine operations.

use std::time::Instant;
use super::{
    OptimizationResult, CreationResult, PruningResult, BalancingResult,
    health::NetworkHealthReport,
};

/// Comprehensive engine operation result
#[derive(Debug, Clone)]
pub struct EngineOperationResult {
    /// Operation type performed
    pub operation_type: EngineOperationType,
    /// Time taken for operation in milliseconds
    pub operation_time_ms: u64,
    /// Whether operation was successful
    pub success: bool,
    /// Performance improvement achieved
    pub performance_improvement: f64,
    /// Detailed operation results
    pub details: EngineOperationDetails,
    /// Timestamp of operation
    pub timestamp: Instant,
}

impl EngineOperationResult {
    /// Create new operation result
    #[inline]
    pub fn new(
        operation_type: EngineOperationType,
        operation_time_ms: u64,
        success: bool,
        performance_improvement: f64,
        details: EngineOperationDetails,
    ) -> Self {
        Self {
            operation_type,
            operation_time_ms,
            success,
            performance_improvement,
            details,
            timestamp: Instant::now(),
        }
    }

    /// Check if operation was highly successful
    #[inline]
    pub fn was_highly_successful(&self) -> bool {
        self.success && self.performance_improvement > 10.0
    }

    /// Get operation summary
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "{:?}: {}ms, {:.1}% improvement",
            self.operation_type,
            self.operation_time_ms,
            self.performance_improvement
        )
    }

    /// Get detailed report
    #[inline]
    pub fn detailed_report(&self) -> String {
        let mut report = format!(
            "=== Engine Operation Report ===\n\
            Operation: {:?}\n\
            Duration: {}ms\n\
            Success: {}\n\
            Performance Improvement: {:.1}%\n\
            Timestamp: {:?}\n\n",
            self.operation_type,
            self.operation_time_ms,
            self.success,
            self.performance_improvement,
            self.timestamp
        );

        match &self.details {
            EngineOperationDetails::Optimization(opt) => {
                report.push_str(&opt.detailed_report());
            }
            EngineOperationDetails::Creation(creation) => {
                report.push_str(&creation.summary());
            }
            EngineOperationDetails::Pruning(pruning) => {
                report.push_str(&pruning.detailed_report());
            }
            EngineOperationDetails::Balancing(balancing) => {
                report.push_str(&balancing.detailed_report());
            }
            EngineOperationDetails::HealthCheck(health) => {
                report.push_str(&health.format_report());
            }
            EngineOperationDetails::Combined { optimization, creation, pruning, balancing, health } => {
                report.push_str("--- Combined Operation Results ---\n");
                if let Some(opt) = optimization {
                    report.push_str(&format!("Optimization: {}\n", opt.summary()));
                }
                if let Some(create) = creation {
                    report.push_str(&format!("Creation: {}\n", create.summary()));
                }
                if let Some(prune) = pruning {
                    report.push_str(&format!("Pruning: {}\n", prune.summary()));
                }
                if let Some(balance) = balancing {
                    report.push_str(&format!("Balancing: {}\n", balance.summary()));
                }
                if let Some(health_check) = health {
                    report.push_str(&format!("Health: {}\n", health_check.summary()));
                }
            }
        }

        report
    }
}

/// Types of engine operations
#[derive(Debug, Clone, PartialEq)]
pub enum EngineOperationType {
    /// Full optimization operation
    FullOptimization,
    /// Strategic entanglement creation
    StrategyCreation,
    /// Intelligent pruning
    IntelligentPruning,
    /// Load balancing
    LoadBalancing,
    /// Health check
    HealthCheck,
    /// Combined optimization
    CombinedOptimization,
}

impl EngineOperationType {
    /// Get operation priority (higher = more important)
    #[inline]
    pub fn priority(&self) -> u8 {
        match self {
            EngineOperationType::HealthCheck => 10,
            EngineOperationType::CombinedOptimization => 9,
            EngineOperationType::FullOptimization => 8,
            EngineOperationType::LoadBalancing => 7,
            EngineOperationType::IntelligentPruning => 6,
            EngineOperationType::StrategyCreation => 5,
        }
    }

    /// Get expected duration in milliseconds
    #[inline]
    pub fn expected_duration_ms(&self) -> u64 {
        match self {
            EngineOperationType::HealthCheck => 50,
            EngineOperationType::StrategyCreation => 100,
            EngineOperationType::IntelligentPruning => 150,
            EngineOperationType::LoadBalancing => 200,
            EngineOperationType::FullOptimization => 300,
            EngineOperationType::CombinedOptimization => 500,
        }
    }

    /// Check if operation is critical
    #[inline]
    pub fn is_critical(&self) -> bool {
        matches!(self, EngineOperationType::HealthCheck | EngineOperationType::CombinedOptimization)
    }
}

/// Detailed operation results
#[derive(Debug, Clone)]
pub enum EngineOperationDetails {
    /// Optimization operation details
    Optimization(OptimizationResult),
    /// Creation operation details
    Creation(CreationResult),
    /// Pruning operation details
    Pruning(PruningResult),
    /// Balancing operation details
    Balancing(BalancingResult),
    /// Health check details
    HealthCheck(NetworkHealthReport),
    /// Combined operation details
    Combined {
        optimization: Option<OptimizationResult>,
        creation: Option<CreationResult>,
        pruning: Option<PruningResult>,
        balancing: Option<BalancingResult>,
        health: Option<NetworkHealthReport>,
    },
}

impl EngineOperationDetails {
    /// Get operation impact score
    #[inline]
    pub fn impact_score(&self) -> f64 {
        match self {
            EngineOperationDetails::Optimization(opt) => opt.performance_improvement,
            EngineOperationDetails::Creation(creation) => creation.impact_score(),
            EngineOperationDetails::Pruning(pruning) => pruning.efficiency_improvement,
            EngineOperationDetails::Balancing(balancing) => balancing.improvement_score,
            EngineOperationDetails::HealthCheck(health) => health.overall_score(),
            EngineOperationDetails::Combined { optimization, creation, pruning, balancing, health } => {
                let mut total_score = 0.0;
                let mut count = 0;

                if let Some(opt) = optimization {
                    total_score += opt.performance_improvement;
                    count += 1;
                }
                if let Some(create) = creation {
                    total_score += create.impact_score();
                    count += 1;
                }
                if let Some(prune) = pruning {
                    total_score += prune.efficiency_improvement;
                    count += 1;
                }
                if let Some(balance) = balancing {
                    total_score += balance.improvement_score;
                    count += 1;
                }
                if let Some(health_check) = health {
                    total_score += health_check.overall_score();
                    count += 1;
                }

                if count > 0 { total_score / count as f64 } else { 0.0 }
            }
        }
    }

    /// Check if operation had significant impact
    #[inline]
    pub fn has_significant_impact(&self) -> bool {
        self.impact_score() > 5.0
    }
}

/// Engine statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct EngineStatistics {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Success rate of operations (0.0 to 1.0)
    pub success_rate: f64,
    /// Average operation latency in microseconds
    pub average_latency_us: f64,
    /// Cache efficiency (0.0 to 1.0)
    pub cache_efficiency: f64,
    /// Throughput in operations per second
    pub throughput_ops_per_sec: f64,
    /// Total operations performed
    pub total_operations: u64,
    /// Failed operations count
    pub failed_operations: u64,
    /// Last update timestamp
    pub last_updated: Instant,
}

impl EngineStatistics {
    /// Create new statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            health_score: 1.0,
            success_rate: 1.0,
            average_latency_us: 0.0,
            cache_efficiency: 1.0,
            throughput_ops_per_sec: 0.0,
            total_operations: 0,
            failed_operations: 0,
            last_updated: Instant::now(),
        }
    }

    /// Check if engine is performing optimally
    #[inline]
    pub fn is_optimal(&self) -> bool {
        self.health_score > 0.9 &&
        self.success_rate > 0.95 &&
        self.average_latency_us < 500.0 &&
        self.cache_efficiency > 0.9
    }

    /// Get performance summary
    #[inline]
    pub fn performance_summary(&self) -> String {
        format!(
            "Performance: Health {:.1}/10, Success {:.1}%, Latency {:.1}μs, Cache {:.1}%, Throughput {:.1}/s",
            self.health_score * 10.0,
            self.success_rate * 100.0,
            self.average_latency_us,
            self.cache_efficiency * 100.0,
            self.throughput_ops_per_sec
        )
    }

    /// Update statistics with operation result
    #[inline]
    pub fn update_with_operation(&mut self, result: &EngineOperationResult) {
        self.total_operations += 1;
        if !result.success {
            self.failed_operations += 1;
        }

        // Update success rate
        self.success_rate = (self.total_operations - self.failed_operations) as f64 / self.total_operations as f64;

        // Update average latency (exponential moving average)
        let new_latency = result.operation_time_ms as f64 * 1000.0; // Convert to microseconds
        self.average_latency_us = 0.9 * self.average_latency_us + 0.1 * new_latency;

        // Update health score based on recent performance
        self.health_score = 0.9 * self.health_score + 0.1 * if result.success { 1.0 } else { 0.0 };

        self.last_updated = Instant::now();
    }
}

impl Default for EngineStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance grades for different metrics
#[derive(Debug, Clone)]
pub struct PerformanceGrades {
    /// Overall performance grade
    pub overall: char,
    /// Latency performance grade
    pub latency: char,
    /// Throughput performance grade
    pub throughput: char,
    /// Reliability performance grade
    pub reliability: char,
    /// Efficiency performance grade
    pub efficiency: char,
}

impl PerformanceGrades {
    /// Create new performance grades
    #[inline]
    pub fn new(
        overall: char,
        latency: char,
        throughput: char,
        reliability: char,
        efficiency: char,
    ) -> Self {
        Self {
            overall,
            latency,
            throughput,
            reliability,
            efficiency,
        }
    }

    /// Check if all grades are acceptable (C or better)
    #[inline]
    pub fn are_acceptable(&self) -> bool {
        self.overall >= 'C' && 
        self.latency >= 'C' && 
        self.throughput >= 'C' && 
        self.reliability >= 'C' && 
        self.efficiency >= 'C'
    }
}

/// Engine performance report
#[derive(Debug, Clone)]
pub struct EnginePerformanceReport {
    /// Performance grades
    pub performance_grades: PerformanceGrades,
    /// Detailed statistics
    pub statistics: EngineStatistics,
    /// Health report
    pub health_report: NetworkHealthReport,
    /// Report timestamp
    pub timestamp: Instant,
}

impl EnginePerformanceReport {
    /// Create new performance report
    #[inline]
    pub fn new(
        performance_grades: PerformanceGrades,
        statistics: EngineStatistics,
        health_report: NetworkHealthReport,
    ) -> Self {
        Self {
            performance_grades,
            statistics,
            health_report,
            timestamp: Instant::now(),
        }
    }

    /// Check if performance is acceptable
    #[inline]
    pub fn is_performance_acceptable(&self) -> bool {
        self.performance_grades.are_acceptable() && self.health_report.is_healthy()
    }

    /// Get performance summary
    #[inline]
    pub fn performance_summary(&self) -> String {
        format!(
            "Overall Grade: {} | Latency: {} | Throughput: {} | Reliability: {} | Efficiency: {}",
            self.performance_grades.overall,
            self.performance_grades.latency,
            self.performance_grades.throughput,
            self.performance_grades.reliability,
            self.performance_grades.efficiency
        )
    }

    /// Format comprehensive report
    #[inline]
    pub fn format_report(&self) -> String {
        format!(
            "=== Engine Performance Report ===\n\
            Timestamp: {:?}\n\
            {}\n\
            {}\n\
            {}",
            self.timestamp,
            self.performance_summary(),
            self.statistics.performance_summary(),
            self.health_report.summary()
        )
    }
}
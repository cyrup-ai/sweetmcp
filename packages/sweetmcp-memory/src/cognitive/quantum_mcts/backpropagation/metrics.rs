//! Backpropagation metrics and result types
//!
//! This module provides comprehensive performance metrics, result structures,
//! and analysis capabilities for quantum backpropagation operations.

use serde::Serialize;
use std::time::Duration;
use crate::cognitive::quantum::Complex64;

/// Comprehensive backpropagation performance metrics
#[derive(Debug, Clone, Default, Serialize)]
pub struct BackpropagationMetrics {
    /// Total number of backpropagations performed
    pub backpropagations_performed: u64,
    /// Total nodes updated across all backpropagations
    pub total_nodes_updated: usize,
    /// Cumulative time spent in backpropagation operations
    pub total_backpropagation_time: Duration,
    /// Total reward magnitude distributed
    pub total_reward_distributed: f64,
    /// Number of batch operations executed
    pub batch_operations: u64,
    /// Number of adaptive backpropagations performed
    pub adaptive_backpropagations: u64,
    /// Number of reward normalization operations
    pub normalization_operations: u64,
    /// Number of cache hits during path traversal
    pub cache_hits: u64,
    /// Number of cache misses during path traversal
    pub cache_misses: u64,
    /// Number of entanglement effects processed
    pub entanglement_effects_processed: u64,
}

impl BackpropagationMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Calculate average nodes updated per backpropagation
    pub fn average_nodes_per_backprop(&self) -> f64 {
        if self.backpropagations_performed > 0 {
            self.total_nodes_updated as f64 / self.backpropagations_performed as f64
        } else {
            0.0
        }
    }
    
    /// Calculate backpropagation throughput (operations per second)
    pub fn throughput(&self) -> f64 {
        let total_seconds = self.total_backpropagation_time.as_secs_f64();
        if total_seconds > 0.0 {
            self.backpropagations_performed as f64 / total_seconds
        } else {
            0.0
        }
    }
    
    /// Calculate average reward per backpropagation
    pub fn average_reward_per_backprop(&self) -> f64 {
        if self.backpropagations_performed > 0 {
            self.total_reward_distributed / self.backpropagations_performed as f64
        } else {
            0.0
        }
    }
    
    /// Calculate cache hit rate percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_requests = self.cache_hits + self.cache_misses;
        if total_cache_requests > 0 {
            self.cache_hits as f64 / total_cache_requests as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate nodes per second processing rate
    pub fn nodes_per_second(&self) -> f64 {
        let total_seconds = self.total_backpropagation_time.as_secs_f64();
        if total_seconds > 0.0 {
            self.total_nodes_updated as f64 / total_seconds
        } else {
            0.0
        }
    }
    
    /// Calculate adaptive backpropagation ratio
    pub fn adaptive_ratio(&self) -> f64 {
        if self.backpropagations_performed > 0 {
            self.adaptive_backpropagations as f64 / self.backpropagations_performed as f64
        } else {
            0.0
        }
    }
    
    /// Calculate entanglement processing rate
    pub fn entanglement_effects_per_backprop(&self) -> f64 {
        if self.backpropagations_performed > 0 {
            self.entanglement_effects_processed as f64 / self.backpropagations_performed as f64
        } else {
            0.0
        }
    }
    
    /// Get performance grade based on metrics
    pub fn performance_grade(&self) -> char {
        let throughput_score = (self.throughput() / 50.0).min(1.0); // Good at 50+ ops/sec
        let cache_score = self.cache_hit_rate() / 100.0; // Convert percentage to 0-1
        let efficiency_score = (self.nodes_per_second() / 1000.0).min(1.0); // Good at 1000+ nodes/sec
        
        let overall_score = (throughput_score + cache_score + efficiency_score) / 3.0 * 100.0;
        
        match overall_score as u32 {
            90..=100 => 'A',
            80..=89 => 'B',
            70..=79 => 'C',
            60..=69 => 'D',
            _ => 'F',
        }
    }
    
    /// Check if metrics indicate good performance
    pub fn is_performing_well(&self) -> bool {
        self.performance_grade() >= 'B' && 
        self.cache_hit_rate() > 70.0 &&
        self.throughput() > 10.0
    }
    
    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Merge metrics from another instance
    pub fn merge(&mut self, other: &BackpropagationMetrics) {
        self.backpropagations_performed += other.backpropagations_performed;
        self.total_nodes_updated += other.total_nodes_updated;
        self.total_backpropagation_time += other.total_backpropagation_time;
        self.total_reward_distributed += other.total_reward_distributed;
        self.batch_operations += other.batch_operations;
        self.adaptive_backpropagations += other.adaptive_backpropagations;
        self.normalization_operations += other.normalization_operations;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
        self.entanglement_effects_processed += other.entanglement_effects_processed;
    }
}

/// Single backpropagation operation result
#[derive(Debug, Clone, Serialize)]
pub struct BackpropagationResult {
    /// Number of nodes that were updated
    pub nodes_updated: usize,
    /// Length of the propagation path traversed
    pub path_length: usize,
    /// Total reward distributed during backpropagation
    pub reward_distributed: Complex64,
    /// Number of entanglement effects that were applied
    pub entanglement_effects_applied: usize,
    /// Time taken for the backpropagation operation
    pub elapsed_time: Duration,
    /// Whether the backpropagation operation succeeded
    pub success: bool,
}

impl BackpropagationResult {
    /// Calculate backpropagation efficiency (nodes updated per millisecond)
    pub fn efficiency(&self) -> f64 {
        let total_ms = self.elapsed_time.as_secs_f64() * 1000.0;
        if total_ms > 0.0 {
            self.nodes_updated as f64 / total_ms
        } else {
            0.0
        }
    }
    
    /// Check if result indicates good performance
    pub fn is_efficient(&self) -> bool {
        self.success && self.efficiency() > 10.0 // More than 10 nodes per millisecond
    }
    
    /// Calculate reward distribution rate (reward magnitude per node)
    pub fn reward_per_node(&self) -> f64 {
        if self.nodes_updated > 0 {
            self.reward_distributed.norm() / self.nodes_updated as f64
        } else {
            0.0
        }
    }
    
    /// Calculate path utilization efficiency
    pub fn path_utilization(&self) -> f64 {
        if self.path_length > 0 {
            self.nodes_updated as f64 / self.path_length as f64
        } else {
            0.0
        }
    }
    
    /// Get performance summary string
    pub fn summary(&self) -> String {
        format!(
            "Backprop: {} nodes, {:.1}ms, {:.2} eff, {} entanglements",
            self.nodes_updated,
            self.elapsed_time.as_secs_f64() * 1000.0,
            self.efficiency(),
            self.entanglement_effects_applied
        )
    }
    
    /// Create failed result for error cases
    pub fn failed() -> Self {
        Self {
            nodes_updated: 0,
            path_length: 0,
            reward_distributed: Complex64::new(0.0, 0.0),
            entanglement_effects_applied: 0,
            elapsed_time: Duration::ZERO,
            success: false,
        }
    }
    
    /// Check if this result should trigger performance warnings
    pub fn needs_performance_attention(&self) -> bool {
        !self.is_efficient() || 
        self.elapsed_time > Duration::from_millis(100) ||
        (self.path_length > 0 && self.path_utilization() < 0.5)
    }
}

/// Reward normalization operation result
#[derive(Debug, Clone, Serialize)]
pub struct NormalizationResult {
    /// Number of nodes that were normalized
    pub nodes_normalized: usize,
    /// Average scaling factor applied to normalized nodes
    pub average_scaling_factor: f64,
    /// Maximum magnitude that was enforced
    pub max_magnitude_enforced: f64,
}

impl NormalizationResult {
    /// Check if significant normalization was needed
    pub fn significant_normalization(&self) -> bool {
        self.nodes_normalized > 0 && self.average_scaling_factor < 0.9
    }
    
    /// Calculate normalization severity (0.0 to 1.0)
    pub fn severity(&self) -> f64 {
        if self.nodes_normalized == 0 {
            return 0.0;
        }
        
        // Severity based on how much scaling was needed
        1.0 - self.average_scaling_factor.clamp(0.0, 1.0)
    }
    
    /// Check if normalization indicates numerical instability
    pub fn indicates_instability(&self) -> bool {
        self.severity() > 0.5 // More than 50% scaling needed
    }
    
    /// Get normalization summary
    pub fn summary(&self) -> String {
        format!(
            "Normalized {} nodes, avg scaling: {:.3}, max: {:.3}",
            self.nodes_normalized,
            self.average_scaling_factor,
            self.max_magnitude_enforced
        )
    }
}

/// Batch backpropagation performance analysis
#[derive(Debug, Clone, Serialize)]
pub struct BatchAnalysis {
    /// Total number of items in the batch
    pub total_items: usize,
    /// Number of successful backpropagations
    pub successful_items: usize,
    /// Number of failed backpropagations
    pub failed_items: usize,
    /// Total time for batch processing
    pub total_time: Duration,
    /// Average time per item
    pub average_time_per_item: Duration,
    /// Total nodes updated across all items
    pub total_nodes_updated: usize,
    /// Total reward distributed across all items
    pub total_reward_distributed: f64,
}

impl BatchAnalysis {
    /// Create batch analysis from results
    pub fn from_results(results: &[BackpropagationResult], total_time: Duration) -> Self {
        let total_items = results.len();
        let successful_items = results.iter().filter(|r| r.success).count();
        let failed_items = total_items - successful_items;
        
        let average_time_per_item = if total_items > 0 {
            total_time / total_items as u32
        } else {
            Duration::ZERO
        };
        
        let total_nodes_updated = results.iter().map(|r| r.nodes_updated).sum();
        let total_reward_distributed = results.iter()
            .map(|r| r.reward_distributed.norm())
            .sum();
        
        Self {
            total_items,
            successful_items,
            failed_items,
            total_time,
            average_time_per_item,
            total_nodes_updated,
            total_reward_distributed,
        }
    }
    
    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_items > 0 {
            self.successful_items as f64 / self.total_items as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate throughput (items per second)
    pub fn throughput(&self) -> f64 {
        let total_seconds = self.total_time.as_secs_f64();
        if total_seconds > 0.0 {
            self.total_items as f64 / total_seconds
        } else {
            0.0
        }
    }
    
    /// Calculate efficiency (nodes per second)
    pub fn efficiency(&self) -> f64 {
        let total_seconds = self.total_time.as_secs_f64();
        if total_seconds > 0.0 {
            self.total_nodes_updated as f64 / total_seconds
        } else {
            0.0
        }
    }
    
    /// Check if batch performance is acceptable
    pub fn is_acceptable(&self) -> bool {
        self.success_rate() > 80.0 && self.throughput() > 5.0
    }
    
    /// Get performance summary
    pub fn summary(&self) -> String {
        format!(
            "Batch: {}/{} success ({:.1}%), {:.1} items/sec, {:.0} nodes/sec",
            self.successful_items,
            self.total_items,
            self.success_rate(),
            self.throughput(),
            self.efficiency()
        )
    }
}

/// Performance trend indicators for backpropagation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Volatile,
    Insufficient, // Not enough data
}

impl PerformanceTrend {
    /// Check if trend is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, PerformanceTrend::Improving | PerformanceTrend::Stable)
    }
    
    /// Get trend score (-1.0 to 1.0)
    pub fn score(&self) -> f64 {
        match self {
            PerformanceTrend::Improving => 1.0,
            PerformanceTrend::Stable => 0.5,
            PerformanceTrend::Declining => -0.5,
            PerformanceTrend::Volatile => -1.0,
            PerformanceTrend::Insufficient => 0.0,
        }
    }
    
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Performance is improving over time",
            PerformanceTrend::Stable => "Performance is stable",
            PerformanceTrend::Declining => "Performance is declining",
            PerformanceTrend::Volatile => "Performance is highly variable",
            PerformanceTrend::Insufficient => "Insufficient data for trend analysis",
        }
    }
}

/// Backpropagation strategy performance comparison
#[derive(Debug, Clone, Serialize)]
pub struct StrategyComparison {
    /// Standard backpropagation metrics
    pub standard_metrics: BackpropagationMetrics,
    /// Adaptive backpropagation metrics
    pub adaptive_metrics: BackpropagationMetrics,
    /// Batch processing metrics
    pub batch_metrics: BackpropagationMetrics,
    /// Performance trend over time
    pub trend: PerformanceTrend,
}

impl StrategyComparison {
    /// Identify the best performing strategy
    pub fn best_strategy(&self) -> &str {
        let standard_score = self.standard_metrics.throughput() * 0.5 + 
                           self.standard_metrics.cache_hit_rate() * 0.5;
        let adaptive_score = self.adaptive_metrics.throughput() * 0.5 + 
                           self.adaptive_metrics.cache_hit_rate() * 0.5;
        let batch_score = self.batch_metrics.throughput() * 0.5 + 
                         self.batch_metrics.cache_hit_rate() * 0.5;
        
        if standard_score >= adaptive_score && standard_score >= batch_score {
            "standard"
        } else if adaptive_score >= batch_score {
            "adaptive"
        } else {
            "batch"
        }
    }
    
    /// Calculate overall performance score
    pub fn overall_score(&self) -> f64 {
        let standard_perf = if self.standard_metrics.is_performing_well() { 1.0 } else { 0.0 };
        let adaptive_perf = if self.adaptive_metrics.is_performing_well() { 1.0 } else { 0.0 };
        let batch_perf = if self.batch_metrics.is_performing_well() { 1.0 } else { 0.0 };
        let trend_score = (self.trend.score() + 1.0) / 2.0; // Convert to 0-1 range
        
        (standard_perf + adaptive_perf + batch_perf + trend_score) / 4.0
    }
    
    /// Get performance recommendations
    pub fn recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !self.standard_metrics.is_performing_well() {
            recommendations.push("Consider optimizing standard backpropagation caching".to_string());
        }
        
        if self.adaptive_metrics.adaptive_ratio() < 0.1 {
            recommendations.push("Increase use of adaptive backpropagation for better learning".to_string());
        }
        
        if self.batch_metrics.batch_operations < 10 {
            recommendations.push("Use batch processing for better throughput".to_string());
        }
        
        if self.trend == PerformanceTrend::Declining {
            recommendations.push("Investigate performance degradation causes".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is acceptable, maintain current strategies".to_string());
        }
        
        recommendations
    }
}
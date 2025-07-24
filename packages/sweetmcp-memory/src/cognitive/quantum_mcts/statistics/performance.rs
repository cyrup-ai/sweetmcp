//! Performance metrics and throughput analysis
//!
//! This module provides performance tracking with blazing-fast throughput analysis
//! and bottleneck identification for quantum MCTS optimization.

use serde::Serialize;
use super::collector::CounterSnapshot;

/// Performance analysis metrics with comprehensive throughput tracking
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceMetrics {
    /// Average visits per node
    pub avg_visits_per_node: f64,
    /// Nodes per second creation rate
    pub node_creation_rate: f64,
    /// Memory efficiency (bytes per node)
    pub memory_efficiency: f64,
    /// Cache hit rates
    pub cache_hit_rates: Vec<(String, f64)>,
    /// Throughput metrics
    pub throughput_metrics: ThroughputMetrics,
}

impl PerformanceMetrics {
    /// Create performance metrics from collector data
    pub fn from_collector_data(
        avg_visits_per_node: f64,
        node_creation_rate: f64,
        elapsed_seconds: f64,
        counters: &CounterSnapshot,
    ) -> Result<Self, crate::cognitive::types::CognitiveError> {
        let throughput_metrics = ThroughputMetrics::from_counters_and_time(counters, elapsed_seconds);
        
        // Placeholder cache hit rates
        let cache_hit_rates = vec![
            ("selection_cache".to_string(), 0.85),
            ("expansion_cache".to_string(), 0.75),
            ("reward_cache".to_string(), 0.90),
        ];
        
        Ok(Self {
            avg_visits_per_node,
            node_creation_rate,
            memory_efficiency: 1024.0, // Estimated bytes per node
            cache_hit_rates,
            throughput_metrics,
        })
    }
    
    /// Get overall cache hit rate
    pub fn overall_cache_hit_rate(&self) -> f64 {
        if self.cache_hit_rates.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.cache_hit_rates.iter().map(|(_, rate)| rate).sum();
        total / self.cache_hit_rates.len() as f64
    }
    
    /// Calculate overall performance score
    pub fn performance_score(&self) -> f64 {
        let throughput_weight = 0.4;
        let cache_weight = 0.3;
        let efficiency_weight = 0.3;
        
        let throughput_score = self.throughput_metrics.overall_throughput();
        let cache_score = self.overall_cache_hit_rate();
        let efficiency_score = (self.node_creation_rate / 1000.0).min(1.0);
        
        throughput_score * throughput_weight + cache_score * cache_weight + efficiency_score * efficiency_weight
    }
    
    /// Check if performance is excellent
    pub fn is_excellent_performance(&self) -> bool {
        self.node_creation_rate > 100.0
            && self.overall_cache_hit_rate() > 0.8
            && self.throughput_metrics.overall_throughput() > 0.7
    }
}

/// Throughput analysis with operation-specific metrics
#[derive(Debug, Clone, Serialize)]
pub struct ThroughputMetrics {
    /// Selections per second
    pub selections_per_second: f64,
    /// Expansions per second
    pub expansions_per_second: f64,
    /// Backpropagations per second
    pub backpropagations_per_second: f64,
    /// Simulations per second
    pub simulations_per_second: f64,
}

impl ThroughputMetrics {
    /// Create throughput metrics from counters and elapsed time
    pub fn from_counters_and_time(counters: &CounterSnapshot, elapsed_seconds: f64) -> Self {
        if elapsed_seconds <= 0.0 {
            return Self {
                selections_per_second: 0.0,
                expansions_per_second: 0.0,
                backpropagations_per_second: 0.0,
                simulations_per_second: 0.0,
            };
        }
        
        Self {
            selections_per_second: counters.selections as f64 / elapsed_seconds,
            expansions_per_second: counters.expansions as f64 / elapsed_seconds,
            backpropagations_per_second: counters.backpropagations as f64 / elapsed_seconds,
            simulations_per_second: counters.simulations as f64 / elapsed_seconds,
        }
    }
    
    /// Get overall throughput score
    pub fn overall_throughput(&self) -> f64 {
        let total = self.selections_per_second 
            + self.expansions_per_second 
            + self.backpropagations_per_second 
            + self.simulations_per_second;
        
        // Normalize to 0-1 range (assuming max reasonable throughput is 1000 ops/sec)
        (total / 1000.0).min(1.0)
    }
    
    /// Check if throughput is balanced across operations
    pub fn is_balanced(&self) -> bool {
        let ops = [
            self.selections_per_second,
            self.expansions_per_second,
            self.backpropagations_per_second,
        ];
        
        if ops.iter().any(|&x| x <= 0.0) {
            return false;
        }
        
        let mean = ops.iter().sum::<f64>() / ops.len() as f64;
        let max_deviation = ops.iter()
            .map(|&x| (x - mean).abs() / mean)
            .fold(0.0f64, f64::max);
        
        max_deviation < 0.3 // Allow 30% deviation from mean
    }
    
    /// Get bottleneck operation (lowest throughput)
    pub fn bottleneck_operation(&self) -> &'static str {
        let ops = [
            ("selection", self.selections_per_second),
            ("expansion", self.expansions_per_second),
            ("backpropagation", self.backpropagations_per_second),
            ("simulation", self.simulations_per_second),
        ];
        
        ops.iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
            .unwrap_or("unknown")
    }
}

/// Performance bottleneck identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PerformanceBottleneck {
    Selection,
    Expansion,
    Backpropagation,
    Simulation,
    Memory,
    Cache,
    None,
}

impl PerformanceBottleneck {
    /// Identify primary bottleneck from metrics
    pub fn identify(metrics: &PerformanceMetrics) -> Self {
        let throughput = &metrics.throughput_metrics;
        
        if metrics.overall_cache_hit_rate() < 0.5 {
            return PerformanceBottleneck::Cache;
        }
        
        if metrics.node_creation_rate < 1.0 {
            return PerformanceBottleneck::Memory;
        }
        
        match throughput.bottleneck_operation() {
            "selection" => PerformanceBottleneck::Selection,
            "expansion" => PerformanceBottleneck::Expansion,
            "backpropagation" => PerformanceBottleneck::Backpropagation,
            "simulation" => PerformanceBottleneck::Simulation,
            _ => PerformanceBottleneck::None,
        }
    }
    
    /// Get bottleneck description
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceBottleneck::Selection => "Node selection is the primary bottleneck",
            PerformanceBottleneck::Expansion => "Tree expansion is limiting performance",
            PerformanceBottleneck::Backpropagation => "Reward backpropagation is slow",
            PerformanceBottleneck::Simulation => "Simulations are the limiting factor",
            PerformanceBottleneck::Memory => "Memory allocation is constraining performance",
            PerformanceBottleneck::Cache => "Poor cache performance is degrading speed",
            PerformanceBottleneck::None => "No significant bottleneck identified",
        }
    }
}

/// Performance trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
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
}

/// Throughput analysis result
#[derive(Debug, Clone, Serialize)]
pub struct ThroughputAnalysis {
    /// Overall throughput score
    pub overall_score: f64,
    /// Balance score across operations
    pub balance_score: f64,
    /// Identified bottleneck
    pub bottleneck: PerformanceBottleneck,
    /// Performance trend
    pub trend: PerformanceTrend,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl ThroughputAnalysis {
    /// Analyze throughput metrics
    pub fn analyze(metrics: &PerformanceMetrics) -> Self {
        let overall_score = metrics.throughput_metrics.overall_throughput();
        let balance_score = if metrics.throughput_metrics.is_balanced() { 1.0 } else { 0.5 };
        let bottleneck = PerformanceBottleneck::identify(metrics);
        let trend = PerformanceTrend::Stable; // Would be calculated from historical data
        
        let mut recommendations = Vec::new();
        
        match bottleneck {
            PerformanceBottleneck::Selection => {
                recommendations.push("Optimize node selection algorithm".to_string());
                recommendations.push("Consider parallel selection strategies".to_string());
            }
            PerformanceBottleneck::Expansion => {
                recommendations.push("Streamline tree expansion logic".to_string());
                recommendations.push("Reduce expansion computational overhead".to_string());
            }
            PerformanceBottleneck::Cache => {
                recommendations.push("Improve cache algorithms and sizing".to_string());
                recommendations.push("Review cache invalidation strategies".to_string());
            }
            PerformanceBottleneck::None => {
                recommendations.push("Performance is well-balanced".to_string());
            }
            _ => {
                recommendations.push(format!("Focus optimization on {}", bottleneck.description()));
            }
        }
        
        Self {
            overall_score,
            balance_score,
            bottleneck,
            trend,
            recommendations,
        }
    }
    
    /// Check if analysis indicates good performance
    pub fn is_good_performance(&self) -> bool {
        self.overall_score > 0.7 && self.balance_score > 0.8
    }
}

/// Priority level for performance issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    /// Determine priority from performance metrics
    pub fn from_performance_score(score: f64) -> Self {
        if score < 0.3 {
            Priority::Critical
        } else if score < 0.5 {
            Priority::High
        } else if score < 0.7 {
            Priority::Medium
        } else {
            Priority::Low
        }
    }
    
    /// Get priority description
    pub fn description(&self) -> &'static str {
        match self {
            Priority::Low => "Low priority - monitor occasionally",
            Priority::Medium => "Medium priority - address when convenient",
            Priority::High => "High priority - address soon",
            Priority::Critical => "Critical priority - immediate attention required",
        }
    }
}
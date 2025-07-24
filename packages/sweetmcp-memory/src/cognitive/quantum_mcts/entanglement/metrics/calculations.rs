//! Performance calculations and derived metrics for quantum entanglement
//!
//! This module provides blazing-fast calculation algorithms with zero-allocation
//! patterns and comprehensive performance analysis capabilities.

use std::time::Duration;
use super::counters::{EntanglementCounters, CounterSnapshot, CounterDelta};

/// Performance calculation utilities for entanglement metrics
pub struct MetricsCalculator;

impl MetricsCalculator {
    /// Calculate success rate for entanglement operations
    pub fn calculate_success_rate(counters: &EntanglementCounters) -> f64 {
        let operations = counters.entanglement_operations();
        let failures = counters.entanglement_failures();
        
        if operations == 0 {
            return 0.0;
        }
        
        let successful_operations = operations.saturating_sub(failures);
        successful_operations as f64 / operations as f64
    }
    
    /// Calculate success rate from snapshot
    pub fn calculate_success_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        if snapshot.entanglement_operations == 0 {
            return 0.0;
        }
        
        let successful_operations = snapshot.entanglement_operations.saturating_sub(snapshot.entanglement_failures);
        successful_operations as f64 / snapshot.entanglement_operations as f64
    }
    
    /// Calculate net entanglement growth
    pub fn calculate_net_entanglements(counters: &EntanglementCounters) -> i64 {
        counters.entanglements_created() as i64 - 
        counters.entanglements_removed() as i64 - 
        counters.entanglements_pruned() as i64
    }
    
    /// Calculate cache hit rate for entanglement decisions
    pub fn calculate_cache_hit_rate(counters: &EntanglementCounters) -> f64 {
        let hits = counters.cache_hits();
        let misses = counters.cache_misses();
        let total = hits + misses;
        
        if total == 0 {
            return 0.0;
        }
        
        hits as f64 / total as f64
    }
    
    /// Calculate cache hit rate from snapshot
    pub fn calculate_cache_hit_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let total = snapshot.cache_hits + snapshot.cache_misses;
        
        if total == 0 {
            return 0.0;
        }
        
        snapshot.cache_hits as f64 / total as f64
    }
    
    /// Calculate average operation time in microseconds
    pub fn calculate_average_operation_time_us(counters: &EntanglementCounters) -> f64 {
        let operations = counters.entanglement_operations();
        let total_time = counters.total_operation_time_us();
        
        if operations == 0 {
            return 0.0;
        }
        
        total_time as f64 / operations as f64
    }
    
    /// Calculate average operation time from snapshot
    pub fn calculate_average_operation_time_us_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        if snapshot.entanglement_operations == 0 {
            return 0.0;
        }
        
        snapshot.total_operation_time_us as f64 / snapshot.entanglement_operations as f64
    }
    
    /// Calculate average influence calculation time in microseconds
    pub fn calculate_average_influence_time_us(counters: &EntanglementCounters) -> f64 {
        let calculations = counters.influence_calculations();
        let total_time = counters.influence_time_us();
        
        if calculations == 0 {
            return 0.0;
        }
        
        total_time as f64 / calculations as f64
    }
    
    /// Calculate average influence time from snapshot
    pub fn calculate_average_influence_time_us_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        if snapshot.influence_calculations == 0 {
            return 0.0;
        }
        
        snapshot.influence_time_us as f64 / snapshot.influence_calculations as f64
    }
    
    /// Calculate operations per second
    pub fn calculate_operations_per_second(counters: &EntanglementCounters) -> f64 {
        let operations = counters.entanglement_operations();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        operations as f64 / uptime_secs
    }
    
    /// Calculate operations per second from snapshot
    pub fn calculate_operations_per_second_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let uptime_secs = snapshot.uptime_duration.as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        snapshot.entanglement_operations as f64 / uptime_secs
    }
    
    /// Calculate influence calculations per second
    pub fn calculate_influence_calculations_per_second(counters: &EntanglementCounters) -> f64 {
        let calculations = counters.influence_calculations();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        calculations as f64 / uptime_secs
    }
    
    /// Calculate influence calculations per second from snapshot
    pub fn calculate_influence_calculations_per_second_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let uptime_secs = snapshot.uptime_duration.as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        snapshot.influence_calculations as f64 / uptime_secs
    }
    
    /// Calculate efficiency score (0.0 to 1.0)
    pub fn calculate_efficiency_score(counters: &EntanglementCounters) -> f64 {
        let success_rate = Self::calculate_success_rate(counters);
        let cache_hit_rate = Self::calculate_cache_hit_rate(counters);
        let avg_time_us = Self::calculate_average_operation_time_us(counters);
        
        // Normalize average time (assume 1000μs is baseline, lower is better)
        let time_efficiency = if avg_time_us > 0.0 {
            (1000.0 / avg_time_us).min(1.0)
        } else {
            1.0
        };
        
        // Weighted combination of factors
        (success_rate * 0.4) + (cache_hit_rate * 0.3) + (time_efficiency * 0.3)
    }
    
    /// Calculate efficiency score from snapshot
    pub fn calculate_efficiency_score_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let success_rate = Self::calculate_success_rate_from_snapshot(snapshot);
        let cache_hit_rate = Self::calculate_cache_hit_rate_from_snapshot(snapshot);
        let avg_time_us = Self::calculate_average_operation_time_us_from_snapshot(snapshot);
        
        let time_efficiency = if avg_time_us > 0.0 {
            (1000.0 / avg_time_us).min(1.0)
        } else {
            1.0
        };
        
        (success_rate * 0.4) + (cache_hit_rate * 0.3) + (time_efficiency * 0.3)
    }
    
    /// Calculate performance grade (A-F) based on efficiency
    pub fn calculate_performance_grade(efficiency_score: f64) -> char {
        match efficiency_score {
            s if s >= 0.95 => 'A',
            s if s >= 0.85 => 'B',
            s if s >= 0.75 => 'C',
            s if s >= 0.65 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate throughput score based on operations per second
    pub fn calculate_throughput_score(ops_per_second: f64) -> f64 {
        // Normalize throughput (assume 100 ops/sec is excellent)
        (ops_per_second / 100.0).min(1.0)
    }
    
    /// Calculate latency score based on average operation time
    pub fn calculate_latency_score(avg_time_us: f64) -> f64 {
        if avg_time_us <= 0.0 {
            return 1.0;
        }
        
        // Lower latency is better, normalize against 100μs baseline
        (100.0 / avg_time_us).min(1.0)
    }
    
    /// Calculate resource utilization score
    pub fn calculate_resource_utilization_score(counters: &EntanglementCounters) -> f64 {
        let total_operations = counters.total_operations();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        // Calculate operations per second across all operation types
        let overall_ops_per_sec = total_operations as f64 / uptime_secs;
        
        // Normalize against expected utilization (50 ops/sec is good utilization)
        (overall_ops_per_sec / 50.0).min(1.0)
    }
    
    /// Calculate failure rate
    pub fn calculate_failure_rate(counters: &EntanglementCounters) -> f64 {
        let operations = counters.entanglement_operations();
        let failures = counters.entanglement_failures();
        
        if operations == 0 {
            return 0.0;
        }
        
        failures as f64 / operations as f64
    }
    
    /// Calculate failure rate from snapshot
    pub fn calculate_failure_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        if snapshot.entanglement_operations == 0 {
            return 0.0;
        }
        
        snapshot.entanglement_failures as f64 / snapshot.entanglement_operations as f64
    }
    
    /// Calculate growth rate (entanglements created per second)
    pub fn calculate_growth_rate(counters: &EntanglementCounters) -> f64 {
        let created = counters.entanglements_created();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        created as f64 / uptime_secs
    }
    
    /// Calculate growth rate from snapshot
    pub fn calculate_growth_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let uptime_secs = snapshot.uptime_duration.as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        snapshot.entanglements_created as f64 / uptime_secs
    }
    
    /// Calculate pruning rate (entanglements pruned per second)
    pub fn calculate_pruning_rate(counters: &EntanglementCounters) -> f64 {
        let pruned = counters.entanglements_pruned();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        pruned as f64 / uptime_secs
    }
    
    /// Calculate pruning rate from snapshot
    pub fn calculate_pruning_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let uptime_secs = snapshot.uptime_duration.as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        snapshot.entanglements_pruned as f64 / uptime_secs
    }
    
    /// Calculate topology analysis rate (analyses per second)
    pub fn calculate_topology_analysis_rate(counters: &EntanglementCounters) -> f64 {
        let analyses = counters.topology_analyses();
        let uptime_secs = counters.uptime().as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        analyses as f64 / uptime_secs
    }
    
    /// Calculate topology analysis rate from snapshot
    pub fn calculate_topology_analysis_rate_from_snapshot(snapshot: &CounterSnapshot) -> f64 {
        let uptime_secs = snapshot.uptime_duration.as_secs_f64();
        
        if uptime_secs <= 0.0 {
            return 0.0;
        }
        
        snapshot.topology_analyses as f64 / uptime_secs
    }
    
    /// Calculate comprehensive performance metrics from counters
    pub fn calculate_comprehensive_metrics(counters: &EntanglementCounters) -> ComprehensiveMetrics {
        ComprehensiveMetrics {
            success_rate: Self::calculate_success_rate(counters),
            cache_hit_rate: Self::calculate_cache_hit_rate(counters),
            average_operation_time_us: Self::calculate_average_operation_time_us(counters),
            average_influence_time_us: Self::calculate_average_influence_time_us(counters),
            operations_per_second: Self::calculate_operations_per_second(counters),
            influence_calculations_per_second: Self::calculate_influence_calculations_per_second(counters),
            efficiency_score: Self::calculate_efficiency_score(counters),
            throughput_score: Self::calculate_throughput_score(Self::calculate_operations_per_second(counters)),
            latency_score: Self::calculate_latency_score(Self::calculate_average_operation_time_us(counters)),
            resource_utilization_score: Self::calculate_resource_utilization_score(counters),
            failure_rate: Self::calculate_failure_rate(counters),
            growth_rate: Self::calculate_growth_rate(counters),
            pruning_rate: Self::calculate_pruning_rate(counters),
            topology_analysis_rate: Self::calculate_topology_analysis_rate(counters),
            net_entanglements: Self::calculate_net_entanglements(counters),
            uptime_seconds: counters.uptime_seconds(),
        }
    }
    
    /// Calculate comprehensive performance metrics from snapshot
    pub fn calculate_comprehensive_metrics_from_snapshot(snapshot: &CounterSnapshot) -> ComprehensiveMetrics {
        let ops_per_sec = Self::calculate_operations_per_second_from_snapshot(snapshot);
        let avg_time_us = Self::calculate_average_operation_time_us_from_snapshot(snapshot);
        
        ComprehensiveMetrics {
            success_rate: Self::calculate_success_rate_from_snapshot(snapshot),
            cache_hit_rate: Self::calculate_cache_hit_rate_from_snapshot(snapshot),
            average_operation_time_us: avg_time_us,
            average_influence_time_us: Self::calculate_average_influence_time_us_from_snapshot(snapshot),
            operations_per_second: ops_per_sec,
            influence_calculations_per_second: Self::calculate_influence_calculations_per_second_from_snapshot(snapshot),
            efficiency_score: Self::calculate_efficiency_score_from_snapshot(snapshot),
            throughput_score: Self::calculate_throughput_score(ops_per_sec),
            latency_score: Self::calculate_latency_score(avg_time_us),
            resource_utilization_score: {
                let total_ops = snapshot.entanglement_operations + snapshot.topology_analyses + snapshot.influence_calculations;
                let uptime_secs = snapshot.uptime_duration.as_secs_f64();
                if uptime_secs > 0.0 {
                    ((total_ops as f64) / uptime_secs / 50.0).min(1.0)
                } else {
                    0.0
                }
            },
            failure_rate: Self::calculate_failure_rate_from_snapshot(snapshot),
            growth_rate: Self::calculate_growth_rate_from_snapshot(snapshot),
            pruning_rate: Self::calculate_pruning_rate_from_snapshot(snapshot),
            topology_analysis_rate: Self::calculate_topology_analysis_rate_from_snapshot(snapshot),
            net_entanglements: snapshot.net_entanglements(),
            uptime_seconds: snapshot.uptime_duration.as_secs(),
        }
    }
    
    /// Calculate performance trend from delta
    pub fn calculate_performance_trend(delta: &CounterDelta) -> PerformanceTrend {
        let time_secs = delta.time_elapsed.as_secs_f64();
        
        if time_secs <= 0.0 {
            return PerformanceTrend::default();
        }
        
        let ops_per_sec = delta.entanglement_operations as f64 / time_secs;
        let success_rate = if delta.entanglement_operations > 0 {
            (delta.entanglement_operations - delta.entanglement_failures) as f64 / delta.entanglement_operations as f64
        } else {
            0.0
        };
        
        let cache_hit_rate = if delta.cache_hits + delta.cache_misses > 0 {
            delta.cache_hits as f64 / (delta.cache_hits + delta.cache_misses) as f64
        } else {
            0.0
        };
        
        let avg_operation_time_us = if delta.entanglement_operations > 0 {
            delta.total_operation_time_us as f64 / delta.entanglement_operations as f64
        } else {
            0.0
        };
        
        PerformanceTrend {
            operations_per_second: ops_per_sec,
            success_rate,
            cache_hit_rate,
            average_operation_time_us: avg_operation_time_us,
            net_entanglement_change: delta.net_entanglement_change(),
            time_period: delta.time_elapsed,
        }
    }
}

/// Comprehensive performance metrics
#[derive(Debug, Clone)]
pub struct ComprehensiveMetrics {
    /// Success rate of operations (0.0 to 1.0)
    pub success_rate: f64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Average operation time in microseconds
    pub average_operation_time_us: f64,
    /// Average influence calculation time in microseconds
    pub average_influence_time_us: f64,
    /// Operations per second
    pub operations_per_second: f64,
    /// Influence calculations per second
    pub influence_calculations_per_second: f64,
    /// Overall efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Throughput score (0.0 to 1.0)
    pub throughput_score: f64,
    /// Latency score (0.0 to 1.0, higher is better)
    pub latency_score: f64,
    /// Resource utilization score (0.0 to 1.0)
    pub resource_utilization_score: f64,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
    /// Growth rate (entanglements created per second)
    pub growth_rate: f64,
    /// Pruning rate (entanglements pruned per second)
    pub pruning_rate: f64,
    /// Topology analysis rate (analyses per second)
    pub topology_analysis_rate: f64,
    /// Net entanglement growth
    pub net_entanglements: i64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

impl ComprehensiveMetrics {
    /// Get overall performance grade
    pub fn performance_grade(&self) -> char {
        MetricsCalculator::calculate_performance_grade(self.efficiency_score)
    }
    
    /// Check if metrics indicate good performance
    pub fn has_good_performance(&self) -> bool {
        self.success_rate > 0.9 && 
        self.cache_hit_rate > 0.8 && 
        self.average_operation_time_us < 1000.0 &&
        self.failure_rate < 0.1
    }
    
    /// Check if metrics indicate performance issues
    pub fn has_performance_issues(&self) -> bool {
        self.success_rate < 0.8 || 
        self.cache_hit_rate < 0.6 || 
        self.average_operation_time_us > 2000.0 ||
        self.failure_rate > 0.2
    }
    
    /// Get performance summary string
    pub fn performance_summary(&self) -> String {
        format!(
            "Performance: {}% success, {}% cache hit, {:.1}μs avg latency, {:.1} ops/sec (Grade: {})",
            (self.success_rate * 100.0) as u32,
            (self.cache_hit_rate * 100.0) as u32,
            self.average_operation_time_us,
            self.operations_per_second,
            self.performance_grade()
        )
    }
}

/// Performance trend over a time period
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    /// Operations per second during the period
    pub operations_per_second: f64,
    /// Success rate during the period
    pub success_rate: f64,
    /// Cache hit rate during the period
    pub cache_hit_rate: f64,
    /// Average operation time during the period
    pub average_operation_time_us: f64,
    /// Net entanglement change during the period
    pub net_entanglement_change: i64,
    /// Time period for the trend
    pub time_period: Duration,
}

impl Default for PerformanceTrend {
    fn default() -> Self {
        Self {
            operations_per_second: 0.0,
            success_rate: 0.0,
            cache_hit_rate: 0.0,
            average_operation_time_us: 0.0,
            net_entanglement_change: 0,
            time_period: Duration::from_secs(0),
        }
    }
}

impl PerformanceTrend {
    /// Check if trend indicates improving performance
    pub fn is_improving(&self) -> bool {
        self.success_rate > 0.9 && 
        self.operations_per_second > 10.0 && 
        self.average_operation_time_us < 1000.0
    }
    
    /// Check if trend indicates declining performance
    pub fn is_declining(&self) -> bool {
        self.success_rate < 0.8 || 
        self.operations_per_second < 5.0 || 
        self.average_operation_time_us > 2000.0
    }
    
    /// Get trend summary
    pub fn summary(&self) -> String {
        let trend_direction = if self.is_improving() {
            "IMPROVING"
        } else if self.is_declining() {
            "DECLINING"
        } else {
            "STABLE"
        };
        
        format!(
            "Trend ({}s): {} - {:.1} ops/sec, {}% success, {:.1}μs latency",
            self.time_period.as_secs(),
            trend_direction,
            self.operations_per_second,
            (self.success_rate * 100.0) as u32,
            self.average_operation_time_us
        )
    }
}
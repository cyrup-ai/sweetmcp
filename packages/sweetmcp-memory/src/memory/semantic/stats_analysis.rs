//! Statistics analysis and trend computation
//!
//! This module provides blazing-fast statistics analysis with zero-allocation
//! trend computation and comparison algorithms.

use super::memory_stats::MemoryStatistics;

/// High-performance statistics aggregator
pub struct StatisticsAggregator;

impl StatisticsAggregator {
    /// Calculate statistics summary with zero allocation
    #[inline]
    pub fn calculate_summary(stats: &super::atomic_stats::AtomicMemoryStatistics) -> StatisticsSummary {
        let item_count = stats.get_item_count();
        let relationship_count = stats.get_relationship_count();
        let memory_usage = stats.get_memory_usage();
        let cleanup_count = stats.get_cleanup_count();
        
        let total_entities = item_count + relationship_count;
        let memory_per_entity = if total_entities > 0 {
            memory_usage as f32 / total_entities as f32
        } else {
            0.0
        };

        StatisticsSummary {
            total_entities,
            memory_per_entity,
            efficiency_score: stats.memory_efficiency(),
            cleanup_frequency: cleanup_count,
            access_efficiency: stats.access_efficiency(),
            needs_optimization: memory_per_entity > 2048.0 || stats.memory_efficiency() < 0.5,
        }
    }

    /// Compare two statistics snapshots
    #[inline]
    pub fn compare_snapshots(before: &MemoryStatistics, after: &MemoryStatistics) -> StatisticsComparison {
        let item_delta = after.total_items as i64 - before.total_items as i64;
        let relationship_delta = after.total_relationships as i64 - before.total_relationships as i64;
        let memory_delta = after.memory_usage_bytes as i64 - before.memory_usage_bytes as i64;
        let efficiency_delta = after.memory_efficiency() - before.memory_efficiency();
        let access_delta = after.total_access_count as i64 - before.total_access_count as i64;

        StatisticsComparison {
            item_delta,
            relationship_delta,
            memory_delta,
            efficiency_delta,
            access_delta,
            improvement_detected: efficiency_delta > 0.01 && memory_delta <= 0,
        }
    }

    /// Calculate trend analysis
    #[inline]
    pub fn analyze_trends(history: &[MemoryStatistics]) -> TrendAnalysis {
        if history.len() < 2 {
            return TrendAnalysis::default();
        }

        let first = &history[0];
        let last = &history[history.len() - 1];
        
        let memory_trend = Self::calculate_trend(
            first.memory_usage_bytes as f32,
            last.memory_usage_bytes as f32,
        );
        
        let efficiency_trend = Self::calculate_trend(
            first.memory_efficiency(),
            last.memory_efficiency(),
        );

        let item_trend = Self::calculate_trend(
            first.total_items as f32,
            last.total_items as f32,
        );

        TrendAnalysis {
            memory_trend,
            efficiency_trend,
            item_trend,
            trend_strength: (memory_trend.abs() + efficiency_trend.abs() + item_trend.abs()) / 3.0,
            is_improving: efficiency_trend > 0.0 && memory_trend <= 0.0,
        }
    }

    /// Calculate trend value (-1.0 = decreasing, 0.0 = stable, 1.0 = increasing)
    #[inline]
    fn calculate_trend(start: f32, end: f32) -> f32 {
        if start == 0.0 {
            return if end > 0.0 { 1.0 } else { 0.0 };
        }
        let change = (end - start) / start;
        change.max(-1.0).min(1.0)
    }

    /// Calculate moving average for smoothed trends
    #[inline]
    pub fn calculate_moving_average(values: &[f32], window_size: usize) -> Vec<f32> {
        if values.len() < window_size || window_size == 0 {
            return values.to_vec();
        }

        let mut averages = Vec::with_capacity(values.len() - window_size + 1);
        
        for i in 0..=(values.len() - window_size) {
            let sum: f32 = values[i..i + window_size].iter().sum();
            averages.push(sum / window_size as f32);
        }
        
        averages
    }

    /// Detect basic anomalies in statistics
    #[inline]
    pub fn detect_anomalies(current: &MemoryStatistics, baseline: &MemoryStatistics) -> bool {
        let threshold = 2.0; // 2x difference threshold

        // Check memory usage anomaly
        if current.memory_usage_bytes > 0 && baseline.memory_usage_bytes > 0 {
            let ratio = current.memory_usage_bytes as f32 / baseline.memory_usage_bytes as f32;
            if ratio > threshold {
                return true;
            }
        }

        // Check efficiency anomaly
        let efficiency_delta = baseline.memory_efficiency() - current.memory_efficiency();
        if efficiency_delta > 0.3 {
            return true;
        }

        // Check relationship ratio anomaly
        let current_ratio = current.relationship_ratio();
        let baseline_ratio = baseline.relationship_ratio();
        current_ratio > baseline_ratio * threshold && current_ratio > 5.0
    }
}

/// Statistics summary for quick analysis
#[derive(Debug, Clone)]
pub struct StatisticsSummary {
    pub total_entities: usize,
    pub memory_per_entity: f32,
    pub efficiency_score: f32,
    pub cleanup_frequency: usize,
    pub access_efficiency: f32,
    pub needs_optimization: bool,
}

impl StatisticsSummary {
    /// Get overall health score
    #[inline]
    pub fn health_score(&self) -> f32 {
        let efficiency_component = self.efficiency_score * 0.4;
        let access_component = self.access_efficiency * 0.3;
        let memory_component = if self.memory_per_entity < 1024.0 {
            0.3
        } else if self.memory_per_entity < 2048.0 {
            0.15
        } else {
            0.0
        };
        
        efficiency_component + access_component + memory_component
    }

    /// Check if summary indicates problems
    #[inline]
    pub fn has_problems(&self) -> bool {
        self.needs_optimization || self.efficiency_score < 0.5 || self.access_efficiency < 0.5
    }
}

/// Statistics comparison result
#[derive(Debug, Clone)]
pub struct StatisticsComparison {
    pub item_delta: i64,
    pub relationship_delta: i64,
    pub memory_delta: i64,
    pub efficiency_delta: f32,
    pub access_delta: i64,
    pub improvement_detected: bool,
}

impl StatisticsComparison {
    /// Check if comparison shows degradation
    #[inline]
    pub fn shows_degradation(&self) -> bool {
        self.efficiency_delta < -0.05 || (self.memory_delta > 0 && self.efficiency_delta <= 0.0)
    }

    /// Get change magnitude
    #[inline]
    pub fn change_magnitude(&self) -> f32 {
        let memory_change = (self.memory_delta.abs() as f32) / 1024.0; // KB
        let efficiency_change = self.efficiency_delta.abs();
        let access_change = (self.access_delta.abs() as f32) / 100.0;
        
        (memory_change + efficiency_change + access_change) / 3.0
    }
}

/// Trend analysis for historical data
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub memory_trend: f32,
    pub efficiency_trend: f32,
    pub item_trend: f32,
    pub trend_strength: f32,
    pub is_improving: bool,
}

impl Default for TrendAnalysis {
    fn default() -> Self {
        Self {
            memory_trend: 0.0,
            efficiency_trend: 0.0,
            item_trend: 0.0,
            trend_strength: 0.0,
            is_improving: false,
        }
    }
}

impl TrendAnalysis {
    /// Check if trends indicate need for intervention
    #[inline]
    pub fn needs_intervention(&self) -> bool {
        !self.is_improving && self.trend_strength > 0.5
    }

    /// Get trend description
    #[inline]
    pub fn description(&self) -> &'static str {
        if self.is_improving {
            "Improving"
        } else if self.trend_strength > 0.7 {
            "Rapidly Degrading"
        } else if self.trend_strength > 0.3 {
            "Slowly Degrading"
        } else {
            "Stable"
        }
    }
}
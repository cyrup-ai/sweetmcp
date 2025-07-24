//! Statistics snapshot and comparison functionality
//!
//! This module provides StatisticsSnapshot for historical tracking and
//! SnapshotComparison for blazing-fast differential analysis.

use serde::Serialize;
use std::time::Instant;
use super::types::QuantumTreeStatistics;

/// Statistics snapshot for historical trend analysis
#[derive(Debug, Clone, Serialize)]
pub struct StatisticsSnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: Instant,
    /// Complete statistics at the snapshot time
    pub statistics: QuantumTreeStatistics,
}

impl StatisticsSnapshot {
    /// Create new snapshot with current timestamp
    pub fn new(statistics: QuantumTreeStatistics) -> Self {
        Self {
            timestamp: Instant::now(),
            statistics,
        }
    }
    
    /// Get age of snapshot in seconds
    pub fn age_seconds(&self) -> f64 {
        self.timestamp.elapsed().as_secs_f64()
    }
    
    /// Get age of snapshot in minutes
    pub fn age_minutes(&self) -> f64 {
        self.age_seconds() / 60.0
    }
    
    /// Get age of snapshot in hours
    pub fn age_hours(&self) -> f64 {
        self.age_seconds() / 3600.0
    }
    
    /// Check if snapshot is recent (within specified seconds)
    pub fn is_recent(&self, seconds: f64) -> bool {
        self.age_seconds() <= seconds
    }
    
    /// Compare with another snapshot to calculate changes
    pub fn compare_with(&self, other: &StatisticsSnapshot) -> SnapshotComparison {
        let time_diff = if self.timestamp > other.timestamp {
            self.timestamp.duration_since(other.timestamp).as_secs_f64()
        } else {
            other.timestamp.duration_since(self.timestamp).as_secs_f64()
        };
        
        let node_change = self.statistics.total_nodes as i64 - other.statistics.total_nodes as i64;
        let visit_change = self.statistics.total_visits as i64 - other.statistics.total_visits as i64;
        let convergence_change = self.statistics.convergence_metrics.overall_convergence - 
                                other.statistics.convergence_metrics.overall_convergence;
        
        SnapshotComparison {
            time_diff_seconds: time_diff,
            node_change,
            visit_change,
            convergence_change,
            newer_snapshot: if self.timestamp > other.timestamp { "self" } else { "other" }.to_string(),
        }
    }
}

/// Comparison between two snapshots with growth rate analysis
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotComparison {
    /// Time difference in seconds
    pub time_diff_seconds: f64,
    /// Change in node count
    pub node_change: i64,
    /// Change in visit count
    pub visit_change: i64,
    /// Change in convergence score
    pub convergence_change: f64,
    /// Which snapshot is newer
    pub newer_snapshot: String,
}

impl SnapshotComparison {
    /// Calculate node growth rate per hour
    pub fn node_growth_rate_per_hour(&self) -> f64 {
        if self.time_diff_seconds > 0.0 {
            (self.node_change as f64) * 3600.0 / self.time_diff_seconds
        } else {
            0.0
        }
    }
    
    /// Calculate visit growth rate per hour
    pub fn visit_growth_rate_per_hour(&self) -> f64 {
        if self.time_diff_seconds > 0.0 {
            (self.visit_change as f64) * 3600.0 / self.time_diff_seconds
        } else {
            0.0
        }
    }
    
    /// Check if showing positive growth
    pub fn is_growing(&self) -> bool {
        self.node_change > 0 && self.visit_change > 0
    }
    
    /// Check if convergence is improving
    pub fn is_converging(&self) -> bool {
        self.convergence_change > 0.01 // Meaningful improvement threshold
    }
    
    /// Calculate overall change score (0.0 to 1.0)
    pub fn change_score(&self) -> f64 {
        let growth_score = if self.is_growing() { 0.4 } else { 0.0 };
        let convergence_score = if self.is_converging() { 0.4 } else { 0.0 };
        let stability_score = if self.time_diff_seconds > 0.0 { 0.2 } else { 0.0 };
        
        growth_score + convergence_score + stability_score
    }
    
    /// Get summary description of changes
    pub fn summary(&self) -> String {
        let growth_desc = if self.is_growing() { "Growing" } else { "Stagnant" };
        let convergence_desc = if self.is_converging() { "Converging" } else { "Diverging" };
        
        format!("{}, {}, Rate: {:.1} nodes/hr", 
                growth_desc, 
                convergence_desc, 
                self.node_growth_rate_per_hour())
    }
}
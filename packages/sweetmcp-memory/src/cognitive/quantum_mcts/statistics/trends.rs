//! Performance trends analysis and snapshot management
//!
//! This module provides comprehensive trend analysis with historical snapshots,
//! performance tracking, and zero-allocation trend calculations.

// Import all trend analysis components
pub use self::{
    snapshot_comparison::{StatisticsSnapshot, SnapshotComparison},
    performance_trends::PerformanceTrends,
    prediction::PerformancePrediction,
    trend_types::{
        PredictionReliability, TrendRecommendation, TrendMomentum, Priority
    },
};

// Import sibling modules
use super::snapshot_comparison;
use super::performance_trends;
use super::prediction;
use super::trend_types;

/// Trend analysis coordinator for unified trend operations
pub struct TrendsCoordinator {
    /// Maximum snapshots to retain
    max_snapshots: usize,
}

impl TrendsCoordinator {
    /// Create new trends coordinator
    pub fn new(max_snapshots: usize) -> Self {
        Self { max_snapshots }
    }
    
    /// Create default trends coordinator
    pub fn default() -> Self {
        Self::new(1000)
    }
    
    /// Analyze trends from snapshot history
    pub fn analyze_trends(&self, snapshots: &[StatisticsSnapshot]) -> PerformanceTrends {
        PerformanceTrends::from_snapshots(snapshots)
    }
    
    /// Create prediction based on trends
    pub fn create_prediction(
        &self, 
        trends: &PerformanceTrends, 
        hours_ahead: f64
    ) -> PerformancePrediction {
        trends.predict_future_performance(hours_ahead)
    }
    
    /// Limit snapshots to maximum count
    pub fn limit_snapshots(&self, snapshots: &mut Vec<StatisticsSnapshot>) {
        if snapshots.len() > self.max_snapshots {
            let excess = snapshots.len() - self.max_snapshots;
            snapshots.drain(0..excess);
        }
    }
    
    /// Get recent snapshots (last N entries)
    pub fn get_recent_snapshots(
        &self, 
        snapshots: &[StatisticsSnapshot], 
        count: usize
    ) -> Vec<StatisticsSnapshot> {
        let start_index = if snapshots.len() > count {
            snapshots.len() - count
        } else {
            0
        };
        snapshots[start_index..].to_vec()
    }
    
    /// Compare two snapshots
    pub fn compare_snapshots(
        &self, 
        snapshot1: &StatisticsSnapshot, 
        snapshot2: &StatisticsSnapshot
    ) -> SnapshotComparison {
        snapshot1.compare_with(snapshot2)
    }
    
    /// Check if snapshots indicate healthy trends
    pub fn is_trending_healthy(&self, snapshots: &[StatisticsSnapshot]) -> bool {
        if snapshots.len() < 2 {
            return false;
        }
        
        let trends = self.analyze_trends(snapshots);
        trends.is_performing_well()
    }
    
    /// Get comprehensive trend analysis
    pub fn comprehensive_analysis(&self, snapshots: &[StatisticsSnapshot]) -> TrendAnalysisResult {
        let trends = self.analyze_trends(snapshots);
        let prediction = self.create_prediction(&trends, 1.0); // 1 hour ahead
        let recommendations = trends.get_recommendations();
        let momentum = if snapshots.len() >= 3 {
            trends.calculate_momentum(snapshots)
        } else {
            TrendMomentum::Insufficient
        };
        
        TrendAnalysisResult {
            trends,
            prediction,
            recommendations,
            momentum,
            is_healthy: trends.is_performing_well(),
            grade: trends.performance_grade(),
        }
    }
}

/// Comprehensive trend analysis result
#[derive(Debug, Clone)]
pub struct TrendAnalysisResult {
    /// Performance trends analysis
    pub trends: PerformanceTrends,
    /// Future performance prediction
    pub prediction: PerformancePrediction,
    /// Recommended actions
    pub recommendations: Vec<TrendRecommendation>,
    /// Trend momentum
    pub momentum: TrendMomentum,
    /// Overall health status
    pub is_healthy: bool,
    /// Performance grade
    pub grade: char,
}

impl TrendAnalysisResult {
    /// Get summary of analysis
    pub fn summary(&self) -> String {
        format!(
            "Grade: {}, Healthy: {}, Momentum: {:?}, {} recommendations",
            self.grade,
            self.is_healthy,
            self.momentum,
            self.recommendations.len()
        )
    }
    
    /// Check if analysis indicates issues requiring attention
    pub fn requires_attention(&self) -> bool {
        !self.is_healthy || 
        self.grade <= 'C' || 
        self.momentum.is_concerning() ||
        self.recommendations.iter().any(|r| r.priority() >= Priority::High)
    }
    
    /// Get high-priority recommendations only
    pub fn high_priority_recommendations(&self) -> Vec<&TrendRecommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.priority() >= Priority::High)
            .collect()
    }
}
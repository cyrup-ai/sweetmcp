//! Performance trends analysis with historical data processing
//!
//! This module provides comprehensive PerformanceTrends analysis with
//! blazing-fast calculations, stability assessment, and trend predictions.

use serde::Serialize;
use super::{
    snapshot_comparison::StatisticsSnapshot,
    prediction::PerformancePrediction,
    trend_types::{TrendRecommendation, TrendMomentum, Priority},
};

/// Performance trends analysis with historical data
#[derive(Debug, Clone, Default, Serialize)]
pub struct PerformanceTrends {
    /// Node creation rate (nodes per hour)
    pub node_growth_rate: f64,
    /// Visit accumulation rate (visits per hour)
    pub visit_growth_rate: f64,
    /// Change in convergence score
    pub convergence_trend: f64,
    /// Stability score (0.0 to 1.0, higher is more stable)
    pub stability_score: f64,
    /// Whether performance is trending upward
    pub trending_up: bool,
    /// Whether performance is stable
    pub is_stable: bool,
}

impl PerformanceTrends {
    /// Create trends from snapshot history
    pub fn from_snapshots(snapshots: &[StatisticsSnapshot]) -> Self {
        if snapshots.len() < 2 {
            return Self::default();
        }
        
        let first = &snapshots[0];
        let last = &snapshots[snapshots.len() - 1];
        let comparison = last.compare_with(first);
        
        let node_growth_rate = comparison.node_growth_rate_per_hour();
        let visit_growth_rate = comparison.visit_growth_rate_per_hour();
        let convergence_trend = comparison.convergence_change;
        
        // Calculate stability from variance in convergence scores
        let stability_score = Self::calculate_stability_score(snapshots);
        
        Self {
            node_growth_rate,
            visit_growth_rate,
            convergence_trend,
            stability_score,
            trending_up: convergence_trend > 0.01 && node_growth_rate > 0.0,
            is_stable: stability_score > 0.8,
        }
    }
    
    /// Calculate stability score from convergence variance
    fn calculate_stability_score(snapshots: &[StatisticsSnapshot]) -> f64 {
        if snapshots.len() < 3 {
            return 0.5; // Neutral score for insufficient data
        }
        
        let convergence_scores: Vec<f64> = snapshots.iter()
            .map(|s| s.statistics.convergence_metrics.overall_convergence)
            .collect();
        
        let mean = convergence_scores.iter().sum::<f64>() / convergence_scores.len() as f64;
        let variance = convergence_scores.iter()
            .map(|&score| (score - mean).powi(2))
            .sum::<f64>() / convergence_scores.len() as f64;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean > 1e-10 {
            std_dev / mean
        } else {
            1.0 // High instability if mean is near zero
        };
        
        // Convert coefficient of variation to stability score (0.0 to 1.0)
        (1.0 / (1.0 + coefficient_of_variation * 2.0)).min(1.0)
    }
    
    /// Check if trends indicate good performance
    pub fn is_performing_well(&self) -> bool {
        self.trending_up && self.is_stable && self.stability_score > 0.7
    }
    
    /// Get performance grade (A-F) based on trends
    pub fn performance_grade(&self) -> char {
        let score = self.calculate_trend_score() * 100.0;
        match score as u32 {
            90..=100 => 'A',
            80..=89 => 'B',
            70..=79 => 'C',
            60..=69 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate overall trend score (0.0 to 1.0)
    pub fn calculate_trend_score(&self) -> f64 {
        let growth_score = if self.trending_up { 0.4 } else { 0.0 };
        let stability_component = self.stability_score * 0.4;
        let convergence_component = if self.convergence_trend > 0.0 { 0.2 } else { 0.0 };
        
        growth_score + stability_component + convergence_component
    }
    
    /// Predict future performance based on current trends
    pub fn predict_future_performance(&self, hours_ahead: f64) -> PerformancePrediction {
        let predicted_nodes = if self.node_growth_rate > 0.0 {
            (self.node_growth_rate * hours_ahead) as usize
        } else {
            0
        };
        
        let predicted_visits = if self.visit_growth_rate > 0.0 {
            (self.visit_growth_rate * hours_ahead) as u64
        } else {
            0
        };
        
        let predicted_convergence_change = self.convergence_trend * (hours_ahead / 24.0); // Daily rate
        
        let confidence = self.prediction_confidence();
        
        PerformancePrediction::new(
            hours_ahead,
            predicted_nodes,
            predicted_visits,
            predicted_convergence_change,
            confidence,
            self.get_prediction_assumptions(),
        )
    }
    
    /// Calculate confidence in predictions (0.0 to 1.0)
    pub fn prediction_confidence(&self) -> f64 {
        // Confidence based on stability and trend consistency
        let stability_factor = self.stability_score;
        let trend_consistency = if self.trending_up && self.convergence_trend > 0.0 { 0.3 } else { 0.0 };
        
        (stability_factor * 0.7 + trend_consistency).min(1.0)
    }
    
    /// Get assumptions underlying the predictions
    pub fn get_prediction_assumptions(&self) -> Vec<String> {
        let mut assumptions = Vec::new();
        
        assumptions.push("Current growth trends continue linearly".to_string());
        assumptions.push("No significant changes in workload or configuration".to_string());
        assumptions.push("System resources remain available".to_string());
        
        if self.is_stable {
            assumptions.push("Performance remains stable".to_string());
        } else {
            assumptions.push("Performance instability may affect predictions".to_string());
        }
        
        if self.trending_up {
            assumptions.push("Positive trend momentum continues".to_string());
        } else {
            assumptions.push("Current stagnation or decline may continue".to_string());
        }
        
        assumptions
    }
    
    /// Identify trend-based recommendations
    pub fn get_recommendations(&self) -> Vec<TrendRecommendation> {
        let mut recommendations = Vec::new();
        
        if !self.trending_up {
            recommendations.push(TrendRecommendation::IncreaseExploration);
        }
        
        if !self.is_stable {
            recommendations.push(TrendRecommendation::ImproveStability);
        }
        
        if self.convergence_trend < 0.0 {
            recommendations.push(TrendRecommendation::ReviewConvergenceStrategy);
        }
        
        if self.node_growth_rate < 10.0 {
            recommendations.push(TrendRecommendation::OptimizeNodeCreation);
        }
        
        if recommendations.is_empty() {
            recommendations.push(TrendRecommendation::MaintainCurrentStrategy);
        }
        
        recommendations
    }
    
    /// Calculate trend momentum (rate of change acceleration)
    pub fn calculate_momentum(&self, recent_snapshots: &[StatisticsSnapshot]) -> TrendMomentum {
        if recent_snapshots.len() < 3 {
            return TrendMomentum::Insufficient;
        }
        
        // Calculate acceleration in convergence improvement
        let len = recent_snapshots.len();
        let recent_change = recent_snapshots[len-1].statistics.convergence_metrics.overall_convergence -
                           recent_snapshots[len-2].statistics.convergence_metrics.overall_convergence;
        let previous_change = recent_snapshots[len-2].statistics.convergence_metrics.overall_convergence -
                             recent_snapshots[len-3].statistics.convergence_metrics.overall_convergence;
        
        let acceleration = recent_change - previous_change;
        
        match acceleration {
            a if a > 0.05 => TrendMomentum::StronglyAccelerating,
            a if a > 0.01 => TrendMomentum::Accelerating,
            a if a < -0.05 => TrendMomentum::StronglyDecelerating,
            a if a < -0.01 => TrendMomentum::Decelerating,
            _ => TrendMomentum::Steady,
        }
    }
    
    /// Get trend analysis summary
    pub fn summary(&self) -> String {
        let grade = self.performance_grade();
        let stability = if self.is_stable { "Stable" } else { "Unstable" };
        let direction = if self.trending_up { "Up" } else { "Down" };
        
        format!(
            "Grade: {}, {}, Trending {}, Growth: {:.1} nodes/hr",
            grade, stability, direction, self.node_growth_rate
        )
    }
    
    /// Check if trends require immediate attention
    pub fn requires_attention(&self) -> bool {
        !self.is_stable || self.convergence_trend < -0.05 || self.node_growth_rate < 1.0
    }
    
    /// Get priority recommendations only
    pub fn priority_recommendations(&self) -> Vec<TrendRecommendation> {
        self.get_recommendations()
            .into_iter()
            .filter(|rec| rec.priority() >= Priority::Medium)
            .collect()
    }
}
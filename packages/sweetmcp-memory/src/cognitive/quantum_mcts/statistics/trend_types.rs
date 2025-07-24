//! Trend analysis types and enums
//!
//! This module provides comprehensive type definitions for trend analysis,
//! including prediction reliability, recommendations, and momentum indicators.

use serde::Serialize;

/// Prediction reliability levels with confidence assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PredictionReliability {
    High,
    Moderate,
    Low,
    VeryLow,
}

impl PredictionReliability {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PredictionReliability::High => "Highly reliable prediction",
            PredictionReliability::Moderate => "Moderately reliable prediction",
            PredictionReliability::Low => "Low reliability prediction",
            PredictionReliability::VeryLow => "Very low reliability prediction",
        }
    }
    
    /// Get confidence score (0.0 to 1.0)
    pub fn confidence_score(&self) -> f64 {
        match self {
            PredictionReliability::High => 0.9,
            PredictionReliability::Moderate => 0.7,
            PredictionReliability::Low => 0.5,
            PredictionReliability::VeryLow => 0.3,
        }
    }
    
    /// Check if reliability is acceptable for decision making
    pub fn is_acceptable(&self) -> bool {
        matches!(self, PredictionReliability::High | PredictionReliability::Moderate)
    }
}

/// Trend-based recommendations for optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TrendRecommendation {
    IncreaseExploration,
    ImproveStability,
    ReviewConvergenceStrategy,
    OptimizeNodeCreation,
    MaintainCurrentStrategy,
    ReduceResourceUsage,
    ScaleUpResources,
}

impl TrendRecommendation {
    /// Get detailed description of the recommendation
    pub fn description(&self) -> String {
        match self {
            TrendRecommendation::IncreaseExploration => 
                "Increase exploration budget or adjust selection policy to improve growth".to_string(),
            TrendRecommendation::ImproveStability => 
                "Focus on improving performance stability through consistent resource allocation".to_string(),
            TrendRecommendation::ReviewConvergenceStrategy => 
                "Review and potentially adjust convergence criteria or exploration balance".to_string(),
            TrendRecommendation::OptimizeNodeCreation => 
                "Optimize node creation processes to improve growth rate".to_string(),
            TrendRecommendation::MaintainCurrentStrategy => 
                "Current strategy is working well, maintain current approach".to_string(),
            TrendRecommendation::ReduceResourceUsage => 
                "Consider optimizing resource usage for better efficiency".to_string(),
            TrendRecommendation::ScaleUpResources => 
                "Consider increasing available resources to support growth".to_string(),
        }
    }
    
    /// Get priority level for implementing the recommendation
    pub fn priority(&self) -> Priority {
        match self {
            TrendRecommendation::ImproveStability => Priority::High,
            TrendRecommendation::ReviewConvergenceStrategy => Priority::High,
            TrendRecommendation::IncreaseExploration => Priority::Medium,
            TrendRecommendation::OptimizeNodeCreation => Priority::Medium,
            TrendRecommendation::ScaleUpResources => Priority::Medium,
            TrendRecommendation::ReduceResourceUsage => Priority::Low,
            TrendRecommendation::MaintainCurrentStrategy => Priority::None,
        }
    }
    
    /// Get estimated impact of implementing the recommendation (0.0 to 1.0)
    pub fn estimated_impact(&self) -> f64 {
        match self {
            TrendRecommendation::ImproveStability => 0.8,
            TrendRecommendation::ReviewConvergenceStrategy => 0.9,
            TrendRecommendation::IncreaseExploration => 0.7,
            TrendRecommendation::OptimizeNodeCreation => 0.6,
            TrendRecommendation::ScaleUpResources => 0.5,
            TrendRecommendation::ReduceResourceUsage => 0.4,
            TrendRecommendation::MaintainCurrentStrategy => 0.0,
        }
    }
    
    /// Get estimated implementation effort (0.0 to 1.0)
    pub fn implementation_effort(&self) -> f64 {
        match self {
            TrendRecommendation::MaintainCurrentStrategy => 0.0,
            TrendRecommendation::IncreaseExploration => 0.3,
            TrendRecommendation::OptimizeNodeCreation => 0.5,
            TrendRecommendation::ReduceResourceUsage => 0.6,
            TrendRecommendation::ImproveStability => 0.7,
            TrendRecommendation::ReviewConvergenceStrategy => 0.8,
            TrendRecommendation::ScaleUpResources => 0.9,
        }
    }
}

/// Trend momentum indicators for acceleration analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrendMomentum {
    StronglyAccelerating,
    Accelerating,
    Steady,
    Decelerating,
    StronglyDecelerating,
    Insufficient, // Not enough data
}

impl TrendMomentum {
    /// Check if momentum is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, TrendMomentum::StronglyAccelerating | TrendMomentum::Accelerating)
    }
    
    /// Get momentum score (-1.0 to 1.0)
    pub fn score(&self) -> f64 {
        match self {
            TrendMomentum::StronglyAccelerating => 1.0,
            TrendMomentum::Accelerating => 0.5,
            TrendMomentum::Steady => 0.0,
            TrendMomentum::Decelerating => -0.5,
            TrendMomentum::StronglyDecelerating => -1.0,
            TrendMomentum::Insufficient => 0.0,
        }
    }
    
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            TrendMomentum::StronglyAccelerating => "Strongly accelerating performance",
            TrendMomentum::Accelerating => "Accelerating performance",
            TrendMomentum::Steady => "Steady performance",
            TrendMomentum::Decelerating => "Decelerating performance",
            TrendMomentum::StronglyDecelerating => "Strongly decelerating performance",
            TrendMomentum::Insufficient => "Insufficient data for analysis",
        }
    }
    
    /// Check if momentum indicates concern
    pub fn is_concerning(&self) -> bool {
        matches!(self, TrendMomentum::StronglyDecelerating | TrendMomentum::Decelerating)
    }
}

/// Priority levels for recommendations with urgency assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Priority {
    None,
    Low,
    Medium,
    High,
}

impl Priority {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Priority::None => "No action needed",
            Priority::Low => "Consider when convenient",
            Priority::Medium => "Should address when possible",
            Priority::High => "Should address promptly",
        }
    }
    
    /// Get urgency score (0.0 to 1.0)
    pub fn urgency_score(&self) -> f64 {
        match self {
            Priority::None => 0.0,
            Priority::Low => 0.3,
            Priority::Medium => 0.6,
            Priority::High => 1.0,
        }
    }
    
    /// Get recommended timeframe for action
    pub fn timeframe(&self) -> &'static str {
        match self {
            Priority::None => "No timeline",
            Priority::Low => "Within weeks",
            Priority::Medium => "Within days",
            Priority::High => "Within hours",
        }
    }
    
    /// Check if priority requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, Priority::High)
    }
}
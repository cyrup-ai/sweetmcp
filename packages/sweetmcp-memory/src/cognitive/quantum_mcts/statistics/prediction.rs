//! Performance prediction with confidence analysis
//!
//! This module provides PerformancePrediction for forecasting future
//! performance with blazing-fast confidence assessment and reliability metrics.

use serde::Serialize;
use super::trend_types::PredictionReliability;

/// Future performance prediction with confidence metrics
#[derive(Debug, Clone, Serialize)]
pub struct PerformancePrediction {
    /// Hours into the future for this prediction
    pub hours_ahead: f64,
    /// Predicted number of additional nodes
    pub predicted_nodes: usize,
    /// Predicted number of additional visits
    pub predicted_visits: u64,
    /// Predicted change in convergence score
    pub predicted_convergence_change: f64,
    /// Confidence in prediction (0.0 to 1.0)
    pub confidence: f64,
    /// Assumptions underlying the prediction
    pub assumptions: Vec<String>,
}

impl PerformancePrediction {
    /// Create new performance prediction
    pub fn new(
        hours_ahead: f64,
        predicted_nodes: usize,
        predicted_visits: u64,
        predicted_convergence_change: f64,
        confidence: f64,
        assumptions: Vec<String>,
    ) -> Self {
        Self {
            hours_ahead,
            predicted_nodes,
            predicted_visits,
            predicted_convergence_change,
            confidence,
            assumptions,
        }
    }
    
    /// Check if prediction indicates positive trajectory
    pub fn is_positive(&self) -> bool {
        self.predicted_nodes > 0 && 
        self.predicted_visits > 0 && 
        self.predicted_convergence_change > 0.0
    }
    
    /// Get reliability assessment of the prediction
    pub fn reliability_assessment(&self) -> PredictionReliability {
        match self.confidence {
            c if c > 0.8 => PredictionReliability::High,
            c if c > 0.6 => PredictionReliability::Moderate,
            c if c > 0.4 => PredictionReliability::Low,
            _ => PredictionReliability::VeryLow,
        }
    }
    
    /// Calculate predicted nodes per hour rate
    pub fn nodes_per_hour_rate(&self) -> f64 {
        if self.hours_ahead > 0.0 {
            self.predicted_nodes as f64 / self.hours_ahead
        } else {
            0.0
        }
    }
    
    /// Calculate predicted visits per hour rate
    pub fn visits_per_hour_rate(&self) -> f64 {
        if self.hours_ahead > 0.0 {
            self.predicted_visits as f64 / self.hours_ahead
        } else {
            0.0
        }
    }
    
    /// Get prediction quality score (0.0 to 1.0)
    pub fn quality_score(&self) -> f64 {
        let positive_score = if self.is_positive() { 0.4 } else { 0.0 };
        let confidence_score = self.confidence * 0.4;
        let reliability_score = self.reliability_assessment().confidence_score() * 0.2;
        
        positive_score + confidence_score + reliability_score
    }
    
    /// Check if prediction is actionable (reliable enough for decisions)
    pub fn is_actionable(&self) -> bool {
        self.reliability_assessment().is_acceptable() && self.confidence > 0.5
    }
    
    /// Get prediction summary
    pub fn summary(&self) -> String {
        let trajectory = if self.is_positive() { "Positive" } else { "Negative" };
        let reliability = self.reliability_assessment().description();
        
        format!(
            "{} trajectory in {:.1}h: {} nodes, {} visits ({})",
            trajectory,
            self.hours_ahead,
            self.predicted_nodes,
            self.predicted_visits,
            reliability
        )
    }
    
    /// Get confidence level description
    pub fn confidence_description(&self) -> String {
        match self.confidence {
            c if c > 0.9 => "Very high confidence".to_string(),
            c if c > 0.7 => "High confidence".to_string(),
            c if c > 0.5 => "Moderate confidence".to_string(),
            c if c > 0.3 => "Low confidence".to_string(),
            _ => "Very low confidence".to_string(),
        }
    }
}
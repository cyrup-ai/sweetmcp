//! Confidence level management for semantic memory
//!
//! This module provides blazing-fast confidence level operations with zero allocation
//! optimizations and elegant ergonomic interfaces for semantic confidence handling.

use serde::{Deserialize, Serialize};

/// Confidence level enum with optimized ordering and conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Very low confidence (0.1)
    VeryLow,
    /// Low confidence (0.3)
    Low,
    /// Medium confidence (0.5)
    Medium,
    /// High confidence (0.7)
    High,
    /// Very high confidence (0.9)
    VeryHigh,
}

impl ConfidenceLevel {
    /// Convert confidence level to float with inlined fast path
    #[inline]
    pub fn to_float(&self) -> f32 {
        match self {
            ConfidenceLevel::VeryLow => 0.1,
            ConfidenceLevel::Low => 0.3,
            ConfidenceLevel::Medium => 0.5,
            ConfidenceLevel::High => 0.7,
            ConfidenceLevel::VeryHigh => 0.9,
        }
    }

    /// Convert float to confidence level with optimized thresholds
    #[inline]
    pub fn from_float(value: f32) -> Self {
        // Use optimized threshold comparison for blazing-fast performance
        if value < 0.2 {
            ConfidenceLevel::VeryLow
        } else if value < 0.4 {
            ConfidenceLevel::Low
        } else if value < 0.6 {
            ConfidenceLevel::Medium
        } else if value < 0.8 {
            ConfidenceLevel::High
        } else {
            ConfidenceLevel::VeryHigh
        }
    }

    /// Get all confidence levels in order
    #[inline]
    pub fn all() -> [ConfidenceLevel; 5] {
        [
            ConfidenceLevel::VeryLow,
            ConfidenceLevel::Low,
            ConfidenceLevel::Medium,
            ConfidenceLevel::High,
            ConfidenceLevel::VeryHigh,
        ]
    }

    /// Get confidence level name as static string for zero allocation
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryLow => "VeryLow",
            ConfidenceLevel::Low => "Low",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::High => "High",
            ConfidenceLevel::VeryHigh => "VeryHigh",
        }
    }

    /// Parse confidence level from string with optimized matching
    #[inline]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "VeryLow" => Some(ConfidenceLevel::VeryLow),
            "Low" => Some(ConfidenceLevel::Low),
            "Medium" => Some(ConfidenceLevel::Medium),
            "High" => Some(ConfidenceLevel::High),
            "VeryHigh" => Some(ConfidenceLevel::VeryHigh),
            _ => None,
        }
    }

    /// Check if confidence level meets minimum threshold
    #[inline]
    pub fn meets_threshold(&self, threshold: ConfidenceLevel) -> bool {
        *self >= threshold
    }

    /// Calculate confidence score between two levels
    #[inline]
    pub fn similarity_score(&self, other: &ConfidenceLevel) -> f32 {
        let diff = (self.to_float() - other.to_float()).abs();
        1.0 - diff
    }

    /// Combine two confidence levels using weighted average
    #[inline]
    pub fn combine(&self, other: &ConfidenceLevel, weight: f32) -> Self {
        let combined_value = self.to_float() * weight + other.to_float() * (1.0 - weight);
        Self::from_float(combined_value)
    }

    /// Boost confidence level by specified amount
    #[inline]
    pub fn boost(&self, amount: f32) -> Self {
        let boosted_value = (self.to_float() + amount).clamp(0.0, 1.0);
        Self::from_float(boosted_value)
    }

    /// Reduce confidence level by specified amount
    #[inline]
    pub fn reduce(&self, amount: f32) -> Self {
        let reduced_value = (self.to_float() - amount).clamp(0.0, 1.0);
        Self::from_float(reduced_value)
    }

    /// Get next higher confidence level
    #[inline]
    pub fn next_higher(&self) -> Option<Self> {
        match self {
            ConfidenceLevel::VeryLow => Some(ConfidenceLevel::Low),
            ConfidenceLevel::Low => Some(ConfidenceLevel::Medium),
            ConfidenceLevel::Medium => Some(ConfidenceLevel::High),
            ConfidenceLevel::High => Some(ConfidenceLevel::VeryHigh),
            ConfidenceLevel::VeryHigh => None,
        }
    }

    /// Get next lower confidence level
    #[inline]
    pub fn next_lower(&self) -> Option<Self> {
        match self {
            ConfidenceLevel::VeryLow => None,
            ConfidenceLevel::Low => Some(ConfidenceLevel::VeryLow),
            ConfidenceLevel::Medium => Some(ConfidenceLevel::Low),
            ConfidenceLevel::High => Some(ConfidenceLevel::Medium),
            ConfidenceLevel::VeryHigh => Some(ConfidenceLevel::High),
        }
    }

    /// Check if confidence level is high (High or VeryHigh)
    #[inline]
    pub fn is_high(&self) -> bool {
        matches!(self, ConfidenceLevel::High | ConfidenceLevel::VeryHigh)
    }

    /// Check if confidence level is low (VeryLow or Low)
    #[inline]
    pub fn is_low(&self) -> bool {
        matches!(self, ConfidenceLevel::VeryLow | ConfidenceLevel::Low)
    }

    /// Check if confidence level is medium
    #[inline]
    pub fn is_medium(&self) -> bool {
        matches!(self, ConfidenceLevel::Medium)
    }
}

impl Default for ConfidenceLevel {
    #[inline]
    fn default() -> Self {
        ConfidenceLevel::Medium
    }
}

impl std::fmt::Display for ConfidenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<f32> for ConfidenceLevel {
    #[inline]
    fn from(value: f32) -> Self {
        Self::from_float(value)
    }
}

impl From<ConfidenceLevel> for f32 {
    #[inline]
    fn from(level: ConfidenceLevel) -> Self {
        level.to_float()
    }
}

/// Confidence calculator for semantic operations
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// Calculate confidence based on multiple factors
    #[inline]
    pub fn calculate_combined_confidence(
        factors: &[(ConfidenceLevel, f32)],
    ) -> ConfidenceLevel {
        if factors.is_empty() {
            return ConfidenceLevel::default();
        }

        let total_weight: f32 = factors.iter().map(|(_, weight)| weight).sum();
        if total_weight == 0.0 {
            return ConfidenceLevel::default();
        }

        let weighted_sum: f32 = factors
            .iter()
            .map(|(level, weight)| level.to_float() * weight)
            .sum();

        let average = weighted_sum / total_weight;
        ConfidenceLevel::from_float(average)
    }

    /// Calculate confidence decay over time
    #[inline]
    pub fn calculate_time_decay(
        initial_confidence: ConfidenceLevel,
        days_elapsed: f32,
        decay_rate: f32,
    ) -> ConfidenceLevel {
        let decay_factor = (-decay_rate * days_elapsed).exp();
        let decayed_value = initial_confidence.to_float() * decay_factor;
        ConfidenceLevel::from_float(decayed_value)
    }

    /// Calculate confidence boost from reinforcement
    #[inline]
    pub fn calculate_reinforcement_boost(
        current_confidence: ConfidenceLevel,
        reinforcement_strength: f32,
    ) -> ConfidenceLevel {
        let boost_amount = reinforcement_strength * (1.0 - current_confidence.to_float());
        let boosted_value = current_confidence.to_float() + boost_amount;
        ConfidenceLevel::from_float(boosted_value.clamp(0.0, 1.0))
    }

    /// Calculate confidence from agreement between multiple sources
    #[inline]
    pub fn calculate_agreement_confidence(
        confidences: &[ConfidenceLevel],
    ) -> ConfidenceLevel {
        if confidences.is_empty() {
            return ConfidenceLevel::default();
        }

        if confidences.len() == 1 {
            return confidences[0];
        }

        // Calculate mean and variance
        let mean: f32 = confidences.iter().map(|c| c.to_float()).sum::<f32>() / confidences.len() as f32;
        
        let variance: f32 = confidences
            .iter()
            .map(|c| (c.to_float() - mean).powi(2))
            .sum::<f32>() / confidences.len() as f32;

        // Higher agreement (lower variance) increases confidence
        let agreement_factor = 1.0 - variance.sqrt().min(1.0);
        let adjusted_confidence = mean * agreement_factor;
        
        ConfidenceLevel::from_float(adjusted_confidence)
    }

    /// Calculate minimum confidence threshold for decision making
    #[inline]
    pub fn calculate_decision_threshold(
        risk_tolerance: f32,
        consequence_severity: f32,
    ) -> ConfidenceLevel {
        // Higher risk tolerance and lower consequences allow lower confidence thresholds
        let base_threshold = 0.5;
        let risk_adjustment = (1.0 - risk_tolerance) * 0.3;
        let severity_adjustment = consequence_severity * 0.2;
        
        let threshold = base_threshold + risk_adjustment + severity_adjustment;
        ConfidenceLevel::from_float(threshold.clamp(0.1, 0.9))
    }
}

/// Confidence statistics for monitoring and analysis
#[derive(Debug, Clone)]
pub struct ConfidenceStatistics {
    pub total_items: usize,
    pub confidence_distribution: [usize; 5], // Count for each confidence level
    pub average_confidence: f32,
    pub confidence_variance: f32,
}

impl ConfidenceStatistics {
    /// Create new confidence statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_items: 0,
            confidence_distribution: [0; 5],
            average_confidence: 0.0,
            confidence_variance: 0.0,
        }
    }

    /// Calculate statistics from confidence levels
    #[inline]
    pub fn from_confidences(confidences: &[ConfidenceLevel]) -> Self {
        if confidences.is_empty() {
            return Self::new();
        }

        let mut distribution = [0; 5];
        let mut total_confidence = 0.0;

        for confidence in confidences {
            let index = match confidence {
                ConfidenceLevel::VeryLow => 0,
                ConfidenceLevel::Low => 1,
                ConfidenceLevel::Medium => 2,
                ConfidenceLevel::High => 3,
                ConfidenceLevel::VeryHigh => 4,
            };
            distribution[index] += 1;
            total_confidence += confidence.to_float();
        }

        let average = total_confidence / confidences.len() as f32;
        
        let variance = confidences
            .iter()
            .map(|c| (c.to_float() - average).powi(2))
            .sum::<f32>() / confidences.len() as f32;

        Self {
            total_items: confidences.len(),
            confidence_distribution: distribution,
            average_confidence: average,
            confidence_variance: variance,
        }
    }

    /// Get percentage distribution of confidence levels
    #[inline]
    pub fn percentage_distribution(&self) -> [f32; 5] {
        if self.total_items == 0 {
            return [0.0; 5];
        }

        let mut percentages = [0.0; 5];
        for (i, &count) in self.confidence_distribution.iter().enumerate() {
            percentages[i] = (count as f32 / self.total_items as f32) * 100.0;
        }
        percentages
    }

    /// Get dominant confidence level
    #[inline]
    pub fn dominant_confidence_level(&self) -> Option<ConfidenceLevel> {
        let max_index = self.confidence_distribution
            .iter()
            .enumerate()
            .max_by_key(|&(_, count)| *count)
            .map(|(index, _)| index)?;

        match max_index {
            0 => Some(ConfidenceLevel::VeryLow),
            1 => Some(ConfidenceLevel::Low),
            2 => Some(ConfidenceLevel::Medium),
            3 => Some(ConfidenceLevel::High),
            4 => Some(ConfidenceLevel::VeryHigh),
            _ => None,
        }
    }

    /// Check if confidence distribution is balanced
    #[inline]
    pub fn is_balanced(&self) -> bool {
        if self.total_items == 0 {
            return true;
        }

        let expected_count = self.total_items as f32 / 5.0;
        let max_deviation = self.confidence_distribution
            .iter()
            .map(|&count| (count as f32 - expected_count).abs())
            .fold(0.0, f32::max);

        max_deviation <= expected_count * 0.5 // Allow 50% deviation
    }
}

impl Default for ConfidenceStatistics {
    fn default() -> Self {
        Self::new()
    }
}
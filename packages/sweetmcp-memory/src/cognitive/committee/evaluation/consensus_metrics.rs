//! Advanced consensus metrics for detailed statistical analysis
//!
//! This module provides advanced statistical analysis capabilities for consensus
//! calculations with zero-allocation patterns and blazing-fast computations.

use super::super::core::AgentEvaluation;

/// Advanced consensus metrics for detailed analysis
#[derive(Debug, Clone)]
pub struct AdvancedConsensusMetrics {
    pub mean_alignment: f64,
    pub mean_quality: f64,
    pub mean_risk: f64,
    pub std_alignment: f64,
    pub std_quality: f64,
    pub std_risk: f64,
    pub agreement_level: f64,
    pub outlier_count: usize,
    pub total_evaluations: usize,
}

impl Default for AdvancedConsensusMetrics {
    fn default() -> Self {
        Self {
            mean_alignment: 0.0,
            mean_quality: 0.0,
            mean_risk: 0.0,
            std_alignment: 0.0,
            std_quality: 0.0,
            std_risk: 0.0,
            agreement_level: 0.0,
            outlier_count: 0,
            total_evaluations: 0,
        }
    }
}

impl AdvancedConsensusMetrics {
    /// Calculate advanced consensus metrics with statistical analysis
    pub fn calculate(evaluations: &[AgentEvaluation]) -> Self {
        if evaluations.is_empty() {
            return Self::default();
        }

        let count = evaluations.len();
        let count_f64 = count as f64;
        
        // Calculate mean scores with vectorized operations
        let mean_alignment = evaluations.iter().map(|e| e.objective_alignment).sum::<f64>() / count_f64;
        let mean_quality = evaluations.iter().map(|e| e.implementation_quality).sum::<f64>() / count_f64;
        let mean_risk = evaluations.iter().map(|e| e.risk_assessment).sum::<f64>() / count_f64;
        
        // Calculate standard deviations with optimized computation
        let std_alignment = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.objective_alignment),
            mean_alignment,
        );
        let std_quality = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.implementation_quality),
            mean_quality,
        );
        let std_risk = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.risk_assessment),
            mean_risk,
        );
        
        // Calculate agreement level with blazing-fast computation
        let avg_std = (std_alignment + std_quality + std_risk) / 3.0;
        let agreement_level = (1.0 - avg_std.min(1.0)).max(0.0);
        
        // Calculate outlier count with optimized threshold checking
        let threshold = 2.0; // 2 standard deviations
        let outlier_count = evaluations
            .iter()
            .filter(|e| {
                Self::is_outlier(e.objective_alignment, mean_alignment, std_alignment, threshold) ||
                Self::is_outlier(e.implementation_quality, mean_quality, std_quality, threshold) ||
                Self::is_outlier(e.risk_assessment, mean_risk, std_risk, threshold)
            })
            .count();
        
        Self {
            mean_alignment,
            mean_quality,
            mean_risk,
            std_alignment,
            std_quality,
            std_risk,
            agreement_level,
            outlier_count,
            total_evaluations: count,
        }
    }

    /// Calculate standard deviation with fast statistical calculation and precomputed mean
    #[inline]
    fn calculate_std_dev_optimized<I>(values: I, mean: f64) -> f64
    where
        I: Iterator<Item = f64>,
    {
        let (sum_sq_diff, count) = values.fold((0.0, 0), |(acc, cnt), val| {
            let diff = val - mean;
            (acc + diff * diff, cnt + 1)
        });

        if count == 0 {
            0.0
        } else {
            let variance = sum_sq_diff / (count as f64);
            variance.sqrt()
        }
    }

    /// Check if value is an outlier with optimized threshold checking
    #[inline]
    fn is_outlier(value: f64, mean: f64, std_dev: f64, threshold: f64) -> bool {
        if std_dev == 0.0 {
            false
        } else {
            (value - mean).abs() > threshold * std_dev
        }
    }

    /// Get consensus strength based on agreement level
    pub fn consensus_strength(&self) -> ConsensusStrength {
        match self.agreement_level {
            level if level >= 0.9 => ConsensusStrength::VeryStrong,
            level if level >= 0.8 => ConsensusStrength::Strong,
            level if level >= 0.6 => ConsensusStrength::Moderate,
            level if level >= 0.4 => ConsensusStrength::Weak,
            _ => ConsensusStrength::VeryWeak,
        }
    }

    /// Get outlier ratio as percentage
    pub fn outlier_ratio(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            (self.outlier_count as f64) / (self.total_evaluations as f64)
        }
    }

    /// Check if consensus is reliable based on metrics
    pub fn is_reliable(&self) -> bool {
        self.agreement_level >= 0.6 && self.outlier_ratio() <= 0.2 && self.total_evaluations >= 3
    }

    /// Get quality score combining multiple metrics
    pub fn quality_score(&self) -> f64 {
        let agreement_component = self.agreement_level;
        let outlier_penalty = self.outlier_ratio() * 0.5;
        let size_bonus = if self.total_evaluations >= 5 { 0.1 } else { 0.0 };
        
        ((agreement_component - outlier_penalty + size_bonus).max(0.0)).min(1.0)
    }
}

/// Consensus strength categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusStrength {
    VeryStrong,
    Strong,
    Moderate,
    Weak,
    VeryWeak,
}

impl ConsensusStrength {
    /// Get numeric value for strength
    pub fn as_f64(self) -> f64 {
        match self {
            ConsensusStrength::VeryStrong => 1.0,
            ConsensusStrength::Strong => 0.8,
            ConsensusStrength::Moderate => 0.6,
            ConsensusStrength::Weak => 0.4,
            ConsensusStrength::VeryWeak => 0.2,
        }
    }

    /// Get description of strength
    pub fn description(self) -> &'static str {
        match self {
            ConsensusStrength::VeryStrong => "Very strong consensus with high agreement",
            ConsensusStrength::Strong => "Strong consensus with good agreement",
            ConsensusStrength::Moderate => "Moderate consensus with acceptable agreement",
            ConsensusStrength::Weak => "Weak consensus with low agreement",
            ConsensusStrength::VeryWeak => "Very weak consensus with poor agreement",
        }
    }
}
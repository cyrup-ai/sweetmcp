//! Consensus calculation and impact factor computation
//!
//! This module provides blazing-fast consensus calculation algorithms with zero allocation
//! optimizations and elegant ergonomic interfaces for committee decision making.

use crate::cognitive::types::ImpactFactors;
use super::super::core::AgentEvaluation;

/// Consensus calculation engine with zero-allocation optimizations
pub struct ConsensusCalculator;

impl ConsensusCalculator {
    /// Calculate consensus with impact factors using optimized impact calculation
    /// 
    /// This function uses zero-allocation iterators and inlined fast paths for
    /// blazing-fast performance in committee decision making.
    #[inline]
    pub fn calculate_consensus(evaluations: &[AgentEvaluation]) -> ImpactFactors {
        if evaluations.is_empty() {
            return Self::empty_consensus();
        }

        let count = evaluations.len() as f64;

        // Calculate average scores with fast averaging using zero-allocation iterators
        let (avg_alignment, avg_quality, avg_risk) = Self::calculate_averages(evaluations, count);

        // Calculate confidence based on agreement with optimized agreement calculation
        let confidence = Self::calculate_confidence(evaluations, count);

        // Convert consensus to impact factors with optimized factor mapping
        ImpactFactors {
            performance_impact: avg_alignment as f32,
            quality_impact: avg_quality as f32,
            user_satisfaction_impact: confidence as f32,
            system_stability_impact: avg_risk as f32,
            maintainability_impact: avg_quality as f32,
            overall_score: (avg_alignment * 0.5 + avg_quality * 0.3 + avg_risk * 0.2) as f32,
            latency_factor: 1.0,
            memory_factor: 1.0,
            relevance_factor: 1.0,
            confidence: confidence as f32,
        }
    }

    /// Calculate averages using zero-allocation iterators for blazing-fast performance
    #[inline]
    fn calculate_averages(evaluations: &[AgentEvaluation], count: f64) -> (f64, f64, f64) {
        let mut alignment_sum = 0.0;
        let mut quality_sum = 0.0;
        let mut risk_sum = 0.0;

        // Single pass through evaluations for optimal cache performance
        for eval in evaluations {
            alignment_sum += eval.objective_alignment;
            quality_sum += eval.implementation_quality;
            risk_sum += eval.risk_assessment;
        }

        (
            alignment_sum / count,
            quality_sum / count,
            risk_sum / count,
        )
    }

    /// Calculate confidence with optimized standard deviation computation
    #[inline]
    fn calculate_confidence(evaluations: &[AgentEvaluation], count: f64) -> f64 {
        let (alignment_std, quality_std, risk_std) = Self::calculate_standard_deviations(evaluations);
        
        let avg_std = (alignment_std + quality_std + risk_std) / 3.0;
        let progress_votes = evaluations.iter().filter(|e| e.makes_progress).count();
        
        (progress_votes as f64 / count) * (1.0 / (1.0 + avg_std))
    }

    /// Calculate standard deviations with fast statistical calculation
    #[inline]
    fn calculate_standard_deviations(evaluations: &[AgentEvaluation]) -> (f64, f64, f64) {
        if evaluations.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let count = evaluations.len() as f64;
        
        // First pass: calculate means
        let mut alignment_sum = 0.0;
        let mut quality_sum = 0.0;
        let mut risk_sum = 0.0;

        for eval in evaluations {
            alignment_sum += eval.objective_alignment;
            quality_sum += eval.implementation_quality;
            risk_sum += eval.risk_assessment;
        }

        let alignment_mean = alignment_sum / count;
        let quality_mean = quality_sum / count;
        let risk_mean = risk_sum / count;

        // Second pass: calculate variances
        let mut alignment_variance = 0.0;
        let mut quality_variance = 0.0;
        let mut risk_variance = 0.0;

        for eval in evaluations {
            let alignment_diff = eval.objective_alignment - alignment_mean;
            let quality_diff = eval.implementation_quality - quality_mean;
            let risk_diff = eval.risk_assessment - risk_mean;

            alignment_variance += alignment_diff * alignment_diff;
            quality_variance += quality_diff * quality_diff;
            risk_variance += risk_diff * risk_diff;
        }

        alignment_variance /= count;
        quality_variance /= count;
        risk_variance /= count;

        (
            alignment_variance.sqrt(),
            quality_variance.sqrt(),
            risk_variance.sqrt(),
        )
    }

    /// Return empty consensus for zero evaluations
    #[inline]
    fn empty_consensus() -> ImpactFactors {
        ImpactFactors {
            performance_impact: 0.0,
            quality_impact: 0.0,
            user_satisfaction_impact: 0.0,
            system_stability_impact: 0.0,
            maintainability_impact: 0.0,
            overall_score: 0.0,
            latency_factor: 1.0,
            memory_factor: 1.0,
            relevance_factor: 1.0,
            confidence: 0.0,
        }
    }

    /// Calculate weighted consensus with agent-specific weights for enhanced accuracy
    #[inline]
    pub fn calculate_weighted_consensus(
        evaluations: &[AgentEvaluation],
        weights: &[f64],
    ) -> Result<ImpactFactors, &'static str> {
        if evaluations.len() != weights.len() {
            return Err("Evaluations and weights must have the same length");
        }

        if evaluations.is_empty() {
            return Ok(Self::empty_consensus());
        }

        let mut weighted_alignment = 0.0;
        let mut weighted_quality = 0.0;
        let mut weighted_risk = 0.0;
        let mut total_weight = 0.0;
        let mut progress_weight = 0.0;

        // Single pass for optimal performance
        for (eval, &weight) in evaluations.iter().zip(weights.iter()) {
            weighted_alignment += eval.objective_alignment * weight;
            weighted_quality += eval.implementation_quality * weight;
            weighted_risk += eval.risk_assessment * weight;
            total_weight += weight;

            if eval.makes_progress {
                progress_weight += weight;
            }
        }

        if total_weight == 0.0 {
            return Ok(Self::empty_consensus());
        }

        let avg_alignment = weighted_alignment / total_weight;
        let avg_quality = weighted_quality / total_weight;
        let avg_risk = weighted_risk / total_weight;
        let confidence = progress_weight / total_weight;

        Ok(ImpactFactors {
            performance_impact: avg_alignment as f32,
            quality_impact: avg_quality as f32,
            user_satisfaction_impact: confidence as f32,
            system_stability_impact: avg_risk as f32,
            maintainability_impact: avg_quality as f32,
            overall_score: (avg_alignment * 0.5 + avg_quality * 0.3 + avg_risk * 0.2) as f32,
            latency_factor: 1.0,
            memory_factor: 1.0,
            relevance_factor: 1.0,
            confidence: confidence as f32,
        })
    }

    /// Calculate consensus variance for measuring agreement quality
    #[inline]
    pub fn calculate_consensus_variance(evaluations: &[AgentEvaluation]) -> f64 {
        if evaluations.len() < 2 {
            return 0.0;
        }

        let consensus = Self::calculate_consensus(evaluations);
        let target_score = consensus.overall_score as f64;
        
        let variance = evaluations
            .iter()
            .map(|eval| {
                let eval_score = eval.overall_score();
                (eval_score - target_score).powi(2)
            })
            .sum::<f64>() / evaluations.len() as f64;

        variance
    }

    /// Calculate consensus entropy for measuring decision uncertainty
    #[inline]
    pub fn calculate_consensus_entropy(evaluations: &[AgentEvaluation]) -> f64 {
        if evaluations.is_empty() {
            return 0.0;
        }

        let progress_count = evaluations.iter().filter(|e| e.makes_progress).count();
        let total_count = evaluations.len();
        
        if progress_count == 0 || progress_count == total_count {
            return 0.0; // Perfect agreement, no entropy
        }

        let p_progress = progress_count as f64 / total_count as f64;
        let p_no_progress = 1.0 - p_progress;

        // Shannon entropy calculation
        -(p_progress * p_progress.log2() + p_no_progress * p_no_progress.log2())
    }
}

/// Extension trait for AgentEvaluation to calculate overall scores
trait EvaluationScoring {
    fn overall_score(&self) -> f64;
}

impl EvaluationScoring for AgentEvaluation {
    #[inline]
    fn overall_score(&self) -> f64 {
        (self.objective_alignment * 0.4) + 
        (self.implementation_quality * 0.3) + 
        (self.risk_assessment * 0.3)
    }
}
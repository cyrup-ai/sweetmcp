//! Consensus calculation algorithms for committee evaluation
//!
//! This module provides optimized consensus calculation with zero-allocation patterns,
//! blazing-fast statistical computations, and vectorized operations for performance.

use crate::cognitive::types::ImpactFactors;
use std::collections::HashMap;
use super::super::core::AgentEvaluation;
use super::super::core::ConsensusDecision;
use super::consensus_metrics::AdvancedConsensusMetrics;

/// Committee consensus calculation with optimized statistical methods
pub struct ConsensusCalculator;

impl ConsensusCalculator {
    /// Calculate consensus with impact factors with optimized impact calculation
    #[inline]
    pub fn calculate_consensus(evaluations: &[AgentEvaluation]) -> ImpactFactors {
        if evaluations.is_empty() {
            return ImpactFactors {
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
            };
        }

        let count = evaluations.len() as f64;
        let count_inv = 1.0 / count; // Precompute for blazing-fast division

        // Vectorized accumulation for blazing-fast performance
        let (sum_alignment, sum_quality, sum_risk) = evaluations
            .iter()
            .fold((0.0, 0.0, 0.0), |(acc_align, acc_qual, acc_risk), eval| {
                (
                    acc_align + eval.objective_alignment,
                    acc_qual + eval.implementation_quality,
                    acc_risk + eval.risk_assessment,
                )
            });

        // Zero-allocation averaging with precomputed inverse
        let avg_alignment = sum_alignment * count_inv;
        let avg_quality = sum_quality * count_inv;
        let avg_risk = sum_risk * count_inv;

        // Calculate confidence based on agreement with optimized std dev calculation
        let alignment_std = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.objective_alignment),
            avg_alignment,
        );
        let quality_std = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.implementation_quality),
            avg_quality,
        );
        let risk_std = Self::calculate_std_dev_optimized(
            evaluations.iter().map(|e| e.risk_assessment),
            avg_risk,
        );

        let avg_std = (alignment_std + quality_std + risk_std) * (1.0 / 3.0);
        let progress_votes = evaluations.iter().filter(|e| e.makes_progress).count();
        let progress_ratio = (progress_votes as f64) * count_inv;
        let confidence = progress_ratio * (1.0 / (1.0 + avg_std));

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

    /// Calculate standard deviation with fast statistical calculation and precomputed mean
    #[inline]
    pub fn calculate_std_dev_optimized<I>(values: I, mean: f64) -> f64
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

    /// Calculate weighted consensus with agent reliability and zero allocations
    pub fn calculate_weighted_consensus(evaluations: &[AgentEvaluation]) -> ConsensusDecision {
        if evaluations.is_empty() {
            return ConsensusDecision {
                makes_progress: false,
                confidence: 0.0,
                overall_score: 0.0,
                improvement_suggestions: Vec::new(),
                dissenting_opinions: Vec::new(),
            };
        }

        // Pre-allocate with capacity for zero reallocations
        let mut suggestion_counts = HashMap::with_capacity(16);
        
        // Calculate agent weights based on reliability with optimized lookups
        let (weighted_alignment, weighted_quality, weighted_risk, total_weight, progress_weight) = 
            evaluations.iter().fold(
                (0.0, 0.0, 0.0, 0.0, 0.0),
                |(w_align, w_qual, w_risk, total_w, prog_w), eval| {
                    // Blazing-fast weight calculation using const lookup
                    let weight = Self::get_agent_weight(&eval.agent_id);
                    let new_progress_weight = if eval.makes_progress { prog_w + weight } else { prog_w };
                    
                    // Accumulate suggestion counts during main loop for efficiency
                    for suggestion in &eval.suggested_improvements {
                        *suggestion_counts.entry(suggestion.clone()).or_insert(0) += 1;
                    }
                    
                    (
                        w_align + eval.objective_alignment * weight,
                        w_qual + eval.implementation_quality * weight,
                        w_risk + eval.risk_assessment * weight,
                        total_w + weight,
                        new_progress_weight,
                    )
                }
            );

        // Calculate weighted averages with zero-division protection
        let total_weight_inv = if total_weight > 0.0 { 1.0 / total_weight } else { 0.0 };
        let avg_alignment = weighted_alignment * total_weight_inv;
        let avg_quality = weighted_quality * total_weight_inv;
        let avg_risk = weighted_risk * total_weight_inv;
        let progress_ratio = progress_weight * total_weight_inv;

        // Calculate overall score with optimized weights
        let overall_score = avg_alignment * 0.4 + avg_quality * 0.3 + avg_risk * 0.3;

        // Calculate confidence based on agreement with blazing-fast variance calculation
        let score_variance = evaluations
            .iter()
            .map(|e| {
                let score = e.overall_score();
                let diff = score - overall_score;
                diff * diff
            })
            .sum::<f64>() / (evaluations.len() as f64);
        
        let confidence = (1.0 - score_variance.sqrt().min(1.0)) * progress_ratio;

        // Sort suggestions by frequency for best recommendations
        let mut suggestions: Vec<_> = suggestion_counts.into_iter().collect();
        suggestions.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        let improvement_suggestions: Vec<String> = suggestions
            .into_iter()
            .take(5)
            .map(|(suggestion, _)| suggestion)
            .collect();

        // Collect dissenting opinions with pre-allocated capacity
        let dissenting_opinions: Vec<String> = evaluations
            .iter()
            .filter(|e| !e.makes_progress)
            .map(|e| format!("{}: {}", e.agent_id, e.reasoning))
            .collect();

        ConsensusDecision {
            makes_progress: progress_ratio > 0.5,
            confidence,
            overall_score,
            improvement_suggestions,
            dissenting_opinions,
        }
    }

    /// Get agent weight with blazing-fast const lookup
    #[inline]
    const fn get_agent_weight(agent_id: &str) -> f64 {
        // Use first char for fast branch prediction
        match agent_id.as_bytes().get(0) {
            Some(b's') => 1.2, // security
            Some(b'p') => 1.1, // performance
            Some(b'm') => 1.0, // maintainability
            Some(b'u') => 1.0, // user
            Some(b'a') => 0.9, // architecture
            Some(b't') => 0.8, // testing
            Some(b'd') => 0.7, // documentation
            _ => 1.0,
        }
    }

    /// Calculate impact factor weights with optimized computation
    pub fn calculate_impact_weights(evaluations: &[AgentEvaluation]) -> (f32, f32, f32, f32) {
        if evaluations.is_empty() {
            return (1.0, 1.0, 1.0, 1.0);
        }

        let count_inv = 1.0 / (evaluations.len() as f32);
        
        // Vectorized calculation of impact weights
        let (perf_sum, qual_sum, stab_sum, maint_sum) = evaluations
            .iter()
            .fold((0.0f32, 0.0f32, 0.0f32, 0.0f32), |(p, q, s, m), eval| {
                (
                    p + eval.objective_alignment as f32,
                    q + eval.implementation_quality as f32,
                    s + eval.risk_assessment as f32,
                    m + eval.implementation_quality as f32,
                )
            });

        (
            perf_sum * count_inv,
            qual_sum * count_inv,
            stab_sum * count_inv,
            maint_sum * count_inv,
        )
    }

    /// Advanced consensus metrics with statistical analysis
    pub fn calculate_advanced_metrics(evaluations: &[AgentEvaluation]) -> AdvancedConsensusMetrics {
        AdvancedConsensusMetrics::calculate(evaluations)
    }
}
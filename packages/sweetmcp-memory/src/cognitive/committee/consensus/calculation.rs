//! Consensus calculation algorithms extracted from consensus.rs

use std::collections::HashMap;
use super::super::core::{AgentEvaluation, ConsensusDecision};

/// High-performance consensus calculation engine
pub struct ConsensusCalculator {
    threshold: f64,
}

impl ConsensusCalculator {
    /// Create new consensus calculator
    #[inline]
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// Calculate consensus from evaluations with blazing-fast optimization
    pub fn calculate_consensus(&self, evaluations: &[AgentEvaluation]) -> ConsensusDecision {
        if evaluations.is_empty() {
            return ConsensusDecision::empty();
        }

        // Fast progress voting with single pass
        let mut progress_votes = 0;
        let mut total_alignment = 0.0;
        let mut total_quality = 0.0;
        let mut total_safety = 0.0;
        let evaluation_count = evaluations.len() as f64;

        // Single-pass aggregation for maximum performance
        for eval in evaluations {
            if eval.makes_progress {
                progress_votes += 1;
            }
            total_alignment += eval.objective_alignment;
            total_quality += eval.implementation_quality;
            total_safety += eval.risk_assessment;
        }

        let makes_progress = (progress_votes as f64 / evaluation_count) > 0.5;

        // Compute averages
        let avg_alignment = total_alignment / evaluation_count;
        let avg_quality = total_quality / evaluation_count;
        let avg_safety = total_safety / evaluation_count;

        // Weighted overall score calculation
        let overall_score = (avg_alignment * 0.4) + (avg_quality * 0.3) + (avg_safety * 0.3);

        // Fast confidence calculation using variance
        let confidence = self.calculate_confidence(evaluations, overall_score);

        // Aggregate improvement suggestions efficiently
        let improvement_suggestions = self.aggregate_suggestions(evaluations);

        // Collect dissenting opinions efficiently
        let dissenting_opinions = self.collect_dissenting_opinions(evaluations);

        ConsensusDecision {
            makes_progress,
            confidence,
            overall_score,
            improvement_suggestions,
            dissenting_opinions,
        }
    }

    /// Calculate confidence based on score variance with optimized variance calculation
    fn calculate_confidence(&self, evaluations: &[AgentEvaluation], overall_score: f64) -> f64 {
        if evaluations.len() <= 1 {
            return 0.5; // Low confidence with insufficient data
        }

        let variance = evaluations
            .iter()
            .map(|e| {
                let score = e.overall_score();
                let diff = score - overall_score;
                diff * diff
            })
            .sum::<f64>() / evaluations.len() as f64;
        
        let std_dev = variance.sqrt();
        
        // Convert standard deviation to confidence (0-1 scale)
        // Higher agreement (lower std dev) = higher confidence
        (1.0 - std_dev).clamp(0.0, 1.0)
    }

    /// Aggregate improvement suggestions with frequency-based ranking
    fn aggregate_suggestions(&self, evaluations: &[AgentEvaluation]) -> Vec<String> {
        let mut suggestion_counts = HashMap::with_capacity(evaluations.len() * 3); // Estimate capacity
        
        for eval in evaluations {
            for suggestion in &eval.suggested_improvements {
                *suggestion_counts.entry(suggestion.clone()).or_insert(0u32) += 1;
            }
        }

        if suggestion_counts.is_empty() {
            return Vec::new();
        }

        // Sort by frequency and take top 5
        let mut suggestions: Vec<(String, u32)> = suggestion_counts.into_iter().collect();
        suggestions.sort_unstable_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        
        suggestions
            .into_iter()
            .take(5)
            .map(|(suggestion, _)| suggestion)
            .collect()
    }

    /// Collect dissenting opinions efficiently
    fn collect_dissenting_opinions(&self, evaluations: &[AgentEvaluation]) -> Vec<String> {
        evaluations
            .iter()
            .filter(|e| !e.makes_progress)
            .map(|e| format!("{}: {}", e.agent_id, e.reasoning))
            .collect()
    }

    /// Check if consensus meets threshold
    #[inline]
    pub fn meets_threshold(&self, consensus: &ConsensusDecision) -> bool {
        consensus.overall_score >= self.threshold
    }

    /// Calculate consensus strength (how strong the agreement is)
    pub fn calculate_strength(&self, evaluations: &[AgentEvaluation]) -> f64 {
        if evaluations.is_empty() {
            return 0.0;
        }

        let progress_ratio = evaluations.iter()
            .filter(|e| e.makes_progress)
            .count() as f64 / evaluations.len() as f64;

        // Strong consensus when most agents agree on progress direction
        let directional_strength = if progress_ratio > 0.8 || progress_ratio < 0.2 {
            1.0 - (progress_ratio - 0.5).abs() * 2.0
        } else {
            0.0
        };

        // Score variance strength
        let avg_score = evaluations.iter()
            .map(|e| e.overall_score())
            .sum::<f64>() / evaluations.len() as f64;
        
        let score_variance = evaluations.iter()
            .map(|e| (e.overall_score() - avg_score).powi(2))
            .sum::<f64>() / evaluations.len() as f64;
        
        let variance_strength = 1.0 - score_variance.sqrt().min(1.0);

        // Combined strength metric
        (directional_strength * 0.6) + (variance_strength * 0.4)
    }

    /// Analyze consensus quality with detailed metrics
    pub fn analyze_quality(&self, evaluations: &[AgentEvaluation]) -> ConsensusQuality {
        if evaluations.is_empty() {
            return ConsensusQuality::default();
        }

        let total_count = evaluations.len();
        let progress_count = evaluations.iter().filter(|e| e.makes_progress).count();
        let progress_ratio = progress_count as f64 / total_count as f64;

        // Calculate score statistics
        let scores: Vec<f64> = evaluations.iter().map(|e| e.overall_score()).collect();
        let mean_score = scores.iter().sum::<f64>() / scores.len() as f64;
        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate variance and standard deviation
        let variance = scores.iter()
            .map(|&score| (score - mean_score).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std_deviation = variance.sqrt();

        ConsensusQuality {
            total_evaluations: total_count,
            progress_votes: progress_count,
            progress_ratio,
            mean_score,
            min_score,
            max_score,
            std_deviation,
            confidence: self.calculate_confidence(evaluations, mean_score),
            strength: self.calculate_strength(evaluations),
        }
    }
}

/// Consensus quality metrics
#[derive(Debug, Clone, Default)]
pub struct ConsensusQuality {
    pub total_evaluations: usize,
    pub progress_votes: usize,
    pub progress_ratio: f64,
    pub mean_score: f64,
    pub min_score: f64,
    pub max_score: f64,
    pub std_deviation: f64,
    pub confidence: f64,
    pub strength: f64,
}

impl ConsensusQuality {
    /// Check if this represents a high-quality consensus
    pub fn is_high_quality(&self, min_evaluations: usize, min_confidence: f64) -> bool {
        self.total_evaluations >= min_evaluations 
            && self.confidence >= min_confidence
            && self.std_deviation < 0.3 // Low variance in scores
    }

    /// Get a quality score (0.0 to 1.0)
    pub fn quality_score(&self) -> f64 {
        let size_factor = (self.total_evaluations as f64 / 5.0).min(1.0); // Diminishing returns after 5 evaluations
        let confidence_factor = self.confidence;
        let consistency_factor = 1.0 - self.std_deviation.min(1.0);
        
        (size_factor * 0.3) + (confidence_factor * 0.4) + (consistency_factor * 0.3)
    }
}

impl ConsensusDecision {
    /// Create an empty consensus decision
    pub fn empty() -> Self {
        Self {
            makes_progress: false,
            confidence: 0.0,
            overall_score: 0.0,
            improvement_suggestions: Vec::new(),
            dissenting_opinions: Vec::new(),
        }
    }

    /// Check if this decision is actionable
    pub fn is_actionable(&self) -> bool {
        self.makes_progress && self.confidence > 0.3
    }

    /// Get decision strength
    pub fn strength(&self) -> f64 {
        if self.makes_progress {
            self.overall_score * self.confidence
        } else {
            (1.0 - self.overall_score) * self.confidence
        }
    }
}
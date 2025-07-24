//! Agent scores for simulation with validation and utilities
//!
//! This module provides the AgentScores struct and related utilities for
//! custom agent evaluation scoring with zero allocation patterns and
//! blazing-fast performance.

use crate::cognitive::types::CognitiveError;

/// Custom agent scores for simulation
#[derive(Debug, Clone)]
pub struct AgentScores {
    pub alignment: f64,
    pub quality: f64,
    pub risk: f64,
}

impl AgentScores {
    /// Create new agent scores with validation
    pub fn new(alignment: f64, quality: f64, risk: f64) -> Result<Self, CognitiveError> {
        if alignment < 0.0 || alignment > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Alignment must be between 0.0 and 1.0".to_string(),
            ));
        }

        if quality < 0.0 || quality > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Quality must be between 0.0 and 1.0".to_string(),
            ));
        }

        if risk < 0.0 || risk > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Risk must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(Self {
            alignment,
            quality,
            risk,
        })
    }

    /// Create default agent scores
    #[inline]
    pub fn default_scores() -> Self {
        Self {
            alignment: 0.6,
            quality: 0.6,
            risk: 0.6,
        }
    }

    /// Create high-performance agent scores
    #[inline]
    pub fn high_performance_scores() -> Self {
        Self {
            alignment: 0.9,
            quality: 0.85,
            risk: 0.3,
        }
    }

    /// Create conservative agent scores
    #[inline]
    pub fn conservative_scores() -> Self {
        Self {
            alignment: 0.7,
            quality: 0.8,
            risk: 0.2,
        }
    }

    /// Create risky agent scores
    #[inline]
    pub fn risky_scores() -> Self {
        Self {
            alignment: 0.8,
            quality: 0.6,
            risk: 0.9,
        }
    }

    /// Create balanced agent scores
    #[inline]
    pub fn balanced_scores() -> Self {
        Self {
            alignment: 0.75,
            quality: 0.75,
            risk: 0.5,
        }
    }

    /// Calculate overall score combining all metrics
    pub fn overall_score(&self) -> f64 {
        // Weighted combination: alignment (40%), quality (40%), risk (20% inverse)
        (self.alignment * 0.4) + (self.quality * 0.4) + ((1.0 - self.risk) * 0.2)
    }

    /// Get score grade (A-F)
    pub fn grade(&self) -> char {
        let score = self.overall_score();
        if score >= 0.9 { 'A' }
        else if score >= 0.8 { 'B' }
        else if score >= 0.7 { 'C' }
        else if score >= 0.6 { 'D' }
        else { 'F' }
    }

    /// Check if scores are acceptable
    pub fn is_acceptable(&self, min_alignment: f64, min_quality: f64, max_risk: f64) -> bool {
        self.alignment >= min_alignment && 
        self.quality >= min_quality && 
        self.risk <= max_risk
    }

    /// Apply noise to scores for testing
    pub fn with_noise(&self, noise_factor: f64) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.alignment.to_bits().hash(&mut hasher);
        self.quality.to_bits().hash(&mut hasher);
        self.risk.to_bits().hash(&mut hasher);
        let hash = hasher.finish();

        let normalized = (hash as f64) / (u64::MAX as f64);
        let noise = (normalized - 0.5) * 2.0 * noise_factor;

        Self {
            alignment: (self.alignment + noise).max(0.0).min(1.0),
            quality: (self.quality + noise * 0.8).max(0.0).min(1.0),
            risk: (self.risk + noise * 1.2).max(0.0).min(1.0),
        }
    }

    /// Interpolate between two score sets
    pub fn interpolate(&self, other: &AgentScores, factor: f64) -> Self {
        let clamped_factor = factor.max(0.0).min(1.0);
        
        Self {
            alignment: self.alignment + (other.alignment - self.alignment) * clamped_factor,
            quality: self.quality + (other.quality - self.quality) * clamped_factor,
            risk: self.risk + (other.risk - self.risk) * clamped_factor,
        }
    }

    /// Create scores optimized for specific agent perspective
    pub fn for_agent_perspective(agent_id: &str) -> Self {
        match agent_id {
            id if id.contains("performance") => Self {
                alignment: 0.85,
                quality: 0.9,
                risk: 0.4,
            },
            id if id.contains("security") => Self {
                alignment: 0.8,
                quality: 0.85,
                risk: 0.2,
            },
            id if id.contains("maintainability") => Self {
                alignment: 0.75,
                quality: 0.9,
                risk: 0.3,
            },
            id if id.contains("user") => Self {
                alignment: 0.9,
                quality: 0.8,
                risk: 0.35,
            },
            id if id.contains("architecture") => Self {
                alignment: 0.7,
                quality: 0.85,
                risk: 0.25,
            },
            id if id.contains("testing") => Self {
                alignment: 0.65,
                quality: 0.8,
                risk: 0.3,
            },
            id if id.contains("documentation") => Self {
                alignment: 0.6,
                quality: 0.75,
                risk: 0.4,
            },
            _ => Self::default_scores(),
        }
    }

    /// Validate scores are within acceptable ranges
    pub fn validate(&self) -> Result<(), CognitiveError> {
        if self.alignment < 0.0 || self.alignment > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Alignment must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.quality < 0.0 || self.quality > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Quality must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.risk < 0.0 || self.risk > 1.0 {
            return Err(CognitiveError::EvaluationFailed(
                "Risk must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate summary string
    pub fn summary(&self) -> String {
        format!(
            "AgentScores {{ alignment: {:.3}, quality: {:.3}, risk: {:.3}, overall: {:.3} ({}) }}",
            self.alignment, self.quality, self.risk, self.overall_score(), self.grade()
        )
    }

    /// Create scores from individual components with validation
    pub fn from_components(alignment: f64, quality: f64, risk: f64) -> Result<Self, CognitiveError> {
        Self::new(alignment, quality, risk)
    }

    /// Adjust scores based on feedback
    pub fn adjust_with_feedback(&self, alignment_delta: f64, quality_delta: f64, risk_delta: f64) -> Self {
        Self {
            alignment: (self.alignment + alignment_delta).max(0.0).min(1.0),
            quality: (self.quality + quality_delta).max(0.0).min(1.0),
            risk: (self.risk + risk_delta).max(0.0).min(1.0),
        }
    }

    /// Create scores that emphasize specific aspects
    pub fn emphasize_alignment(&self, factor: f64) -> Self {
        let enhanced_alignment = (self.alignment * factor).min(1.0);
        Self {
            alignment: enhanced_alignment,
            quality: self.quality,
            risk: self.risk,
        }
    }

    /// Create scores that emphasize quality
    pub fn emphasize_quality(&self, factor: f64) -> Self {
        let enhanced_quality = (self.quality * factor).min(1.0);
        Self {
            alignment: self.alignment,
            quality: enhanced_quality,
            risk: self.risk,
        }
    }

    /// Create scores that reduce risk
    pub fn reduce_risk(&self, factor: f64) -> Self {
        let reduced_risk = (self.risk * (1.0 - factor)).max(0.0);
        Self {
            alignment: self.alignment,
            quality: self.quality,
            risk: reduced_risk,
        }
    }
}

impl Default for AgentScores {
    fn default() -> Self {
        Self::default_scores()
    }
}

/// Utility functions for working with agent scores
pub mod score_utils {
    use super::*;

    /// Calculate average scores from a collection
    pub fn average_scores(scores: &[AgentScores]) -> Option<AgentScores> {
        if scores.is_empty() {
            return None;
        }

        let count = scores.len() as f64;
        let avg_alignment = scores.iter().map(|s| s.alignment).sum::<f64>() / count;
        let avg_quality = scores.iter().map(|s| s.quality).sum::<f64>() / count;
        let avg_risk = scores.iter().map(|s| s.risk).sum::<f64>() / count;

        Some(AgentScores {
            alignment: avg_alignment,
            quality: avg_quality,
            risk: avg_risk,
        })
    }

    /// Find best scores from a collection
    pub fn best_scores(scores: &[AgentScores]) -> Option<&AgentScores> {
        scores.iter().max_by(|a, b| {
            a.overall_score().partial_cmp(&b.overall_score()).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Find worst scores from a collection
    pub fn worst_scores(scores: &[AgentScores]) -> Option<&AgentScores> {
        scores.iter().min_by(|a, b| {
            a.overall_score().partial_cmp(&b.overall_score()).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Filter scores by criteria
    pub fn filter_scores(
        scores: &[AgentScores],
        min_alignment: Option<f64>,
        min_quality: Option<f64>,
        max_risk: Option<f64>,
    ) -> Vec<&AgentScores> {
        scores.iter().filter(|score| {
            if let Some(min_align) = min_alignment {
                if score.alignment < min_align {
                    return false;
                }
            }
            
            if let Some(min_qual) = min_quality {
                if score.quality < min_qual {
                    return false;
                }
            }
            
            if let Some(max_r) = max_risk {
                if score.risk > max_r {
                    return false;
                }
            }
            
            true
        }).collect()
    }

    /// Generate score distribution statistics
    pub fn score_statistics(scores: &[AgentScores]) -> ScoreStatistics {
        if scores.is_empty() {
            return ScoreStatistics::default();
        }

        let alignments: Vec<f64> = scores.iter().map(|s| s.alignment).collect();
        let qualities: Vec<f64> = scores.iter().map(|s| s.quality).collect();
        let risks: Vec<f64> = scores.iter().map(|s| s.risk).collect();

        ScoreStatistics {
            count: scores.len(),
            avg_alignment: alignments.iter().sum::<f64>() / scores.len() as f64,
            avg_quality: qualities.iter().sum::<f64>() / scores.len() as f64,
            avg_risk: risks.iter().sum::<f64>() / scores.len() as f64,
            min_alignment: alignments.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max_alignment: alignments.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            min_quality: qualities.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max_quality: qualities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            min_risk: risks.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            max_risk: risks.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
        }
    }
}

/// Statistics for a collection of agent scores
#[derive(Debug, Clone)]
pub struct ScoreStatistics {
    pub count: usize,
    pub avg_alignment: f64,
    pub avg_quality: f64,
    pub avg_risk: f64,
    pub min_alignment: f64,
    pub max_alignment: f64,
    pub min_quality: f64,
    pub max_quality: f64,
    pub min_risk: f64,
    pub max_risk: f64,
}

impl Default for ScoreStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            avg_alignment: 0.0,
            avg_quality: 0.0,
            avg_risk: 0.0,
            min_alignment: 0.0,
            max_alignment: 0.0,
            min_quality: 0.0,
            max_quality: 0.0,
            min_risk: 0.0,
            max_risk: 0.0,
        }
    }
}

impl ScoreStatistics {
    /// Generate summary report
    pub fn summary_report(&self) -> String {
        format!(
            "Score Statistics (n={}):
             Alignment: avg={:.3}, range=[{:.3}, {:.3}]
             Quality: avg={:.3}, range=[{:.3}, {:.3}]
             Risk: avg={:.3}, range=[{:.3}, {:.3}]",
            self.count,
            self.avg_alignment, self.min_alignment, self.max_alignment,
            self.avg_quality, self.min_quality, self.max_quality,
            self.avg_risk, self.min_risk, self.max_risk
        )
    }
}

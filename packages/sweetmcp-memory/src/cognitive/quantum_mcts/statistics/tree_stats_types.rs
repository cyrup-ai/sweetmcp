//! Tree statistics type definitions and quality assessments
//!
//! This module provides comprehensive type definitions for tree statistics analysis
//! with blazing-fast performance and zero-allocation patterns.

use serde::Serialize;
use super::{
    types::QuantumTreeStatistics,
    metrics::{RewardStatistics, ConvergenceMetrics},
};

/// Reward quality assessment enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RewardQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl RewardQuality {
    /// Assess reward quality from reward statistics
    pub fn from_reward_stats(stats: &RewardStatistics) -> Self {
        let quality_score = stats.quality_score();
        
        if quality_score > 0.9 {
            RewardQuality::Excellent
        } else if quality_score > 0.8 {
            RewardQuality::Good
        } else if quality_score > 0.6 {
            RewardQuality::Fair
        } else if quality_score > 0.4 {
            RewardQuality::Poor
        } else {
            RewardQuality::Critical
        }
    }
    
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            RewardQuality::Excellent => "Outstanding reward distribution and stability",
            RewardQuality::Good => "Good reward quality with strong patterns",
            RewardQuality::Fair => "Acceptable reward quality with room for improvement",
            RewardQuality::Poor => "Poor reward quality requiring attention",
            RewardQuality::Critical => "Critical reward issues requiring immediate action",
        }
    }
    
    /// Check if quality is acceptable
    pub fn is_acceptable(&self) -> bool {
        matches!(self, RewardQuality::Excellent | RewardQuality::Good | RewardQuality::Fair)
    }
    
    /// Get quality score (0.0 to 1.0)
    pub fn score(&self) -> f64 {
        match self {
            RewardQuality::Excellent => 1.0,
            RewardQuality::Good => 0.8,
            RewardQuality::Fair => 0.6,
            RewardQuality::Poor => 0.4,
            RewardQuality::Critical => 0.2,
        }
    }

    /// Get color code for visualization
    pub fn color_code(&self) -> &'static str {
        match self {
            RewardQuality::Excellent => "green",
            RewardQuality::Good => "lightgreen",
            RewardQuality::Fair => "yellow",
            RewardQuality::Poor => "orange",
            RewardQuality::Critical => "red",
        }
    }

    /// Check if quality requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, RewardQuality::Critical | RewardQuality::Poor)
    }

    /// Get improvement priority (higher = more urgent)
    pub fn improvement_priority(&self) -> u8 {
        match self {
            RewardQuality::Critical => 5,
            RewardQuality::Poor => 4,
            RewardQuality::Fair => 3,
            RewardQuality::Good => 2,
            RewardQuality::Excellent => 1,
        }
    }
}

/// Convergence phase analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConvergencePhase {
    Initial,
    Exploring,
    Converging,
    Converged,
    Stagnant,
}

impl ConvergencePhase {
    /// Determine convergence phase from metrics
    pub fn from_convergence_metrics(metrics: &ConvergenceMetrics) -> Self {
        let convergence_score = metrics.overall_convergence;
        let amplitude_concentration = metrics.amplitude_concentration;
        let visit_concentration = metrics.visit_concentration;
        
        if convergence_score > 0.9 && amplitude_concentration > 0.8 {
            ConvergencePhase::Converged
        } else if convergence_score > 0.7 && (amplitude_concentration > 0.6 || visit_concentration > 0.6) {
            ConvergencePhase::Converging
        } else if convergence_score > 0.4 {
            ConvergencePhase::Exploring
        } else if convergence_score < 0.1 {
            ConvergencePhase::Initial
        } else {
            ConvergencePhase::Stagnant
        }
    }
    
    /// Get phase description
    pub fn description(&self) -> &'static str {
        match self {
            ConvergencePhase::Initial => "Initial exploration phase - building search tree",
            ConvergencePhase::Exploring => "Active exploration - gathering information",
            ConvergencePhase::Converging => "Converging - focusing on promising areas",
            ConvergencePhase::Converged => "Converged - search highly focused",
            ConvergencePhase::Stagnant => "Stagnant - limited progress detected",
        }
    }
    
    /// Check if phase indicates good progress
    pub fn is_making_progress(&self) -> bool {
        !matches!(self, ConvergencePhase::Stagnant)
    }
    
    /// Get expected next phase
    pub fn expected_next_phase(&self) -> Option<ConvergencePhase> {
        match self {
            ConvergencePhase::Initial => Some(ConvergencePhase::Exploring),
            ConvergencePhase::Exploring => Some(ConvergencePhase::Converging),
            ConvergencePhase::Converging => Some(ConvergencePhase::Converged),
            ConvergencePhase::Converged => None, // Terminal state
            ConvergencePhase::Stagnant => Some(ConvergencePhase::Exploring), // Recovery
        }
    }

    /// Get phase duration estimate in iterations
    pub fn typical_duration_estimate(&self) -> Option<u32> {
        match self {
            ConvergencePhase::Initial => Some(100),
            ConvergencePhase::Exploring => Some(500),
            ConvergencePhase::Converging => Some(200),
            ConvergencePhase::Converged => None, // Indefinite
            ConvergencePhase::Stagnant => None, // Problem state
        }
    }

    /// Get phase efficiency score (0.0 to 1.0)
    pub fn efficiency_score(&self) -> f64 {
        match self {
            ConvergencePhase::Initial => 0.3,
            ConvergencePhase::Exploring => 0.7,
            ConvergencePhase::Converging => 0.9,
            ConvergencePhase::Converged => 1.0,
            ConvergencePhase::Stagnant => 0.1,
        }
    }

    /// Check if phase is terminal (no further progression expected)
    pub fn is_terminal(&self) -> bool {
        matches!(self, ConvergencePhase::Converged)
    }

    /// Check if phase indicates problems
    pub fn indicates_problems(&self) -> bool {
        matches!(self, ConvergencePhase::Stagnant)
    }
}

/// Overall convergence health assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConvergenceHealth {
    Excellent,
    Healthy,
    Concerning,
    Poor,
    Critical,
}

impl ConvergenceHealth {
    /// Assess convergence health from metrics and phase
    pub fn from_metrics_and_phase(metrics: &ConvergenceMetrics, phase: ConvergencePhase) -> Self {
        let convergence_score = metrics.overall_convergence;
        let stability = metrics.stability_score();
        
        // Adjust assessment based on phase appropriateness
        let phase_appropriate = match phase {
            ConvergencePhase::Initial => convergence_score < 0.3,
            ConvergencePhase::Exploring => convergence_score >= 0.2 && convergence_score < 0.8,
            ConvergencePhase::Converging => convergence_score >= 0.6 && convergence_score < 0.95,
            ConvergencePhase::Converged => convergence_score >= 0.85,
            ConvergencePhase::Stagnant => false, // Never appropriate
        };
        
        if phase == ConvergencePhase::Stagnant {
            ConvergenceHealth::Critical
        } else if phase_appropriate && stability > 0.8 && convergence_score > 0.7 {
            ConvergenceHealth::Excellent
        } else if phase_appropriate && stability > 0.6 && convergence_score > 0.5 {
            ConvergenceHealth::Healthy
        } else if stability > 0.4 && convergence_score > 0.3 {
            ConvergenceHealth::Concerning
        } else if convergence_score > 0.1 {
            ConvergenceHealth::Poor
        } else {
            ConvergenceHealth::Critical
        }
    }
    
    /// Get health description
    pub fn description(&self) -> &'static str {
        match self {
            ConvergenceHealth::Excellent => "Excellent convergence with stable progress",
            ConvergenceHealth::Healthy => "Healthy convergence patterns",
            ConvergenceHealth::Concerning => "Concerning convergence trends",
            ConvergenceHealth::Poor => "Poor convergence requiring intervention",
            ConvergenceHealth::Critical => "Critical convergence failure",
        }
    }
    
    /// Check if health is acceptable
    pub fn is_acceptable(&self) -> bool {
        matches!(self, ConvergenceHealth::Excellent | ConvergenceHealth::Healthy | ConvergenceHealth::Concerning)
    }
    
    /// Get recommendations for improvement
    pub fn get_recommendations(&self) -> Vec<&'static str> {
        match self {
            ConvergenceHealth::Excellent => vec![
                "Continue current strategy",
                "Monitor for any degradation",
            ],
            ConvergenceHealth::Healthy => vec![
                "Maintain current approach",
                "Consider minor optimizations",
            ],
            ConvergenceHealth::Concerning => vec![
                "Monitor convergence trends closely",
                "Consider adjusting exploration parameters",
                "Review reward function stability",
            ],
            ConvergenceHealth::Poor => vec![
                "Increase exploration rate",
                "Review and adjust MCTS parameters",
                "Check for reward function issues",
                "Consider tree pruning",
            ],
            ConvergenceHealth::Critical => vec![
                "Restart search with different parameters",
                "Investigate fundamental algorithm issues",
                "Review problem formulation",
                "Consider alternative search strategies",
            ],
        }
    }

    /// Get health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        match self {
            ConvergenceHealth::Excellent => 1.0,
            ConvergenceHealth::Healthy => 0.8,
            ConvergenceHealth::Concerning => 0.6,
            ConvergenceHealth::Poor => 0.4,
            ConvergenceHealth::Critical => 0.2,
        }
    }

    /// Get urgency level (0-5, higher = more urgent)
    pub fn urgency_level(&self) -> u8 {
        match self {
            ConvergenceHealth::Excellent => 0,
            ConvergenceHealth::Healthy => 1,
            ConvergenceHealth::Concerning => 2,
            ConvergenceHealth::Poor => 4,
            ConvergenceHealth::Critical => 5,
        }
    }

    /// Check if health requires immediate intervention
    pub fn requires_intervention(&self) -> bool {
        matches!(self, ConvergenceHealth::Poor | ConvergenceHealth::Critical)
    }

    /// Get color code for visualization
    pub fn color_code(&self) -> &'static str {
        match self {
            ConvergenceHealth::Excellent => "darkgreen",
            ConvergenceHealth::Healthy => "green",
            ConvergenceHealth::Concerning => "yellow",
            ConvergenceHealth::Poor => "orange",
            ConvergenceHealth::Critical => "red",
        }
    }
}

/// Quality assessment utilities
pub mod quality_utils {
    use super::*;

    /// Calculate combined quality score from multiple assessments
    pub fn combined_quality_score(
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
        phase: ConvergencePhase,
    ) -> f64 {
        let reward_weight = 0.4;
        let health_weight = 0.4;
        let phase_weight = 0.2;

        (reward_quality.score() * reward_weight) +
        (convergence_health.health_score() * health_weight) +
        (phase.efficiency_score() * phase_weight)
    }

    /// Get overall system grade based on quality assessments
    pub fn overall_grade(
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
        phase: ConvergencePhase,
    ) -> char {
        let score = combined_quality_score(reward_quality, convergence_health, phase);
        
        if score >= 0.9 { 'A' }
        else if score >= 0.8 { 'B' }
        else if score >= 0.7 { 'C' }
        else if score >= 0.6 { 'D' }
        else { 'F' }
    }

    /// Check if system is performing acceptably
    pub fn is_system_healthy(
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
        phase: ConvergencePhase,
    ) -> bool {
        reward_quality.is_acceptable() &&
        convergence_health.is_acceptable() &&
        phase.is_making_progress()
    }

    /// Get priority issues that need attention
    pub fn priority_issues(
        reward_quality: RewardQuality,
        convergence_health: ConvergenceHealth,
        phase: ConvergencePhase,
    ) -> Vec<String> {
        let mut issues = Vec::new();

        if reward_quality.requires_immediate_action() {
            issues.push(format!("Reward quality is {}: {}", 
                format!("{:?}", reward_quality).to_lowercase(), 
                reward_quality.description()));
        }

        if convergence_health.requires_intervention() {
            issues.push(format!("Convergence health is {}: {}", 
                format!("{:?}", convergence_health).to_lowercase(), 
                convergence_health.description()));
        }

        if phase.indicates_problems() {
            issues.push(format!("Search is in {} phase: {}", 
                format!("{:?}", phase).to_lowercase(), 
                phase.description()));
        }

        issues
    }
}

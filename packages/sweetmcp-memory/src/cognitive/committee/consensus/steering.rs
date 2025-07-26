//! Steering feedback system extracted from consensus.rs

use super::evaluation_phases::{EvaluationRound, RoundStatistics};

/// Steering feedback generator with intelligent analysis
pub struct SteeringSystem {
    min_rounds_for_feedback: usize,
    improvement_threshold: f64,
}

impl SteeringSystem {
    /// Create new steering system
    pub fn new(min_rounds_for_feedback: usize, improvement_threshold: f64) -> Self {
        Self {
            min_rounds_for_feedback,
            improvement_threshold,
        }
    }

    /// Generate steering feedback with advanced analysis
    pub fn generate_steering_feedback(&self, rounds: &[EvaluationRound]) -> Option<SteeringFeedback> {
        if rounds.len() < self.min_rounds_for_feedback {
            return None;
        }

        let latest_round = &rounds[rounds.len() - 1];
        let previous_round = &rounds[rounds.len() - 2];

        let latest_stats = latest_round.statistics();
        let previous_stats = previous_round.statistics();

        // Analyze score progression
        let score_improvement = latest_stats.average_score - previous_stats.average_score;
        let progress_improvement = latest_stats.progress_ratio - previous_stats.progress_ratio;

        // Determine feedback type based on improvement patterns
        let feedback_type = self.determine_feedback_type(score_improvement, progress_improvement);
        
        let message = match feedback_type {
            FeedbackType::Encouraging => self.generate_encouraging_feedback(&latest_stats, &previous_stats),
            FeedbackType::Corrective => self.generate_corrective_feedback(&latest_stats, &previous_stats, rounds),
            FeedbackType::Refocusing => self.generate_refocusing_feedback(&latest_stats, latest_round),
            FeedbackType::Concluding => self.generate_concluding_feedback(&latest_stats, rounds.len()),
        };

        Some(SteeringFeedback {
            feedback_type,
            message,
            suggested_focus_areas: self.identify_focus_areas(&latest_stats, latest_round),
            continue_evaluation: feedback_type != FeedbackType::Concluding,
            confidence: self.calculate_feedback_confidence(&latest_stats, score_improvement),
        })
    }

    /// Determine appropriate feedback type
    fn determine_feedback_type(&self, score_improvement: f64, progress_improvement: f64) -> FeedbackType {
        if score_improvement >= self.improvement_threshold && progress_improvement >= 0.1 {
            FeedbackType::Encouraging
        } else if score_improvement < -0.1 || progress_improvement < -0.2 {
            FeedbackType::Corrective
        } else if score_improvement.abs() < 0.05 && progress_improvement.abs() < 0.05 {
            FeedbackType::Refocusing
        } else {
            FeedbackType::Concluding
        }
    }

    /// Generate encouraging feedback for positive progress
    fn generate_encouraging_feedback(&self, latest: &RoundStatistics, previous: &RoundStatistics) -> String {
        format!(
            "Excellent progress! Score improved from {:.2} to {:.2} (+{:.2}). \
            Progress consensus increased from {:.1}% to {:.1}%. \
            Continue refining based on the strongest suggestions. \
            Focus areas: {}",
            previous.average_score,
            latest.average_score,
            latest.average_score - previous.average_score,
            previous.progress_ratio * 100.0,
            latest.progress_ratio * 100.0,
            self.format_focus_areas(&["quality improvements", "risk mitigation"])
        )
    }

    /// Generate corrective feedback for declining performance
    fn generate_corrective_feedback(
        &self, 
        latest: &RoundStatistics, 
        previous: &RoundStatistics,
        rounds: &[EvaluationRound]
    ) -> String {
        let declining_areas = self.identify_declining_areas(latest, previous);
        let top_concerns = self.extract_top_concerns(rounds);

        format!(
            "Scores are declining (current: {:.2}, previous: {:.2}, change: {:+.2}). \
            Declining areas: {}. \
            Address these critical concerns: {}. \
            Refocus on fundamental requirements and risk mitigation.",
            latest.average_score,
            previous.average_score,
            latest.average_score - previous.average_score,
            declining_areas.join(", "),
            top_concerns.join(", ")
        )
    }

    /// Generate refocusing feedback for stagnant progress
    fn generate_refocusing_feedback(&self, latest: &RoundStatistics, round: &EvaluationRound) -> String {
        let stagnation_areas = self.identify_stagnation_areas(latest);
        let fresh_perspectives = self.suggest_fresh_perspectives(round);

        format!(
            "Progress has stagnated (score: {:.2}, progress: {:.1}%). \
            Stagnant areas: {}. \
            Try these fresh perspectives: {}. \
            Consider alternative approaches or break down the problem differently.",
            latest.average_score,
            latest.progress_ratio * 100.0,
            stagnation_areas.join(", "),
            fresh_perspectives.join(", ")
        )
    }

    /// Generate concluding feedback for final decisions
    fn generate_concluding_feedback(&self, latest: &RoundStatistics, total_rounds: usize) -> String {
        let recommendation = if latest.average_score >= 0.7 {
            "RECOMMENDED"
        } else if latest.progress_ratio >= 0.6 {
            "CONDITIONALLY RECOMMENDED"
        } else {
            "NOT RECOMMENDED"
        };

        format!(
            "Final assessment after {} rounds: {} \
            (score: {:.2}, progress consensus: {:.1}%). \
            Decision confidence: {}. \
            Ready for implementation decision.",
            total_rounds,
            recommendation,
            latest.average_score,
            latest.progress_ratio * 100.0,
            if latest.average_score >= 0.8 { "HIGH" } else if latest.average_score >= 0.6 { "MEDIUM" } else { "LOW" }
        )
    }

    /// Identify key focus areas based on current statistics
    fn identify_focus_areas(&self, stats: &RoundStatistics, round: &EvaluationRound) -> Vec<String> {
        let mut focus_areas = Vec::new();

        if stats.average_alignment < 0.6 {
            focus_areas.push("objective alignment".to_string());
        }
        if stats.average_quality < 0.6 {
            focus_areas.push("implementation quality".to_string());
        }
        if stats.average_safety < 0.6 {
            focus_areas.push("risk assessment".to_string());
        }
        if stats.error_count > 0 {
            focus_areas.push("evaluation reliability".to_string());
        }
        if round.evaluations.len() < 3 {
            focus_areas.push("evaluation coverage".to_string());
        }

        // Default focus areas if none identified
        if focus_areas.is_empty() {
            focus_areas.extend_from_slice(&[
                "fine-tuning details".to_string(),
                "edge case handling".to_string(),
                "performance optimization".to_string(),
            ]);
        }

        focus_areas
    }

    /// Identify declining performance areas
    fn identify_declining_areas(&self, latest: &RoundStatistics, previous: &RoundStatistics) -> Vec<String> {
        let mut declining = Vec::new();

        if latest.average_alignment < previous.average_alignment - 0.05 {
            declining.push("alignment".to_string());
        }
        if latest.average_quality < previous.average_quality - 0.05 {
            declining.push("quality".to_string());
        }
        if latest.average_safety < previous.average_safety - 0.05 {
            declining.push("safety".to_string());
        }
        if latest.progress_ratio < previous.progress_ratio - 0.1 {
            declining.push("progress consensus".to_string());
        }

        if declining.is_empty() {
            declining.push("overall confidence".to_string());
        }

        declining
    }

    /// Extract top concerns from evaluation rounds
    fn extract_top_concerns(&self, rounds: &[EvaluationRound]) -> Vec<String> {
        let mut all_suggestions = Vec::new();
        
        for round in rounds.iter().rev().take(2) { // Look at last 2 rounds
            for eval in &round.evaluations {
                if !eval.makes_progress {
                    all_suggestions.extend(eval.suggested_improvements.iter().cloned());
                }
            }
        }

        // Take unique concerns, limited to top 3
        let mut unique_concerns: Vec<String> = all_suggestions.into_iter().collect();
        unique_concerns.sort();
        unique_concerns.dedup();
        unique_concerns.into_iter().take(3).collect()
    }

    /// Identify areas showing stagnation
    fn identify_stagnation_areas(&self, stats: &RoundStatistics) -> Vec<String> {
        let mut stagnant = Vec::new();

        if stats.average_score < 0.6 {
            stagnant.push("overall scoring".to_string());
        }
        if stats.progress_ratio < 0.5 && stats.progress_ratio > 0.3 {
            stagnant.push("progress consensus".to_string());
        }
        if stats.error_count > 0 {
            stagnant.push("evaluation quality".to_string());
        }

        if stagnant.is_empty() {
            stagnant.push("incremental improvement".to_string());
        }

        stagnant
    }

    /// Suggest fresh perspectives for stagnant evaluations
    fn suggest_fresh_perspectives(&self, round: &EvaluationRound) -> Vec<String> {
        let suggestions = vec![
            "architectural alternatives".to_string(),
            "user experience impact".to_string(),
            "performance implications".to_string(),
            "maintainability concerns".to_string(),
            "security considerations".to_string(),
        ];

        // Return subset based on phase
        match round.phase {
            super::evaluation_phases::EvaluationPhase::Initial => suggestions.into_iter().take(2).collect(),
            super::evaluation_phases::EvaluationPhase::Review => suggestions.into_iter().skip(1).take(2).collect(),
            super::evaluation_phases::EvaluationPhase::Refine => suggestions.into_iter().skip(2).collect(),
        }
    }

    /// Calculate confidence in the steering feedback
    fn calculate_feedback_confidence(&self, stats: &RoundStatistics, score_improvement: f64) -> f64 {
        let base_confidence = if stats.total_evaluations >= 5 { 0.8 } else { 0.6 };
        let improvement_factor = (score_improvement * 2.0).clamp(-0.2, 0.2);
        let error_penalty = stats.error_count as f64 * 0.1;
        
        (base_confidence + improvement_factor - error_penalty).clamp(0.0, 1.0)
    }

    /// Format focus areas into readable string
    fn format_focus_areas(&self, areas: &[&str]) -> String {
        if areas.is_empty() {
            "general improvement".to_string()
        } else {
            areas.join(", ")
        }
    }
}

/// Types of steering feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackType {
    /// Encouraging feedback for positive progress
    Encouraging,
    /// Corrective feedback for declining performance
    Corrective,
    /// Refocusing feedback for stagnant progress
    Refocusing,
    /// Concluding feedback for final decision
    Concluding,
}

/// Steering feedback with comprehensive guidance
#[derive(Debug, Clone)]
pub struct SteeringFeedback {
    pub feedback_type: FeedbackType,
    pub message: String,
    pub suggested_focus_areas: Vec<String>,
    pub continue_evaluation: bool,
    pub confidence: f64,
}

impl SteeringFeedback {
    /// Check if feedback suggests continuing evaluation
    pub fn should_continue(&self) -> bool {
        self.continue_evaluation && self.confidence > 0.3
    }

    /// Get priority focus area
    pub fn primary_focus(&self) -> Option<&str> {
        self.suggested_focus_areas.first().map(|s| s.as_str())
    }

    /// Get feedback urgency level
    pub fn urgency(&self) -> FeedbackUrgency {
        match self.feedback_type {
            FeedbackType::Corrective => FeedbackUrgency::High,
            FeedbackType::Refocusing => FeedbackUrgency::Medium,
            FeedbackType::Encouraging => FeedbackUrgency::Low,
            FeedbackType::Concluding => FeedbackUrgency::None,
        }
    }
}

/// Feedback urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackUrgency {
    None,
    Low,
    Medium,
    High,
}

impl Default for SteeringSystem {
    fn default() -> Self {
        Self::new(2, 0.1) // Require 2 rounds minimum, 0.1 improvement threshold
    }
}
//! Result types and analysis for quantum improvement operations
//!
//! This module provides comprehensive result structures with blazing-fast analysis
//! methods and zero-allocation performance tracking for improvement operations.

use std::time::Duration;
use super::simulation::DepthResult;

/// Complete improvement result with comprehensive analysis
#[derive(Debug, Clone)]
pub struct ImprovementResult {
    /// Number of recursive depths completed
    pub total_depths: u32,
    /// Final convergence score achieved
    pub final_convergence_score: f64,
    /// Best convergence score during improvement
    pub best_convergence_score: f64,
    /// History of results for each depth
    pub improvement_history: Vec<DepthResult>,
    /// Total time for improvement process
    pub total_time: Duration,
    /// Peak memory usage during improvement
    pub memory_peak: usize,
    /// Overall success indicator
    pub success: bool,
    /// Reason for termination
    pub termination_reason: TerminationReason,
}

impl ImprovementResult {
    /// Create new improvement result
    pub fn new(
        total_depths: u32,
        final_convergence_score: f64,
        best_convergence_score: f64,
        improvement_history: Vec<DepthResult>,
        total_time: Duration,
        memory_peak: usize,
        success: bool,
        termination_reason: TerminationReason,
    ) -> Self {
        Self {
            total_depths,
            final_convergence_score,
            best_convergence_score,
            improvement_history,
            total_time,
            memory_peak,
            success,
            termination_reason,
        }
    }
    
    /// Analyze convergence trend with blazing-fast comparison
    pub fn convergence_trend(&self) -> ConvergenceTrend {
        if self.improvement_history.len() < 2 {
            return ConvergenceTrend::Insufficient;
        }
        
        let first_score = self.improvement_history[0].convergence_score;
        let last_score = self.improvement_history.last().unwrap().convergence_score;
        
        if last_score > first_score + 0.1 {
            ConvergenceTrend::Improving
        } else if last_score < first_score - 0.1 {
            ConvergenceTrend::Degrading
        } else {
            ConvergenceTrend::Stable
        }
    }
    
    /// Calculate average convergence with zero-allocation computation
    pub fn average_convergence(&self) -> f64 {
        if self.improvement_history.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.improvement_history.iter()
            .map(|depth| depth.convergence_score)
            .sum();
        
        sum / self.improvement_history.len() as f64
    }
    
    /// Get improvement efficiency (convergence per time)
    pub fn improvement_efficiency(&self) -> f64 {
        if self.total_time.as_secs_f64() > 0.0 {
            self.final_convergence_score / self.total_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get memory efficiency (convergence per peak memory)
    pub fn memory_efficiency(&self) -> f64 {
        if self.memory_peak > 0 {
            self.final_convergence_score / self.memory_peak as f64
        } else {
            0.0
        }
    }
    
    /// Get improvement velocity (convergence per depth)
    pub fn improvement_velocity(&self) -> f64 {
        if self.total_depths > 0 {
            self.final_convergence_score / self.total_depths as f64
        } else {
            0.0
        }
    }
    
    /// Calculate quality score based on multiple factors
    pub fn quality_score(&self) -> f64 {
        let convergence_weight = 0.4;
        let efficiency_weight = 0.3;
        let consistency_weight = 0.2;
        let success_weight = 0.1;
        
        let convergence_component = self.final_convergence_score;
        let efficiency_component = self.improvement_efficiency().min(1.0);
        let consistency_component = self.calculate_consistency();
        let success_component = if self.success { 1.0 } else { 0.0 };
        
        convergence_component * convergence_weight
            + efficiency_component * efficiency_weight
            + consistency_component * consistency_weight
            + success_component * success_weight
    }
    
    /// Calculate consistency of improvement across depths
    fn calculate_consistency(&self) -> f64 {
        if self.improvement_history.len() < 2 {
            return 1.0;
        }
        
        let scores: Vec<f64> = self.improvement_history.iter()
            .map(|depth| depth.convergence_score)
            .collect();
        
        let mean = self.average_convergence();
        let variance: f64 = scores.iter()
            .map(|score| (score - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;
        
        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean > 0.0 { std_dev / mean } else { 0.0 };
        
        // Convert to consistency score (lower CV = higher consistency)
        (1.0 - coefficient_of_variation).max(0.0)
    }
    
    /// Get improvement acceleration (change in velocity over time)
    pub fn improvement_acceleration(&self) -> f64 {
        if self.improvement_history.len() < 3 {
            return 0.0;
        }
        
        let mid_point = self.improvement_history.len() / 2;
        let early_avg = self.improvement_history[0..mid_point].iter()
            .map(|d| d.convergence_score)
            .sum::<f64>() / mid_point as f64;
        
        let late_avg = self.improvement_history[mid_point..].iter()
            .map(|d| d.convergence_score)
            .sum::<f64>() / (self.improvement_history.len() - mid_point) as f64;
        
        late_avg - early_avg
    }
    
    /// Check if improvement shows exponential growth pattern
    pub fn has_exponential_growth(&self) -> bool {
        if self.improvement_history.len() < 3 {
            return false;
        }
        
        let acceleration = self.improvement_acceleration();
        let trend = self.convergence_trend();
        
        acceleration > 0.1 && trend == ConvergenceTrend::Improving
    }
    
    /// Get depth with maximum convergence
    pub fn best_depth(&self) -> Option<&DepthResult> {
        self.improvement_history.iter()
            .max_by(|a, b| a.convergence_score.partial_cmp(&b.convergence_score).unwrap_or(std::cmp::Ordering::Equal))
    }
    
    /// Get depth with minimum convergence
    pub fn worst_depth(&self) -> Option<&DepthResult> {
        self.improvement_history.iter()
            .min_by(|a, b| a.convergence_score.partial_cmp(&b.convergence_score).unwrap_or(std::cmp::Ordering::Equal))
    }
    
    /// Get performance summary statistics
    pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            quality_score: self.quality_score(),
            efficiency: self.improvement_efficiency(),
            velocity: self.improvement_velocity(),
            acceleration: self.improvement_acceleration(),
            consistency: self.calculate_consistency(),
            memory_efficiency: self.memory_efficiency(),
            success_rate: if self.success { 1.0 } else { 0.0 },
            convergence_trend: self.convergence_trend(),
        }
    }
    
    /// Check if result indicates successful optimization
    pub fn is_optimization_successful(&self) -> bool {
        self.success 
            && self.final_convergence_score > 0.7 
            && self.quality_score() > 0.6
            && matches!(self.convergence_trend(), ConvergenceTrend::Improving | ConvergenceTrend::Stable)
    }
    
    /// Get detailed analysis report as string
    pub fn analysis_report(&self) -> String {
        let summary = self.performance_summary();
        format!(
            "Improvement Analysis Report:\n\
             - Final Convergence: {:.3} (trend: {:?})\n\
             - Success: {} (reason: {:?})\n\
             - Depths: {} completed in {:.3}s\n\
             - Memory Peak: {} bytes\n\
             - Quality Score: {:.3}\n\
             - Efficiency: {:.3} convergence/sec\n\
             - Velocity: {:.3} convergence/depth\n\
             - Acceleration: {:.3}\n\
             - Consistency: {:.3}\n\
             - Memory Efficiency: {:.6} convergence/byte",
            self.final_convergence_score,
            self.convergence_trend(),
            self.success,
            self.termination_reason,
            self.total_depths,
            self.total_time.as_secs_f64(),
            self.memory_peak,
            summary.quality_score,
            summary.efficiency,
            summary.velocity,
            summary.acceleration,
            summary.consistency,
            summary.memory_efficiency
        )
    }
}

/// Performance summary for quick analysis
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub quality_score: f64,
    pub efficiency: f64,
    pub velocity: f64,
    pub acceleration: f64,
    pub consistency: f64,
    pub memory_efficiency: f64,
    pub success_rate: f64,
    pub convergence_trend: ConvergenceTrend,
}

impl PerformanceSummary {
    /// Check if performance is excellent
    pub fn is_excellent(&self) -> bool {
        self.quality_score > 0.8 
            && self.efficiency > 0.1 
            && self.consistency > 0.8
            && matches!(self.convergence_trend, ConvergenceTrend::Improving)
    }
    
    /// Check if performance is good
    pub fn is_good(&self) -> bool {
        self.quality_score > 0.6 
            && self.efficiency > 0.05 
            && self.consistency > 0.6
    }
    
    /// Check if performance needs improvement
    pub fn needs_improvement(&self) -> bool {
        self.quality_score < 0.4 
            || self.efficiency < 0.02
            || matches!(self.convergence_trend, ConvergenceTrend::Degrading)
    }
}

/// Termination reason enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationReason {
    /// High convergence achieved
    HighConvergence,
    /// No improvement detected
    NoImprovement,
    /// Memory pressure detected
    MemoryPressure,
    /// Maximum depth reached
    MaxDepthReached,
    /// Operation timeout
    Timeout,
    /// Error occurred
    Error,
    /// User requested termination
    UserRequested,
    /// Resource exhaustion
    ResourceExhaustion,
}

impl TerminationReason {
    /// Check if termination reason indicates success
    pub fn is_successful(self) -> bool {
        matches!(self, 
            TerminationReason::HighConvergence 
            | TerminationReason::MaxDepthReached
            | TerminationReason::UserRequested
        )
    }
    
    /// Check if termination reason indicates failure
    pub fn is_failure(self) -> bool {
        matches!(self,
            TerminationReason::Error
            | TerminationReason::ResourceExhaustion
            | TerminationReason::MemoryPressure
        )
    }
    
    /// Get human-readable description
    pub fn description(self) -> &'static str {
        match self {
            TerminationReason::HighConvergence => "High convergence achieved",
            TerminationReason::NoImprovement => "No improvement detected",
            TerminationReason::MemoryPressure => "Memory pressure detected",
            TerminationReason::MaxDepthReached => "Maximum depth reached",
            TerminationReason::Timeout => "Operation timeout",
            TerminationReason::Error => "Error occurred",
            TerminationReason::UserRequested => "User requested termination",
            TerminationReason::ResourceExhaustion => "Resource exhaustion",
        }
    }
}

/// Convergence trend analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceTrend {
    /// Convergence is improving over time
    Improving,
    /// Convergence is stable
    Stable,
    /// Convergence is degrading
    Degrading,
    /// Insufficient data for analysis
    Insufficient,
}

impl ConvergenceTrend {
    /// Check if trend is positive
    pub fn is_positive(self) -> bool {
        matches!(self, ConvergenceTrend::Improving | ConvergenceTrend::Stable)
    }
    
    /// Check if trend is negative
    pub fn is_negative(self) -> bool {
        matches!(self, ConvergenceTrend::Degrading)
    }
    
    /// Get trend strength indicator
    pub fn strength_indicator(self) -> &'static str {
        match self {
            ConvergenceTrend::Improving => "↗️ Strong",
            ConvergenceTrend::Stable => "→ Stable", 
            ConvergenceTrend::Degrading => "↘️ Weak",
            ConvergenceTrend::Insufficient => "? Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_improvement_result_analysis() {
        let result = ImprovementResult {
            total_depths: 3,
            final_convergence_score: 0.85,
            best_convergence_score: 0.85,
            improvement_history: vec![
                DepthResult {
                    depth: 0,
                    convergence_score: 0.5,
                    ..Default::default()
                },
                DepthResult {
                    depth: 1,
                    convergence_score: 0.7,
                    ..Default::default()
                },
                DepthResult {
                    depth: 2,
                    convergence_score: 0.85,
                    ..Default::default()
                },
            ],
            total_time: Duration::from_secs(30),
            memory_peak: 500,
            success: true,
            termination_reason: TerminationReason::HighConvergence,
        };
        
        assert_eq!(result.convergence_trend(), ConvergenceTrend::Improving);
        assert!((result.average_convergence() - 0.683).abs() < 0.01); // (0.5+0.7+0.85)/3
        assert!(result.is_optimization_successful());
        assert!(result.has_exponential_growth());
        
        let summary = result.performance_summary();
        assert!(summary.is_good());
    }
    
    #[test]
    fn test_termination_reason_classification() {
        assert!(TerminationReason::HighConvergence.is_successful());
        assert!(!TerminationReason::HighConvergence.is_failure());
        
        assert!(TerminationReason::Error.is_failure());
        assert!(!TerminationReason::Error.is_successful());
        
        assert_eq!(
            TerminationReason::HighConvergence.description(),
            "High convergence achieved"
        );
    }
    
    #[test]
    fn test_convergence_trend_indicators() {
        assert!(ConvergenceTrend::Improving.is_positive());
        assert!(!ConvergenceTrend::Improving.is_negative());
        
        assert!(ConvergenceTrend::Degrading.is_negative());
        assert!(!ConvergenceTrend::Degrading.is_positive());
        
        assert_eq!(
            ConvergenceTrend::Improving.strength_indicator(),
            "↗️ Strong"
        );
    }
}
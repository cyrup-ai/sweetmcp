//! Core type definitions for quantum entanglement module
//!
//! This module defines the fundamental types and enums used throughout the
//! quantum entanglement system with zero allocation patterns and blazing-fast performance.

use std::time::SystemTime;

// Re-export analysis types from parent module
pub use super::analysis::{NetworkTopology, EntanglementDistribution, NetworkAnalysisReport};
pub use super::metrics::EntanglementMetrics;

// Re-export core engine types
pub use super::engine_core::QuantumEntanglementEngine;
pub use super::engine_operations::OptimizationResult;
pub use super::engine_optimization::OptimizationPrediction;
pub use super::engine_analysis::{EngineHealthReport, OptimizationPriority};
pub use super::engine_health::NetworkPerformanceMetrics;
pub use super::engine_health_types::{CriticalNode, CriticalityType, HealthStatus};
pub use super::engine_issue_types::{NetworkIssue, IssueSeverity, IssueCategory};
pub use super::engine_issue_collection::{IssueCollection, IssueSummaryStats};

/// Optimization strategy recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    Emergency,    // Immediate comprehensive optimization required
    Aggressive,   // Significant optimization needed
    Moderate,     // Regular optimization recommended
    Maintenance,  // Light maintenance optimization only
}

impl OptimizationStrategy {
    /// Get strategy description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            OptimizationStrategy::Emergency => "Emergency optimization - network is in critical state",
            OptimizationStrategy::Aggressive => "Aggressive optimization - significant improvements needed",
            OptimizationStrategy::Moderate => "Regular optimization recommended",
            OptimizationStrategy::Maintenance => "Light maintenance optimization only",
        }
    }
    
    /// Get recommended optimization frequency in hours
    #[inline]
    pub const fn frequency_hours(&self) -> u32 {
        match self {
            OptimizationStrategy::Emergency => 1,    // Every hour
            OptimizationStrategy::Aggressive => 4,   // Every 4 hours
            OptimizationStrategy::Moderate => 12,    // Twice daily
            OptimizationStrategy::Maintenance => 24, // Daily
        }
    }

    /// Get optimization intensity level (0.0 to 1.0)
    #[inline]
    pub const fn intensity_level(&self) -> f64 {
        match self {
            OptimizationStrategy::Emergency => 1.0,
            OptimizationStrategy::Aggressive => 0.8,
            OptimizationStrategy::Moderate => 0.5,
            OptimizationStrategy::Maintenance => 0.2,
        }
    }

    /// Check if strategy requires immediate action
    #[inline]
    pub const fn requires_immediate_action(&self) -> bool {
        matches!(self, OptimizationStrategy::Emergency)
    }
}

/// Optimization urgency levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationUrgency {
    Low,
    Medium,
    High,
    Immediate,
}

impl OptimizationUrgency {
    /// Get urgency description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            OptimizationUrgency::Low => "Low urgency - optimize during next maintenance window",
            OptimizationUrgency::Medium => "Medium urgency - optimize within 24 hours",
            OptimizationUrgency::High => "High urgency - optimize within 4 hours",
            OptimizationUrgency::Immediate => "Immediate urgency - optimize now",
        }
    }
    
    /// Get maximum wait time in minutes
    #[inline]
    pub const fn max_wait_minutes(&self) -> u32 {
        match self {
            OptimizationUrgency::Low => 1440,      // 24 hours
            OptimizationUrgency::Medium => 720,    // 12 hours
            OptimizationUrgency::High => 240,      // 4 hours
            OptimizationUrgency::Immediate => 0,   // Now
        }
    }

    /// Get priority score (higher = more urgent)
    #[inline]
    pub const fn priority_score(&self) -> u8 {
        match self {
            OptimizationUrgency::Low => 1,
            OptimizationUrgency::Medium => 2,
            OptimizationUrgency::High => 3,
            OptimizationUrgency::Immediate => 4,
        }
    }

    /// Check if urgency requires immediate attention
    #[inline]
    pub const fn is_critical(&self) -> bool {
        matches!(self, OptimizationUrgency::High | OptimizationUrgency::Immediate)
    }
}

/// Comprehensive health report combining multiple analysis types
#[derive(Debug, Clone)]
pub struct ComprehensiveHealthReport {
    pub analysis_report: NetworkAnalysisReport,
    pub optimization_prediction: OptimizationPrediction,
    pub generated_at: SystemTime,
}

impl ComprehensiveHealthReport {
    /// Create new comprehensive health report
    pub fn new(
        analysis_report: NetworkAnalysisReport,
        optimization_prediction: OptimizationPrediction,
    ) -> Self {
        Self {
            analysis_report,
            optimization_prediction,
            generated_at: SystemTime::now(),
        }
    }

    /// Get overall health grade
    pub fn health_grade(&self) -> char {
        let health_score = self.analysis_report.health.overall_health;
        if health_score >= 90.0 {
            'A'
        } else if health_score >= 80.0 {
            'B'
        } else if health_score >= 70.0 {
            'C'
        } else if health_score >= 60.0 {
            'D'
        } else {
            'F'
        }
    }
    
    /// Check if optimization is strongly recommended
    pub fn strongly_recommends_optimization(&self) -> bool {
        self.analysis_report.recommends_optimization() && 
        self.optimization_prediction.is_optimization_recommended()
    }

    /// Get health score as percentage
    #[inline]
    pub fn health_percentage(&self) -> f64 {
        self.analysis_report.health.overall_health
    }

    /// Get report age in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.generated_at)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Check if report is stale (older than threshold)
    pub fn is_stale(&self, threshold_seconds: u64) -> bool {
        self.age_seconds() > threshold_seconds
    }
}

/// Health trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum HealthTrend {
    Improving,
    Stable,
    Declining,
    Insufficient, // Not enough data
}

impl HealthTrend {
    /// Get trend description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            HealthTrend::Improving => "Network health is improving over time",
            HealthTrend::Stable => "Network health is stable",
            HealthTrend::Declining => "Network health is declining - attention needed",
            HealthTrend::Insufficient => "Insufficient data to determine trend",
        }
    }
    
    /// Check if trend indicates problems
    #[inline]
    pub const fn indicates_problems(&self) -> bool {
        matches!(self, HealthTrend::Declining)
    }

    /// Get trend score (-1.0 to 1.0, where 1.0 is best)
    #[inline]
    pub const fn trend_score(&self) -> f64 {
        match self {
            HealthTrend::Improving => 1.0,
            HealthTrend::Stable => 0.0,
            HealthTrend::Declining => -1.0,
            HealthTrend::Insufficient => 0.0,
        }
    }

    /// Check if trend is positive
    #[inline]
    pub const fn is_positive(&self) -> bool {
        matches!(self, HealthTrend::Improving | HealthTrend::Stable)
    }
}

/// Network optimization context
#[derive(Debug, Clone)]
pub struct OptimizationContext {
    /// Current optimization strategy
    pub strategy: OptimizationStrategy,
    
    /// Current urgency level
    pub urgency: OptimizationUrgency,
    
    /// Health trend
    pub trend: HealthTrend,
    
    /// Last optimization timestamp
    pub last_optimization: Option<SystemTime>,
    
    /// Optimization count since last reset
    pub optimization_count: u32,
}

impl OptimizationContext {
    /// Create new optimization context
    pub fn new(
        strategy: OptimizationStrategy,
        urgency: OptimizationUrgency,
        trend: HealthTrend,
    ) -> Self {
        Self {
            strategy,
            urgency,
            trend,
            last_optimization: None,
            optimization_count: 0,
        }
    }

    /// Update context after optimization
    pub fn update_after_optimization(&mut self) {
        self.last_optimization = Some(SystemTime::now());
        self.optimization_count += 1;
    }

    /// Check if optimization is due based on strategy frequency
    pub fn is_optimization_due(&self) -> bool {
        if let Some(last_opt) = self.last_optimization {
            let elapsed = SystemTime::now()
                .duration_since(last_opt)
                .map(|d| d.as_secs() / 3600) // Convert to hours
                .unwrap_or(u64::MAX);
            
            elapsed >= self.strategy.frequency_hours() as u64
        } else {
            true // Never optimized before
        }
    }

    /// Get time until next scheduled optimization in hours
    pub fn hours_until_next_optimization(&self) -> u64 {
        if let Some(last_opt) = self.last_optimization {
            let elapsed = SystemTime::now()
                .duration_since(last_opt)
                .map(|d| d.as_secs() / 3600)
                .unwrap_or(0);
            
            let frequency = self.strategy.frequency_hours() as u64;
            if elapsed >= frequency {
                0
            } else {
                frequency - elapsed
            }
        } else {
            0 // Never optimized, due now
        }
    }
}
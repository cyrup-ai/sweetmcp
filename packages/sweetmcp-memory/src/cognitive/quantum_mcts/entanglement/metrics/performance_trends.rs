//! Performance trend analysis and indicators
//!
//! This module provides blazing-fast trend analysis with zero-allocation
//! pattern recognition for continuous performance monitoring.

/// Performance trend indicators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceTrend {
    /// Performance is improving (getting faster)
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is degrading (getting slower)
    Degrading,
}

impl PerformanceTrend {
    /// Get trend description
    pub fn description(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Improving",
            PerformanceTrend::Stable => "Stable",
            PerformanceTrend::Degrading => "Degrading",
        }
    }
    
    /// Check if trend is positive
    pub fn is_positive(&self) -> bool {
        matches!(self, PerformanceTrend::Improving | PerformanceTrend::Stable)
    }
    
    /// Check if trend requires attention
    pub fn requires_attention(&self) -> bool {
        matches!(self, PerformanceTrend::Degrading)
    }
    
    /// Get trend score (0.0 to 1.0, higher is better)
    pub fn score(&self) -> f64 {
        match self {
            PerformanceTrend::Improving => 1.0,
            PerformanceTrend::Stable => 0.8,
            PerformanceTrend::Degrading => 0.3,
        }
    }
    
    /// Get trend grade
    pub fn grade(&self) -> char {
        match self {
            PerformanceTrend::Improving => 'A',
            PerformanceTrend::Stable => 'B',
            PerformanceTrend::Degrading => 'D',
        }
    }
    
    /// Get trend emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "📈",
            PerformanceTrend::Stable => "➡️",
            PerformanceTrend::Degrading => "📉",
        }
    }
    
    /// Get detailed description
    pub fn detailed_description(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Performance is improving over time with faster execution",
            PerformanceTrend::Stable => "Performance is stable with consistent execution times",
            PerformanceTrend::Degrading => "Performance is degrading with slower execution times",
        }
    }
    
    /// Get recommendation based on trend
    pub fn recommendation(&self) -> &'static str {
        match self {
            PerformanceTrend::Improving => "Continue current optimizations",
            PerformanceTrend::Stable => "Monitor for any changes",
            PerformanceTrend::Degrading => "Investigate performance bottlenecks",
        }
    }
    
    /// Get action priority level
    pub fn action_priority(&self) -> ActionPriority {
        match self {
            PerformanceTrend::Improving => ActionPriority::Low,
            PerformanceTrend::Stable => ActionPriority::Medium,
            PerformanceTrend::Degrading => ActionPriority::High,
        }
    }
}

/// Action priority levels for performance trends
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionPriority {
    /// Low priority - monitoring only
    Low,
    /// Medium priority - periodic review
    Medium,
    /// High priority - immediate attention needed
    High,
}

impl ActionPriority {
    /// Get priority description
    pub fn description(&self) -> &'static str {
        match self {
            ActionPriority::Low => "Low Priority",
            ActionPriority::Medium => "Medium Priority",
            ActionPriority::High => "High Priority",
        }
    }
    
    /// Get priority level as number (1-3, higher is more urgent)
    pub fn level(&self) -> u8 {
        match self {
            ActionPriority::Low => 1,
            ActionPriority::Medium => 2,
            ActionPriority::High => 3,
        }
    }
    
    /// Check if priority requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, ActionPriority::High)
    }
    
    /// Get priority color code
    pub fn color_code(&self) -> &'static str {
        match self {
            ActionPriority::Low => "green",
            ActionPriority::Medium => "yellow",
            ActionPriority::High => "red",
        }
    }
}

/// Trend analysis utilities
pub struct TrendAnalyzer;

impl TrendAnalyzer {
    /// Analyze trend from sequence of values (zero-allocation)
    pub fn analyze_values(values: &[f64]) -> PerformanceTrend {
        if values.len() < 4 {
            return PerformanceTrend::Stable;
        }
        
        let mid_point = values.len() / 2;
        let first_half = &values[..mid_point];
        let second_half = &values[mid_point..];
        
        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;
        
        let improvement_ratio = first_avg / second_avg;
        
        if improvement_ratio > 1.1 {
            PerformanceTrend::Improving
        } else if improvement_ratio < 0.9 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }
    
    /// Calculate trend confidence (0.0 to 1.0)
    pub fn trend_confidence(values: &[f64]) -> f64 {
        if values.len() < 4 {
            return 0.0;
        }
        
        // Calculate variance to determine confidence
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let coefficient_of_variation = if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            1.0
        };
        
        // Lower variance means higher confidence
        (1.0 - coefficient_of_variation.min(1.0)).max(0.0)
    }
    
    /// Get trend strength (0.0 to 1.0)
    pub fn trend_strength(values: &[f64]) -> f64 {
        if values.len() < 4 {
            return 0.0;
        }
        
        let mid_point = values.len() / 2;
        let first_half = &values[..mid_point];
        let second_half = &values[mid_point..];
        
        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;
        
        if first_avg == 0.0 && second_avg == 0.0 {
            return 0.0;
        }
        
        let max_avg = first_avg.max(second_avg);
        let min_avg = first_avg.min(second_avg);
        
        if max_avg > 0.0 {
            (max_avg - min_avg) / max_avg
        } else {
            0.0
        }
    }
}
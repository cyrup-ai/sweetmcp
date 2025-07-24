//! Benchmark comparison and analysis utilities
//!
//! This module provides comprehensive benchmark comparison capabilities,
//! performance regression detection, and trend analysis with zero-allocation
//! patterns and blazing-fast performance.

use std::time::Duration;
use super::benchmarking_results::{BenchmarkResults, PerformanceCategory, PerformanceTrend, ResultQuality};

/// Comprehensive benchmark comparison analysis
#[derive(Debug, Clone)]
pub struct BenchmarkComparison {
    /// Baseline benchmark results
    pub baseline: BenchmarkResults,
    /// Current benchmark results
    pub current: BenchmarkResults,
    /// Comparison metrics
    pub metrics: ComparisonMetrics,
    /// Regression analysis
    pub regression: RegressionAnalysis,
    /// Improvement analysis
    pub improvement: ImprovementAnalysis,
}

impl BenchmarkComparison {
    /// Create comparison between two benchmark results
    pub fn new(baseline: BenchmarkResults, current: BenchmarkResults) -> Self {
        let metrics = ComparisonMetrics::calculate(&baseline, &current);
        let regression = RegressionAnalysis::analyze(&baseline, &current);
        let improvement = ImprovementAnalysis::analyze(&baseline, &current);
        
        Self {
            baseline,
            current,
            metrics,
            regression,
            improvement,
        }
    }
    
    /// Generate comprehensive comparison report
    pub fn comparison_report(&self) -> String {
        format!(
            "Benchmark Comparison Report\n\
             ===========================\n\
             Baseline: {} ({})\n\
             Current:  {} ({})\n\
             \n\
             Performance Change:\n\
             - Throughput: {:.1}% ({:.1} -> {:.1} ops/sec)\n\
             - Latency: {:.1}% ({:.2}ms -> {:.2}ms avg)\n\
             - Consistency: {:.1}% change in variability\n\
             \n\
             Regression Analysis:\n\
             - Status: {:?}\n\
             - Severity: {:?}\n\
             - Affected Metrics: {:?}\n\
             \n\
             Improvement Analysis:\n\
             - Status: {:?}\n\
             - Magnitude: {:?}\n\
             - Key Improvements: {:?}\n\
             \n\
             Recommendations:\n\
             {}",
            self.baseline.benchmark_name,
            self.baseline.stats.performance_grade(),
            self.current.benchmark_name,
            self.current.stats.performance_grade(),
            self.metrics.throughput_change_percent,
            self.baseline.stats.operations_per_second,
            self.current.stats.operations_per_second,
            self.metrics.latency_change_percent,
            self.baseline.stats.average_duration.as_secs_f64() * 1000.0,
            self.current.stats.average_duration.as_secs_f64() * 1000.0,
            self.metrics.consistency_change_percent,
            self.regression.status,
            self.regression.severity,
            self.regression.affected_metrics,
            self.improvement.status,
            self.improvement.magnitude,
            self.improvement.key_improvements,
            self.generate_recommendations().join("\n- ")
        )
    }
    
    /// Generate actionable recommendations
    pub fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match self.regression.status {
            RegressionStatus::Significant => {
                recommendations.push("Significant performance regression detected - immediate investigation required".to_string());
                recommendations.push("Roll back recent changes and identify root cause".to_string());
            }
            RegressionStatus::Minor => {
                recommendations.push("Minor performance regression - monitor closely".to_string());
            }
            RegressionStatus::None => {
                if matches!(self.improvement.status, ImprovementStatus::Significant) {
                    recommendations.push("Excellent performance improvement achieved".to_string());
                }
            }
        }
        
        if self.metrics.consistency_degraded() {
            recommendations.push("Performance consistency has degraded - investigate variability sources".to_string());
        }
        
        if !self.current.quality.is_sufficient() {
            recommendations.push("Current benchmark quality is insufficient - collect more samples".to_string());
        }
        
        recommendations
    }
    
    /// Check if comparison indicates critical issues
    pub fn has_critical_issues(&self) -> bool {
        matches!(self.regression.status, RegressionStatus::Significant) ||
        matches!(self.current.category, PerformanceCategory::Critical)
    }
    
    /// Get overall comparison verdict
    pub fn verdict(&self) -> ComparisonVerdict {
        if self.has_critical_issues() {
            ComparisonVerdict::Critical
        } else if matches!(self.regression.status, RegressionStatus::Minor) {
            ComparisonVerdict::Concerning
        } else if matches!(self.improvement.status, ImprovementStatus::Significant) {
            ComparisonVerdict::Excellent
        } else {
            ComparisonVerdict::Acceptable
        }
    }
}

/// Detailed comparison metrics
#[derive(Debug, Clone)]
pub struct ComparisonMetrics {
    /// Throughput change percentage
    pub throughput_change_percent: f64,
    /// Latency change percentage (negative = improvement)
    pub latency_change_percent: f64,
    /// Consistency change percentage
    pub consistency_change_percent: f64,
    /// Statistical significance
    pub statistical_significance: f64,
}

impl ComparisonMetrics {
    /// Calculate comparison metrics
    pub fn calculate(baseline: &BenchmarkResults, current: &BenchmarkResults) -> Self {
        let throughput_change = Self::calculate_percentage_change(
            baseline.stats.operations_per_second,
            current.stats.operations_per_second,
        );
        
        let latency_change = Self::calculate_percentage_change(
            baseline.stats.average_duration.as_nanos() as f64,
            current.stats.average_duration.as_nanos() as f64,
        );
        
        let baseline_cv = baseline.stats.coefficient_of_variation();
        let current_cv = current.stats.coefficient_of_variation();
        let consistency_change = Self::calculate_percentage_change(baseline_cv, current_cv);
        
        let significance = Self::calculate_statistical_significance(baseline, current);
        
        Self {
            throughput_change_percent: throughput_change,
            latency_change_percent: latency_change,
            consistency_change_percent: consistency_change,
            statistical_significance: significance,
        }
    }
    
    /// Calculate percentage change
    fn calculate_percentage_change(baseline: f64, current: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }
    
    /// Calculate statistical significance (simplified)
    fn calculate_statistical_significance(baseline: &BenchmarkResults, current: &BenchmarkResults) -> f64 {
        // Simplified significance calculation based on sample sizes and effect size
        let min_samples = baseline.stats.sample_count.min(current.stats.sample_count) as f64;
        let effect_size = (baseline.stats.operations_per_second - current.stats.operations_per_second).abs() 
            / baseline.stats.operations_per_second.max(1.0);
        
        // Higher sample count and larger effect size = higher significance
        (min_samples.sqrt() * effect_size).min(1.0)
    }
    
    /// Check if consistency has degraded significantly
    pub fn consistency_degraded(&self) -> bool {
        self.consistency_change_percent > 20.0 // 20% increase in variability
    }
    
    /// Check if results are statistically significant
    pub fn is_statistically_significant(&self) -> bool {
        self.statistical_significance > 0.7
    }
}

/// Regression analysis results
#[derive(Debug, Clone)]
pub struct RegressionAnalysis {
    /// Regression status
    pub status: RegressionStatus,
    /// Severity level
    pub severity: RegressionSeverity,
    /// Affected performance metrics
    pub affected_metrics: Vec<String>,
    /// Confidence in regression detection
    pub confidence: f64,
}

impl RegressionAnalysis {
    /// Analyze for performance regression
    pub fn analyze(baseline: &BenchmarkResults, current: &BenchmarkResults) -> Self {
        let throughput_regression = baseline.stats.operations_per_second > current.stats.operations_per_second;
        let latency_regression = baseline.stats.average_duration < current.stats.average_duration;
        
        let throughput_change = (current.stats.operations_per_second - baseline.stats.operations_per_second) 
            / baseline.stats.operations_per_second.max(1.0);
        
        let mut affected_metrics = Vec::new();
        let mut severity_score = 0.0;
        
        if throughput_regression && throughput_change < -0.1 {
            affected_metrics.push("Throughput".to_string());
            severity_score += throughput_change.abs();
        }
        
        if latency_regression {
            let latency_change = (current.stats.average_duration.as_nanos() as f64 
                - baseline.stats.average_duration.as_nanos() as f64) 
                / baseline.stats.average_duration.as_nanos() as f64;
            if latency_change > 0.1 {
                affected_metrics.push("Latency".to_string());
                severity_score += latency_change;
            }
        }
        
        let (status, severity) = if severity_score > 0.3 {
            (RegressionStatus::Significant, RegressionSeverity::High)
        } else if severity_score > 0.1 {
            (RegressionStatus::Minor, RegressionSeverity::Medium)
        } else {
            (RegressionStatus::None, RegressionSeverity::Low)
        };
        
        let confidence = if affected_metrics.is_empty() {
            0.0
        } else {
            (severity_score * 2.0).min(1.0)
        };
        
        Self {
            status,
            severity,
            affected_metrics,
            confidence,
        }
    }
}

/// Improvement analysis results
#[derive(Debug, Clone)]
pub struct ImprovementAnalysis {
    /// Improvement status
    pub status: ImprovementStatus,
    /// Improvement magnitude
    pub magnitude: ImprovementMagnitude,
    /// Key improvement areas
    pub key_improvements: Vec<String>,
    /// Confidence in improvement detection
    pub confidence: f64,
}

impl ImprovementAnalysis {
    /// Analyze for performance improvements
    pub fn analyze(baseline: &BenchmarkResults, current: &BenchmarkResults) -> Self {
        let throughput_improvement = current.stats.operations_per_second > baseline.stats.operations_per_second;
        let latency_improvement = current.stats.average_duration < baseline.stats.average_duration;
        
        let throughput_change = (current.stats.operations_per_second - baseline.stats.operations_per_second) 
            / baseline.stats.operations_per_second.max(1.0);
        
        let mut key_improvements = Vec::new();
        let mut improvement_score = 0.0;
        
        if throughput_improvement && throughput_change > 0.1 {
            key_improvements.push("Throughput".to_string());
            improvement_score += throughput_change;
        }
        
        if latency_improvement {
            let latency_change = (baseline.stats.average_duration.as_nanos() as f64 
                - current.stats.average_duration.as_nanos() as f64) 
                / baseline.stats.average_duration.as_nanos() as f64;
            if latency_change > 0.1 {
                key_improvements.push("Latency".to_string());
                improvement_score += latency_change;
            }
        }
        
        let (status, magnitude) = if improvement_score > 0.5 {
            (ImprovementStatus::Significant, ImprovementMagnitude::Major)
        } else if improvement_score > 0.2 {
            (ImprovementStatus::Moderate, ImprovementMagnitude::Moderate)
        } else if improvement_score > 0.05 {
            (ImprovementStatus::Minor, ImprovementMagnitude::Minor)
        } else {
            (ImprovementStatus::None, ImprovementMagnitude::None)
        };
        
        let confidence = if key_improvements.is_empty() {
            0.0
        } else {
            (improvement_score * 1.5).min(1.0)
        };
        
        Self {
            status,
            magnitude,
            key_improvements,
            confidence,
        }
    }
}

/// Regression status levels
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionStatus {
    None,
    Minor,
    Significant,
}

/// Regression severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionSeverity {
    Low,
    Medium,
    High,
}

/// Improvement status levels
#[derive(Debug, Clone, PartialEq)]
pub enum ImprovementStatus {
    None,
    Minor,
    Moderate,
    Significant,
}

/// Improvement magnitude levels
#[derive(Debug, Clone, PartialEq)]
pub enum ImprovementMagnitude {
    None,
    Minor,
    Moderate,
    Major,
}

/// Overall comparison verdict
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonVerdict {
    Critical,    // Significant regression or critical performance
    Concerning,  // Minor regression or concerning trends
    Acceptable,  // No significant changes
    Excellent,   // Significant improvements
}

impl ComparisonVerdict {
    /// Get verdict description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            ComparisonVerdict::Critical => "Critical issues detected - immediate action required",
            ComparisonVerdict::Concerning => "Concerning trends detected - monitoring recommended",
            ComparisonVerdict::Acceptable => "Performance changes are within acceptable limits",
            ComparisonVerdict::Excellent => "Excellent performance improvements achieved",
        }
    }
}
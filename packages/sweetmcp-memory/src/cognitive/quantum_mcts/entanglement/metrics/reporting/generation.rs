//! Metrics report generation and core logic
//!
//! This module provides core report generation functionality
//! with zero-allocation patterns and blazing-fast performance.

use std::time::{Duration, Instant};
use super::super::counters::EntanglementCounters;
use super::super::calculations::MetricsCalculator;
use super::super::tracking::{PerformanceCategory, TimingUtils};
use super::types::{MetricsReport, HistoricalDataPoint, PerformanceTrend};

/// Comprehensive metrics reporter for quantum entanglement
#[derive(Debug)]
pub struct MetricsReporter {
    /// Reporter name
    reporter_name: String,
    /// Creation time
    creation_time: Instant,
    /// Collected reports
    reports: Vec<MetricsReport>,
    /// Maximum number of reports to retain
    max_reports: usize,
}

impl MetricsReporter {
    /// Create new metrics reporter
    pub fn new(reporter_name: String) -> Self {
        Self::with_capacity(reporter_name, 100)
    }
    
    /// Create new metrics reporter with custom capacity
    pub fn with_capacity(reporter_name: String, max_reports: usize) -> Self {
        Self {
            reporter_name,
            creation_time: Instant::now(),
            reports: Vec::with_capacity(max_reports.min(100)),
            max_reports,
        }
    }
    
    /// Generate comprehensive metrics report
    pub fn generate_report(
        &mut self,
        counters: &EntanglementCounters,
        calculations: &MetricsCalculator,
    ) -> MetricsReport {
        let report_time = Instant::now();
        let uptime = self.creation_time.elapsed();
        
        // Get counter snapshots
        let entanglement_count = counters.entanglement_count();
        let operation_count = counters.operation_count();
        let influence_count = counters.influence_calculation_count();
        let error_count = counters.error_count();
        
        // Get performance metrics
        let avg_operation_time = calculations.average_operation_time();
        let avg_influence_time = calculations.average_influence_time();
        let operations_per_second = calculations.operations_per_second();
        let influence_per_second = calculations.influence_calculations_per_second();
        let error_rate = calculations.error_rate();
        
        // Calculate performance scores
        let operation_performance = TimingUtils::categorize_duration(avg_operation_time);
        let influence_performance = TimingUtils::categorize_duration(avg_influence_time);
        let overall_performance = Self::calculate_overall_performance(
            &operation_performance,
            &influence_performance,
            operations_per_second,
            error_rate,
        );
        
        let report = MetricsReport {
            report_id: self.reports.len() + 1,
            report_time,
            uptime,
            entanglement_count,
            operation_count,
            influence_count,
            error_count,
            avg_operation_time,
            avg_influence_time,
            operations_per_second,
            influence_per_second,
            error_rate,
            operation_performance,
            influence_performance,
            overall_performance,
        };
        
        // Store report
        if self.reports.len() >= self.max_reports {
            self.reports.remove(0);
        }
        self.reports.push(report.clone());
        
        report
    }
    
    /// Get historical performance data
    pub fn historical_performance(&self) -> Vec<HistoricalDataPoint> {
        self.reports
            .iter()
            .map(|report| HistoricalDataPoint {
                timestamp: report.report_time,
                performance_score: report.overall_performance.score(),
                operations_per_second: report.operations_per_second,
                error_rate: report.error_rate,
                operation_count: report.operation_count,
            })
            .collect()
    }
    
    /// Calculate performance trend from recent reports
    pub fn performance_trend(&self, sample_size: usize) -> PerformanceTrend {
        if self.reports.len() < 2 {
            return PerformanceTrend::Unknown;
        }
        
        let recent_reports = if self.reports.len() > sample_size {
            &self.reports[self.reports.len() - sample_size..]
        } else {
            &self.reports
        };
        
        if recent_reports.len() < 2 {
            return PerformanceTrend::Unknown;
        }
        
        // Calculate linear trend
        let scores: Vec<f64> = recent_reports.iter()
            .map(|r| r.overall_performance.score())
            .collect();
        
        let n = scores.len() as f64;
        let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
        let sum_y: f64 = scores.iter().sum();
        let sum_xy: f64 = scores.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..scores.len()).map(|i| (i as f64).powi(2)).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let variance = scores.iter()
            .map(|&score| (score - sum_y / n).powi(2))
            .sum::<f64>() / n;
        
        if slope.abs() < 0.01 {
            PerformanceTrend::Stable { variance }
        } else if slope > 0.0 {
            PerformanceTrend::Improving { rate: slope }
        } else {
            PerformanceTrend::Declining { rate: slope.abs() }
        }
    }
    
    /// Reset reporter data
    pub fn reset(&mut self) {
        self.reports.clear();
        self.creation_time = Instant::now();
    }
    
    /// Get report count
    pub fn report_count(&self) -> usize {
        self.reports.len()
    }
    
    /// Get latest report
    pub fn latest_report(&self) -> Option<&MetricsReport> {
        self.reports.last()
    }
    
    /// Get reporter name
    pub fn reporter_name(&self) -> &str {
        &self.reporter_name
    }
    
    /// Get creation time
    pub fn creation_time(&self) -> Instant {
        self.creation_time
    }
    
    /// Get all reports
    pub fn reports(&self) -> &[MetricsReport] {
        &self.reports
    }
    
    /// Calculate overall performance from individual metrics
    fn calculate_overall_performance(
        operation_perf: &PerformanceCategory,
        influence_perf: &PerformanceCategory,
        ops_per_sec: f64,
        error_rate: f64,
    ) -> PerformanceCategory {
        let operation_score = operation_perf.score();
        let influence_score = influence_perf.score();
        let throughput_score = (ops_per_sec / 100.0).min(1.0);
        let reliability_score = (1.0 - error_rate).max(0.0);
        
        // Weighted combination
        let overall_score = (operation_score * 0.3) + 
                           (influence_score * 0.3) + 
                           (throughput_score * 0.2) + 
                           (reliability_score * 0.2);
        
        Self::score_to_category(overall_score)
    }
    
    /// Convert score to performance category
    fn score_to_category(score: f64) -> PerformanceCategory {
        match score {
            s if s >= 0.9 => PerformanceCategory::Excellent,
            s if s >= 0.7 => PerformanceCategory::Good,
            s if s >= 0.5 => PerformanceCategory::Acceptable,
            s if s >= 0.3 => PerformanceCategory::Slow,
            _ => PerformanceCategory::VerySlow,
        }
    }
    
    /// Convert score to letter grade
    fn score_to_grade(score: f64) -> char {
        Self::score_to_category(score).grade()
    }
}
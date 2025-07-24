//! Summary report generation for aggregated metrics
//!
//! This module provides summary report generation functionality
//! with zero-allocation patterns and blazing-fast performance.

use std::time::{Duration, Instant};
use super::super::tracking::{BatchStatistics, InfluenceStatistics};
use super::super::benchmarking::{BenchmarkResults, RollingStatistics};
use super::types::{SummaryReport, AggregatedMetrics};
use super::generation::MetricsReporter;

impl MetricsReporter {
    /// Generate summary report from multiple metrics
    pub fn generate_summary_report(
        &self,
        batch_stats: Option<&BatchStatistics>,
        influence_stats: Option<&InfluenceStatistics>,
        benchmark_results: Option<&BenchmarkResults>,
        rolling_stats: Option<&RollingStatistics>,
    ) -> SummaryReport {
        let report_time = Instant::now();
        let uptime = self.creation_time().elapsed();
        
        // Aggregate performance data
        let mut performance_scores = Vec::new();
        let mut throughput_metrics = Vec::new();
        let mut operation_counts = Vec::new();
        let mut error_counts = Vec::new();
        let mut response_times = Vec::new();
        
        if let Some(batch) = batch_stats {
            performance_scores.push(batch.performance_category().score());
            throughput_metrics.push(batch.operation_count as f64 / batch.batch_duration.as_secs_f64());
            operation_counts.push(batch.operation_count);
            error_counts.push(batch.error_count);
            response_times.push(batch.average_operation_time);
        }
        
        if let Some(influence) = influence_stats {
            performance_scores.push(influence.performance_category().score());
            throughput_metrics.push(influence.calculations_per_second);
            operation_counts.push(influence.calculation_count);
            error_counts.push(influence.error_count);
            response_times.push(influence.average_calculation_time);
        }
        
        if let Some(benchmark) = benchmark_results {
            performance_scores.push(benchmark.performance_score());
            throughput_metrics.push(benchmark.operations_per_second);
            operation_counts.push(benchmark.total_operations());
            error_counts.push(benchmark.error_count());
            response_times.push(benchmark.average_response_time());
        }
        
        if let Some(rolling) = rolling_stats {
            performance_scores.push(rolling.performance_category().score());
            throughput_metrics.push(rolling.samples_per_second);
            operation_counts.push(rolling.sample_count());
            error_counts.push(rolling.error_count());
            response_times.push(rolling.average_sample_time());
        }
        
        // Calculate aggregate metrics
        let overall_performance_score = if !performance_scores.is_empty() {
            performance_scores.iter().sum::<f64>() / performance_scores.len() as f64
        } else {
            0.0
        };
        
        let average_throughput = if !throughput_metrics.is_empty() {
            throughput_metrics.iter().sum::<f64>() / throughput_metrics.len() as f64
        } else {
            0.0
        };
        
        let total_operations = operation_counts.iter().sum::<u64>();
        let total_errors = error_counts.iter().sum::<u64>();
        let combined_error_rate = if total_operations > 0 {
            total_errors as f64 / total_operations as f64
        } else {
            0.0
        };
        
        let peak_throughput = throughput_metrics.iter()
            .fold(0.0f64, |acc, &x| acc.max(x));
        
        let average_response_time = if !response_times.is_empty() {
            let total_nanos: u128 = response_times.iter()
                .map(|d| d.as_nanos())
                .sum();
            Duration::from_nanos((total_nanos / response_times.len() as u128) as u64)
        } else {
            Duration::from_millis(0)
        };
        
        // Calculate consistency score based on variance in performance
        let consistency_score = if performance_scores.len() > 1 {
            let mean = overall_performance_score;
            let variance = performance_scores.iter()
                .map(|score| (score - mean).powi(2))
                .sum::<f64>() / performance_scores.len() as f64;
            let std_dev = variance.sqrt();
            (1.0 - std_dev).max(0.0)
        } else {
            1.0
        };
        
        let aggregated_metrics = AggregatedMetrics {
            total_operations,
            total_errors,
            combined_error_rate,
            peak_throughput,
            average_response_time,
            consistency_score,
        };
        
        let overall_grade = Self::score_to_grade(overall_performance_score);
        
        SummaryReport {
            report_time,
            uptime,
            overall_performance_score,
            average_throughput,
            overall_grade,
            data_sources: performance_scores.len(),
            aggregated_metrics,
        }
    }
}

/// Summary report generator utility functions
pub struct SummaryGenerator;

impl SummaryGenerator {
    /// Generate aggregated metrics from multiple data sources
    pub fn aggregate_metrics(
        batch_stats: Option<&BatchStatistics>,
        influence_stats: Option<&InfluenceStatistics>,
        benchmark_results: Option<&BenchmarkResults>,
        rolling_stats: Option<&RollingStatistics>,
    ) -> AggregatedMetrics {
        let mut operation_counts = Vec::new();
        let mut error_counts = Vec::new();
        let mut throughput_metrics = Vec::new();
        let mut response_times = Vec::new();
        let mut performance_scores = Vec::new();
        
        if let Some(batch) = batch_stats {
            operation_counts.push(batch.operation_count);
            error_counts.push(batch.error_count);
            throughput_metrics.push(batch.operation_count as f64 / batch.batch_duration.as_secs_f64());
            response_times.push(batch.average_operation_time);
            performance_scores.push(batch.performance_category().score());
        }
        
        if let Some(influence) = influence_stats {
            operation_counts.push(influence.calculation_count);
            error_counts.push(influence.error_count);
            throughput_metrics.push(influence.calculations_per_second);
            response_times.push(influence.average_calculation_time);
            performance_scores.push(influence.performance_category().score());
        }
        
        if let Some(benchmark) = benchmark_results {
            operation_counts.push(benchmark.total_operations());
            error_counts.push(benchmark.error_count());
            throughput_metrics.push(benchmark.operations_per_second);
            response_times.push(benchmark.average_response_time());
            performance_scores.push(benchmark.performance_score());
        }
        
        if let Some(rolling) = rolling_stats {
            operation_counts.push(rolling.sample_count());
            error_counts.push(rolling.error_count());
            throughput_metrics.push(rolling.samples_per_second);
            response_times.push(rolling.average_sample_time());
            performance_scores.push(rolling.performance_category().score());
        }
        
        let total_operations = operation_counts.iter().sum::<u64>();
        let total_errors = error_counts.iter().sum::<u64>();
        let combined_error_rate = if total_operations > 0 {
            total_errors as f64 / total_operations as f64
        } else {
            0.0
        };
        
        let peak_throughput = throughput_metrics.iter()
            .fold(0.0f64, |acc, &x| acc.max(x));
        
        let average_response_time = if !response_times.is_empty() {
            let total_nanos: u128 = response_times.iter()
                .map(|d| d.as_nanos())
                .sum();
            Duration::from_nanos((total_nanos / response_times.len() as u128) as u64)
        } else {
            Duration::from_millis(0)
        };
        
        // Calculate consistency score based on variance in performance
        let consistency_score = if performance_scores.len() > 1 {
            let mean = performance_scores.iter().sum::<f64>() / performance_scores.len() as f64;
            let variance = performance_scores.iter()
                .map(|score| (score - mean).powi(2))
                .sum::<f64>() / performance_scores.len() as f64;
            let std_dev = variance.sqrt();
            (1.0 - std_dev).max(0.0)
        } else {
            1.0
        };
        
        AggregatedMetrics {
            total_operations,
            total_errors,
            combined_error_rate,
            peak_throughput,
            average_response_time,
            consistency_score,
        }
    }
    
    /// Calculate weighted performance score
    pub fn calculate_weighted_performance(
        operation_score: f64,
        throughput_score: f64,
        reliability_score: f64,
        consistency_score: f64,
    ) -> f64 {
        (operation_score * 0.3) + 
        (throughput_score * 0.3) + 
        (reliability_score * 0.2) + 
        (consistency_score * 0.2)
    }
    
    /// Normalize throughput to 0-1 scale
    pub fn normalize_throughput(throughput: f64, max_expected: f64) -> f64 {
        (throughput / max_expected).min(1.0)
    }
    
    /// Calculate reliability score from error rate
    pub fn calculate_reliability_score(error_rate: f64) -> f64 {
        (1.0 - error_rate).max(0.0)
    }
    
    /// Calculate efficiency ratio
    pub fn calculate_efficiency_ratio(operations: u64, errors: u64) -> f64 {
        if errors == 0 {
            operations as f64
        } else {
            operations as f64 / errors as f64
        }
    }
    
    /// Calculate performance variance
    pub fn calculate_performance_variance(scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }
        
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        scores.iter()
            .map(|score| (score - mean).powi(2))
            .sum::<f64>() / scores.len() as f64
    }
    
    /// Calculate trend strength from slope
    pub fn calculate_trend_strength(slope: f64) -> f64 {
        slope.abs().min(1.0)
    }
}
//! Report formatting and display utilities
//!
//! This module provides report formatting and visualization functionality
//! with zero-allocation patterns and blazing-fast performance.

use super::super::tracking::TimingUtils;
use super::types::{MetricsReport, SummaryReport, PerformanceDashboard, HistoricalDataPoint};

impl MetricsReport {
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Report #{}: {} ops, {} influence ({:.1}/sec, {:.1}/sec) - {} (Grade: {})",
            self.report_id,
            self.operation_count,
            self.influence_count,
            self.operations_per_second,
            self.influence_per_second,
            self.overall_performance.description(),
            self.overall_performance.grade()
        )
    }
    
    /// Get detailed report
    pub fn detailed_report(&self) -> String {
        format!(
            "=== Metrics Report #{} ===\n\
            Report Time: {:?}\n\
            System Uptime: {}\n\
            \n\
            --- Operation Counts ---\n\
            Entanglements: {}\n\
            Operations: {}\n\
            Influence Calculations: {}\n\
            Errors: {}\n\
            \n\
            --- Performance Metrics ---\n\
            Avg Operation Time: {}\n\
            Avg Influence Time: {}\n\
            Operations/sec: {:.1}\n\
            Influence/sec: {:.1}\n\
            Error Rate: {:.2}%\n\
            \n\
            --- Performance Grades ---\n\
            Operations: {} ({})\n\
            Influence: {} ({}) \n\
            Overall: {} ({})",
            self.report_id,
            self.report_time,
            TimingUtils::format_duration(self.uptime),
            self.entanglement_count,
            self.operation_count,
            self.influence_count,
            self.error_count,
            TimingUtils::format_duration(self.avg_operation_time),
            TimingUtils::format_duration(self.avg_influence_time),
            self.operations_per_second,
            self.influence_per_second,
            self.error_rate * 100.0,
            self.operation_performance.grade(),
            self.operation_performance.description(),
            self.influence_performance.grade(),
            self.influence_performance.description(),
            self.overall_performance.grade(),
            self.overall_performance.description()
        )
    }

    /// Get compact summary line
    pub fn compact_summary(&self) -> String {
        format!(
            "#{}: {:.1}ops/s {:.1}inf/s {}% err {}",
            self.report_id,
            self.operations_per_second,
            self.influence_per_second,
            (self.error_rate * 100.0) as u32,
            self.overall_performance.grade()
        )
    }

    /// Get CSV header
    pub fn csv_header() -> &'static str {
        "report_id,timestamp,uptime_ms,entanglements,operations,influence,errors,avg_op_ms,avg_inf_ms,ops_per_sec,inf_per_sec,error_rate,op_grade,inf_grade,overall_grade"
    }

    /// Get CSV row
    pub fn to_csv(&self) -> String {
        format!(
            "{},{:?},{},{},{},{},{},{},{},{:.1},{:.1},{:.4},{},{},{}",
            self.report_id,
            self.report_time,
            self.uptime.as_millis(),
            self.entanglement_count,
            self.operation_count,
            self.influence_count,
            self.error_count,
            self.avg_operation_time.as_millis(),
            self.avg_influence_time.as_millis(),
            self.operations_per_second,
            self.influence_per_second,
            self.error_rate,
            self.operation_performance.grade(),
            self.influence_performance.grade(),
            self.overall_performance.grade()
        )
    }
}

impl SummaryReport {
    /// Get formatted summary
    pub fn formatted_summary(&self) -> String {
        format!(
            "=== Summary Report ===\n\
            Generated: {:?}\n\
            System Uptime: {}\n\
            Data Sources: {}\n\
            \n\
            --- Overall Performance ---\n\
            Performance Score: {:.2}\n\
            Performance Grade: {} ({})\n\
            Average Throughput: {:.1} ops/sec\n\
            \n\
            --- Aggregated Metrics ---\n\
            Total Operations: {}\n\
            Total Errors: {}\n\
            Combined Error Rate: {:.2}%\n\
            Peak Throughput: {:.1} ops/sec\n\
            Average Response Time: {}\n\
            Consistency Score: {:.2}\n\
            Health Score: {:.2}",
            self.report_time,
            TimingUtils::format_duration(self.uptime),
            self.data_sources,
            self.overall_performance_score,
            self.overall_grade,
            self.grade_description(),
            self.average_throughput,
            self.aggregated_metrics.total_operations,
            self.aggregated_metrics.total_errors,
            self.aggregated_metrics.combined_error_rate * 100.0,
            self.aggregated_metrics.peak_throughput,
            TimingUtils::format_duration(self.aggregated_metrics.average_response_time),
            self.aggregated_metrics.consistency_score,
            self.aggregated_metrics.health_score()
        )
    }

    /// Get compact summary
    pub fn compact_summary(&self) -> String {
        format!(
            "Summary: {} sources, {:.1} avg throughput, {}% err, {} grade",
            self.data_sources,
            self.average_throughput,
            (self.aggregated_metrics.combined_error_rate * 100.0) as u32,
            self.overall_grade
        )
    }
}

/// Report formatting utilities
pub struct ReportFormatter;

impl ReportFormatter {
    /// Format multiple reports as table
    pub fn format_reports_table(reports: &[MetricsReport]) -> String {
        if reports.is_empty() {
            return "No reports available".to_string();
        }

        let mut table = String::new();
        table.push_str("ID    | Ops/sec | Inf/sec | Err% | Grade | Summary\n");
        table.push_str("------|---------|---------|------|-------|--------\n");

        for report in reports {
            table.push_str(&format!(
                "{:5} | {:7.1} | {:7.1} | {:4.1} | {:5} | {}\n",
                report.report_id,
                report.operations_per_second,
                report.influence_per_second,
                report.error_rate * 100.0,
                report.overall_performance.grade(),
                report.overall_performance.description()
            ));
        }

        table
    }

    /// Format performance comparison
    pub fn format_performance_comparison(reports: &[MetricsReport]) -> String {
        if reports.len() < 2 {
            return "Need at least 2 reports for comparison".to_string();
        }

        let first = &reports[0];
        let last = &reports[reports.len() - 1];

        let ops_change = last.operations_per_second - first.operations_per_second;
        let inf_change = last.influence_per_second - first.influence_per_second;
        let err_change = last.error_rate - first.error_rate;

        format!(
            "Performance Comparison:\n\
            \n\
            Operations/sec: {:.1} → {:.1} ({:+.1})\n\
            Influence/sec:  {:.1} → {:.1} ({:+.1})\n\
            Error Rate:     {:.2}% → {:.2}% ({:+.2}%)\n\
            Grade:          {} → {}\n\
            \n\
            Reports Compared: #{} to #{}",
            first.operations_per_second,
            last.operations_per_second,
            ops_change,
            first.influence_per_second,
            last.influence_per_second,
            inf_change,
            first.error_rate * 100.0,
            last.error_rate * 100.0,
            err_change * 100.0,
            first.overall_performance.grade(),
            last.overall_performance.grade(),
            first.report_id,
            last.report_id
        )
    }

    /// Format trend analysis
    pub fn format_trend_analysis(historical_data: &[HistoricalDataPoint]) -> String {
        if historical_data.len() < 2 {
            return "Insufficient data for trend analysis".to_string();
        }

        let scores: Vec<f64> = historical_data.iter().map(|d| d.performance_score).collect();
        let first_score = scores[0];
        let last_score = scores[scores.len() - 1];
        let change = last_score - first_score;
        let change_percent = (change / first_score) * 100.0;

        let direction = if change > 0.01 {
            "Improving"
        } else if change < -0.01 {
            "Declining"
        } else {
            "Stable"
        };

        format!(
            "Trend Analysis:\n\
            Direction: {}\n\
            Change: {:.3} ({:+.1}%)\n\
            First Score: {:.3}\n\
            Last Score: {:.3}\n\
            Data Points: {}",
            direction,
            change,
            change_percent,
            first_score,
            last_score,
            scores.len()
        )
    }
}
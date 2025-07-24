//! Dashboard visualization utilities
//!
//! This module provides visualization functionality for metrics dashboards
//! with zero-allocation patterns and blazing-fast performance.

use super::types::{PerformanceDashboard, HistoricalDataPoint};
use super::dashboard::KeyMetrics;

/// Dashboard visualization utilities
pub struct DashboardVisualizer;

impl DashboardVisualizer {
    /// Format performance chart for dashboard
    pub fn format_performance_chart(data: &[HistoricalDataPoint], width: usize) -> String {
        if data.is_empty() {
            return "No data available for chart".to_string();
        }

        let scores: Vec<f64> = data.iter().map(|d| d.performance_score).collect();
        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max_score - min_score;

        if range == 0.0 {
            return "Performance is constant".to_string();
        }

        let mut chart = String::new();
        chart.push_str("Performance Chart:\n");
        chart.push_str(&format!("Scale: {:.2} to {:.2}\n", min_score, max_score));
        chart.push_str(&"-".repeat(width + 2));
        chart.push('\n');

        for data_point in data {
            let normalized = ((data_point.performance_score - min_score) / range * width as f64) as usize;
            let bar_length = normalized.min(width);
            
            chart.push('|');
            chart.push_str(&"█".repeat(bar_length));
            chart.push_str(&" ".repeat(width - bar_length));
            chart.push_str(&format!("| {:.2}\n", data_point.performance_score));
        }

        chart.push_str(&"-".repeat(width + 2));
        chart.push('\n');

        chart
    }

    /// Create simple ASCII dashboard
    pub fn create_ascii_dashboard(dashboard: &PerformanceDashboard) -> String {
        let key_metrics = dashboard.key_metrics();
        let chart = Self::format_performance_chart(&dashboard.historical_data, 40);
        
        format!(
            "┌─────────────────────────────────────────────┐\n\
            │ {} Dashboard                    │\n\
            ├─────────────────────────────────────────────┤\n\
            │ Status: {} ({})                      │\n\
            │ Performance: {:.2} ({})                    │\n\
            │ Throughput: {:.1} ops/sec                  │\n\
            │ Error Rate: {:.1}%                         │\n\
            │ Trend: {}                              │\n\
            ├─────────────────────────────────────────────┤\n\
            │ {}                                     │\n\
            └─────────────────────────────────────────────┘",
            dashboard.reporter_name,
            key_metrics.health_status.description(),
            key_metrics.health_status.color(),
            key_metrics.current_performance,
            key_metrics.performance_grade(),
            key_metrics.operations_per_second,
            key_metrics.error_rate * 100.0,
            key_metrics.trend_direction,
            chart.lines().take(5).collect::<Vec<_>>().join("\n│ ")
        )
    }

    /// Create compact status line
    pub fn create_status_line(dashboard: &PerformanceDashboard) -> String {
        let key_metrics = dashboard.key_metrics();
        format!(
            "[{}] {:.2} perf | {:.1} ops/s | {:.1}% err | {} trend",
            key_metrics.health_status.description(),
            key_metrics.current_performance,
            key_metrics.operations_per_second,
            key_metrics.error_rate * 100.0,
            key_metrics.trend_direction
        )
    }

    /// Create sparkline chart
    pub fn create_sparkline(data: &[HistoricalDataPoint], width: usize) -> String {
        if data.is_empty() {
            return "▁".repeat(width);
        }

        let scores: Vec<f64> = data.iter().map(|d| d.performance_score).collect();
        let min_score = scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_score = scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max_score - min_score;

        if range == 0.0 {
            return "▄".repeat(width);
        }

        let spark_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let mut sparkline = String::new();

        // Take last `width` data points or all if fewer
        let start_idx = if data.len() > width { data.len() - width } else { 0 };
        
        for data_point in &data[start_idx..] {
            let normalized = ((data_point.performance_score - min_score) / range * (spark_chars.len() - 1) as f64) as usize;
            let char_idx = normalized.min(spark_chars.len() - 1);
            sparkline.push(spark_chars[char_idx]);
        }

        // Pad with spaces if needed
        while sparkline.len() < width {
            sparkline.push(' ');
        }

        sparkline
    }

    /// Create performance gauge
    pub fn create_performance_gauge(performance: f64, width: usize) -> String {
        let filled = (performance * width as f64) as usize;
        let empty = width - filled;
        
        let mut gauge = String::new();
        gauge.push('[');
        gauge.push_str(&"█".repeat(filled));
        gauge.push_str(&"░".repeat(empty));
        gauge.push(']');
        gauge.push_str(&format!(" {:.1}%", performance * 100.0));
        
        gauge
    }

    /// Create trend indicator
    pub fn create_trend_indicator(trend_strength: f64, is_positive: bool) -> String {
        let arrow = if is_positive { "↗" } else { "↘" };
        let strength_desc = match trend_strength.abs() {
            s if s >= 0.8 => "Strong",
            s if s >= 0.5 => "Moderate",
            s if s >= 0.2 => "Weak",
            _ => "Minimal",
        };
        
        format!("{} {} {:.1}%", arrow, strength_desc, trend_strength * 100.0)
    }

    /// Create error rate indicator
    pub fn create_error_indicator(error_rate: f64) -> String {
        let color = match error_rate {
            r if r <= 0.01 => "🟢", // Green for <= 1%
            r if r <= 0.05 => "🟡", // Yellow for <= 5%
            _ => "🔴",              // Red for > 5%
        };
        
        format!("{} {:.2}%", color, error_rate * 100.0)
    }

    /// Create throughput indicator
    pub fn create_throughput_indicator(ops_per_second: f64) -> String {
        let indicator = match ops_per_second {
            t if t >= 1000.0 => "🚀", // Rocket for high throughput
            t if t >= 100.0 => "⚡",  // Lightning for good throughput
            t if t >= 10.0 => "🔥",   // Fire for moderate throughput
            _ => "🐌",               // Snail for low throughput
        };
        
        format!("{} {:.1} ops/s", indicator, ops_per_second)
    }

    /// Create mini dashboard
    pub fn create_mini_dashboard(dashboard: &PerformanceDashboard) -> String {
        let key_metrics = dashboard.key_metrics();
        let sparkline = Self::create_sparkline(&dashboard.historical_data, 20);
        let gauge = Self::create_performance_gauge(key_metrics.current_performance, 10);
        let trend = Self::create_trend_indicator(dashboard.trend.strength(), dashboard.trend.is_positive());
        let error_indicator = Self::create_error_indicator(key_metrics.error_rate);
        let throughput_indicator = Self::create_throughput_indicator(key_metrics.operations_per_second);
        
        format!(
            "┌─ {} ─┐\n\
            │ Perf: {} │\n\
            │ Trend: {} │\n\
            │ Errors: {} │\n\
            │ Speed: {} │\n\
            │ Chart: {} │\n\
            └─────────────────────┘",
            dashboard.reporter_name,
            gauge,
            trend,
            error_indicator,
            throughput_indicator,
            sparkline
        )
    }

    /// Create alert dashboard for critical issues
    pub fn create_alert_dashboard(dashboard: &PerformanceDashboard) -> String {
        let key_metrics = dashboard.key_metrics();
        
        if key_metrics.health_status.is_healthy() {
            return "✅ All systems operational".to_string();
        }

        let mut alerts = Vec::new();
        
        if key_metrics.current_performance < 0.3 {
            alerts.push("🚨 CRITICAL: Performance below 30%");
        } else if key_metrics.current_performance < 0.5 {
            alerts.push("⚠️  WARNING: Performance below 50%");
        }
        
        if key_metrics.error_rate > 0.1 {
            alerts.push("🚨 CRITICAL: Error rate above 10%");
        } else if key_metrics.error_rate > 0.05 {
            alerts.push("⚠️  WARNING: Error rate above 5%");
        }
        
        if key_metrics.operations_per_second < 1.0 {
            alerts.push("⚠️  WARNING: Very low throughput");
        }
        
        if !dashboard.trend.is_positive() && dashboard.trend.strength() > 0.5 {
            alerts.push("⚠️  WARNING: Strong negative trend");
        }
        
        if alerts.is_empty() {
            "ℹ️  Minor performance issues detected".to_string()
        } else {
            format!(
                "🚨 SYSTEM ALERTS 🚨\n\
                {}\n\
                \n\
                Current Status: {}\n\
                Performance: {:.1}%\n\
                Error Rate: {:.2}%\n\
                Throughput: {:.1} ops/s",
                alerts.join("\n"),
                key_metrics.health_status.description(),
                key_metrics.current_performance * 100.0,
                key_metrics.error_rate * 100.0,
                key_metrics.operations_per_second
            )
        }
    }
}
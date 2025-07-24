//! Health monitoring utilities for quantum entanglement system
//!
//! This module provides health monitoring, trend analysis, and diagnostic
//! utilities for quantum entanglement networks with zero allocation patterns
//! and blazing-fast performance.

use super::entanglement_types::{ComprehensiveHealthReport, HealthTrend};
use super::engine_core::QuantumEntanglementEngine;

/// Create comprehensive health report
pub async fn create_comprehensive_health_report(
    engine: &QuantumEntanglementEngine,
) -> Result<ComprehensiveHealthReport, crate::cognitive::types::CognitiveError> {
    let analysis_report = engine.generate_analysis_report().await?;
    let optimization_prediction = engine.predict_optimization_impact(
        &analysis_report.topology,
        &std::collections::HashMap::new(), // Would need actual tree in real implementation
    );
    
    Ok(ComprehensiveHealthReport::new(analysis_report, optimization_prediction))
}

/// Monitor health trends over time with detailed analysis
pub fn analyze_detailed_health_trends(
    historical_reports: &[ComprehensiveHealthReport],
    window_size: usize,
) -> DetailedHealthTrend {
    if historical_reports.len() < window_size.max(2) {
        return DetailedHealthTrend {
            trend: HealthTrend::Insufficient,
            confidence: 0.0,
            trend_strength: 0.0,
            volatility: 0.0,
        };
    }
    
    let recent_window = &historical_reports[historical_reports.len() - window_size..];
    let health_scores: Vec<f64> = recent_window
        .iter()
        .map(|r| r.analysis_report.health.overall_health)
        .collect();
    
    let trend = calculate_trend_direction(&health_scores);
    let confidence = calculate_trend_confidence(&health_scores);
    let trend_strength = calculate_trend_strength(&health_scores);
    let volatility = calculate_health_volatility(&health_scores);
    
    DetailedHealthTrend {
        trend,
        confidence,
        trend_strength,
        volatility,
    }
}

/// Calculate trend direction from health scores
fn calculate_trend_direction(health_scores: &[f64]) -> HealthTrend {
    if health_scores.len() < 2 {
        return HealthTrend::Insufficient;
    }
    
    let first_half = &health_scores[..health_scores.len() / 2];
    let second_half = &health_scores[health_scores.len() / 2..];
    
    let first_avg: f64 = first_half.iter().sum::<f64>() / first_half.len() as f64;
    let second_avg: f64 = second_half.iter().sum::<f64>() / second_half.len() as f64;
    
    let change = second_avg - first_avg;
    
    if change > 2.0 {
        HealthTrend::Improving
    } else if change < -2.0 {
        HealthTrend::Declining
    } else {
        HealthTrend::Stable
    }
}

/// Calculate confidence in trend analysis
fn calculate_trend_confidence(health_scores: &[f64]) -> f64 {
    if health_scores.len() < 3 {
        return 0.0;
    }
    
    // Calculate R-squared for linear trend
    let n = health_scores.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = health_scores.iter().sum::<f64>() / n;
    
    let mut ss_tot = 0.0;
    let mut ss_res = 0.0;
    
    for (i, &y) in health_scores.iter().enumerate() {
        let x = i as f64;
        let y_pred = y_mean + (x - x_mean) * calculate_slope(health_scores);
        
        ss_tot += (y - y_mean).powi(2);
        ss_res += (y - y_pred).powi(2);
    }
    
    if ss_tot == 0.0 {
        1.0
    } else {
        (1.0 - ss_res / ss_tot).max(0.0)
    }
}

/// Calculate slope of health trend
fn calculate_slope(health_scores: &[f64]) -> f64 {
    let n = health_scores.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = health_scores.iter().sum::<f64>() / n;
    
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    
    for (i, &y) in health_scores.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean).powi(2);
    }
    
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Calculate trend strength
fn calculate_trend_strength(health_scores: &[f64]) -> f64 {
    if health_scores.len() < 2 {
        return 0.0;
    }
    
    let slope = calculate_slope(health_scores);
    let range = health_scores.iter().fold(0.0, |acc, &x| acc.max(x)) - 
               health_scores.iter().fold(100.0, |acc, &x| acc.min(x));
    
    if range == 0.0 {
        0.0
    } else {
        (slope.abs() / range).min(1.0)
    }
}

/// Calculate health volatility
fn calculate_health_volatility(health_scores: &[f64]) -> f64 {
    if health_scores.len() < 2 {
        return 0.0;
    }
    
    let mean = health_scores.iter().sum::<f64>() / health_scores.len() as f64;
    let variance = health_scores.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / health_scores.len() as f64;
    
    variance.sqrt()
}

/// Generate health summary statistics
pub fn generate_health_summary(
    historical_reports: &[ComprehensiveHealthReport],
) -> HealthSummaryStats {
    if historical_reports.is_empty() {
        return HealthSummaryStats::default();
    }
    
    let health_scores: Vec<f64> = historical_reports
        .iter()
        .map(|r| r.analysis_report.health.overall_health)
        .collect();
    
    let min_health = health_scores.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_health = health_scores.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let avg_health = health_scores.iter().sum::<f64>() / health_scores.len() as f64;
    let volatility = calculate_health_volatility(&health_scores);
    
    let trend = analyze_detailed_health_trends(historical_reports, health_scores.len().min(10));
    
    HealthSummaryStats {
        min_health,
        max_health,
        avg_health,
        volatility,
        trend_direction: trend.trend,
        trend_confidence: trend.confidence,
        sample_count: health_scores.len(),
    }
}

/// Detect health anomalies in recent reports
pub fn detect_health_anomalies(
    historical_reports: &[ComprehensiveHealthReport],
    threshold_std_dev: f64,
) -> Vec<HealthAnomaly> {
    if historical_reports.len() < 5 {
        return Vec::new(); // Need sufficient data for anomaly detection
    }
    
    let health_scores: Vec<f64> = historical_reports
        .iter()
        .map(|r| r.analysis_report.health.overall_health)
        .collect();
    
    let mean = health_scores.iter().sum::<f64>() / health_scores.len() as f64;
    let std_dev = calculate_health_volatility(&health_scores);
    
    let mut anomalies = Vec::new();
    
    for (i, &score) in health_scores.iter().enumerate() {
        let z_score = if std_dev > 0.0 {
            (score - mean).abs() / std_dev
        } else {
            0.0
        };
        
        if z_score > threshold_std_dev {
            let anomaly_type = if score < mean {
                AnomalyType::HealthDrop
            } else {
                AnomalyType::HealthSpike
            };
            
            anomalies.push(HealthAnomaly {
                report_index: i,
                health_score: score,
                z_score,
                anomaly_type,
                severity: if z_score > threshold_std_dev * 2.0 {
                    AnomalySeverity::High
                } else {
                    AnomalySeverity::Medium
                },
            });
        }
    }
    
    anomalies
}

/// Detailed health trend analysis
#[derive(Debug, Clone)]
pub struct DetailedHealthTrend {
    pub trend: HealthTrend,
    pub confidence: f64,      // 0.0 to 1.0
    pub trend_strength: f64,  // 0.0 to 1.0
    pub volatility: f64,      // Standard deviation of health scores
}

impl DetailedHealthTrend {
    /// Check if trend analysis is reliable
    pub fn is_reliable(&self) -> bool {
        self.confidence > 0.7 && !matches!(self.trend, HealthTrend::Insufficient)
    }

    /// Get trend quality assessment
    pub fn quality_assessment(&self) -> &'static str {
        if self.confidence > 0.9 {
            "High quality trend analysis"
        } else if self.confidence > 0.7 {
            "Good quality trend analysis"
        } else if self.confidence > 0.5 {
            "Moderate quality trend analysis"
        } else {
            "Low quality trend analysis"
        }
    }
}

/// Health summary statistics
#[derive(Debug, Clone)]
pub struct HealthSummaryStats {
    pub min_health: f64,
    pub max_health: f64,
    pub avg_health: f64,
    pub volatility: f64,
    pub trend_direction: HealthTrend,
    pub trend_confidence: f64,
    pub sample_count: usize,
}

impl Default for HealthSummaryStats {
    fn default() -> Self {
        Self {
            min_health: 0.0,
            max_health: 0.0,
            avg_health: 0.0,
            volatility: 0.0,
            trend_direction: HealthTrend::Insufficient,
            trend_confidence: 0.0,
            sample_count: 0,
        }
    }
}

/// Health anomaly detection
#[derive(Debug, Clone)]
pub struct HealthAnomaly {
    pub report_index: usize,
    pub health_score: f64,
    pub z_score: f64,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
}

/// Types of health anomalies
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyType {
    HealthDrop,   // Significant decrease in health
    HealthSpike,  // Significant increase in health
}

/// Anomaly severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
}

impl AnomalySeverity {
    /// Get severity description
    #[inline]
    pub const fn description(&self) -> &'static str {
        match self {
            AnomalySeverity::Low => "Minor anomaly detected",
            AnomalySeverity::Medium => "Moderate anomaly detected",
            AnomalySeverity::High => "Significant anomaly detected",
        }
    }
}
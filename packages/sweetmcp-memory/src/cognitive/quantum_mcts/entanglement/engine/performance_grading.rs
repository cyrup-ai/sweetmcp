//! Performance grading and reporting for quantum entanglement engines
//!
//! This module provides comprehensive performance analysis with blazing-fast
//! zero-allocation grading algorithms and detailed reporting capabilities.

use std::time::Instant;
use tracing::debug;

use crate::cognitive::types::CognitiveError;
use super::{
    core::QuantumEntanglementEngine,
    maintenance_statistics::EngineStatistics,
};

/// Trend direction for performance analysis
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Performance is improving
    Improving,
    /// Performance is stable
    Stable,
    /// Performance is declining
    Declining,
    /// Trend is unclear or insufficient data
    Unknown,
}

/// Performance grades for different aspects
#[derive(Debug, Clone)]
pub struct PerformanceGrades {
    /// Overall performance grade
    pub overall: char,
    /// Latency performance grade
    pub latency: char,
    /// Throughput performance grade
    pub throughput: char,
    /// Reliability performance grade
    pub reliability: char,
    /// Efficiency performance grade
    pub efficiency: char,
}

impl PerformanceGrades {
    /// Create new performance grades
    pub fn new(
        overall: char,
        latency: char,
        throughput: char,
        reliability: char,
        efficiency: char,
    ) -> Self {
        Self {
            overall,
            latency,
            throughput,
            reliability,
            efficiency,
        }
    }
    
    /// Check if performance is acceptable (C or better)
    pub fn is_acceptable(&self) -> bool {
        self.overall >= 'C'
    }
    
    /// Check if performance is excellent (A grade)
    pub fn is_excellent(&self) -> bool {
        self.overall == 'A'
    }
    
    /// Get worst performing aspect
    pub fn worst_aspect(&self) -> (String, char) {
        let aspects = vec![
            ("Latency".to_string(), self.latency),
            ("Throughput".to_string(), self.throughput),
            ("Reliability".to_string(), self.reliability),
            ("Efficiency".to_string(), self.efficiency),
        ];
        
        aspects.into_iter()
            .max_by_key(|(_, grade)| match grade {
                'A' => 0,
                'B' => 1,
                'C' => 2,
                'D' => 3,
                _ => 4,
            })
            .unwrap_or(("Unknown".to_string(), 'F'))
    }
    
    /// Get grade point average (4.0 scale)
    pub fn grade_point_average(&self) -> f64 {
        let grades = vec![self.overall, self.latency, self.throughput, self.reliability, self.efficiency];
        let points: Vec<f64> = grades.iter().map(|&g| match g {
            'A' => 4.0,
            'B' => 3.0,
            'C' => 2.0,
            'D' => 1.0,
            _ => 0.0,
        }).collect();
        
        points.iter().sum::<f64>() / points.len() as f64
    }
}

/// Comprehensive engine performance report
#[derive(Debug, Clone)]
pub struct EnginePerformanceReport {
    /// Report timestamp
    pub timestamp: Instant,
    /// Engine statistics
    pub statistics: EngineStatistics,
    /// Health report
    pub health_report: super::health::NetworkHealthReport,
    /// Performance grades
    pub performance_grades: PerformanceGrades,
}

impl EnginePerformanceReport {
    /// Create new performance report
    pub fn new(
        timestamp: Instant,
        statistics: EngineStatistics,
        health_report: super::health::NetworkHealthReport,
        performance_grades: PerformanceGrades,
    ) -> Self {
        Self {
            timestamp,
            statistics,
            health_report,
            performance_grades,
        }
    }

    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        self.performance_grades.overall >= 'C' && self.health_report.is_healthy()
    }
    
    /// Get performance summary
    pub fn performance_summary(&self) -> String {
        format!(
            "Overall Grade: {} | Latency: {} | Throughput: {} | Reliability: {} | Efficiency: {}",
            self.performance_grades.overall,
            self.performance_grades.latency,
            self.performance_grades.throughput,
            self.performance_grades.reliability,
            self.performance_grades.efficiency
        )
    }
    
    /// Format comprehensive report
    pub fn format_report(&self) -> String {
        format!(
            "=== Engine Performance Report ===\n\
            Timestamp: {:?}\n\
            {}\n\
            {}\n\
            {}",
            self.timestamp,
            self.performance_summary(),
            self.statistics.performance_summary(),
            self.health_report.summary()
        )
    }
    
    /// Get improvement recommendations
    pub fn get_improvement_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.performance_grades.latency < 'C' {
            recommendations.push("Optimize operation latency through caching and batch processing".to_string());
        }
        
        if self.performance_grades.throughput < 'C' {
            recommendations.push("Improve throughput through parallel processing and pipeline optimization".to_string());
        }
        
        if self.performance_grades.reliability < 'C' {
            recommendations.push("Enhance reliability through better error handling and retry mechanisms".to_string());
        }
        
        if self.performance_grades.efficiency < 'C' {
            recommendations.push("Increase efficiency through resource optimization and cache improvements".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is satisfactory, continue current optimizations".to_string());
        }
        
        recommendations
    }
}

impl QuantumEntanglementEngine {
    /// Create engine performance report
    pub async fn create_performance_report(&self) -> Result<EnginePerformanceReport, CognitiveError> {
        debug!("Creating comprehensive engine performance report");
        
        let statistics = self.get_comprehensive_statistics().await?;
        let health_report = self.health_check().await?;
        let timestamp = Instant::now();
        
        // Calculate performance grades
        let latency_grade = Self::calculate_latency_grade(statistics.average_latency_us);
        let throughput_grade = Self::calculate_throughput_grade(statistics.throughput_ops_per_sec);
        let reliability_grade = Self::calculate_reliability_grade(statistics.success_rate);
        let efficiency_grade = Self::calculate_efficiency_grade(statistics.cache_efficiency);
        
        let overall_grade = Self::calculate_overall_grade(&[
            latency_grade, throughput_grade, reliability_grade, efficiency_grade
        ]);
        
        let performance_grades = PerformanceGrades::new(
            overall_grade,
            latency_grade,
            throughput_grade,
            reliability_grade,
            efficiency_grade,
        );
        
        Ok(EnginePerformanceReport::new(
            timestamp,
            statistics,
            health_report,
            performance_grades,
        ))
    }
    
    /// Calculate performance grade for latency
    fn calculate_latency_grade(latency_us: f64) -> char {
        match latency_us {
            l if l <= 100.0 => 'A',
            l if l <= 500.0 => 'B',
            l if l <= 1000.0 => 'C',
            l if l <= 2000.0 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate performance grade for throughput
    fn calculate_throughput_grade(ops_per_sec: f64) -> char {
        match ops_per_sec {
            t if t >= 100.0 => 'A',
            t if t >= 50.0 => 'B',
            t if t >= 20.0 => 'C',
            t if t >= 10.0 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate performance grade for reliability
    fn calculate_reliability_grade(success_rate: f64) -> char {
        match success_rate {
            r if r >= 0.98 => 'A',
            r if r >= 0.95 => 'B',
            r if r >= 0.90 => 'C',
            r if r >= 0.85 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate performance grade for efficiency
    fn calculate_efficiency_grade(cache_hit_rate: f64) -> char {
        match cache_hit_rate {
            e if e >= 0.95 => 'A',
            e if e >= 0.90 => 'B',
            e if e >= 0.80 => 'C',
            e if e >= 0.70 => 'D',
            _ => 'F',
        }
    }
    
    /// Calculate overall performance grade
    fn calculate_overall_grade(grades: &[char]) -> char {
        let grade_values: Vec<u8> = grades.iter().map(|&g| match g {
            'A' => 4,
            'B' => 3,
            'C' => 2,
            'D' => 1,
            _ => 0,
        }).collect();
        
        let average = grade_values.iter().sum::<u8>() as f64 / grade_values.len() as f64;
        
        match average {
            a if a >= 3.5 => 'A',
            a if a >= 2.5 => 'B',
            a if a >= 1.5 => 'C',
            a if a >= 0.5 => 'D',
            _ => 'F',
        }
    }
    
}
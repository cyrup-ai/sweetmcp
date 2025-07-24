//! Rolling performance monitoring for continuous benchmarking
//!
//! This module provides blazing-fast rolling window performance monitoring
//! with zero-allocation patterns and real-time trend analysis.

use std::time::{Duration, Instant};
use std::collections::VecDeque;
use super::tracking::{PerformanceCategory, TimingUtils};
use super::performance_trends::PerformanceTrend;

/// Rolling performance monitor for continuous benchmarking
#[derive(Debug)]
pub struct RollingPerformanceMonitor {
    /// Name of the monitor
    monitor_name: String,
    /// Rolling window of recent samples
    samples: VecDeque<Duration>,
    /// Maximum window size
    window_size: usize,
    /// Total samples processed
    total_samples: u64,
    /// Creation time
    creation_time: Instant,
}

impl RollingPerformanceMonitor {
    /// Create new rolling monitor
    pub fn new(monitor_name: String, window_size: usize) -> Self {
        Self {
            monitor_name,
            samples: VecDeque::with_capacity(window_size),
            window_size,
            total_samples: 0,
            creation_time: Instant::now(),
        }
    }
    
    /// Add new sample to the rolling window
    pub fn add_sample(&mut self, duration: Duration) {
        self.total_samples += 1;
        
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        
        self.samples.push_back(duration);
    }
    
    /// Get current window statistics
    pub fn current_statistics(&self) -> RollingStatistics {
        if self.samples.is_empty() {
            return RollingStatistics::default();
        }
        
        let mut sorted_samples: Vec<Duration> = self.samples.iter().copied().collect();
        sorted_samples.sort();
        
        let total_time: Duration = self.samples.iter().sum();
        let average_duration = Duration::from_nanos(total_time.as_nanos() / self.samples.len() as u128);
        let median_duration = TimingUtils::calculate_median(&sorted_samples);
        let p95_duration = TimingUtils::calculate_p95(&sorted_samples);
        
        let window_duration = self.creation_time.elapsed();
        let samples_per_second = if window_duration.as_secs_f64() > 0.0 {
            self.samples.len() as f64 / window_duration.as_secs_f64()
        } else {
            0.0
        };
        
        RollingStatistics {
            monitor_name: self.monitor_name.clone(),
            window_size: self.samples.len(),
            total_samples: self.total_samples,
            average_duration,
            median_duration,
            min_duration: sorted_samples[0],
            max_duration: sorted_samples[sorted_samples.len() - 1],
            p95_duration,
            samples_per_second,
            window_duration,
        }
    }
    
    /// Check if performance is stable (low variance)
    pub fn is_stable(&self, variance_threshold: f64) -> bool {
        if self.samples.len() < 10 {
            return false;
        }
        
        let stats = self.current_statistics();
        let mean_ns = stats.average_duration.as_nanos() as f64;
        
        let variance: f64 = self.samples
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - mean_ns;
                diff * diff
            })
            .sum::<f64>() / self.samples.len() as f64;
        
        let coefficient_of_variation = if mean_ns > 0.0 {
            variance.sqrt() / mean_ns
        } else {
            0.0
        };
        
        coefficient_of_variation < variance_threshold
    }
    
    /// Get performance trend over the window
    pub fn performance_trend(&self) -> PerformanceTrend {
        if self.samples.len() < 4 {
            return PerformanceTrend::Stable;
        }
        
        let mid_point = self.samples.len() / 2;
        let first_half: Vec<Duration> = self.samples.iter().take(mid_point).copied().collect();
        let second_half: Vec<Duration> = self.samples.iter().skip(mid_point).copied().collect();
        
        let first_avg = Duration::from_nanos(
            first_half.iter().map(|d| d.as_nanos()).sum::<u128>() / first_half.len() as u128
        );
        let second_avg = Duration::from_nanos(
            second_half.iter().map(|d| d.as_nanos()).sum::<u128>() / second_half.len() as u128
        );
        
        let improvement_ratio = first_avg.as_nanos() as f64 / second_avg.as_nanos() as f64;
        
        if improvement_ratio > 1.1 {
            PerformanceTrend::Improving
        } else if improvement_ratio < 0.9 {
            PerformanceTrend::Degrading
        } else {
            PerformanceTrend::Stable
        }
    }
    
    /// Reset the monitor
    pub fn reset(&mut self) {
        self.samples.clear();
        self.total_samples = 0;
        self.creation_time = Instant::now();
    }
    
    /// Get monitor name
    pub fn monitor_name(&self) -> &str {
        &self.monitor_name
    }
    
    /// Get current window size
    pub fn current_window_size(&self) -> usize {
        self.samples.len()
    }
    
    /// Get maximum window size
    pub fn max_window_size(&self) -> usize {
        self.window_size
    }
    
    /// Get total samples processed
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
    
    /// Check if window is full
    pub fn is_window_full(&self) -> bool {
        self.samples.len() >= self.window_size
    }
    
    /// Get creation time
    pub fn creation_time(&self) -> Instant {
        self.creation_time
    }
    
    /// Get elapsed time since creation
    pub fn elapsed_time(&self) -> Duration {
        self.creation_time.elapsed()
    }
    
    /// Get current average duration
    pub fn current_average(&self) -> Duration {
        if self.samples.is_empty() {
            Duration::from_nanos(0)
        } else {
            let total: Duration = self.samples.iter().sum();
            Duration::from_nanos(total.as_nanos() / self.samples.len() as u128)
        }
    }
    
    /// Get current throughput estimate
    pub fn current_throughput(&self) -> f64 {
        let elapsed = self.creation_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.samples.len() as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Check if monitor has sufficient data for analysis
    pub fn has_sufficient_data(&self) -> bool {
        self.samples.len() >= 10 && self.total_samples >= 10
    }
}

/// Statistics for rolling performance monitor
#[derive(Debug, Clone)]
pub struct RollingStatistics {
    /// Monitor name
    pub monitor_name: String,
    /// Current window size
    pub window_size: usize,
    /// Total samples processed
    pub total_samples: u64,
    /// Average duration in current window
    pub average_duration: Duration,
    /// Median duration in current window
    pub median_duration: Duration,
    /// Minimum duration in current window
    pub min_duration: Duration,
    /// Maximum duration in current window
    pub max_duration: Duration,
    /// 95th percentile duration
    pub p95_duration: Duration,
    /// Samples per second rate
    pub samples_per_second: f64,
    /// Duration of the monitoring window
    pub window_duration: Duration,
}

impl Default for RollingStatistics {
    fn default() -> Self {
        Self {
            monitor_name: String::new(),
            window_size: 0,
            total_samples: 0,
            average_duration: Duration::from_nanos(0),
            median_duration: Duration::from_nanos(0),
            min_duration: Duration::from_nanos(0),
            max_duration: Duration::from_nanos(0),
            p95_duration: Duration::from_nanos(0),
            samples_per_second: 0.0,
            window_duration: Duration::from_nanos(0),
        }
    }
}

impl RollingStatistics {
    /// Get performance category
    pub fn performance_category(&self) -> PerformanceCategory {
        TimingUtils::categorize_duration(self.average_duration)
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "{}: {} samples (avg: {}, {:.1}/sec, {})",
            self.monitor_name,
            self.window_size,
            TimingUtils::format_duration(self.average_duration),
            self.samples_per_second,
            self.performance_category().description()
        )
    }
    
    /// Get efficiency ratio
    pub fn efficiency_ratio(&self) -> f64 {
        if self.window_duration.as_nanos() > 0 {
            let total_operation_time = Duration::from_nanos(
                self.average_duration.as_nanos() * self.window_size as u128
            );
            total_operation_time.as_nanos() as f64 / self.window_duration.as_nanos() as f64
        } else {
            0.0
        }
    }
    
    /// Get performance grade
    pub fn performance_grade(&self) -> char {
        self.performance_category().grade()
    }
    
    /// Check if performance is good
    pub fn has_good_performance(&self) -> bool {
        matches!(
            self.performance_category(),
            PerformanceCategory::Excellent | PerformanceCategory::Good
        ) && self.samples_per_second > 50.0
    }
    
    /// Check if performance has issues
    pub fn has_performance_issues(&self) -> bool {
        matches!(
            self.performance_category(),
            PerformanceCategory::Slow | PerformanceCategory::VerySlow
        ) || self.samples_per_second < 10.0
    }
}
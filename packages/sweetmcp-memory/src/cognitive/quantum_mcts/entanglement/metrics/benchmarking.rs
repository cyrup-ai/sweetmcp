//! Benchmarking utilities and performance assessment for quantum entanglement
//!
//! This module provides blazing-fast benchmarking capabilities with zero-allocation
//! patterns and comprehensive performance assessment tools.

// Re-export core benchmarking components
pub use benchmark_core::EntanglementBenchmark;
pub use benchmark_results::{BenchmarkResults, BenchmarkComparison};
pub use rolling_monitor::{RollingPerformanceMonitor, RollingStatistics};
pub use performance_trends::{PerformanceTrend, ActionPriority, TrendAnalyzer};

// Module declarations
mod benchmark_core;
mod benchmark_results;
mod rolling_monitor;
mod performance_trends;

use std::time::{Duration, Instant};
use std::collections::HashMap;
use super::tracking::{PerformanceCategory, TimingUtils};

/// Comprehensive benchmarking suite for quantum entanglement operations
#[derive(Debug)]
pub struct EntanglementBenchmarkSuite {
    /// Collection of active benchmarks
    benchmarks: HashMap<String, EntanglementBenchmark>,
    /// Collection of rolling monitors
    monitors: HashMap<String, RollingPerformanceMonitor>,
    /// Suite creation time
    creation_time: Instant,
    /// Total operations across all benchmarks
    total_operations: u64,
}

impl EntanglementBenchmarkSuite {
    /// Create new benchmark suite
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
            monitors: HashMap::new(),
            creation_time: Instant::now(),
            total_operations: 0,
        }
    }
    
    /// Add new benchmark to the suite
    pub fn add_benchmark(&mut self, name: String, benchmark: EntanglementBenchmark) {
        self.benchmarks.insert(name, benchmark);
    }
    
    /// Create and add benchmark with default settings
    pub fn create_benchmark(&mut self, name: String) {
        let benchmark = EntanglementBenchmark::new(name.clone());
        self.add_benchmark(name, benchmark);
    }
    
    /// Create and add benchmark with custom sample limit
    pub fn create_benchmark_with_limit(&mut self, name: String, sample_limit: usize) {
        let benchmark = EntanglementBenchmark::with_sample_limit(name.clone(), sample_limit);
        self.add_benchmark(name, benchmark);
    }
    
    /// Add rolling monitor to the suite
    pub fn add_monitor(&mut self, name: String, window_size: usize) {
        let monitor = RollingPerformanceMonitor::new(name.clone(), window_size);
        self.monitors.insert(name, monitor);
    }
    
    /// Record operation for specific benchmark
    pub fn record_operation(&mut self, benchmark_name: &str, duration: Duration) -> Result<(), String> {
        match self.benchmarks.get_mut(benchmark_name) {
            Some(benchmark) => {
                benchmark.record_sample(duration);
                self.total_operations += 1;
                
                // Also record in monitor if exists
                if let Some(monitor) = self.monitors.get_mut(benchmark_name) {
                    monitor.add_sample(duration);
                }
                
                Ok(())
            }
            None => Err(format!("Benchmark '{}' not found", benchmark_name)),
        }
    }
    
    /// Record batch operation for specific benchmark
    pub fn record_batch_operation(
        &mut self,
        benchmark_name: &str,
        operation_count: u64,
        total_duration: Duration,
    ) -> Result<(), String> {
        match self.benchmarks.get_mut(benchmark_name) {
            Some(benchmark) => {
                benchmark.record_batch(operation_count, total_duration);
                self.total_operations += operation_count;
                Ok(())
            }
            None => Err(format!("Benchmark '{}' not found", benchmark_name)),
        }
    }
    
    /// Get benchmark results
    pub fn get_results(&self, benchmark_name: &str) -> Result<BenchmarkResults, String> {
        match self.benchmarks.get(benchmark_name) {
            Some(benchmark) => Ok(benchmark.results()),
            None => Err(format!("Benchmark '{}' not found", benchmark_name)),
        }
    }
    
    /// Get all benchmark results
    pub fn get_all_results(&self) -> Vec<BenchmarkResults> {
        self.benchmarks
            .values()
            .map(|benchmark| benchmark.results())
            .collect()
    }
    
    /// Get rolling statistics for monitor
    pub fn get_monitor_statistics(&self, monitor_name: &str) -> Result<RollingStatistics, String> {
        match self.monitors.get(monitor_name) {
            Some(monitor) => Ok(monitor.current_statistics()),
            None => Err(format!("Monitor '{}' not found", monitor_name)),
        }
    }
    
    /// Get performance trend for monitor
    pub fn get_performance_trend(&self, monitor_name: &str) -> Result<PerformanceTrend, String> {
        match self.monitors.get(monitor_name) {
            Some(monitor) => Ok(monitor.performance_trend()),
            None => Err(format!("Monitor '{}' not found", monitor_name)),
        }
    }
    
    /// Compare two benchmarks
    pub fn compare_benchmarks(
        &self,
        current_name: &str,
        baseline_name: &str,
    ) -> Result<BenchmarkComparison, String> {
        let current_results = self.get_results(current_name)?;
        let baseline_results = self.get_results(baseline_name)?;
        Ok(current_results.compare_with(&baseline_results))
    }
    
    /// Reset specific benchmark
    pub fn reset_benchmark(&mut self, benchmark_name: &str) -> Result<(), String> {
        match self.benchmarks.get_mut(benchmark_name) {
            Some(benchmark) => {
                benchmark.reset();
                Ok(())
            }
            None => Err(format!("Benchmark '{}' not found", benchmark_name)),
        }
    }
    
    /// Reset specific monitor
    pub fn reset_monitor(&mut self, monitor_name: &str) -> Result<(), String> {
        match self.monitors.get_mut(monitor_name) {
            Some(monitor) => {
                monitor.reset();
                Ok(())
            }
            None => Err(format!("Monitor '{}' not found", monitor_name)),
        }
    }
    
    /// Reset all benchmarks and monitors
    pub fn reset_all(&mut self) {
        for benchmark in self.benchmarks.values_mut() {
            benchmark.reset();
        }
        for monitor in self.monitors.values_mut() {
            monitor.reset();
        }
        self.creation_time = Instant::now();
        self.total_operations = 0;
    }
    
    /// Get suite summary
    pub fn suite_summary(&self) -> SuiteSummary {
        let benchmark_count = self.benchmarks.len();
        let monitor_count = self.monitors.len();
        let suite_duration = self.creation_time.elapsed();
        
        let (active_benchmarks, total_samples) = self.benchmarks
            .values()
            .fold((0, 0), |(active, samples), benchmark| {
                let is_active = benchmark.total_operations() > 0;
                (
                    active + if is_active { 1 } else { 0 },
                    samples + benchmark.sample_count(),
                )
            });
        
        let (active_monitors, total_monitor_samples) = self.monitors
            .values()
            .fold((0, 0), |(active, samples), monitor| {
                let is_active = monitor.total_samples() > 0;
                (
                    active + if is_active { 1 } else { 0 },
                    samples + monitor.current_window_size(),
                )
            });
        
        let average_throughput = if suite_duration.as_secs_f64() > 0.0 {
            self.total_operations as f64 / suite_duration.as_secs_f64()
        } else {
            0.0
        };
        
        SuiteSummary {
            benchmark_count,
            monitor_count,
            active_benchmarks,
            active_monitors,
            total_operations: self.total_operations,
            total_samples,
            total_monitor_samples,
            suite_duration,
            average_throughput,
        }
    }
    
    /// Get comprehensive report
    pub fn comprehensive_report(&self) -> String {
        let summary = self.suite_summary();
        format!(
            "=== Entanglement Benchmark Suite Report ===\n\
            Suite Duration: {}\n\
            Total Operations: {}\n\
            Average Throughput: {:.1} ops/sec\n\
            Active Benchmarks: {}/{}\n\
            Active Monitors: {}/{}",
            TimingUtils::format_duration(summary.suite_duration),
            summary.total_operations,
            summary.average_throughput,
            summary.active_benchmarks,
            summary.benchmark_count,
            summary.active_monitors,
            summary.monitor_count
        )
    }
    
    /// Get benchmark names
    pub fn benchmark_names(&self) -> Vec<String> {
        self.benchmarks.keys().cloned().collect()
    }
    
    /// Check if benchmark exists
    pub fn has_benchmark(&self, name: &str) -> bool {
        self.benchmarks.contains_key(name)
    }
    
    /// Get total operations across all benchmarks
    pub fn total_operations(&self) -> u64 {
        self.total_operations
    }
}

impl Default for EntanglementBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics for the benchmark suite
#[derive(Debug, Clone)]
pub struct SuiteSummary {
    pub benchmark_count: usize,
    pub monitor_count: usize,
    pub active_benchmarks: usize,
    pub active_monitors: usize,
    pub total_operations: u64,
    pub total_samples: usize,
    pub total_monitor_samples: usize,
    pub suite_duration: Duration,
    pub average_throughput: f64,
}

impl SuiteSummary {
    /// Get efficiency ratio
    pub fn efficiency_ratio(&self) -> f64 {
        if self.benchmark_count > 0 && self.active_benchmarks > 0 {
            self.active_benchmarks as f64 / self.benchmark_count as f64
        } else {
            0.0
        }
    }
    
    /// Check if suite is performing well
    pub fn is_performing_well(&self) -> bool {
        self.average_throughput > 50.0 && self.efficiency_ratio() > 0.5
    }
}
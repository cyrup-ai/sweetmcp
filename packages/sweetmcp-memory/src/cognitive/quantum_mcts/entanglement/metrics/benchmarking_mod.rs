//! Benchmarking module integration for quantum entanglement metrics
//!
//! This module provides ergonomic re-exports and integration for all benchmarking
//! components with zero allocation patterns and blazing-fast performance.

// Re-export from sibling modules
pub use super::benchmarking_core::*;
pub use super::benchmarking_results::*;
pub use super::benchmarking_comparison::*;
pub use super::benchmarking_monitor::*;

// Re-export core types and functionality
pub use benchmarking_core::{
    EntanglementBenchmark, BenchmarkStats,
};

pub use benchmarking_results::{
    BenchmarkResults, PerformanceCategory, PerformanceTrend, ResultQuality,
};

pub use benchmarking_comparison::{
    BenchmarkComparison, ComparisonMetrics, RegressionAnalysis, ImprovementAnalysis,
    RegressionStatus, RegressionSeverity, ImprovementStatus, ImprovementMagnitude,
    ComparisonVerdict,
};

pub use benchmarking_monitor::{
    RollingPerformanceMonitor, PerformanceSample, RollingStatistics,
    PerformanceTrend as MonitorTrend,
};

/// Benchmarking suite for comprehensive performance analysis
pub struct BenchmarkingSuite {
    /// Active benchmarks
    benchmarks: std::collections::HashMap<String, EntanglementBenchmark>,
    /// Rolling monitors
    monitors: std::collections::HashMap<String, RollingPerformanceMonitor>,
    /// Historical results
    results_history: Vec<BenchmarkResults>,
    /// Maximum history size
    max_history: usize,
}

impl BenchmarkingSuite {
    /// Create new benchmarking suite
    pub fn new() -> Self {
        Self::with_history_limit(1000)
    }
    
    /// Create suite with custom history limit
    pub fn with_history_limit(max_history: usize) -> Self {
        Self {
            benchmarks: std::collections::HashMap::new(),
            monitors: std::collections::HashMap::new(),
            results_history: Vec::with_capacity(max_history.min(100)),
            max_history,
        }
    }
    
    /// Create or get benchmark
    pub fn benchmark(&mut self, name: &str) -> &mut EntanglementBenchmark {
        self.benchmarks.entry(name.to_string())
            .or_insert_with(|| EntanglementBenchmark::new(name.to_string()))
    }
    
    /// Create or get rolling monitor
    pub fn monitor(&mut self, name: &str, window_size: usize) -> &mut RollingPerformanceMonitor {
        self.monitors.entry(name.to_string())
            .or_insert_with(|| RollingPerformanceMonitor::new(name.to_string(), window_size))
    }
    
    /// Record benchmark results
    pub fn record_results(&mut self, name: &str) -> Option<BenchmarkResults> {
        if let Some(benchmark) = self.benchmarks.get(name) {
            let results = BenchmarkResults::from_benchmark(benchmark);
            
            // Add to history
            if self.results_history.len() >= self.max_history {
                self.results_history.remove(0);
            }
            self.results_history.push(results.clone());
            
            // Update monitor if exists
            if let Some(monitor) = self.monitors.get_mut(name) {
                monitor.add_benchmark_sample(&results.stats);
            }
            
            Some(results)
        } else {
            None
        }
    }
    
    /// Compare current results with baseline
    pub fn compare_with_baseline(&self, name: &str, baseline_name: &str) -> Option<BenchmarkComparison> {
        let current = self.results_history.iter()
            .filter(|r| r.benchmark_name == name)
            .last()?;
        
        let baseline = self.results_history.iter()
            .filter(|r| r.benchmark_name == baseline_name)
            .last()?;
        
        Some(BenchmarkComparison::new(baseline.clone(), current.clone()))
    }
    
    /// Get performance summary for all benchmarks
    pub fn performance_summary(&self) -> String {
        let mut summary = String::from("Benchmarking Suite Performance Summary\n");
        summary.push_str("=====================================\n\n");
        
        // Active benchmarks
        if !self.benchmarks.is_empty() {
            summary.push_str("Active Benchmarks:\n");
            for (name, benchmark) in &self.benchmarks {
                let stats = benchmark.current_stats();
                summary.push_str(&format!(
                    "- {}: {} ops/sec (Grade: {})\n",
                    name, stats.operations_per_second, stats.performance_grade()
                ));
            }
            summary.push('\n');
        }
        
        // Rolling monitors
        if !self.monitors.is_empty() {
            summary.push_str("Rolling Monitors:\n");
            for (name, monitor) in &self.monitors {
                let stats = monitor.current_stats();
                summary.push_str(&format!(
                    "- {}: {:.1} ops/sec avg, trend: {:?}\n",
                    name, stats.avg_operations_per_second, stats.trend
                ));
            }
            summary.push('\n');
        }
        
        // Recent results
        if !self.results_history.is_empty() {
            summary.push_str("Recent Results:\n");
            for result in self.results_history.iter().rev().take(5) {
                summary.push_str(&format!(
                    "- {} ({}): {}\n",
                    result.benchmark_name,
                    result.stats.performance_grade(),
                    result.stats.summary()
                ));
            }
        }
        
        summary
    }
    
    /// Reset all benchmarks and monitors
    pub fn reset_all(&mut self) {
        for benchmark in self.benchmarks.values_mut() {
            benchmark.reset();
        }
        
        for monitor in self.monitors.values_mut() {
            monitor.reset();
        }
        
        self.results_history.clear();
    }
    
    /// Get benchmark names
    pub fn benchmark_names(&self) -> Vec<String> {
        self.benchmarks.keys().cloned().collect()
    }
    
    /// Get monitor names
    pub fn monitor_names(&self) -> Vec<String> {
        self.monitors.keys().cloned().collect()
    }
    
    /// Get results history
    pub fn results_history(&self) -> &[BenchmarkResults] {
        &self.results_history
    }
    
    /// Check if any benchmarks show performance issues
    pub fn has_performance_issues(&self) -> bool {
        // Check recent results for poor performance
        self.results_history.iter().rev().take(3).any(|r| {
            matches!(r.category, PerformanceCategory::Poor | PerformanceCategory::Critical)
        }) ||
        // Check monitors for declining trends
        self.monitors.values().any(|m| {
            m.current_stats().is_declining() || m.current_stats().is_volatile()
        })
    }
    
    /// Get performance issues summary
    pub fn performance_issues_summary(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check recent results
        for result in self.results_history.iter().rev().take(3) {
            if matches!(result.category, PerformanceCategory::Poor | PerformanceCategory::Critical) {
                issues.push(format!(
                    "Benchmark '{}' shows {} performance",
                    result.benchmark_name,
                    match result.category {
                        PerformanceCategory::Poor => "poor",
                        PerformanceCategory::Critical => "critical",
                        _ => "concerning"
                    }
                ));
            }
        }
        
        // Check monitors
        for (name, monitor) in &self.monitors {
            let stats = monitor.current_stats();
            if stats.is_declining() {
                issues.push(format!("Monitor '{}' shows declining performance trend", name));
            } else if stats.is_volatile() {
                issues.push(format!("Monitor '{}' shows volatile performance", name));
            }
        }
        
        issues
    }
}

impl Default for BenchmarkingSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick benchmarking utilities
pub mod quick_bench {
    use super::*;
    use std::time::Instant;
    
    /// Quick benchmark a closure
    pub fn quick_benchmark<F, R>(name: &str, iterations: usize, operation: F) -> BenchmarkResults
    where
        F: Fn() -> R,
    {
        let mut benchmark = EntanglementBenchmark::new(name.to_string());
        
        for _ in 0..iterations {
            benchmark.time_operation(&operation);
        }
        
        BenchmarkResults::from_benchmark(&benchmark)
    }
    
    /// Quick async benchmark
    pub async fn quick_async_benchmark<F, Fut, R>(
        name: &str,
        iterations: usize,
        operation: F,
    ) -> BenchmarkResults
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let mut benchmark = EntanglementBenchmark::new(name.to_string());
        
        for _ in 0..iterations {
            benchmark.time_async_operation(&operation).await;
        }
        
        BenchmarkResults::from_benchmark(&benchmark)
    }
    
    /// Time a single operation
    pub fn time_operation<F, R>(operation: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        (result, duration)
    }
}
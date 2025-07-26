//! Quantum MCTS entanglement metrics coordination module
//!
//! This module coordinates all metric collection, calculation, tracking, benchmarking,
//! and reporting capabilities with blazing-fast zero-allocation patterns.

pub mod benchmark_core;
pub mod benchmark_results;
pub mod benchmark_suite;
pub mod benchmarking;
pub mod benchmarking_comparison;
pub mod benchmarking_core;
pub mod benchmarking_mod;
pub mod benchmarking_monitor;
pub mod benchmarking_results;
pub mod calculations;
pub mod counters;
pub mod performance_trends;
pub mod reporting;
pub mod rolling_monitor;
pub mod tracking;

// Re-export the benchmarking module for ergonomic access
// Note: Removed duplicate pub use benchmarking_mod since it's already included via mod benchmarking_mod

use std::time::{Duration, Instant};
use std::sync::Arc;
use crate::monitoring::metrics::MetricsCollector;

// Re-export key types for ergonomic access
pub use counters::{EntanglementCounters, CounterSnapshot};
pub use calculations::{MetricsCalculator, ComprehensiveMetrics, PerformanceCalculations, PerformanceSnapshot, TrendAnalysis};
pub use tracking::{
    PerformanceTracker, BatchPerformanceTracker, BatchStatistics, 
    InfluenceTracker, InfluenceStatistics, PerformanceCategory, TimingUtils
};
pub use benchmarking::{
    EntanglementBenchmark, BenchmarkResults,
    RollingPerformanceMonitor, RollingStatistics, PerformanceTrend as BenchmarkTrend
};
pub use benchmarking_comparison::{
    BenchmarkComparison, ComparisonMetrics, RegressionAnalysis, ImprovementAnalysis
};
pub use crate::cognitive::quantum_mcts::entanglement::benchmarking_mod::BenchmarkingSuite;
pub use reporting::{
    MetricsReporter, MetricsReport, SummaryReport, HistoricalDataPoint,
    PerformanceDashboard, DashboardHealthStatus, KeyMetrics, ReportFormatter,
    DashboardVisualizer, MetricsReporting, PerformanceGrades, AggregatedMetrics,
    PerformanceTrend
};

// Type aliases for ergonomic access
pub type EntanglementMetricsSummary = AggregatedMetrics;
pub type EntanglementMetricsCollector = MetricsReporter;

/// Comprehensive metrics collection and analysis system
#[derive(Debug)]
pub struct EntanglementMetrics {
    /// Atomic counters for operations
    counters: Arc<EntanglementCounters>,
    /// Performance calculations
    calculations: Arc<PerformanceCalculations>,
    /// Metrics reporter
    reporter: MetricsReporter,
    /// Rolling performance monitor
    rolling_monitor: RollingPerformanceMonitor,
    /// Current benchmark (if active)
    active_benchmark: Option<EntanglementBenchmark>,
    /// Creation time
    creation_time: Instant,
    /// System name
    system_name: String,
}

impl EntanglementMetrics {
    /// Create new metrics system
    pub fn new(system_name: String) -> Self {
        let counters = Arc::new(EntanglementCounters::new());
        let calculations = Arc::new(PerformanceCalculations::new(Arc::clone(&counters)));
        let reporter = MetricsReporter::new(format!("{}_reporter", system_name));
        let rolling_monitor = RollingPerformanceMonitor::new(
            format!("{}_rolling", system_name), 
            1000
        );
        
        Self {
            counters,
            calculations,
            reporter,
            rolling_monitor,
            active_benchmark: None,
            creation_time: Instant::now(),
            system_name,
        }
    }
    
    /// Create optimized metrics system for high-throughput scenarios
    pub fn new_high_throughput(system_name: String) -> Self {
        let counters = Arc::new(EntanglementCounters::new());
        let calculations = Arc::new(PerformanceCalculations::new(Arc::clone(&counters)));
        let reporter = MetricsReporter::with_capacity(format!("{}_reporter", system_name), 1000);
        let rolling_monitor = RollingPerformanceMonitor::new(
            format!("{}_rolling", system_name), 
            5000
        );
        
        Self {
            counters,
            calculations,
            reporter,
            rolling_monitor,
            active_benchmark: None,
            creation_time: Instant::now(),
            system_name,
        }
    }
    
    /// Create lightweight metrics system for low-overhead scenarios
    pub fn new_lightweight(system_name: String) -> Self {
        let counters = Arc::new(EntanglementCounters::new());
        let calculations = Arc::new(PerformanceCalculations::new(Arc::clone(&counters)));
        let reporter = MetricsReporter::with_capacity(format!("{}_reporter", system_name), 50);
        let rolling_monitor = RollingPerformanceMonitor::new(
            format!("{}_rolling", system_name), 
            100
        );
        
        Self {
            counters,
            calculations,
            reporter,
            rolling_monitor,
            active_benchmark: None,
            creation_time: Instant::now(),
            system_name,
        }
    }
    
    /// Get shared reference to counters
    pub fn counters(&self) -> Arc<EntanglementCounters> {
        Arc::clone(&self.counters)
    }
    
    /// Get shared reference to calculations
    pub fn calculations(&self) -> Arc<PerformanceCalculations> {
        Arc::clone(&self.calculations)
    }
    
    /// Record entanglement creation
    #[inline]
    pub fn record_entanglement(&self) {
        self.counters.record_entanglement();
    }
    
    /// Record entanglement operation with timing
    #[inline]
    pub fn record_operation(&mut self, duration: Duration) {
        self.counters.record_entanglement_operation(duration);
        self.rolling_monitor.add_sample(duration);
        
        if let Some(benchmark) = &mut self.active_benchmark {
            benchmark.record_sample(duration);
        }
    }
    
    /// Record multiple operations
    #[inline]
    pub fn record_operations(&self, count: u64, total_duration: Duration) {
        self.counters.record_entanglement_operations(count, total_duration);
        
        if count > 0 {
            let avg_duration = Duration::from_nanos(total_duration.as_nanos() / count as u128);
            self.rolling_monitor.add_sample(avg_duration);
            
            if let Some(benchmark) = &mut self.active_benchmark {
                benchmark.record_batch(count, total_duration);
            }
        }
    }
    
    /// Record influence calculation
    #[inline]
    pub fn record_influence_calculation(&self, duration: Duration) {
        self.counters.record_influence_calculation(duration);
    }
    
    /// Record multiple influence calculations
    #[inline]
    pub fn record_influence_calculations(&self, count: u64, total_duration: Duration) {
        self.counters.record_influence_calculations(count, total_duration);
    }
    
    /// Record error
    #[inline]
    pub fn record_error(&self) {
        self.counters.record_error();
    }
    
    /// Start performance tracking for an operation
    pub fn start_tracking(&self) -> PerformanceTracker {
        PerformanceTracker::start()
    }
    
    /// Start named performance tracking
    pub fn start_tracking_named(&self, operation_name: String) -> PerformanceTracker {
        PerformanceTracker::start_named(operation_name)
    }
    
    /// Start batch performance tracking
    pub fn start_batch_tracking(&self) -> BatchPerformanceTracker {
        BatchPerformanceTracker::new()
    }
    
    /// Start named batch performance tracking
    pub fn start_batch_tracking_named(&self, batch_name: String) -> BatchPerformanceTracker {
        BatchPerformanceTracker::new_named(batch_name)
    }
    
    /// Start influence tracking
    pub fn start_influence_tracking(&self) -> InfluenceTracker {
        InfluenceTracker::start()
    }
    
    /// Start benchmark
    pub fn start_benchmark(&mut self, benchmark_name: String) -> Result<(), String> {
        if self.active_benchmark.is_some() {
            return Err("Benchmark already active".to_string());
        }
        
        self.active_benchmark = Some(EntanglementBenchmark::new(benchmark_name));
        Ok(())
    }
    
    /// Stop benchmark and get results
    pub fn stop_benchmark(&mut self) -> Option<BenchmarkResults> {
        self.active_benchmark.take().map(|benchmark| benchmark.results())
    }
    
    /// Check if benchmark is active
    pub fn has_active_benchmark(&self) -> bool {
        self.active_benchmark.is_some()
    }
    
    /// Get current benchmark name
    pub fn active_benchmark_name(&self) -> Option<String> {
        self.active_benchmark.as_ref().map(|b| b.benchmark_name().to_string())
    }
    
    /// Generate comprehensive metrics report
    pub fn generate_report(&mut self) -> MetricsReport {
        self.reporter.generate_report(&self.counters, &self.calculations)
    }
    
    /// Generate summary report with all available metrics
    pub fn generate_summary_report(&self) -> SummaryReport {
        let rolling_stats = self.rolling_monitor.current_statistics();
        let benchmark_results = self.active_benchmark.as_ref().map(|b| b.results());
        
        self.reporter.generate_summary_report(
            None, // batch_stats
            None, // influence_stats  
            benchmark_results.as_ref(),
            Some(&rolling_stats),
        )
    }
    
    /// Generate performance dashboard
    pub fn generate_dashboard(&self) -> PerformanceDashboard {
        self.reporter.generate_dashboard()
    }
    
    /// Get current performance snapshot
    pub fn performance_snapshot(&self) -> PerformanceSnapshot {
        self.calculations.performance_snapshot()
    }
    
    /// Get counter snapshot
    pub fn counter_snapshot(&self) -> CounterSnapshot {
        self.counters.snapshot()
    }
    
    /// Get rolling statistics
    pub fn rolling_statistics(&self) -> RollingStatistics {
        self.rolling_monitor.current_statistics()
    }
    
    /// Get performance trend
    pub fn performance_trend(&self) -> PerformanceTrend {
        self.reporter.performance_trend(20)
    }
    
    /// Get system uptime
    pub fn uptime(&self) -> Duration {
        self.creation_time.elapsed()
    }
    
    /// Get system name
    pub fn system_name(&self) -> &str {
        &self.system_name
    }
    
    /// Check if performance is stable
    pub fn is_performance_stable(&self) -> bool {
        self.rolling_monitor.is_stable(0.2) // 20% coefficient of variation threshold
    }
    
    /// Get current throughput estimate
    pub fn current_throughput(&self) -> f64 {
        let rolling_stats = self.rolling_monitor.current_statistics();
        rolling_stats.samples_per_second
    }
    
    /// Get overall health status
    pub fn health_status(&self) -> DashboardHealthStatus {
        let dashboard = self.generate_dashboard();
        dashboard.health_status()
    }
    
    /// Record error
    pub fn record_error(&self) {
        self.counters.record_entanglement_failure();
        self.calculations.record_error();
    }
    
    /// Record engine operation with performance metrics
    /// 
    /// Tracks detailed metrics for all operation types including timing, success/failure,
    /// and type-specific performance characteristics.
    pub fn record_engine_operation(
        &self,
        op_type: &crate::cognitive::quantum_mcts::entanglement::engine::operation_types::EngineOperationType,
        duration_ms: u64,
        success: bool,
        performance_improvement: Option<f64>,
    ) {
        use crate::cognitive::quantum_mcts::entanglement::engine::operation_types::EngineOperationType;
        
        // Record the operation timing with high precision
        let duration = Duration::from_millis(duration_ms);
        self.counters.record_entanglement_operation(duration);
        
        // Record success/failure state
        if !success {
            self.counters.record_entanglement_failure();
        }
        
        // Record performance improvement if provided
        if let Some(improvement) = performance_improvement {
            self.calculations.record_performance_improvement(improvement);
        }
        
        // Record operation type specific metrics
        match op_type {
            EngineOperationType::FullOptimization => {
                self.counters.record_entanglements_created(1);
                self.calculations.record_optimization_metrics(
                    performance_improvement.unwrap_or(0.0),
                    1, // Single operation
                    duration_ms,
                );
            }
            EngineOperationType::StrategyCreation => {
                self.counters.record_entanglements_created(1);
                self.calculations.record_creation_metrics(
                    1, // entanglements_created
                    performance_improvement.unwrap_or(0.0),
                    duration_ms,
                );
            }
            EngineOperationType::IntelligentPruning => {
                self.counters.record_entanglements_pruned(1);
                self.calculations.record_pruning_metrics(
                    1, // entanglements_removed
                    performance_improvement.unwrap_or(0.0),
                    duration_ms,
                );
            }
            EngineOperationType::LoadBalancing => {
                self.calculations.record_balancing_metrics(
                    performance_improvement.unwrap_or(0.0),
                    duration_ms,
                );
            }
            EngineOperationType::HealthCheck => {
                // Health checks might not have performance improvement metrics
                let health_score = if success { 1.0 } else { 0.0 };
                self.calculations.record_health_metrics(
                    health_score,
                    0, // No issues by default, actual issues come from HealthCheck details
                    duration_ms,
                );
            }
            EngineOperationType::CombinedOptimization => {
                // Combined operations will be broken down by their components
                // We still record the overall operation timing
                self.calculations.record_combined_operation_metrics(
                    performance_improvement.unwrap_or(0.0),
                    duration_ms,
                );
            }
        }
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.counters.reset();
        self.calculations.reset();
        self.reporter.reset();
        self.rolling_monitor.reset();
        self.active_benchmark = None;
        self.creation_time = Instant::now();
    }
    
    /// Get comprehensive status summary
    pub fn status_summary(&self) -> String {
        let snapshot = self.performance_snapshot();
        let rolling_stats = self.rolling_monitor.current_statistics();
        let health = self.health_status();
        
        format!(
            "=== {} Metrics Status ===\n\
            Uptime: {}\n\
            Health: {}\n\
            \n\
            --- Current Performance ---\n\
            Operations: {} ({:.1}/sec)\n\
            Avg Operation Time: {}\n\
            Influence Calculations: {} ({:.1}/sec)\n\
            Avg Influence Time: {}\n\
            Error Rate: {:.2}%\n\
            \n\
            --- Rolling Statistics ---\n\
            {}\n\
            Stable: {}\n\
            Trend: {}\n\
            \n\
            --- Benchmark Status ---\n\
            Active: {}",
            self.system_name,
            TimingUtils::format_duration(self.uptime()),
            health.description(),
            snapshot.operation_count,
            snapshot.operations_per_second,
            TimingUtils::format_duration(snapshot.average_operation_time),
            snapshot.influence_calculation_count,
            snapshot.influence_calculations_per_second,
            TimingUtils::format_duration(snapshot.average_influence_time),
            snapshot.error_rate * 100.0,
            rolling_stats.summary(),
            self.is_performance_stable(),
            self.performance_trend().description(),
            self.has_active_benchmark()
        )
    }
    
    /// Time a closure and record the operation
    pub fn time_operation<F, R>(&mut self, f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = TimingUtils::time_and_record(f, &self.counters);
        self.rolling_monitor.add_sample(duration);
        
        if let Some(benchmark) = &mut self.active_benchmark {
            benchmark.record_sample(duration);
        }
        
        (result, duration)
    }
    
    /// Time an async closure and record the operation
    pub async fn time_operation_async<F, Fut, R>(&mut self, f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let (result, duration) = TimingUtils::time_async_and_record(f, &self.counters).await;
        self.rolling_monitor.add_sample(duration);
        
        if let Some(benchmark) = &mut self.active_benchmark {
            benchmark.record_sample(duration);
        }
        
        (result, duration)
    }
}

impl Default for EntanglementMetrics {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

/// Factory for creating optimized metrics instances
pub struct MetricsFactory;

impl MetricsFactory {
    /// Create metrics for high-frequency operations
    pub fn create_high_frequency(system_name: String) -> EntanglementMetrics {
        EntanglementMetrics::new_high_throughput(system_name)
    }
    
    /// Create metrics for low-overhead scenarios
    pub fn create_low_overhead(system_name: String) -> EntanglementMetrics {
        EntanglementMetrics::new_lightweight(system_name)
    }
    
    /// Create metrics for development/debugging
    pub fn create_debug(system_name: String) -> EntanglementMetrics {
        let mut metrics = EntanglementMetrics::new(system_name);
        // Start a default benchmark for debugging
        let _ = metrics.start_benchmark("debug_benchmark".to_string());
        metrics
    }
    
    /// Create metrics optimized for specific use cases
    pub fn create_optimized(system_name: String, use_case: MetricsUseCase) -> EntanglementMetrics {
        match use_case {
            MetricsUseCase::HighThroughput => Self::create_high_frequency(system_name),
            MetricsUseCase::LowOverhead => Self::create_low_overhead(system_name),
            MetricsUseCase::Development => Self::create_debug(system_name),
            MetricsUseCase::Production => EntanglementMetrics::new(system_name),
        }
    }
}

/// Metrics use case optimization targets
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricsUseCase {
    /// High-throughput scenarios with many operations
    HighThroughput,
    /// Low-overhead scenarios where metrics should be minimal
    LowOverhead,
    /// Development scenarios with detailed tracking
    Development,
    /// Production scenarios with balanced performance and monitoring
    Production,
}

/// Convenience macro for timing operations with metrics
#[macro_export]
macro_rules! time_with_metrics {
    ($metrics:expr, $operation:expr) => {{
        let tracker = $metrics.start_tracking();
        let result = $operation;
        let duration = tracker.record_and_stop($metrics.counters());
        (result, duration)
    }};
    
    ($metrics:expr, $name:expr, $operation:expr) => {{
        let tracker = $metrics.start_tracking_named($name.to_string());
        let result = $operation;
        let duration = tracker.record_and_stop($metrics.counters());
        (result, duration)
    }};
}

/// Convenience macro for timing async operations with metrics
#[macro_export]
macro_rules! time_async_with_metrics {
    ($metrics:expr, $operation:expr) => {{
        let tracker = $metrics.start_tracking();
        let result = $operation.await;
        let duration = tracker.record_and_stop($metrics.counters());
        (result, duration)
    }};
    
    ($metrics:expr, $name:expr, $operation:expr) => {{
        let tracker = $metrics.start_tracking_named($name.to_string());
        let result = $operation.await;
        let duration = tracker.record_and_stop($metrics.counters());
        (result, duration)
    }};
}

// Re-export macros
pub use time_with_metrics;
pub use time_async_with_metrics;
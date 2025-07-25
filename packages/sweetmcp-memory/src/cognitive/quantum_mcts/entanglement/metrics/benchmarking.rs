//! Benchmarking utilities and performance assessment for quantum entanglement
//!
//! This module provides blazing-fast benchmarking capabilities with zero-allocation
//! patterns and comprehensive performance assessment tools.

// Re-export core benchmarking components
pub use crate::cognitive::quantum_mcts::entanglement::metrics::benchmark_core::EntanglementBenchmark;
pub use crate::cognitive::quantum_mcts::entanglement::metrics::benchmark_results::{BenchmarkResults, BenchmarkComparison};
pub use crate::cognitive::quantum_mcts::entanglement::metrics::rolling_monitor::{RollingPerformanceMonitor, RollingStatistics};
pub use crate::cognitive::quantum_mcts::entanglement::metrics::performance_trends::{PerformanceTrend, ActionPriority, TrendAnalyzer};
pub use crate::cognitive::quantum_mcts::entanglement::metrics::benchmark_suite::{EntanglementBenchmarkSuite, SuiteSummary};
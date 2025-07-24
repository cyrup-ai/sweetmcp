//! Improvement module coordination
//!
//! This module coordinates all improvement components with optimized access patterns
//! and zero-allocation re-exports for blazing-fast performance.

pub mod engine;
pub mod parallel_execution;
pub mod amplification;
pub mod convergence;
pub mod metrics;
pub mod memory_tracking;
pub mod memory_health;
pub mod simulation;
pub mod amplitude_amplifier;
pub mod result_types;

// Re-export key types for convenient access
pub use engine::RecursiveImprovementEngine;
pub use parallel_execution::ParallelExecutor;
pub use amplification::AmplificationEngine;
pub use convergence::ConvergenceAnalyzer;
pub use memory_tracking::{MemoryTracker, MemoryStats, MemoryHealthStatus};
pub use memory_health::{
    AllocationStats, MemoryTrend, CleanupRecommendation, 
    MemoryHealth, MemoryHealthStatus as HealthStatus
};
pub use metrics::{
    ImprovementMetrics, MetricsSummary, PerformanceTrend,
    AmplificationResult, calculate_quantum_state_quality,
};
pub use simulation::{
    SimulationResult, IterationResult, DepthResult
};
pub use amplitude_amplifier::{
    QuantumAmplitudeAmplifier, AmplifierConfig
};
pub use result_types::{
    ImprovementResult, TerminationReason, ConvergenceTrend
};
//! Cognitive orchestrator module
//!
//! This module provides comprehensive infinite agentic orchestrator
//! functionality for committee-driven optimization with zero allocation
//! patterns and blazing-fast performance.

pub mod core;
pub mod iteration;

// Decomposed parsing modules
pub mod parsing_core;
pub mod parsing_extraction;
pub mod parsing_validation;
pub mod parsing_mod;

// Alias for backward compatibility
pub use parsing_mod as parsing;

// Re-export key types and functions for ergonomic usage
pub use core::{
    InfiniteOrchestrator, OrchestratorStatus, PerformanceBaseline, ImprovementMetrics,
};

pub use iteration::{
    IterationPlan, IterationStats,
};

pub use parsing::{
    parse_spec, validate_spec, extract_percentage, extract_number,
    parse_constraints, parse_success_criteria, determine_optimization_type,
    extract_config_values, parse_baseline_metrics, create_default_spec,
    merge_specs, ConfigValues, BaselineMetrics,
};

/// Create a new infinite orchestrator with default settings
pub fn orchestrator<P: AsRef<std::path::Path>>(
    spec_file: P,
    output_dir: P,
    initial_code: String,
    user_objective: String,
) -> Result<InfiniteOrchestrator, crate::cognitive::types::CognitiveError> {
    InfiniteOrchestrator::new(
        spec_file,
        output_dir,
        initial_code,
        100.0, // default latency
        50.0,  // default memory
        75.0,  // default relevance
        user_objective,
    )
}

/// Create a new infinite orchestrator with custom metrics
pub fn orchestrator_with_metrics<P: AsRef<std::path::Path>>(
    spec_file: P,
    output_dir: P,
    initial_code: String,
    initial_latency: f64,
    initial_memory: f64,
    initial_relevance: f64,
    user_objective: String,
) -> Result<InfiniteOrchestrator, crate::cognitive::types::CognitiveError> {
    InfiniteOrchestrator::new(
        spec_file,
        output_dir,
        initial_code,
        initial_latency,
        initial_memory,
        initial_relevance,
        user_objective,
    )
}

/// Validate orchestrator configuration
pub fn validate_orchestrator_config(
    orchestrator: &InfiniteOrchestrator,
) -> Result<(), crate::cognitive::types::CognitiveError> {
    orchestrator.validate_config()
}

/// Get orchestrator performance summary
pub fn get_performance_summary(
    orchestrator: &InfiniteOrchestrator,
    current_latency: f64,
    current_memory: f64,
    current_relevance: f64,
) -> ImprovementMetrics {
    orchestrator.calculate_improvement(current_latency, current_memory, current_relevance)
}
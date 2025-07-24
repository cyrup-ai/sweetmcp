//! Parsing integration module with ergonomic re-exports
//!
//! This module provides a unified interface for optimization specification parsing
//! with convenient re-exports and utility functions for blazing-fast performance.

// Re-export all public types and functions
pub use super::parsing_core::{
    parse_spec, parse_markdown_spec, parse_json_spec, parse_spec_content,
    normalize_spec, parse_multiple_specs, create_simple_spec, parse_spec_with_defaults,
};

pub use super::parsing_extraction::{
    extract_percentage, extract_number, extract_config_values, parse_baseline_metrics,
    extract_optimization_indicators, extract_constraint_patterns, extract_success_patterns,
    extract_thresholds, extract_time_values, text_utils,
    ConfigValues, BaselineMetrics, ThresholdValues, TimeValues,
};

pub use super::parsing_validation::{
    validate_spec, parse_constraints, parse_success_criteria, determine_optimization_type,
    validate_constraint_consistency, validate_success_criteria_achievability,
    analyze_spec_completeness, SpecCompletenessAnalysis,
};

use crate::cognitive::types::{CognitiveError, OptimizationSpec, OptimizationType};
use std::path::Path;

/// Create a default optimization specification
pub fn create_default_spec(objective: &str) -> OptimizationSpec {
    OptimizationSpec {
        objective: objective.to_string(),
        constraints: vec![
            "Zero allocation patterns".to_string(),
            "Blazing-fast performance".to_string(),
            "No unsafe code".to_string(),
            "Idiomatic Rust".to_string(),
        ],
        success_criteria: vec![
            "Improves performance metrics".to_string(),
            "Maintains code quality".to_string(),
            "Passes all tests".to_string(),
        ],
        optimization_type: OptimizationType::Performance,
        timeout_ms: Some(300_000),
        max_iterations: Some(100),
        target_quality: 0.8,
    }
}

/// Merge multiple specifications into one
pub fn merge_specs(specs: &[OptimizationSpec]) -> Result<OptimizationSpec, CognitiveError> {
    if specs.is_empty() {
        return Err(CognitiveError::SpecError(
            "Cannot merge empty specification list".to_string()
        ));
    }
    
    if specs.len() == 1 {
        return Ok(specs[0].clone());
    }
    
    let mut merged = specs[0].clone();
    
    for spec in &specs[1..] {
        // Merge constraints
        for constraint in &spec.constraints {
            if !merged.constraints.contains(constraint) {
                merged.constraints.push(constraint.clone());
            }
        }
        
        // Merge success criteria
        for criterion in &spec.success_criteria {
            if !merged.success_criteria.contains(criterion) {
                merged.success_criteria.push(criterion.clone());
            }
        }
        
        // Use most restrictive timeout
        if let (Some(merged_timeout), Some(spec_timeout)) = (merged.timeout_ms, spec.timeout_ms) {
            merged.timeout_ms = Some(merged_timeout.min(spec_timeout));
        } else if spec.timeout_ms.is_some() {
            merged.timeout_ms = spec.timeout_ms;
        }
        
        // Use most restrictive max iterations
        if let (Some(merged_max), Some(spec_max)) = (merged.max_iterations, spec.max_iterations) {
            merged.max_iterations = Some(merged_max.min(spec_max));
        } else if spec.max_iterations.is_some() {
            merged.max_iterations = spec.max_iterations;
        }
        
        // Use highest target quality
        merged.target_quality = merged.target_quality.max(spec.target_quality);
    }
    
    validate_spec(&merged)?;
    Ok(merged)
}

/// Quick parsing utilities for common use cases
pub mod quick {
    use super::*;

    /// Parse specification from file with validation
    pub fn parse_file<P: AsRef<Path>>(spec_file: P) -> Result<OptimizationSpec, CognitiveError> {
        parse_spec(spec_file)
    }

    /// Parse specification from string content
    pub fn parse_string(content: &str) -> Result<OptimizationSpec, CognitiveError> {
        parse_spec_content(content)
    }

    /// Create and validate a simple specification
    pub fn create_simple(objective: &str) -> OptimizationSpec {
        create_simple_spec(objective)
    }

    /// Quick validation check
    pub fn is_valid(spec: &OptimizationSpec) -> bool {
        validate_spec(spec).is_ok()
    }

    /// Get completeness score (0.0 to 1.0)
    pub fn completeness_score(spec: &OptimizationSpec) -> f64 {
        analyze_spec_completeness(spec).overall_completeness
    }

    /// Check if specification is ready for optimization
    pub fn is_ready(spec: &OptimizationSpec) -> bool {
        let analysis = analyze_spec_completeness(spec);
        analysis.is_complete() && validate_spec(spec).is_ok()
    }

    /// Get optimization type from text
    pub fn detect_type(text: &str) -> OptimizationType {
        determine_optimization_type(text)
    }

    /// Extract key metrics from text
    pub fn extract_metrics(text: &str) -> BaselineMetrics {
        parse_baseline_metrics(text)
    }
}

/// Parsing presets for different scenarios
pub mod presets {
    use super::*;

    /// Performance optimization preset
    pub fn performance_optimization(objective: &str) -> OptimizationSpec {
        OptimizationSpec {
            objective: objective.to_string(),
            constraints: vec![
                "Zero allocation patterns".to_string(),
                "Blazing-fast performance".to_string(),
                "No unsafe code".to_string(),
                "Max latency increase: 5%".to_string(),
                "Maintain memory efficiency".to_string(),
            ],
            success_criteria: vec![
                "Reduces latency by at least 10%".to_string(),
                "Maintains or improves memory usage".to_string(),
                "Passes all performance benchmarks".to_string(),
                "No regression in functionality".to_string(),
            ],
            optimization_type: OptimizationType::Performance,
            timeout_ms: Some(300_000),
            max_iterations: Some(150),
            target_quality: 0.85,
        }
    }

    /// Memory optimization preset
    pub fn memory_optimization(objective: &str) -> OptimizationSpec {
        OptimizationSpec {
            objective: objective.to_string(),
            constraints: vec![
                "Zero allocation patterns".to_string(),
                "No unsafe code".to_string(),
                "Max memory increase: 0%".to_string(),
                "Maintain performance".to_string(),
                "Idiomatic Rust".to_string(),
            ],
            success_criteria: vec![
                "Reduces memory usage by at least 15%".to_string(),
                "No performance degradation".to_string(),
                "Eliminates memory leaks".to_string(),
                "Passes all tests".to_string(),
            ],
            optimization_type: OptimizationType::Memory,
            timeout_ms: Some(450_000),
            max_iterations: Some(200),
            target_quality: 0.8,
        }
    }

    /// Quality optimization preset
    pub fn quality_optimization(objective: &str) -> OptimizationSpec {
        OptimizationSpec {
            objective: objective.to_string(),
            constraints: vec![
                "Maintain performance".to_string(),
                "No unsafe code".to_string(),
                "Idiomatic Rust".to_string(),
                "Preserve API compatibility".to_string(),
            ],
            success_criteria: vec![
                "Improves code quality metrics".to_string(),
                "Increases test coverage".to_string(),
                "Reduces complexity".to_string(),
                "Improves maintainability".to_string(),
            ],
            optimization_type: OptimizationType::Quality,
            timeout_ms: Some(600_000),
            max_iterations: Some(100),
            target_quality: 0.9,
        }
    }

    /// Security optimization preset
    pub fn security_optimization(objective: &str) -> OptimizationSpec {
        OptimizationSpec {
            objective: objective.to_string(),
            constraints: vec![
                "No unsafe code".to_string(),
                "Zero allocation patterns".to_string(),
                "Validate all inputs".to_string(),
                "No information leakage".to_string(),
                "Maintain performance".to_string(),
            ],
            success_criteria: vec![
                "Eliminates security vulnerabilities".to_string(),
                "Passes security audit".to_string(),
                "Implements proper error handling".to_string(),
                "No sensitive data exposure".to_string(),
            ],
            optimization_type: OptimizationType::Security,
            timeout_ms: Some(900_000),
            max_iterations: Some(75),
            target_quality: 0.95,
        }
    }

    /// Balanced optimization preset
    pub fn balanced_optimization(objective: &str) -> OptimizationSpec {
        OptimizationSpec {
            objective: objective.to_string(),
            constraints: vec![
                "Zero allocation patterns".to_string(),
                "Blazing-fast performance".to_string(),
                "No unsafe code".to_string(),
                "Idiomatic Rust".to_string(),
                "Maintainable code".to_string(),
            ],
            success_criteria: vec![
                "Improves overall performance".to_string(),
                "Maintains code quality".to_string(),
                "Reduces resource usage".to_string(),
                "Passes all tests".to_string(),
            ],
            optimization_type: OptimizationType::Performance,
            timeout_ms: Some(450_000),
            max_iterations: Some(125),
            target_quality: 0.82,
        }
    }
}

/// Builder pattern for customized specification creation
pub struct SpecificationBuilder {
    spec: OptimizationSpec,
}

impl Default for SpecificationBuilder {
    fn default() -> Self {
        Self {
            spec: create_default_spec("Custom optimization"),
        }
    }
}

impl SpecificationBuilder {
    /// Create new specification builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set objective
    pub fn objective(mut self, objective: &str) -> Self {
        self.spec.objective = objective.to_string();
        self
    }

    /// Add constraint
    pub fn constraint(mut self, constraint: &str) -> Self {
        self.spec.constraints.push(constraint.to_string());
        self
    }

    /// Add multiple constraints
    pub fn constraints(mut self, constraints: &[&str]) -> Self {
        for constraint in constraints {
            self.spec.constraints.push(constraint.to_string());
        }
        self
    }

    /// Add success criterion
    pub fn success_criterion(mut self, criterion: &str) -> Self {
        self.spec.success_criteria.push(criterion.to_string());
        self
    }

    /// Add multiple success criteria
    pub fn success_criteria(mut self, criteria: &[&str]) -> Self {
        for criterion in criteria {
            self.spec.success_criteria.push(criterion.to_string());
        }
        self
    }

    /// Set optimization type
    pub fn optimization_type(mut self, opt_type: OptimizationType) -> Self {
        self.spec.optimization_type = opt_type;
        self
    }

    /// Set timeout in milliseconds
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.spec.timeout_ms = Some(timeout);
        self
    }

    /// Set timeout in seconds
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.spec.timeout_ms = Some(seconds * 1000);
        self
    }

    /// Set timeout in minutes
    pub fn timeout_minutes(mut self, minutes: u64) -> Self {
        self.spec.timeout_ms = Some(minutes * 60 * 1000);
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, max_iter: u32) -> Self {
        self.spec.max_iterations = Some(max_iter);
        self
    }

    /// Set target quality
    pub fn target_quality(mut self, quality: f64) -> Self {
        self.spec.target_quality = quality.clamp(0.1, 1.0);
        self
    }

    /// Build and validate specification
    pub fn build(self) -> Result<OptimizationSpec, CognitiveError> {
        validate_spec(&self.spec)?;
        Ok(self.spec)
    }

    /// Build without validation (for testing)
    pub fn build_unchecked(self) -> OptimizationSpec {
        self.spec
    }
}

/// Utility functions for specification management
pub mod utils {
    use super::*;

    /// Compare two specifications and highlight differences
    pub fn compare_specs(spec1: &OptimizationSpec, spec2: &OptimizationSpec) -> SpecComparison {
        SpecComparison {
            objective_differs: spec1.objective != spec2.objective,
            constraints_differ: spec1.constraints != spec2.constraints,
            criteria_differ: spec1.success_criteria != spec2.success_criteria,
            type_differs: spec1.optimization_type != spec2.optimization_type,
            timeout_differs: spec1.timeout_ms != spec2.timeout_ms,
            iterations_differ: spec1.max_iterations != spec2.max_iterations,
            quality_differs: (spec1.target_quality - spec2.target_quality).abs() > 0.01,
        }
    }

    /// Generate specification summary
    pub fn summarize_spec(spec: &OptimizationSpec) -> String {
        format!(
            "Optimization Specification Summary:\n\
             - Objective: {}\n\
             - Type: {:?}\n\
             - Constraints: {} items\n\
             - Success Criteria: {} items\n\
             - Timeout: {}ms\n\
             - Max Iterations: {}\n\
             - Target Quality: {:.1}%",
            spec.objective,
            spec.optimization_type,
            spec.constraints.len(),
            spec.success_criteria.len(),
            spec.timeout_ms.unwrap_or(0),
            spec.max_iterations.unwrap_or(0),
            spec.target_quality * 100.0
        )
    }

    /// Export specification to JSON string
    pub fn to_json(spec: &OptimizationSpec) -> Result<String, CognitiveError> {
        serde_json::to_string_pretty(spec)
            .map_err(|e| CognitiveError::SpecError(format!("JSON serialization error: {}", e)))
    }

    /// Clone and modify specification
    pub fn modify_spec<F>(spec: &OptimizationSpec, modifier: F) -> OptimizationSpec
    where
        F: FnOnce(&mut OptimizationSpec),
    {
        let mut modified = spec.clone();
        modifier(&mut modified);
        modified
    }

    /// Check if specification meets minimum requirements
    pub fn meets_minimum_requirements(spec: &OptimizationSpec) -> bool {
        !spec.objective.trim().is_empty() &&
        !spec.constraints.is_empty() &&
        !spec.success_criteria.is_empty() &&
        spec.target_quality > 0.0 &&
        spec.target_quality <= 1.0
    }

    /// Get specification complexity score (0.0 to 1.0)
    pub fn complexity_score(spec: &OptimizationSpec) -> f64 {
        let constraint_complexity = (spec.constraints.len().min(10) as f64) / 10.0;
        let criteria_complexity = (spec.success_criteria.len().min(10) as f64) / 10.0;
        let objective_complexity = if spec.objective.len() > 100 { 1.0 } else { spec.objective.len() as f64 / 100.0 };
        
        (constraint_complexity + criteria_complexity + objective_complexity) / 3.0
    }
}

/// Specification comparison result
#[derive(Debug)]
pub struct SpecComparison {
    pub objective_differs: bool,
    pub constraints_differ: bool,
    pub criteria_differ: bool,
    pub type_differs: bool,
    pub timeout_differs: bool,
    pub iterations_differ: bool,
    pub quality_differs: bool,
}

impl SpecComparison {
    /// Check if specifications are identical
    pub fn are_identical(&self) -> bool {
        !self.objective_differs &&
        !self.constraints_differ &&
        !self.criteria_differ &&
        !self.type_differs &&
        !self.timeout_differs &&
        !self.iterations_differ &&
        !self.quality_differs
    }

    /// Count number of differences
    pub fn difference_count(&self) -> usize {
        [
            self.objective_differs,
            self.constraints_differ,
            self.criteria_differ,
            self.type_differs,
            self.timeout_differs,
            self.iterations_differ,
            self.quality_differs,
        ].iter().filter(|&&x| x).count()
    }

    /// Get similarity score (0.0 to 1.0)
    pub fn similarity_score(&self) -> f64 {
        1.0 - (self.difference_count() as f64 / 7.0)
    }
}

/// Convenience macro for creating specifications
#[macro_export]
macro_rules! optimization_spec {
    ($objective:expr) => {
        $crate::cognitive::orchestrator::parsing_mod::create_default_spec($objective)
    };
    ($objective:expr, $opt_type:expr) => {{
        let mut spec = $crate::cognitive::orchestrator::parsing_mod::create_default_spec($objective);
        spec.optimization_type = $opt_type;
        spec
    }};
}

/// Re-export the macro
pub use optimization_spec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_specs() {
        let spec1 = create_default_spec("Objective 1");
        let spec2 = create_default_spec("Objective 2");
        
        let merged = merge_specs(&[spec1, spec2]).expect("Should merge specs");
        assert_eq!(merged.objective, "Objective 1"); // First spec's objective
        assert!(merged.constraints.len() >= 4); // Should have constraints from both
    }

    #[test]
    fn test_specification_builder() {
        let spec = SpecificationBuilder::new()
            .objective("Test optimization")
            .constraint("No unsafe code")
            .success_criterion("Passes tests")
            .timeout_seconds(60)
            .max_iterations(50)
            .target_quality(0.9)
            .build()
            .expect("Should build valid spec");
        
        assert_eq!(spec.objective, "Test optimization");
        assert_eq!(spec.timeout_ms, Some(60000));
        assert_eq!(spec.target_quality, 0.9);
    }

    #[test]
    fn test_presets() {
        let perf_spec = presets::performance_optimization("Optimize performance");
        assert_eq!(perf_spec.optimization_type, OptimizationType::Performance);
        
        let mem_spec = presets::memory_optimization("Optimize memory");
        assert_eq!(mem_spec.optimization_type, OptimizationType::Memory);
    }

    #[test]
    fn test_quick_utilities() {
        let spec = create_default_spec("Test");
        assert!(quick::is_valid(&spec));
        assert!(quick::is_ready(&spec));
        assert!(quick::completeness_score(&spec) > 0.7);
    }

    #[test]
    fn test_utils() {
        let spec1 = create_default_spec("Test 1");
        let spec2 = create_default_spec("Test 2");
        
        let comparison = utils::compare_specs(&spec1, &spec2);
        assert!(comparison.objective_differs);
        assert!(!comparison.are_identical());
        assert!(comparison.similarity_score() < 1.0);
    }
}

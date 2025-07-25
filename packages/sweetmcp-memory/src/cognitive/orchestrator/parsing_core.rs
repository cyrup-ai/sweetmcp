//! Core parsing functionality for optimization specifications
//!
//! This module provides the main parsing logic for optimization specifications
//! with zero allocation patterns and blazing-fast performance.

use crate::cognitive::types::{CognitiveError, OptimizationType};
use crate::vector::async_vector_optimization::OptimizationSpec;
use super::parsing_extraction::{extract_number, extract_percentage};
use super::parsing_validation::validate_spec;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::{debug, warn};

/// Parse optimization specification from file
pub fn parse_spec<P: AsRef<Path>>(spec_file: P) -> Result<OptimizationSpec, CognitiveError> {
    let mut file = File::open(spec_file)
        .map_err(|e| CognitiveError::SpecError(e.to_string()))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| CognitiveError::SpecError(e.to_string()))?;

    // Try JSON first
    if let Ok(spec) = serde_json::from_str::<OptimizationSpec>(&contents) {
        debug!("Parsed optimization spec from JSON format");
        validate_spec(&spec)?;
        return Ok(spec);
    }

    // Fall back to markdown parsing
    debug!("JSON parsing failed, attempting markdown parsing");
    let spec = parse_markdown_spec(&contents)?;
    validate_spec(&spec)?;
    Ok(spec)
}

/// Parse optimization specification from markdown format
pub fn parse_markdown_spec(contents: &str) -> Result<OptimizationSpec, CognitiveError> {
    let mut user_objective = "Optimize code performance".to_string();
    let mut baseline_latency = 100.0;
    let mut baseline_memory = 50.0;
    let mut baseline_relevance = 75.0;
    let mut max_latency_increase = 10.0;
    let mut max_memory_increase = 20.0;
    let mut min_relevance_improvement = 5.0;

    for line in contents.lines() {
        let line = line.trim();
        
        if line.contains("Latency:") {
            if let Some(num) = extract_number(line) {
                baseline_latency = num;
            }
        } else if line.contains("Memory:") {
            if let Some(num) = extract_number(line) {
                baseline_memory = num;
            }
        } else if line.contains("Relevance:") {
            if let Some(num) = extract_percentage(line) {
                baseline_relevance = num;
            }
        } else if line.starts_with("# ") || line.starts_with("## ") {
            // Extract objective from markdown headers
            user_objective = line.trim_start_matches('#').trim().to_string();
        } else if line.contains("Objective:") || line.contains("Goal:") {
            // Extract objective from explicit objective lines
            if let Some(colon_pos) = line.find(':') {
                user_objective = line[colon_pos + 1..].trim().to_string();
            }
        } else if line.contains("Max latency increase:") {
            if let Some(num) = extract_percentage(line) {
                max_latency_increase = num;
            }
        } else if line.contains("Max memory increase:") {
            if let Some(num) = extract_percentage(line) {
                max_memory_increase = num;
            }
        } else if line.contains("Min relevance improvement:") {
            if let Some(num) = extract_percentage(line) {
                min_relevance_improvement = num;
            }
        }
    }

    let spec = OptimizationSpec {
        objective: user_objective,
        constraints: vec![
            format!("Max latency increase: {}%", max_latency_increase),
            format!("Max memory increase: {}%", max_memory_increase),
            format!("Min relevance improvement: {}%", min_relevance_improvement),
            "Idiomatic Rust".to_string(),
        ],
        success_criteria: vec![
            "Reduces latency by at least 5%".to_string(),
            "Maintains or improves memory usage".to_string(),
            "Improves relevance score".to_string(),
        ],
        optimization_type: OptimizationType::Performance,
        timeout_ms: Some(300_000), // 5 minutes
        max_iterations: Some(100),
        target_quality: 0.8,
    };

    debug!("Successfully parsed markdown specification");
    Ok(spec)
}

/// Parse optimization specification from JSON string
pub fn parse_json_spec(json_content: &str) -> Result<OptimizationSpec, CognitiveError> {
    let spec = serde_json::from_str::<OptimizationSpec>(json_content)
        .map_err(|e| CognitiveError::SpecError(format!("JSON parsing error: {}", e)))?;
    
    validate_spec(&spec)?;
    debug!("Successfully parsed JSON specification");
    Ok(spec)
}

/// Parse specification from raw text content with format detection
pub fn parse_spec_content(content: &str) -> Result<OptimizationSpec, CognitiveError> {
    // Detect format based on content
    let trimmed = content.trim();
    
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        // Looks like JSON
        parse_json_spec(content)
    } else if trimmed.contains('#') || trimmed.contains("Objective:") || trimmed.contains("Goal:") {
        // Looks like markdown
        parse_markdown_spec(content)
    } else {
        // Try JSON first, then markdown
        match parse_json_spec(content) {
            Ok(spec) => Ok(spec),
            Err(_) => parse_markdown_spec(content),
        }
    }
}

/// Validate and normalize a parsed specification
pub fn normalize_spec(mut spec: OptimizationSpec) -> Result<OptimizationSpec, CognitiveError> {
    // Ensure objective is not empty
    if spec.objective.trim().is_empty() {
        spec.objective = "Optimize code performance".to_string();
    }

    // Ensure minimum constraints
    let required_constraints = vec![
        "Zero allocation patterns".to_string(),
        "Blazing-fast performance".to_string(),
        "No unsafe code".to_string(),
        "Idiomatic Rust".to_string(),
    ];

    for constraint in required_constraints {
        if !spec.constraints.iter().any(|c| c.contains(&constraint)) {
            spec.constraints.push(constraint);
        }
    }

    // Ensure minimum success criteria
    if spec.success_criteria.is_empty() {
        spec.success_criteria = vec![
            "Improves performance metrics".to_string(),
            "Maintains code quality".to_string(),
            "Passes all tests".to_string(),
        ];
    }

    // Set reasonable defaults for missing values
    if spec.timeout_ms.is_none() {
        spec.timeout_ms = Some(300_000); // 5 minutes
    }

    if spec.max_iterations.is_none() {
        spec.max_iterations = Some(100);
    }

    if spec.target_quality <= 0.0 || spec.target_quality > 1.0 {
        spec.target_quality = 0.8;
    }

    validate_spec(&spec)?;
    Ok(spec)
}

/// Parse multiple specifications from a single content string
pub fn parse_multiple_specs(content: &str) -> Result<Vec<OptimizationSpec>, CognitiveError> {
    let mut specs = Vec::new();
    
    // Split by common delimiters
    let sections = if content.contains("---") {
        content.split("---").collect::<Vec<_>>()
    } else if content.contains("===") {
        content.split("===").collect::<Vec<_>>()
    } else {
        // Try to parse as single spec
        return Ok(vec![parse_spec_content(content)?]);
    };

    for section in sections {
        let section = section.trim();
        if !section.is_empty() {
            match parse_spec_content(section) {
                Ok(spec) => specs.push(spec),
                Err(e) => {
                    warn!("Failed to parse specification section: {}", e);
                    // Continue with other sections
                }
            }
        }
    }

    if specs.is_empty() {
        return Err(CognitiveError::SpecError(
            "No valid specifications found in content".to_string()
        ));
    }

    Ok(specs)
}

/// Create a specification from a simple objective string
pub fn create_simple_spec(objective: &str) -> OptimizationSpec {
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

/// Parse specification with custom defaults
pub fn parse_spec_with_defaults(
    content: &str,
    default_timeout: Option<u64>,
    default_iterations: Option<u32>,
    default_quality: f64,
) -> Result<OptimizationSpec, CognitiveError> {
    let mut spec = parse_spec_content(content)?;
    
    // Apply custom defaults if values are missing
    if spec.timeout_ms.is_none() && default_timeout.is_some() {
        spec.timeout_ms = default_timeout;
    }
    
    if spec.max_iterations.is_none() && default_iterations.is_some() {
        spec.max_iterations = default_iterations;
    }
    
    if spec.target_quality <= 0.0 || spec.target_quality > 1.0 {
        spec.target_quality = default_quality.clamp(0.1, 1.0);
    }
    
    validate_spec(&spec)?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let content = r#"
# Optimize Memory Usage

Objective: Reduce memory allocation by 20%

Baseline:
- Memory: 100MB
- Latency: 50ms
- Relevance: 80%

Constraints:
- Max latency increase: 5%
- Max memory increase: 0%
- Min relevance improvement: 2%
"#;
        
        let spec = parse_markdown_spec(content).expect("Should parse markdown");
        assert_eq!(spec.objective, "Optimize Memory Usage");
        assert!(spec.constraints.len() >= 3);
    }

    #[test]
    fn test_parse_json_spec() {
        let json = r#"{
            "objective": "Test optimization",
            "constraints": ["No unsafe code"],
            "success_criteria": ["Passes tests"],
            "optimization_type": "Performance",
            "timeout_ms": 60000,
            "max_iterations": 50,
            "target_quality": 0.9
        }"#;
        
        let spec = parse_json_spec(json).expect("Should parse JSON");
        assert_eq!(spec.objective, "Test optimization");
        assert_eq!(spec.target_quality, 0.9);
    }

    #[test]
    fn test_normalize_spec() {
        let mut spec = OptimizationSpec {
            objective: "".to_string(),
            constraints: vec![],
            success_criteria: vec![],
            optimization_type: OptimizationType::Performance,
            timeout_ms: None,
            max_iterations: None,
            target_quality: 0.0,
        };
        
        spec = normalize_spec(spec).expect("Should normalize");
        assert!(!spec.objective.is_empty());
        assert!(!spec.constraints.is_empty());
        assert!(!spec.success_criteria.is_empty());
        assert!(spec.timeout_ms.is_some());
        assert!(spec.max_iterations.is_some());
        assert!(spec.target_quality > 0.0);
    }
}

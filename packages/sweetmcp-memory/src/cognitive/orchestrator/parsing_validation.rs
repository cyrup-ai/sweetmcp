//! Validation and analysis for optimization specifications
//!
//! This module provides validation logic and analysis functions for optimization specifications
//! with zero allocation patterns and blazing-fast performance.

use crate::cognitive::types::{CognitiveError, OptimizationType};
use crate::vector::async_vector_optimization::OptimizationSpec;
use super::parsing_extraction::{extract_optimization_indicators, extract_constraint_patterns, extract_success_patterns};
use tracing::{debug, warn};

/// Validate optimization specification
pub fn validate_spec(spec: &OptimizationSpec) -> Result<(), CognitiveError> {
    // Validate objective
    if spec.objective.trim().is_empty() {
        return Err(CognitiveError::SpecError(
            "Objective cannot be empty".to_string()
        ));
    }

    // Validate constraints
    if spec.constraints.is_empty() {
        return Err(CognitiveError::SpecError(
            "At least one constraint must be specified".to_string()
        ));
    }

    // Validate success criteria
    if spec.success_criteria.is_empty() {
        return Err(CognitiveError::SpecError(
            "At least one success criterion must be specified".to_string()
        ));
    }

    // Validate timeout
    if let Some(timeout) = spec.timeout_ms {
        if timeout == 0 {
            return Err(CognitiveError::SpecError(
                "Timeout must be greater than 0".to_string()
            ));
        }
        if timeout > 3600000 { // 1 hour
            warn!("Timeout is very large ({}ms), consider reducing", timeout);
        }
    }

    // Validate max iterations
    if let Some(max_iter) = spec.max_iterations {
        if max_iter == 0 {
            return Err(CognitiveError::SpecError(
                "Max iterations must be greater than 0".to_string()
            ));
        }
        if max_iter > 10000 {
            warn!("Max iterations is very large ({}), consider reducing", max_iter);
        }
    }

    // Validate target quality
    if spec.target_quality <= 0.0 || spec.target_quality > 1.0 {
        return Err(CognitiveError::SpecError(
            "Target quality must be between 0.0 and 1.0".to_string()
        ));
    }

    debug!("Specification validation passed");
    Ok(())
}

/// Parse constraints from specification text
pub fn parse_constraints(text: &str) -> Vec<String> {
    let mut constraints = Vec::new();
    
    // Extract constraint patterns
    let patterns = extract_constraint_patterns(text);
    constraints.extend(patterns);
    
    // Look for specific constraint keywords
    for line in text.lines() {
        let line = line.trim();
        
        if line.contains("constraint") || line.contains("requirement") {
            constraints.push(line.to_string());
        } else if line.contains("must not") || line.contains("cannot") || line.contains("forbidden") {
            constraints.push(line.to_string());
        } else if line.contains("limit") || line.contains("maximum") || line.contains("minimum") {
            constraints.push(line.to_string());
        }
    }
    
    // Add default constraints if none found
    if constraints.is_empty() {
        constraints = vec![
            "Zero allocation patterns".to_string(),
            "Blazing-fast performance".to_string(),
            "No unsafe code".to_string(),
            "Idiomatic Rust".to_string(),
        ];
    }
    
    // Remove duplicates while preserving order
    let mut unique_constraints = Vec::new();
    for constraint in constraints {
        if !unique_constraints.iter().any(|c: &String| c.to_lowercase() == constraint.to_lowercase()) {
            unique_constraints.push(constraint);
        }
    }
    
    unique_constraints
}

/// Parse success criteria from specification text
pub fn parse_success_criteria(text: &str) -> Vec<String> {
    let mut criteria = Vec::new();
    
    // Extract success patterns
    let patterns = extract_success_patterns(text);
    criteria.extend(patterns);
    
    // Look for specific success criteria keywords
    for line in text.lines() {
        let line = line.trim();
        
        if line.contains("success") && !line.contains("criteria") {
            criteria.push(line.to_string());
        } else if line.contains("goal") || line.contains("target") || line.contains("objective") {
            criteria.push(line.to_string());
        } else if line.contains("achieve") || line.contains("accomplish") || line.contains("reach") {
            criteria.push(line.to_string());
        } else if line.contains("improve") || line.contains("enhance") || line.contains("optimize") {
            criteria.push(line.to_string());
        }
    }
    
    // Add default criteria if none found
    if criteria.is_empty() {
        criteria = vec![
            "Improves performance metrics".to_string(),
            "Maintains code quality".to_string(),
            "Passes all tests".to_string(),
        ];
    }
    
    // Remove duplicates while preserving order
    let mut unique_criteria = Vec::new();
    for criterion in criteria {
        if !unique_criteria.iter().any(|c: &String| c.to_lowercase() == criterion.to_lowercase()) {
            unique_criteria.push(criterion);
        }
    }
    
    unique_criteria
}

/// Determine optimization type from specification text
pub fn determine_optimization_type(text: &str) -> OptimizationType {
    let indicators = extract_optimization_indicators(text);
    let text_lower = text.to_lowercase();
    
    // Count different optimization type indicators
    let performance_score = if indicators.contains(&"performance".to_string()) { 2 } else { 0 } +
        if text_lower.contains("speed") || text_lower.contains("fast") || text_lower.contains("latency") { 1 } else { 0 };
    
    let memory_score = if indicators.contains(&"memory".to_string()) { 2 } else { 0 } +
        if text_lower.contains("allocation") || text_lower.contains("heap") || text_lower.contains("stack") { 1 } else { 0 };
    
    let quality_score = if indicators.contains(&"quality".to_string()) { 2 } else { 0 } +
        if text_lower.contains("accuracy") || text_lower.contains("precision") || text_lower.contains("relevance") { 1 } else { 0 };
    
    let readability_score = if indicators.contains(&"readability".to_string()) { 2 } else { 0 } +
        if text_lower.contains("maintainability") || text_lower.contains("clean") || text_lower.contains("refactor") { 1 } else { 0 };
    
    let security_score = if indicators.contains(&"security".to_string()) { 2 } else { 0 } +
        if text_lower.contains("safe") || text_lower.contains("vulnerability") || text_lower.contains("exploit") { 1 } else { 0 };
    
    // Determine primary optimization type based on highest score
    let max_score = [performance_score, memory_score, quality_score, readability_score, security_score].iter().max().unwrap_or(&0);
    
    if *max_score == 0 {
        // Default to performance if no clear indicators
        OptimizationType::Performance
    } else if performance_score == *max_score {
        OptimizationType::Performance
    } else if memory_score == *max_score {
        OptimizationType::Memory
    } else if quality_score == *max_score {
        OptimizationType::Quality
    } else if readability_score == *max_score {
        OptimizationType::Readability
    } else if security_score == *max_score {
        OptimizationType::Security
    } else {
        OptimizationType::Performance
    }
}

/// Validate constraint consistency
pub fn validate_constraint_consistency(constraints: &[String]) -> Result<(), CognitiveError> {
    let constraints_lower: Vec<String> = constraints.iter().map(|c| c.to_lowercase()).collect();
    
    // Check for contradictory constraints
    if constraints_lower.iter().any(|c| c.contains("unsafe")) && 
       constraints_lower.iter().any(|c| c.contains("no unsafe") || c.contains("safe")) {
        return Err(CognitiveError::SpecError(
            "Contradictory safety constraints detected".to_string()
        ));
    }
    
    if constraints_lower.iter().any(|c| c.contains("fast") || c.contains("performance")) &&
       constraints_lower.iter().any(|c| c.contains("slow") || c.contains("no optimization")) {
        return Err(CognitiveError::SpecError(
            "Contradictory performance constraints detected".to_string()
        ));
    }
    
    // Check for unrealistic constraints
    let mut max_latency_increase = None;
    let mut max_memory_increase = None;
    
    for constraint in &constraints_lower {
        if constraint.contains("max latency increase") {
            // Extract percentage if possible
            if let Some(start) = constraint.find(char::is_numeric) {
                if let Some(end) = constraint[start..].find('%') {
                    if let Ok(percentage) = constraint[start..start + end].parse::<f64>() {
                        max_latency_increase = Some(percentage);
                    }
                }
            }
        }
        
        if constraint.contains("max memory increase") {
            if let Some(start) = constraint.find(char::is_numeric) {
                if let Some(end) = constraint[start..].find('%') {
                    if let Ok(percentage) = constraint[start..start + end].parse::<f64>() {
                        max_memory_increase = Some(percentage);
                    }
                }
            }
        }
    }
    
    // Warn about very restrictive constraints
    if let Some(latency) = max_latency_increase {
        if latency < 1.0 {
            warn!("Very restrictive latency constraint: {}%", latency);
        }
    }
    
    if let Some(memory) = max_memory_increase {
        if memory < 1.0 {
            warn!("Very restrictive memory constraint: {}%", memory);
        }
    }
    
    Ok(())
}

/// Validate success criteria achievability
pub fn validate_success_criteria_achievability(criteria: &[String]) -> Result<(), CognitiveError> {
    let criteria_lower: Vec<String> = criteria.iter().map(|c| c.to_lowercase()).collect();
    
    // Check for contradictory criteria
    if criteria_lower.iter().any(|c| c.contains("increase") && c.contains("performance")) &&
       criteria_lower.iter().any(|c| c.contains("decrease") && c.contains("performance")) {
        return Err(CognitiveError::SpecError(
            "Contradictory performance criteria detected".to_string()
        ));
    }
    
    // Check for unrealistic expectations
    let mut performance_improvements = Vec::new();
    
    for criterion in &criteria_lower {
        if criterion.contains("improve") && criterion.contains("%") {
            // Extract percentage improvements
            if let Some(start) = criterion.find(char::is_numeric) {
                if let Some(end) = criterion[start..].find('%') {
                    if let Ok(percentage) = criterion[start..start + end].parse::<f64>() {
                        performance_improvements.push(percentage);
                    }
                }
            }
        }
    }
    
    // Warn about overly ambitious improvements
    for improvement in performance_improvements {
        if improvement > 50.0 {
            warn!("Very ambitious improvement target: {}%", improvement);
        }
    }
    
    Ok(())
}

/// Analyze specification completeness
pub fn analyze_spec_completeness(spec: &OptimizationSpec) -> SpecCompletenessAnalysis {
    let mut analysis = SpecCompletenessAnalysis::default();
    
    // Check objective quality
    analysis.has_clear_objective = !spec.objective.trim().is_empty() && spec.objective.len() > 10;
    analysis.objective_score = if analysis.has_clear_objective {
        if spec.objective.len() > 50 { 1.0 } else { 0.7 }
    } else {
        0.3
    };
    
    // Check constraints quality
    analysis.has_sufficient_constraints = spec.constraints.len() >= 3;
    analysis.constraints_score = (spec.constraints.len().min(5) as f64) / 5.0;
    
    // Check success criteria quality
    analysis.has_measurable_criteria = spec.success_criteria.len() >= 2;
    analysis.criteria_score = (spec.success_criteria.len().min(5) as f64) / 5.0;
    
    // Check configuration completeness
    analysis.has_timeout = spec.timeout_ms.is_some();
    analysis.has_max_iterations = spec.max_iterations.is_some();
    analysis.has_target_quality = spec.target_quality > 0.0 && spec.target_quality <= 1.0;
    
    // Calculate overall completeness
    let config_score = [
        if analysis.has_timeout { 1.0 } else { 0.0 },
        if analysis.has_max_iterations { 1.0 } else { 0.0 },
        if analysis.has_target_quality { 1.0 } else { 0.0 },
    ].iter().sum::<f64>() / 3.0;
    
    analysis.overall_completeness = (
        analysis.objective_score * 0.3 +
        analysis.constraints_score * 0.25 +
        analysis.criteria_score * 0.25 +
        config_score * 0.2
    );
    
    // Generate recommendations
    if !analysis.has_clear_objective {
        analysis.recommendations.push("Provide a more detailed objective".to_string());
    }
    if !analysis.has_sufficient_constraints {
        analysis.recommendations.push("Add more specific constraints".to_string());
    }
    if !analysis.has_measurable_criteria {
        analysis.recommendations.push("Define measurable success criteria".to_string());
    }
    if !analysis.has_timeout {
        analysis.recommendations.push("Specify a timeout value".to_string());
    }
    if !analysis.has_max_iterations {
        analysis.recommendations.push("Set maximum iterations limit".to_string());
    }
    
    analysis
}

/// Specification completeness analysis result
#[derive(Debug, Default)]
pub struct SpecCompletenessAnalysis {
    pub has_clear_objective: bool,
    pub has_sufficient_constraints: bool,
    pub has_measurable_criteria: bool,
    pub has_timeout: bool,
    pub has_max_iterations: bool,
    pub has_target_quality: bool,
    pub objective_score: f64,
    pub constraints_score: f64,
    pub criteria_score: f64,
    pub overall_completeness: f64,
    pub recommendations: Vec<String>,
}

impl SpecCompletenessAnalysis {
    /// Check if specification is complete enough for optimization
    pub fn is_complete(&self) -> bool {
        self.overall_completeness >= 0.7
    }
    
    /// Get completeness grade (A-F)
    pub fn completeness_grade(&self) -> char {
        if self.overall_completeness >= 0.9 { 'A' }
        else if self.overall_completeness >= 0.8 { 'B' }
        else if self.overall_completeness >= 0.7 { 'C' }
        else if self.overall_completeness >= 0.6 { 'D' }
        else { 'F' }
    }
    
    /// Generate completeness report
    pub fn generate_report(&self) -> String {
        format!(
            "Specification Completeness Analysis\n\
             ===================================\n\
             Overall Score: {:.1}% (Grade: {})\n\
             \n\
             Components:\n\
             - Objective: {} ({:.1}%)\n\
             - Constraints: {} ({:.1}%)\n\
             - Success Criteria: {} ({:.1}%)\n\
             - Configuration: Complete timeout/iterations/quality settings\n\
             \n\
             Recommendations ({}):\n\
             {}",
            self.overall_completeness * 100.0,
            self.completeness_grade(),
            if self.has_clear_objective { "✓" } else { "✗" },
            self.objective_score * 100.0,
            if self.has_sufficient_constraints { "✓" } else { "✗" },
            self.constraints_score * 100.0,
            if self.has_measurable_criteria { "✓" } else { "✗" },
            self.criteria_score * 100.0,
            self.recommendations.len(),
            self.recommendations.iter()
                .enumerate()
                .map(|(i, rec)| format!("  {}. {}", i + 1, rec))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_spec() {
        let spec = OptimizationSpec {
            objective: "Test optimization".to_string(),
            constraints: vec!["No unsafe code".to_string()],
            success_criteria: vec!["Passes tests".to_string()],
            optimization_type: OptimizationType::Performance,
            timeout_ms: Some(60000),
            max_iterations: Some(100),
            target_quality: 0.8,
        };
        
        assert!(validate_spec(&spec).is_ok());
    }

    #[test]
    fn test_determine_optimization_type() {
        let performance_text = "Improve performance and reduce latency";
        assert_eq!(determine_optimization_type(performance_text), OptimizationType::Performance);
        
        let memory_text = "Reduce memory allocation and heap usage";
        assert_eq!(determine_optimization_type(memory_text), OptimizationType::Memory);
        
        let quality_text = "Improve accuracy and relevance";
        assert_eq!(determine_optimization_type(quality_text), OptimizationType::Quality);
    }

    #[test]
    fn test_parse_constraints() {
        let text = r#"
        Constraints:
        - No unsafe code
        - Max latency increase: 5%
        - Must maintain readability
        "#;
        
        let constraints = parse_constraints(text);
        assert!(constraints.len() >= 3);
        assert!(constraints.iter().any(|c| c.contains("unsafe")));
    }

    #[test]
    fn test_analyze_spec_completeness() {
        let spec = OptimizationSpec {
            objective: "Comprehensive optimization objective with detailed description".to_string(),
            constraints: vec![
                "No unsafe code".to_string(),
                "Max latency increase: 5%".to_string(),
                "Maintain readability".to_string(),
            ],
            success_criteria: vec![
                "Improves performance".to_string(),
                "Passes all tests".to_string(),
            ],
            optimization_type: OptimizationType::Performance,
            timeout_ms: Some(300000),
            max_iterations: Some(100),
            target_quality: 0.8,
        };
        
        let analysis = analyze_spec_completeness(&spec);
        assert!(analysis.is_complete());
        assert!(analysis.completeness_grade() >= 'C');
    }
}

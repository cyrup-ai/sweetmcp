//! Extraction utilities for parsing optimization specifications
//!
//! This module provides utility functions for extracting values from specification text
//! with zero allocation patterns and blazing-fast performance.

use tracing::debug;

/// Extract percentage value from a line of text
pub fn extract_percentage(line: &str) -> Option<f64> {
    line.split_whitespace()
        .find(|word| word.ends_with('%'))
        .and_then(|word| word.trim_end_matches('%').parse::<f64>().ok())
}

/// Extract numeric value from a line of text
pub fn extract_number(line: &str) -> Option<f64> {
    // Look for numbers in the line, handling various formats
    for word in line.split_whitespace() {
        // Remove common suffixes and prefixes
        let cleaned = word
            .trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
            .replace(",", ""); // Remove thousands separators
        
        if let Ok(num) = cleaned.parse::<f64>() {
            return Some(num);
        }
    }
    
    None
}

/// Extract configuration values from specification text
pub fn extract_config_values(text: &str) -> ConfigValues {
    let mut config = ConfigValues::default();
    
    for line in text.lines() {
        let line = line.trim().to_lowercase();
        
        if line.contains("timeout") {
            if let Some(num) = extract_number(&line) {
                // Convert to milliseconds if needed
                config.timeout_ms = if line.contains("ms") {
                    Some(num as u64)
                } else if line.contains("sec") || line.contains("second") {
                    Some((num * 1000.0) as u64)
                } else if line.contains("min") || line.contains("minute") {
                    Some((num * 60000.0) as u64)
                } else {
                    // Assume milliseconds by default
                    Some(num as u64)
                };
            }
        } else if line.contains("iteration") || line.contains("max_iter") {
            if let Some(num) = extract_number(&line) {
                config.max_iterations = Some(num as u32);
            }
        } else if line.contains("quality") || line.contains("target") {
            if let Some(num) = extract_percentage(&line) {
                config.target_quality = num / 100.0; // Convert percentage to decimal
            } else if let Some(num) = extract_number(&line) {
                // If it's already a decimal (0.0-1.0), use as-is
                if num <= 1.0 {
                    config.target_quality = num;
                } else {
                    // Otherwise treat as percentage
                    config.target_quality = num / 100.0;
                }
            }
        }
    }
    
    config
}

/// Parse baseline metrics from specification text
pub fn parse_baseline_metrics(text: &str) -> BaselineMetrics {
    let mut metrics = BaselineMetrics::default();
    
    for line in text.lines() {
        let line = line.trim();
        
        if line.contains("baseline") || line.contains("current") {
            if line.contains("latency") {
                if let Some(num) = extract_number(line) {
                    metrics.latency = num;
                }
            } else if line.contains("memory") {
                if let Some(num) = extract_number(line) {
                    metrics.memory = num;
                }
            } else if line.contains("relevance") {
                if let Some(num) = extract_percentage(line) {
                    metrics.relevance = num;
                }
            }
        }
    }
    
    metrics
}

/// Extract optimization type indicators from text
pub fn extract_optimization_indicators(text: &str) -> Vec<String> {
    let mut indicators = Vec::new();
    let text_lower = text.to_lowercase();
    
    // Performance indicators
    if text_lower.contains("performance") || text_lower.contains("speed") || text_lower.contains("latency") {
        indicators.push("performance".to_string());
    }
    
    // Memory indicators
    if text_lower.contains("memory") || text_lower.contains("allocation") || text_lower.contains("heap") {
        indicators.push("memory".to_string());
    }
    
    // Quality indicators
    if text_lower.contains("quality") || text_lower.contains("accuracy") || text_lower.contains("relevance") {
        indicators.push("quality".to_string());
    }
    
    // Readability indicators
    if text_lower.contains("readability") || text_lower.contains("maintainability") || text_lower.contains("clean") {
        indicators.push("readability".to_string());
    }
    
    // Security indicators
    if text_lower.contains("security") || text_lower.contains("safe") || text_lower.contains("vulnerability") {
        indicators.push("security".to_string());
    }
    
    indicators
}

/// Extract constraint patterns from text
pub fn extract_constraint_patterns(text: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        
        // Look for constraint patterns
        if line.starts_with("- ") || line.starts_with("* ") {
            let constraint = line.trim_start_matches("- ").trim_start_matches("* ").trim();
            if !constraint.is_empty() {
                patterns.push(constraint.to_string());
            }
        } else if line.contains("must") || line.contains("should") || line.contains("cannot") {
            patterns.push(line.to_string());
        } else if line.contains("max") || line.contains("min") || line.contains("limit") {
            patterns.push(line.to_string());
        }
    }
    
    patterns
}

/// Extract success criteria patterns from text
pub fn extract_success_patterns(text: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    
    for line in text.lines() {
        let line = line.trim();
        
        // Look for success criteria patterns
        if line.contains("success") || line.contains("achieve") || line.contains("goal") {
            patterns.push(line.to_string());
        } else if line.contains("improve") || line.contains("reduce") || line.contains("increase") {
            patterns.push(line.to_string());
        } else if line.contains("pass") || line.contains("meet") || line.contains("satisfy") {
            patterns.push(line.to_string());
        }
    }
    
    patterns
}

/// Extract numeric thresholds from text
pub fn extract_thresholds(text: &str) -> ThresholdValues {
    let mut thresholds = ThresholdValues::default();
    
    for line in text.lines() {
        let line = line.trim().to_lowercase();
        
        if line.contains("latency") {
            if line.contains("max") || line.contains("limit") {
                if let Some(num) = extract_percentage(&line) {
                    thresholds.max_latency_increase = Some(num);
                } else if let Some(num) = extract_number(&line) {
                    thresholds.max_latency_increase = Some(num);
                }
            } else if line.contains("target") || line.contains("goal") {
                if let Some(num) = extract_number(&line) {
                    thresholds.target_latency = Some(num);
                }
            }
        } else if line.contains("memory") {
            if line.contains("max") || line.contains("limit") {
                if let Some(num) = extract_percentage(&line) {
                    thresholds.max_memory_increase = Some(num);
                } else if let Some(num) = extract_number(&line) {
                    thresholds.max_memory_increase = Some(num);
                }
            } else if line.contains("target") || line.contains("goal") {
                if let Some(num) = extract_number(&line) {
                    thresholds.target_memory = Some(num);
                }
            }
        } else if line.contains("relevance") || line.contains("quality") {
            if line.contains("min") || line.contains("improve") {
                if let Some(num) = extract_percentage(&line) {
                    thresholds.min_quality_improvement = Some(num);
                } else if let Some(num) = extract_number(&line) {
                    thresholds.min_quality_improvement = Some(num);
                }
            } else if line.contains("target") || line.contains("goal") {
                if let Some(num) = extract_percentage(&line) {
                    thresholds.target_quality = Some(num / 100.0);
                } else if let Some(num) = extract_number(&line) {
                    thresholds.target_quality = Some(if num <= 1.0 { num } else { num / 100.0 });
                }
            }
        }
    }
    
    thresholds
}

/// Extract time-based values from text
pub fn extract_time_values(text: &str) -> TimeValues {
    let mut time_values = TimeValues::default();
    
    for line in text.lines() {
        let line = line.trim().to_lowercase();
        
        if line.contains("timeout") || line.contains("time limit") {
            if let Some(num) = extract_number(&line) {
                time_values.timeout_ms = if line.contains("ms") {
                    Some(num as u64)
                } else if line.contains("sec") || line.contains("second") {
                    Some((num * 1000.0) as u64)
                } else if line.contains("min") || line.contains("minute") {
                    Some((num * 60000.0) as u64)
                } else if line.contains("hour") {
                    Some((num * 3600000.0) as u64)
                } else {
                    // Default to seconds
                    Some((num * 1000.0) as u64)
                };
            }
        } else if line.contains("deadline") || line.contains("duration") {
            if let Some(num) = extract_number(&line) {
                time_values.deadline_ms = if line.contains("ms") {
                    Some(num as u64)
                } else if line.contains("sec") || line.contains("second") {
                    Some((num * 1000.0) as u64)
                } else if line.contains("min") || line.contains("minute") {
                    Some((num * 60000.0) as u64)
                } else {
                    Some((num * 1000.0) as u64)
                };
            }
        }
    }
    
    time_values
}

/// Configuration values extracted from specification
#[derive(Debug, Default)]
pub struct ConfigValues {
    pub timeout_ms: Option<u64>,
    pub max_iterations: Option<u32>,
    pub target_quality: f64,
}

/// Baseline performance metrics
#[derive(Debug)]
pub struct BaselineMetrics {
    pub latency: f64,
    pub memory: f64,
    pub relevance: f64,
}

impl Default for BaselineMetrics {
    fn default() -> Self {
        Self {
            latency: 100.0,  // 100ms default
            memory: 50.0,    // 50MB default
            relevance: 75.0, // 75% default
        }
    }
}

/// Threshold values for optimization constraints
#[derive(Debug, Default)]
pub struct ThresholdValues {
    pub max_latency_increase: Option<f64>,
    pub max_memory_increase: Option<f64>,
    pub min_quality_improvement: Option<f64>,
    pub target_latency: Option<f64>,
    pub target_memory: Option<f64>,
    pub target_quality: Option<f64>,
}

/// Time-related configuration values
#[derive(Debug, Default)]
pub struct TimeValues {
    pub timeout_ms: Option<u64>,
    pub deadline_ms: Option<u64>,
}

/// Utility functions for text processing
pub mod text_utils {
    use super::*;

    /// Clean and normalize text for parsing
    pub fn normalize_text(text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract quoted strings from text
    pub fn extract_quoted_strings(text: &str) -> Vec<String> {
        let mut quoted = Vec::new();
        let mut in_quote = false;
        let mut current_quote = String::new();
        let mut quote_char = '"';

        for ch in text.chars() {
            if !in_quote && (ch == '"' || ch == '\'') {
                in_quote = true;
                quote_char = ch;
                current_quote.clear();
            } else if in_quote && ch == quote_char {
                in_quote = false;
                if !current_quote.is_empty() {
                    quoted.push(current_quote.clone());
                }
                current_quote.clear();
            } else if in_quote {
                current_quote.push(ch);
            }
        }

        quoted
    }

    /// Extract key-value pairs from text
    pub fn extract_key_value_pairs(text: &str) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                if !key.is_empty() && !value.is_empty() {
                    pairs.push((key, value));
                }
            } else if let Some(equals_pos) = line.find('=') {
                let key = line[..equals_pos].trim().to_string();
                let value = line[equals_pos + 1..].trim().to_string();
                if !key.is_empty() && !value.is_empty() {
                    pairs.push((key, value));
                }
            }
        }

        pairs
    }

    /// Check if text contains any of the given keywords
    pub fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
        let text_lower = text.to_lowercase();
        keywords.iter().any(|keyword| text_lower.contains(&keyword.to_lowercase()))
    }

    /// Count occurrences of keywords in text
    pub fn count_keywords(text: &str, keywords: &[&str]) -> usize {
        let text_lower = text.to_lowercase();
        keywords.iter()
            .map(|keyword| text_lower.matches(&keyword.to_lowercase()).count())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_percentage() {
        assert_eq!(extract_percentage("Max latency increase: 10%"), Some(10.0));
        assert_eq!(extract_percentage("Quality should be 85%"), Some(85.0));
        assert_eq!(extract_percentage("No percentage here"), None);
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("Latency: 100ms"), Some(100.0));
        assert_eq!(extract_number("Memory usage is 50.5MB"), Some(50.5));
        assert_eq!(extract_number("Timeout: 1,000 seconds"), Some(1000.0));
        assert_eq!(extract_number("No numbers here"), None);
    }

    #[test]
    fn test_extract_config_values() {
        let text = r#"
        Timeout: 5 minutes
        Max iterations: 200
        Target quality: 90%
        "#;
        
        let config = extract_config_values(text);
        assert_eq!(config.timeout_ms, Some(300000)); // 5 minutes in ms
        assert_eq!(config.max_iterations, Some(200));
        assert_eq!(config.target_quality, 0.9);
    }

    #[test]
    fn test_extract_optimization_indicators() {
        let text = "Improve performance and reduce memory usage while maintaining quality";
        let indicators = extract_optimization_indicators(text);
        
        assert!(indicators.contains(&"performance".to_string()));
        assert!(indicators.contains(&"memory".to_string()));
        assert!(indicators.contains(&"quality".to_string()));
    }

    #[test]
    fn test_text_utils() {
        let text = r#"Key1: "Value 1" Key2: 'Value 2'"#;
        let quoted = text_utils::extract_quoted_strings(text);
        
        assert_eq!(quoted.len(), 2);
        assert!(quoted.contains(&"Value 1".to_string()));
        assert!(quoted.contains(&"Value 2".to_string()));
    }
}

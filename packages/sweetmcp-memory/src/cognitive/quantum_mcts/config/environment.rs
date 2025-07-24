//! Environment variable loading for quantum MCTS configuration
//!
//! This module provides blazing-fast environment variable parsing with
//! validation and reasonable bounds checking for optimal performance.

use std::env;
use super::core::QuantumMCTSConfig;

/// Configuration source types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSource {
    Environment,
    File(String),
    Default,
}

/// Environment loading errors
#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentError {
    InvalidValue(String),
    MissingRequired(String),
    ParseError(String),
}

/// Environment variable loader with validation
#[derive(Debug, Clone)]
pub struct EnvironmentLoader {
    source: ConfigSource,
}

impl EnvironmentLoader {
    /// Create new environment loader
    pub fn new() -> Self {
        Self {
            source: ConfigSource::Environment,
        }
    }
    
    /// Load configuration from environment
    pub fn load(&self, config: &mut QuantumMCTSConfig) -> Result<(), EnvironmentError> {
        load_from_environment(config);
        Ok(())
    }
}

impl Default for EnvironmentLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load configuration from environment variables with validation
pub fn load_from_environment(config: &mut QuantumMCTSConfig) {
    load_parallelism_config(config);
    load_quantum_parameters(config);
    load_performance_parameters(config);
    load_system_parameters(config);
    load_debug_parameters(config);
}

/// Load parallelism and concurrency configuration
fn load_parallelism_config(config: &mut QuantumMCTSConfig) {
    if let Ok(val) = env::var("QUANTUM_MCTS_PARALLEL") {
        if let Ok(parallel) = val.parse::<usize>() {
            config.max_quantum_parallel = parallel.min(64).max(1); // Reasonable bounds
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_BATCH_SIZE") {
        if let Ok(batch_size) = val.parse::<usize>() {
            config.batch_size = batch_size.min(1000).max(1); // Reasonable bounds
        }
    }
}

/// Load quantum-specific parameters
fn load_quantum_parameters(config: &mut QuantumMCTSConfig) {
    if let Ok(val) = env::var("QUANTUM_MCTS_EXPLORATION") {
        if let Ok(exploration) = val.parse::<f64>() {
            if exploration > 0.0 && exploration <= 10.0 {
                config.quantum_exploration = exploration;
            }
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_DECOHERENCE") {
        if let Ok(threshold) = val.parse::<f64>() {
            if threshold > 0.0 && threshold <= 1.0 {
                config.decoherence_threshold = threshold;
            }
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_ENTANGLEMENT") {
        if let Ok(strength) = val.parse::<f64>() {
            if strength >= 0.0 && strength <= 1.0 {
                config.entanglement_strength = strength;
            }
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_AMPLITUDE_THRESHOLD") {
        if let Ok(threshold) = val.parse::<f64>() {
            if threshold > 0.0 && threshold <= 1.0 {
                config.amplitude_threshold = threshold;
            }
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_PHASE_RATE") {
        if let Ok(rate) = val.parse::<f64>() {
            if rate > 0.0 && rate <= 1.0 {
                config.phase_evolution_rate = rate;
            }
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_PRECISION") {
        if let Ok(precision) = val.parse::<f64>() {
            if precision > 0.0 && precision <= 1.0 {
                config.measurement_precision = precision;
            }
        }
    }
}

/// Load performance and optimization parameters
fn load_performance_parameters(config: &mut QuantumMCTSConfig) {
    if let Ok(val) = env::var("QUANTUM_MCTS_ITERATIONS") {
        if let Ok(iterations) = val.parse::<u32>() {
            config.recursive_iterations = iterations.min(20).max(1); // Reasonable bounds
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_TIMEOUT_MS") {
        if let Ok(timeout) = val.parse::<u64>() {
            config.simulation_timeout_ms = timeout.min(600_000).max(1_000); // 1s to 10min bounds
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_ERROR_CORRECTION") {
        config.enable_error_correction = parse_boolean(&val);
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_MAX_ENTANGLEMENTS") {
        if let Ok(max_entanglements) = val.parse::<usize>() {
            config.max_entanglements_per_node = max_entanglements.min(100).max(1); // Reasonable bounds
        }
    }
    
    if let Ok(val) = env::var("QUANTUM_MCTS_PRUNING_THRESHOLD") {
        if let Ok(threshold) = val.parse::<f64>() {
            if threshold >= 0.0 && threshold <= 1.0 {
                config.pruning_threshold = threshold;
            }
        }
    }
}

/// Load system resource parameters
fn load_system_parameters(config: &mut QuantumMCTSConfig) {
    if let Ok(val) = env::var("QUANTUM_MCTS_MAX_TREE_SIZE") {
        if let Ok(size) = val.parse::<usize>() {
            config.max_tree_size = size.min(10_000_000).max(1_000); // 1k to 10M bounds
        }
    }
}

/// Load debug and development parameters
fn load_debug_parameters(config: &mut QuantumMCTSConfig) {
    if let Ok(val) = env::var("QUANTUM_MCTS_DEBUG") {
        config.debug_mode = parse_boolean(&val);
    }
}

/// Parse boolean values from environment variables
fn parse_boolean(value: &str) -> bool {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => true,
        "false" | "0" | "no" | "off" | "disabled" => false,
        _ => false, // Default to false for invalid values
    }
}

/// Get all supported environment variables with descriptions
pub fn get_supported_variables() -> Vec<(String, String, String)> {
    vec![
        // (Variable name, Description, Example)
        ("QUANTUM_MCTS_PARALLEL".to_string(), 
         "Maximum parallel quantum circuits".to_string(), 
         "8".to_string()),
        ("QUANTUM_MCTS_BATCH_SIZE".to_string(), 
         "Batch size for parallel operations".to_string(), 
         "20".to_string()),
        ("QUANTUM_MCTS_EXPLORATION".to_string(), 
         "Quantum exploration factor".to_string(), 
         "2.0".to_string()),
        ("QUANTUM_MCTS_DECOHERENCE".to_string(), 
         "Decoherence threshold".to_string(), 
         "0.1".to_string()),
        ("QUANTUM_MCTS_ENTANGLEMENT".to_string(), 
         "Entanglement strength".to_string(), 
         "0.7".to_string()),
        ("QUANTUM_MCTS_AMPLITUDE_THRESHOLD".to_string(), 
         "Minimum amplitude threshold".to_string(), 
         "0.01".to_string()),
        ("QUANTUM_MCTS_PHASE_RATE".to_string(), 
         "Phase evolution rate".to_string(), 
         "0.1".to_string()),
        ("QUANTUM_MCTS_PRECISION".to_string(), 
         "Measurement precision".to_string(), 
         "1e-10".to_string()),
        ("QUANTUM_MCTS_ITERATIONS".to_string(), 
         "Recursive improvement iterations".to_string(), 
         "3".to_string()),
        ("QUANTUM_MCTS_TIMEOUT_MS".to_string(), 
         "Simulation timeout in milliseconds".to_string(), 
         "30000".to_string()),
        ("QUANTUM_MCTS_ERROR_CORRECTION".to_string(), 
         "Enable quantum error correction".to_string(), 
         "true".to_string()),
        ("QUANTUM_MCTS_MAX_ENTANGLEMENTS".to_string(), 
         "Maximum entanglements per node".to_string(), 
         "10".to_string()),
        ("QUANTUM_MCTS_PRUNING_THRESHOLD".to_string(), 
         "Threshold for pruning weak entanglements".to_string(), 
         "0.1".to_string()),
        ("QUANTUM_MCTS_MAX_TREE_SIZE".to_string(), 
         "Maximum tree size in nodes".to_string(), 
         "100000".to_string()),
        ("QUANTUM_MCTS_DEBUG".to_string(), 
         "Enable debug mode".to_string(), 
         "false".to_string()),
    ]
}

/// Print help text for environment variables
pub fn print_environment_help() {
    println!("Quantum MCTS Configuration Environment Variables:");
    println!("==============================================");
    
    for (var, description, example) in get_supported_variables() {
        println!("{:<35} - {}", var, description);
        println!("{:<35}   Example: {}", "", example);
        println!();
    }
}

/// Load configuration from a .env file format
pub fn load_from_file(config: &mut QuantumMCTSConfig, file_path: &str) -> Result<(), String> {
    use std::fs;
    
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read config file {}: {}", file_path, e))?;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse key=value pairs
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            
            // Temporarily set environment variable
            env::set_var(key, value);
        }
    }
    
    // Load from the temporarily set environment variables
    load_from_environment(config);
    
    Ok(())
}

/// Save current configuration to a .env file format
pub fn save_to_file(config: &QuantumMCTSConfig, file_path: &str) -> Result<(), String> {
    use std::fs;
    
    let mut content = String::new();
    content.push_str("# Quantum MCTS Configuration\n");
    content.push_str("# Generated automatically\n\n");
    
    content.push_str(&format!("QUANTUM_MCTS_PARALLEL={}\n", config.max_quantum_parallel));
    content.push_str(&format!("QUANTUM_MCTS_BATCH_SIZE={}\n", config.batch_size));
    content.push_str(&format!("QUANTUM_MCTS_EXPLORATION={}\n", config.quantum_exploration));
    content.push_str(&format!("QUANTUM_MCTS_DECOHERENCE={}\n", config.decoherence_threshold));
    content.push_str(&format!("QUANTUM_MCTS_ENTANGLEMENT={}\n", config.entanglement_strength));
    content.push_str(&format!("QUANTUM_MCTS_AMPLITUDE_THRESHOLD={}\n", config.amplitude_threshold));
    content.push_str(&format!("QUANTUM_MCTS_PHASE_RATE={}\n", config.phase_evolution_rate));
    content.push_str(&format!("QUANTUM_MCTS_PRECISION={}\n", config.measurement_precision));
    content.push_str(&format!("QUANTUM_MCTS_ITERATIONS={}\n", config.recursive_iterations));
    content.push_str(&format!("QUANTUM_MCTS_TIMEOUT_MS={}\n", config.simulation_timeout_ms));
    content.push_str(&format!("QUANTUM_MCTS_ERROR_CORRECTION={}\n", config.enable_error_correction));
    content.push_str(&format!("QUANTUM_MCTS_MAX_ENTANGLEMENTS={}\n", config.max_entanglements_per_node));
    content.push_str(&format!("QUANTUM_MCTS_PRUNING_THRESHOLD={}\n", config.pruning_threshold));
    content.push_str(&format!("QUANTUM_MCTS_MAX_TREE_SIZE={}\n", config.max_tree_size));
    content.push_str(&format!("QUANTUM_MCTS_DEBUG={}\n", config.debug_mode));
    
    fs::write(file_path, content)
        .map_err(|e| format!("Failed to write config file {}: {}", file_path, e))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_boolean_parsing() {
        assert!(parse_boolean("true"));
        assert!(parse_boolean("TRUE"));
        assert!(parse_boolean("1"));
        assert!(parse_boolean("yes"));
        assert!(parse_boolean("on"));
        assert!(parse_boolean("enabled"));
        
        assert!(!parse_boolean("false"));
        assert!(!parse_boolean("FALSE"));
        assert!(!parse_boolean("0"));
        assert!(!parse_boolean("no"));
        assert!(!parse_boolean("off"));
        assert!(!parse_boolean("disabled"));
        assert!(!parse_boolean("invalid"));
    }
    
    #[test]
    fn test_environment_loading() {
        let mut config = QuantumMCTSConfig::default();
        
        // Set some test environment variables
        env::set_var("QUANTUM_MCTS_PARALLEL", "8");
        env::set_var("QUANTUM_MCTS_EXPLORATION", "3.0");
        env::set_var("QUANTUM_MCTS_ERROR_CORRECTION", "false");
        
        load_from_environment(&mut config);
        
        assert_eq!(config.max_quantum_parallel, 8);
        assert!((config.quantum_exploration - 3.0).abs() < f64::EPSILON);
        assert!(!config.enable_error_correction);
        
        // Clean up
        env::remove_var("QUANTUM_MCTS_PARALLEL");
        env::remove_var("QUANTUM_MCTS_EXPLORATION");
        env::remove_var("QUANTUM_MCTS_ERROR_CORRECTION");
    }
    
    #[test]
    fn test_invalid_environment_values() {
        let mut config = QuantumMCTSConfig::default();
        let original_exploration = config.quantum_exploration;
        
        // Set invalid values
        env::set_var("QUANTUM_MCTS_EXPLORATION", "invalid");
        env::set_var("QUANTUM_MCTS_DECOHERENCE", "2.0"); // > 1.0
        
        load_from_environment(&mut config);
        
        // Should keep original values for invalid inputs
        assert_eq!(config.quantum_exploration, original_exploration);
        assert_eq!(config.decoherence_threshold, QuantumMCTSConfig::default().decoherence_threshold);
        
        // Clean up
        env::remove_var("QUANTUM_MCTS_EXPLORATION");
        env::remove_var("QUANTUM_MCTS_DECOHERENCE");
    }
    
    #[test]
    fn test_supported_variables() {
        let vars = get_supported_variables();
        assert!(!vars.is_empty());
        
        // Check that all variables have proper format
        for (var, desc, example) in vars {
            assert!(var.starts_with("QUANTUM_MCTS_"));
            assert!(!desc.is_empty());
            assert!(!example.is_empty());
        }
    }
}
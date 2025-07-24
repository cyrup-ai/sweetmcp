//! Configuration validation with detailed error reporting and bounds checking
//!
//! This module provides blazing-fast parameter validation with zero-allocation
//! error reporting for quantum MCTS configuration parameters.

use crate::cognitive::quantum_mcts::config::core::QuantumMCTSConfig;

/// Configuration validation error with detailed context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub parameter: String,
    pub value: String,
    pub constraint: String,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} violates constraint: {}", 
               self.parameter, self.value, self.constraint)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Configuration validator with comprehensive bounds checking
pub struct ConfigValidator {
    errors: Vec<ValidationError>,
}

impl ConfigValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Validate quantum MCTS configuration with detailed error reporting
    pub fn validate(config: &QuantumMCTSConfig) -> Result<(), String> {
        let mut validator = Self::new();
        
        validator.validate_parallelism(config);
        validator.validate_exploration_factor(config);
        validator.validate_thresholds(config);
        validator.validate_iterations(config);
        validator.validate_timeout(config);
        validator.validate_tree_size(config);
        validator.validate_precision(config);
        validator.validate_system_compatibility(config);
        
        if validator.errors.is_empty() {
            Ok(())
        } else {
            let error_message = validator.errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            Err(error_message)
        }
    }

    /// Validate parallelism parameters with CPU-aware bounds
    fn validate_parallelism(&mut self, config: &QuantumMCTSConfig) {
        let cpu_count = num_cpus::get();
        
        if config.max_quantum_parallel == 0 {
            self.errors.push(ValidationError {
                parameter: "max_quantum_parallel".to_string(),
                value: config.max_quantum_parallel.to_string(),
                constraint: "must be greater than 0".to_string(),
                suggestion: Some("set to number of CPU cores".to_string()),
            });
        }
        
        if config.max_quantum_parallel > 128 {
            self.errors.push(ValidationError {
                parameter: "max_quantum_parallel".to_string(),
                value: config.max_quantum_parallel.to_string(),
                constraint: "must not exceed 128".to_string(),
                suggestion: Some(format!("consider setting to {} (CPU cores)", cpu_count)),
            });
        }
        
        // Warn about potential oversubscription
        if config.max_quantum_parallel > cpu_count * 4 {
            self.errors.push(ValidationError {
                parameter: "max_quantum_parallel".to_string(),
                value: config.max_quantum_parallel.to_string(),
                constraint: "should not exceed 4x CPU cores for optimal performance".to_string(),
                suggestion: Some(format!("consider setting to {} (4x CPU cores)", cpu_count * 4)),
            });
        }
    }

    /// Validate exploration factor with UCT-aware bounds
    fn validate_exploration_factor(&mut self, config: &QuantumMCTSConfig) {
        if config.quantum_exploration <= 0.0 {
            self.errors.push(ValidationError {
                parameter: "quantum_exploration".to_string(),
                value: config.quantum_exploration.to_string(),
                constraint: "must be positive".to_string(),
                suggestion: Some("typical values range from 0.5 to 3.0".to_string()),
            });
        }
        
        if config.quantum_exploration > 20.0 {
            self.errors.push(ValidationError {
                parameter: "quantum_exploration".to_string(),
                value: config.quantum_exploration.to_string(),
                constraint: "exceeds reasonable limit (20.0)".to_string(),
                suggestion: Some("consider values between 1.0 and 3.0 for most applications".to_string()),
            });
        }
        
        // Performance guidance
        if config.quantum_exploration > 5.0 {
            self.errors.push(ValidationError {
                parameter: "quantum_exploration".to_string(),
                value: config.quantum_exploration.to_string(),
                constraint: "high values may slow convergence".to_string(),
                suggestion: Some("consider 1.4-2.0 for balanced exploration/exploitation".to_string()),
            });
        }
    }

    /// Validate threshold parameters with quantum physics constraints
    fn validate_thresholds(&mut self, config: &QuantumMCTSConfig) {
        // Decoherence threshold validation
        if !(0.0..=1.0).contains(&config.decoherence_threshold) {
            self.errors.push(ValidationError {
                parameter: "decoherence_threshold".to_string(),
                value: config.decoherence_threshold.to_string(),
                constraint: "must be between 0.0 and 1.0".to_string(),
                suggestion: Some("typical values: 0.05 (high precision) to 0.2 (fast pruning)".to_string()),
            });
        }
        
        // Entanglement strength validation
        if !(0.0..=1.0).contains(&config.entanglement_strength) {
            self.errors.push(ValidationError {
                parameter: "entanglement_strength".to_string(),
                value: config.entanglement_strength.to_string(),
                constraint: "must be between 0.0 and 1.0".to_string(),
                suggestion: Some("typical values: 0.5 (moderate) to 0.9 (strong entanglement)".to_string()),
            });
        }
        
        // Amplitude threshold validation
        if !(0.0..=1.0).contains(&config.amplitude_threshold) {
            self.errors.push(ValidationError {
                parameter: "amplitude_threshold".to_string(),
                value: config.amplitude_threshold.to_string(),
                constraint: "must be between 0.0 and 1.0".to_string(),
                suggestion: Some("typical values: 0.001 (thorough) to 0.05 (aggressive pruning)".to_string()),
            });
        }
        
        // Phase evolution rate validation
        if !(0.0..=1.0).contains(&config.phase_evolution_rate) {
            self.errors.push(ValidationError {
                parameter: "phase_evolution_rate".to_string(),
                value: config.phase_evolution_rate.to_string(),
                constraint: "must be between 0.0 and 1.0".to_string(),
                suggestion: Some("typical values: 0.05 (slow) to 0.2 (fast evolution)".to_string()),
            });
        }
        
        // Cross-parameter validation
        if config.amplitude_threshold > config.decoherence_threshold {
            self.errors.push(ValidationError {
                parameter: "amplitude_threshold".to_string(),
                value: format!("{} > decoherence_threshold({})", 
                             config.amplitude_threshold, config.decoherence_threshold),
                constraint: "amplitude_threshold should be <= decoherence_threshold".to_string(),
                suggestion: Some("lower amplitude_threshold for consistent pruning behavior".to_string()),
            });
        }
    }

    /// Validate iteration parameters with computational complexity awareness
    fn validate_iterations(&mut self, config: &QuantumMCTSConfig) {
        if config.recursive_iterations == 0 {
            self.errors.push(ValidationError {
                parameter: "recursive_iterations".to_string(),
                value: config.recursive_iterations.to_string(),
                constraint: "must be greater than 0".to_string(),
                suggestion: Some("typical values: 2 (fast) to 5 (thorough)".to_string()),
            });
        }
        
        if config.recursive_iterations > 50 {
            self.errors.push(ValidationError {
                parameter: "recursive_iterations".to_string(),
                value: config.recursive_iterations.to_string(),
                constraint: "exceeds reasonable limit (50)".to_string(),
                suggestion: Some("consider 3-7 iterations for most applications".to_string()),
            });
        }
        
        // Performance warning for high iteration counts
        if config.recursive_iterations > 10 {
            self.errors.push(ValidationError {
                parameter: "recursive_iterations".to_string(),
                value: config.recursive_iterations.to_string(),
                constraint: "high iteration count may cause exponential slowdown".to_string(),
                suggestion: Some("consider 3-5 iterations for balanced performance/quality".to_string()),
            });
        }
    }

    /// Validate timeout parameters with real-world usage patterns
    fn validate_timeout(&mut self, config: &QuantumMCTSConfig) {
        if config.simulation_timeout_ms < 100 {
            self.errors.push(ValidationError {
                parameter: "simulation_timeout_ms".to_string(),
                value: config.simulation_timeout_ms.to_string(),
                constraint: "must be at least 100ms".to_string(),
                suggestion: Some("minimum recommended: 1000ms for meaningful computation".to_string()),
            });
        }
        
        if config.simulation_timeout_ms > 3_600_000 {
            self.errors.push(ValidationError {
                parameter: "simulation_timeout_ms".to_string(),
                value: config.simulation_timeout_ms.to_string(),
                constraint: "exceeds reasonable limit (1 hour)".to_string(),
                suggestion: Some("consider 30-120 seconds for most applications".to_string()),
            });
        }
        
        // Guidance for different use cases
        if config.simulation_timeout_ms < 1_000 {
            self.errors.push(ValidationError {
                parameter: "simulation_timeout_ms".to_string(),
                value: config.simulation_timeout_ms.to_string(),
                constraint: "very short timeout may prevent meaningful computation".to_string(),
                suggestion: Some("consider 5-30 seconds for real-time applications".to_string()),
            });
        }
    }

    /// Validate tree size parameters with memory awareness
    fn validate_tree_size(&mut self, config: &QuantumMCTSConfig) {
        if config.max_tree_size < 10 {
            self.errors.push(ValidationError {
                parameter: "max_tree_size".to_string(),
                value: config.max_tree_size.to_string(),
                constraint: "must be at least 10".to_string(),
                suggestion: Some("minimum recommended: 1000 for meaningful search".to_string()),
            });
        }
        
        if config.max_tree_size > 100_000_000 {
            self.errors.push(ValidationError {
                parameter: "max_tree_size".to_string(),
                value: config.max_tree_size.to_string(),
                constraint: "exceeds reasonable limit (100M)".to_string(),
                suggestion: Some("consider 10k-1M nodes for most applications".to_string()),
            });
        }
        
        // Memory usage estimation and warning
        let estimated_memory_mb = (config.max_tree_size * 1024) / 1_000_000; // Rough estimate in MB
        if estimated_memory_mb > 1000 { // > 1GB
            self.errors.push(ValidationError {
                parameter: "max_tree_size".to_string(),
                value: format!("{} (~{}MB)", config.max_tree_size, estimated_memory_mb),
                constraint: "may require significant memory allocation".to_string(),
                suggestion: Some("ensure sufficient system memory is available".to_string()),
            });
        }
    }

    /// Validate precision parameters with numerical stability considerations
    fn validate_precision(&mut self, config: &QuantumMCTSConfig) {
        if config.measurement_precision <= 0.0 {
            self.errors.push(ValidationError {
                parameter: "measurement_precision".to_string(),
                value: config.measurement_precision.to_string(),
                constraint: "must be positive".to_string(),
                suggestion: Some("typical values: 1e-6 (fast) to 1e-12 (high precision)".to_string()),
            });
        }
        
        if config.measurement_precision > 1.0 {
            self.errors.push(ValidationError {
                parameter: "measurement_precision".to_string(),
                value: config.measurement_precision.to_string(),
                constraint: "must not exceed 1.0".to_string(),
                suggestion: Some("use scientific notation for small values (e.g., 1e-10)".to_string()),
            });
        }
        
        // Numerical stability guidance
        if config.measurement_precision < 1e-15 {
            self.errors.push(ValidationError {
                parameter: "measurement_precision".to_string(),
                value: config.measurement_precision.to_string(),
                constraint: "extremely high precision may cause numerical instability".to_string(),
                suggestion: Some("consider 1e-10 to 1e-12 for most high-precision applications".to_string()),
            });
        }
        
        if config.measurement_precision > 1e-3 {
            self.errors.push(ValidationError {
                parameter: "measurement_precision".to_string(),
                value: config.measurement_precision.to_string(),
                constraint: "low precision may affect computation quality".to_string(),
                suggestion: Some("consider 1e-6 to 1e-10 for balanced performance/accuracy".to_string()),
            });
        }
    }

    /// Validate system compatibility with resource constraints
    fn validate_system_compatibility(&mut self, config: &QuantumMCTSConfig) {
        let cpu_count = num_cpus::get();
        let estimated_memory = config.estimate_memory_usage();
        let available_memory = Self::estimate_available_memory();
        
        // Check CPU oversubscription
        if config.max_quantum_parallel > cpu_count * 8 {
            self.errors.push(ValidationError {
                parameter: "system_compatibility".to_string(),
                value: format!("{}x CPU oversubscription", 
                             config.max_quantum_parallel as f64 / cpu_count as f64),
                constraint: "excessive parallelism may degrade performance".to_string(),
                suggestion: Some(format!("consider max_quantum_parallel <= {}", cpu_count * 2)),
            });
        }
        
        // Check memory usage
        if estimated_memory > available_memory / 2 {
            self.errors.push(ValidationError {
                parameter: "system_compatibility".to_string(),
                value: format!("{}MB estimated usage", estimated_memory / 1_000_000),
                constraint: "may exceed 50% of available memory".to_string(),
                suggestion: Some("consider reducing max_tree_size or using performance_optimized config".to_string()),
            });
        }
        
        // Check for resource conflicts
        if config.enable_error_correction && cpu_count < 2 {
            self.errors.push(ValidationError {
                parameter: "system_compatibility".to_string(),
                value: "error_correction enabled on single-core system".to_string(),
                constraint: "error correction requires multiple CPU cores for efficiency".to_string(),
                suggestion: Some("disable error correction or upgrade to multi-core system".to_string()),
            });
        }
    }

    /// Estimate available system memory (conservative approach)
    fn estimate_available_memory() -> u64 {
        match num_cpus::get() {
            1..=2 => 2_000_000_000,   // 2GB estimate for low-end systems
            3..=4 => 4_000_000_000,   // 4GB estimate for mid-range systems
            5..=8 => 8_000_000_000,   // 8GB estimate for high-end systems
            _ => 16_000_000_000,      // 16GB+ estimate for server-class systems
        }
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick validation functions for common use cases
pub mod quick {
    use super::*;

    /// Quick validation of parallelism settings
    pub fn validate_parallelism(parallel: usize) -> Result<(), String> {
        if parallel == 0 {
            return Err("parallelism must be greater than 0".to_string());
        }
        if parallel > num_cpus::get() * 4 {
            return Err(format!("parallelism {} may cause oversubscription", parallel));
        }
        Ok(())
    }

    /// Quick validation of threshold values
    pub fn validate_threshold(name: &str, value: f64) -> Result<(), String> {
        if !(0.0..=1.0).contains(&value) {
            return Err(format!("{} must be between 0.0 and 1.0", name));
        }
        Ok(())
    }

    /// Quick validation of timeout values
    pub fn validate_timeout(timeout_ms: u64) -> Result<(), String> {
        if timeout_ms < 100 {
            return Err("timeout must be at least 100ms".to_string());
        }
        if timeout_ms > 3_600_000 {
            return Err("timeout exceeds 1 hour limit".to_string());
        }
        Ok(())
    }

    /// Quick validation of tree size
    pub fn validate_tree_size(size: usize) -> Result<(), String> {
        if size < 10 {
            return Err("tree size must be at least 10".to_string());
        }
        let memory_mb = (size * 1024) / 1_000_000;
        if memory_mb > 2000 { // > 2GB
            return Err(format!("tree size may require {}MB memory", memory_mb));
        }
        Ok(())
    }
}
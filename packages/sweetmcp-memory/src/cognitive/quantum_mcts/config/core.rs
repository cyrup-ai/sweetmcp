//! Core quantum MCTS configuration with optimized defaults and validation
//!
//! This module provides the main QuantumMCTSConfig struct with zero-copy serialization
//! and compile-time optimization for blazing-fast quantum MCTS performance.

use serde::{Deserialize, Serialize};

/// Quantum MCTS configuration with compile-time optimization
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct QuantumMCTSConfig {
    /// Maximum parallel quantum circuits - optimized for CPU cores
    pub max_quantum_parallel: usize,
    /// Quantum exploration factor - UCT exploration parameter
    pub quantum_exploration: f64,
    /// Decoherence threshold - quantum coherence loss limit
    pub decoherence_threshold: f64,
    /// Entanglement strength - connection strength between nodes
    pub entanglement_strength: f64,
    /// Recursive improvement iterations - maximum recursion depth
    pub recursive_iterations: u32,
    /// Quantum amplitude threshold - minimum amplitude for consideration
    pub amplitude_threshold: f64,
    /// Phase evolution rate - quantum phase change rate
    pub phase_evolution_rate: f64,
    /// Simulation timeout in milliseconds - prevents infinite computation
    pub simulation_timeout_ms: u64,
    /// Memory limit for quantum tree - prevents excessive memory usage
    pub max_tree_size: usize,
    /// Enable quantum error correction - computational overhead vs accuracy
    pub enable_error_correction: bool,
    /// Quantum measurement precision - higher precision = slower computation
    pub measurement_precision: f64,
    /// Maximum entanglements per node for network effects
    pub max_entanglements_per_node: usize,
    /// Pruning threshold for weak entanglements
    pub pruning_threshold: f64,
    /// Batch size for parallel operations
    pub batch_size: usize,
    /// Debug mode for additional logging and validation
    pub debug_mode: bool,
}

impl Default for QuantumMCTSConfig {
    fn default() -> Self {
        Self {
            max_quantum_parallel: num_cpus::get().min(16).max(1), // Optimized for available cores
            quantum_exploration: 2.0,
            decoherence_threshold: 0.1,
            entanglement_strength: 0.7,
            recursive_iterations: 3,
            amplitude_threshold: 0.01,
            phase_evolution_rate: 0.1,
            simulation_timeout_ms: 30_000, // 30 seconds default timeout
            max_tree_size: 100_000, // 100k nodes maximum
            enable_error_correction: true,
            measurement_precision: 1e-10,
            max_entanglements_per_node: 10,
            pruning_threshold: 0.1,
            batch_size: 20,
            debug_mode: false,
        }
    }
}

impl QuantumMCTSConfig {
    /// Create new configuration with validation and environment loading
    pub fn new() -> Self {
        let mut config = Self::default();
        config.load_from_environment();
        
        if let Err(e) = config.validate() {
            eprintln!("Warning: Invalid quantum MCTS configuration: {}. Using defaults.", e);
            config = Self::default();
        }
        
        config
    }
    
    /// Create configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            max_quantum_parallel: num_cpus::get(),
            quantum_exploration: 1.4, // Lower exploration for faster convergence
            decoherence_threshold: 0.2, // Higher threshold for more aggressive pruning
            entanglement_strength: 0.5, // Lower entanglement for less computation
            recursive_iterations: 2, // Fewer iterations for speed
            amplitude_threshold: 0.05, // Higher threshold for pruning
            phase_evolution_rate: 0.2, // Faster phase evolution
            simulation_timeout_ms: 10_000, // Shorter timeout
            max_tree_size: 50_000, // Smaller tree for memory efficiency
            enable_error_correction: false, // Disable for maximum speed
            measurement_precision: 1e-6, // Lower precision for speed
            max_entanglements_per_node: 5, // Fewer entanglements for speed
            pruning_threshold: 0.2, // More aggressive pruning
            batch_size: 50, // Larger batches for throughput
            debug_mode: false,
        }
    }
    
    /// Create configuration optimized for accuracy
    pub fn accuracy_optimized() -> Self {
        Self {
            max_quantum_parallel: (num_cpus::get() / 2).max(1), // Use fewer cores for stability
            quantum_exploration: 3.0, // Higher exploration for better coverage
            decoherence_threshold: 0.05, // Lower threshold for higher precision
            entanglement_strength: 0.9, // Higher entanglement for more information
            recursive_iterations: 5, // More iterations for convergence
            amplitude_threshold: 0.001, // Lower threshold for completeness
            phase_evolution_rate: 0.05, // Slower evolution for stability
            simulation_timeout_ms: 120_000, // Longer timeout for complex problems
            max_tree_size: 500_000, // Larger tree for more thorough search
            enable_error_correction: true, // Enable for maximum accuracy
            measurement_precision: 1e-12, // Higher precision
            max_entanglements_per_node: 20, // More entanglements for information
            pruning_threshold: 0.05, // Conservative pruning
            batch_size: 10, // Smaller batches for precision
            debug_mode: false,
        }
    }
    
    /// Create balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }
    
    /// Create minimal configuration for resource-constrained environments
    pub fn minimal() -> Self {
        Self {
            max_quantum_parallel: 1,
            quantum_exploration: 1.0,
            decoherence_threshold: 0.3,
            entanglement_strength: 0.3,
            recursive_iterations: 1,
            amplitude_threshold: 0.1,
            phase_evolution_rate: 0.3,
            simulation_timeout_ms: 5_000,
            max_tree_size: 1_000,
            enable_error_correction: false,
            measurement_precision: 1e-4,
            max_entanglements_per_node: 3,
            pruning_threshold: 0.3,
            batch_size: 5,
            debug_mode: false,
        }
    }
    
    /// Create debug configuration with extensive validation
    pub fn debug() -> Self {
        Self {
            debug_mode: true,
            simulation_timeout_ms: 60_000, // Longer timeout for debugging
            enable_error_correction: true, // Always enable for debugging
            measurement_precision: 1e-12, // High precision for debugging
            ..Self::default()
        }
    }
    
    /// Load configuration from environment variables
    pub fn load_from_environment(&mut self) {
        super::environment::load_from_environment(self);
    }
    
    /// Validate configuration parameters with detailed error reporting
    pub fn validate(&self) -> Result<(), String> {
        super::validation::validate_config(self)
    }
    
    /// Calculate expected memory usage for this configuration
    pub fn estimate_memory_usage(&self) -> u64 {
        super::system::estimate_memory_usage(self)
    }
    
    /// Check if configuration is suitable for current system
    pub fn is_system_compatible(&self) -> bool {
        super::system::is_system_compatible(self)
    }
    
    /// Get recommended configuration for the current system
    pub fn recommended() -> Self {
        super::system::get_recommended_config()
    }
    
    /// Get system-optimized configuration
    pub fn system_optimized() -> Self {
        super::system::get_system_optimized_config()
    }
    
    /// Create a builder for gradual configuration construction
    pub fn builder() -> super::builder::QuantumMCTSConfigBuilder {
        super::builder::QuantumMCTSConfigBuilder::new()
    }
    
    /// Clone configuration with modifications
    pub fn with_parallel(&self, parallel: usize) -> Self {
        Self {
            max_quantum_parallel: parallel,
            ..self.clone()
        }
    }
    
    /// Clone configuration with exploration modification
    pub fn with_exploration(&self, exploration: f64) -> Self {
        Self {
            quantum_exploration: exploration,
            ..self.clone()
        }
    }
    
    /// Clone configuration with timeout modification
    pub fn with_timeout(&self, timeout_ms: u64) -> Self {
        Self {
            simulation_timeout_ms: timeout_ms,
            ..self.clone()
        }
    }
    
    /// Clone configuration with tree size modification
    pub fn with_tree_size(&self, size: usize) -> Self {
        Self {
            max_tree_size: size,
            ..self.clone()
        }
    }
    
    /// Get configuration summary as string
    pub fn summary(&self) -> String {
        format!(
            "QuantumMCTSConfig: parallel={}, exploration={:.2}, threshold={:.3}, timeout={}ms, tree_size={}",
            self.max_quantum_parallel,
            self.quantum_exploration,
            self.decoherence_threshold,
            self.simulation_timeout_ms,
            self.max_tree_size
        )
    }
    
    /// Check if configuration is performance-oriented
    pub fn is_performance_oriented(&self) -> bool {
        !self.enable_error_correction && 
        self.measurement_precision >= 1e-8 &&
        self.max_tree_size <= 100_000
    }
    
    /// Check if configuration is accuracy-oriented
    pub fn is_accuracy_oriented(&self) -> bool {
        self.enable_error_correction && 
        self.measurement_precision <= 1e-10 &&
        self.recursive_iterations >= 4
    }
    
    /// Get performance score (0.0 to 1.0, higher = more performance-oriented)
    pub fn performance_score(&self) -> f64 {
        let mut score = 0.0;
        
        // Parallelism score (more cores = higher performance)
        score += (self.max_quantum_parallel as f64 / 16.0).min(1.0) * 0.2;
        
        // Error correction penalty
        if !self.enable_error_correction {
            score += 0.2;
        }
        
        // Precision score (lower precision = higher performance)
        if self.measurement_precision >= 1e-6 {
            score += 0.2;
        } else if self.measurement_precision >= 1e-8 {
            score += 0.1;
        }
        
        // Tree size score (smaller = higher performance)
        if self.max_tree_size <= 50_000 {
            score += 0.2;
        } else if self.max_tree_size <= 100_000 {
            score += 0.1;
        }
        
        // Timeout score (shorter = higher performance)
        if self.simulation_timeout_ms <= 10_000 {
            score += 0.2;
        } else if self.simulation_timeout_ms <= 30_000 {
            score += 0.1;
        }
        
        score.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = QuantumMCTSConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.max_quantum_parallel > 0);
        assert!(config.quantum_exploration > 0.0);
    }
    
    #[test]
    fn test_performance_optimized_config() {
        let config = QuantumMCTSConfig::performance_optimized();
        assert!(config.validate().is_ok());
        assert!(!config.enable_error_correction);
        assert!(config.is_performance_oriented());
        assert!(config.performance_score() > 0.5);
    }
    
    #[test]
    fn test_accuracy_optimized_config() {
        let config = QuantumMCTSConfig::accuracy_optimized();
        assert!(config.validate().is_ok());
        assert!(config.enable_error_correction);
        assert!(config.is_accuracy_oriented());
    }
    
    #[test]
    fn test_minimal_config() {
        let config = QuantumMCTSConfig::minimal();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_quantum_parallel, 1);
        assert!(config.max_tree_size <= 1_000);
    }
    
    #[test]
    fn test_config_modifications() {
        let config = QuantumMCTSConfig::default();
        
        let modified = config.with_parallel(8);
        assert_eq!(modified.max_quantum_parallel, 8);
        
        let modified = config.with_exploration(3.0);
        assert!((modified.quantum_exploration - 3.0).abs() < f64::EPSILON);
        
        let modified = config.with_timeout(60_000);
        assert_eq!(modified.simulation_timeout_ms, 60_000);
        
        let modified = config.with_tree_size(200_000);
        assert_eq!(modified.max_tree_size, 200_000);
    }
    
    #[test]
    fn test_config_summary() {
        let config = QuantumMCTSConfig::default();
        let summary = config.summary();
        
        assert!(summary.contains("QuantumMCTSConfig"));
        assert!(summary.contains(&config.max_quantum_parallel.to_string()));
        assert!(summary.contains(&config.max_tree_size.to_string()));
    }
    
    #[test]
    fn test_performance_score() {
        let perf_config = QuantumMCTSConfig::performance_optimized();
        let acc_config = QuantumMCTSConfig::accuracy_optimized();
        
        assert!(perf_config.performance_score() > acc_config.performance_score());
    }
}
//! Configuration builder pattern with fluent API and validation
//!
//! This module provides a blazing-fast, zero-allocation builder pattern
//! for constructing quantum MCTS configurations with comprehensive validation.

use crate::cognitive::quantum_mcts::config::{
    core::QuantumMCTSConfig,
    validation::ConfigValidator,
    system::SystemAnalyzer,
};
use std::marker::PhantomData;

/// Configuration builder with compile-time validation and fluent API
#[derive(Debug)]
pub struct QuantumMCTSConfigBuilder<State = Unvalidated> {
    config: QuantumMCTSConfig,
    _state: PhantomData<State>,
}

/// Builder state markers for compile-time validation
#[derive(Debug)]
pub struct Unvalidated;

#[derive(Debug)]
pub struct Validated;

/// Configuration preset for rapid builder initialization
#[derive(Debug, Clone, Copy)]
pub enum ConfigPreset {
    Default,
    Performance,
    Accuracy,
    SystemOptimized,
    Minimal,
}

impl QuantumMCTSConfigBuilder<Unvalidated> {
    /// Create new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: QuantumMCTSConfig::default(),
            _state: PhantomData,
        }
    }

    /// Create builder from configuration preset
    pub fn from_preset(preset: ConfigPreset) -> Self {
        let config = match preset {
            ConfigPreset::Default => QuantumMCTSConfig::default(),
            ConfigPreset::Performance => QuantumMCTSConfig::performance_optimized(),
            ConfigPreset::Accuracy => QuantumMCTSConfig::accuracy_optimized(),
            ConfigPreset::SystemOptimized => {
                SystemAnalyzer::new().system_optimized_config()
            },
            ConfigPreset::Minimal => QuantumMCTSConfig {
                max_quantum_parallel: 1,
                quantum_exploration: 1.0,
                decoherence_threshold: 0.2,
                entanglement_strength: 0.3,
                recursive_iterations: 1,
                amplitude_threshold: 0.1,
                phase_evolution_rate: 0.1,
                simulation_timeout_ms: 5_000,
                max_tree_size: 1_000,
                enable_error_correction: false,
                measurement_precision: 1e-6,
            },
        };

        Self {
            config,
            _state: PhantomData,
        }
    }

    /// Set maximum parallel quantum circuits with bounds checking
    pub fn max_quantum_parallel(mut self, parallel: usize) -> Self {
        self.config.max_quantum_parallel = parallel.clamp(1, 128);
        self
    }

    /// Set quantum exploration factor with UCT-aware bounds
    pub fn quantum_exploration(mut self, exploration: f64) -> Self {
        self.config.quantum_exploration = exploration.clamp(0.1, 10.0);
        self
    }

    /// Set decoherence threshold with quantum physics bounds
    pub fn decoherence_threshold(mut self, threshold: f64) -> Self {
        self.config.decoherence_threshold = threshold.clamp(0.001, 1.0);
        self
    }

    /// Set entanglement strength with correlation bounds
    pub fn entanglement_strength(mut self, strength: f64) -> Self {
        self.config.entanglement_strength = strength.clamp(0.0, 1.0);
        self
    }

    /// Set recursive improvement iterations with complexity bounds
    pub fn recursive_iterations(mut self, iterations: u32) -> Self {
        self.config.recursive_iterations = iterations.clamp(1, 20);
        self
    }

    /// Set amplitude threshold with pruning bounds
    pub fn amplitude_threshold(mut self, threshold: f64) -> Self {
        self.config.amplitude_threshold = threshold.clamp(0.0001, 1.0);
        self
    }

    /// Set phase evolution rate with stability bounds
    pub fn phase_evolution_rate(mut self, rate: f64) -> Self {
        self.config.phase_evolution_rate = rate.clamp(0.001, 1.0);
        self
    }

    /// Set simulation timeout with practical bounds
    pub fn simulation_timeout_ms(mut self, timeout: u64) -> Self {
        self.config.simulation_timeout_ms = timeout.clamp(100, 3_600_000);
        self
    }

    /// Set maximum tree size with memory-aware bounds
    pub fn max_tree_size(mut self, size: usize) -> Self {
        self.config.max_tree_size = size.clamp(10, 10_000_000);
        self
    }

    /// Enable or disable quantum error correction
    pub fn enable_error_correction(mut self, enable: bool) -> Self {
        self.config.enable_error_correction = enable;
        self
    }

    /// Set measurement precision with numerical stability bounds
    pub fn measurement_precision(mut self, precision: f64) -> Self {
        self.config.measurement_precision = precision.clamp(1e-15, 1.0);
        self
    }

    /// Configure for real-time applications with optimized parameters
    pub fn for_real_time(mut self) -> Self {
        self.config.simulation_timeout_ms = 1_000; // 1 second max
        self.config.max_tree_size = 5_000; // Small tree for speed
        self.config.enable_error_correction = false; // Disable for speed
        self.config.recursive_iterations = 1; // Minimal iterations
        self.config.measurement_precision = 1e-6; // Lower precision for speed
        self
    }

    /// Configure for batch processing with accuracy focus
    pub fn for_batch_processing(mut self) -> Self {
        self.config.simulation_timeout_ms = 300_000; // 5 minutes max
        self.config.max_tree_size = 1_000_000; // Large tree for accuracy
        self.config.enable_error_correction = true; // Enable for accuracy
        self.config.recursive_iterations = 5; // More iterations
        self.config.measurement_precision = 1e-10; // High precision
        self
    }

    /// Configure for mobile/embedded systems
    pub fn for_mobile(mut self) -> Self {
        self.config.max_quantum_parallel = 2; // Conservative parallelism
        self.config.simulation_timeout_ms = 2_000; // Short timeout
        self.config.max_tree_size = 1_000; // Very small tree
        self.config.enable_error_correction = false; // Disable for battery
        self.config.recursive_iterations = 1; // Minimal computation
        self.config.measurement_precision = 1e-4; // Very low precision
        self
    }

    /// Configure for high-performance computing environments
    pub fn for_hpc(mut self) -> Self {
        let cpu_count = num_cpus::get();
        self.config.max_quantum_parallel = cpu_count * 2; // Aggressive parallelism
        self.config.simulation_timeout_ms = 1_800_000; // 30 minutes
        self.config.max_tree_size = 10_000_000; // Maximum tree size
        self.config.enable_error_correction = true; // Full accuracy
        self.config.recursive_iterations = 10; // Deep recursion
        self.config.measurement_precision = 1e-12; // Maximum precision
        self
    }

    /// Apply system-specific optimizations automatically
    pub fn auto_optimize(mut self) -> Self {
        let analyzer = SystemAnalyzer::new();
        let optimized = analyzer.system_optimized_config();
        self.config = optimized;
        self
    }

    /// Apply custom configuration transformation
    pub fn transform<F>(mut self, f: F) -> Self 
    where
        F: FnOnce(QuantumMCTSConfig) -> QuantumMCTSConfig,
    {
        self.config = f(self.config);
        self
    }

    /// Validate configuration and transition to validated state
    pub fn validate(self) -> Result<QuantumMCTSConfigBuilder<Validated>, String> {
        ConfigValidator::validate(&self.config)?;
        
        Ok(QuantumMCTSConfigBuilder {
            config: self.config,
            _state: PhantomData,
        })
    }

    /// Build configuration without validation (unsafe)
    pub fn build_unchecked(self) -> QuantumMCTSConfig {
        self.config
    }
}

impl QuantumMCTSConfigBuilder<Validated> {
    /// Build the validated configuration
    pub fn build(self) -> QuantumMCTSConfig {
        self.config
    }

    /// Check system compatibility before building
    pub fn check_system_compatibility(self) -> Result<QuantumMCTSConfig, String> {
        let analyzer = SystemAnalyzer::new();
        analyzer.is_compatible(&self.config)?;
        Ok(self.config)
    }

    /// Build with final validation and system compatibility check
    pub fn build_with_system_check(self) -> Result<QuantumMCTSConfig, String> {
        let analyzer = SystemAnalyzer::new();
        analyzer.is_compatible(&self.config)?;
        Ok(self.config)
    }
}

impl Default for QuantumMCTSConfigBuilder<Unvalidated> {
    fn default() -> Self {
        Self::new()
    }
}

/// Fluent configuration builder with method chaining
pub struct FluentConfigBuilder {
    builder: QuantumMCTSConfigBuilder<Unvalidated>,
}

impl FluentConfigBuilder {
    /// Create new fluent builder
    pub fn new() -> Self {
        Self {
            builder: QuantumMCTSConfigBuilder::new(),
        }
    }

    /// Start with a preset configuration
    pub fn preset(preset: ConfigPreset) -> Self {
        Self {
            builder: QuantumMCTSConfigBuilder::from_preset(preset),
        }
    }

    /// Configure parallelism with intelligent defaults
    pub fn parallelism(&mut self) -> ParallelismConfig<'_> {
        ParallelismConfig::new(&mut self.builder)
    }

    /// Configure thresholds with quantum-aware defaults
    pub fn thresholds(&mut self) -> ThresholdConfig<'_> {
        ThresholdConfig::new(&mut self.builder)
    }

    /// Configure performance parameters
    pub fn performance(&mut self) -> PerformanceConfig<'_> {
        PerformanceConfig::new(&mut self.builder)
    }

    /// Configure accuracy parameters
    pub fn accuracy(&mut self) -> AccuracyConfig<'_> {
        AccuracyConfig::new(&mut self.builder)
    }

    /// Finalize and build configuration
    pub fn build(self) -> Result<QuantumMCTSConfig, String> {
        self.builder.validate()?.build_with_system_check()
    }

    /// Build without validation or system checks
    pub fn build_unchecked(self) -> QuantumMCTSConfig {
        self.builder.build_unchecked()
    }
}

/// Parallelism configuration helper
pub struct ParallelismConfig<'a> {
    builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>,
}

impl<'a> ParallelismConfig<'a> {
    fn new(builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>) -> Self {
        Self { builder }
    }

    /// Set parallelism to CPU count
    pub fn cpu_count(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        let cpu_count = num_cpus::get();
        self.builder.config.max_quantum_parallel = cpu_count;
        self.builder
    }

    /// Set parallelism to half CPU count (conservative)
    pub fn conservative(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        let cpu_count = num_cpus::get();
        self.builder.config.max_quantum_parallel = (cpu_count / 2).max(1);
        self.builder
    }

    /// Set parallelism to 2x CPU count (aggressive)
    pub fn aggressive(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        let cpu_count = num_cpus::get();
        self.builder.config.max_quantum_parallel = cpu_count * 2;
        self.builder
    }

    /// Set custom parallelism value
    pub fn custom(self, parallel: usize) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.max_quantum_parallel = parallel;
        self.builder
    }
}

/// Threshold configuration helper
pub struct ThresholdConfig<'a> {
    builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>,
}

impl<'a> ThresholdConfig<'a> {
    fn new(builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>) -> Self {
        Self { builder }
    }

    /// Set conservative thresholds for high precision
    pub fn conservative(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.decoherence_threshold = 0.05;
        self.builder.config.amplitude_threshold = 0.001;
        self.builder
    }

    /// Set aggressive thresholds for performance
    pub fn aggressive(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.decoherence_threshold = 0.2;
        self.builder.config.amplitude_threshold = 0.05;
        self.builder
    }

    /// Set balanced thresholds
    pub fn balanced(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.decoherence_threshold = 0.1;
        self.builder.config.amplitude_threshold = 0.01;
        self.builder
    }
}

/// Performance configuration helper
pub struct PerformanceConfig<'a> {
    builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>,
}

impl<'a> PerformanceConfig<'a> {
    fn new(builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>) -> Self {
        Self { builder }
    }

    /// Optimize for maximum speed
    pub fn speed(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.enable_error_correction = false;
        self.builder.config.recursive_iterations = 1;
        self.builder.config.simulation_timeout_ms = 5_000;
        self.builder.config.max_tree_size = 10_000;
        self.builder
    }

    /// Balance speed and quality
    pub fn balanced(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.enable_error_correction = true;
        self.builder.config.recursive_iterations = 3;
        self.builder.config.simulation_timeout_ms = 30_000;
        self.builder.config.max_tree_size = 100_000;
        self.builder
    }
}

/// Accuracy configuration helper
pub struct AccuracyConfig<'a> {
    builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>,
}

impl<'a> AccuracyConfig<'a> {
    fn new(builder: &'a mut QuantumMCTSConfigBuilder<Unvalidated>) -> Self {
        Self { builder }
    }

    /// Optimize for maximum accuracy
    pub fn maximum(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.enable_error_correction = true;
        self.builder.config.recursive_iterations = 7;
        self.builder.config.measurement_precision = 1e-12;
        self.builder.config.max_tree_size = 1_000_000;
        self.builder
    }

    /// High accuracy with reasonable performance
    pub fn high(self) -> &'a mut QuantumMCTSConfigBuilder<Unvalidated> {
        self.builder.config.enable_error_correction = true;
        self.builder.config.recursive_iterations = 5;
        self.builder.config.measurement_precision = 1e-10;
        self.builder.config.max_tree_size = 500_000;
        self.builder
    }
}

impl Default for FluentConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
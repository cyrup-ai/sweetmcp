//! Quantum MCTS configuration management with zero-allocation patterns
//!
//! This module provides comprehensive configuration management for quantum MCTS
//! with blazing-fast validation, system optimization, and builder patterns.

pub mod core;
pub mod environment;
pub mod validation;
pub mod system;
pub mod builder;

// Re-export core types for backward compatibility
pub use core::QuantumMCTSConfig;
pub use builder::{
    QuantumMCTSConfigBuilder, FluentConfigBuilder, ConfigPreset,
    ParallelismConfig, ThresholdConfig, PerformanceConfig, AccuracyConfig,
};
pub use validation::{ConfigValidator, ValidationError};
pub use system::{
    SystemAnalyzer, SystemResources, SystemCapabilitySummary,
    ArchType, PerformanceTier, ConfigType,
};
pub use environment::{EnvironmentLoader, ConfigSource, EnvironmentError};

use std::sync::OnceLock;

/// Global configuration coordinator for centralized access
pub struct ConfigCoordinator {
    analyzer: SystemAnalyzer,
    env_loader: environment::EnvironmentLoader,
}

static CONFIG_COORDINATOR: OnceLock<ConfigCoordinator> = OnceLock::new();

impl ConfigCoordinator {
    /// Get global configuration coordinator instance
    pub fn global() -> &'static ConfigCoordinator {
        CONFIG_COORDINATOR.get_or_init(|| Self::new())
    }

    /// Create new configuration coordinator
    pub fn new() -> Self {
        Self {
            analyzer: SystemAnalyzer::new(),
            env_loader: environment::EnvironmentLoader::new(),
        }
    }

    /// Create optimal configuration for current system
    pub fn create_optimal_config() -> QuantumMCTSConfig {
        let coordinator = Self::global();
        let mut config = coordinator.analyzer.system_optimized_config();
        
        // Apply environment overrides
        if let Ok(env_config) = coordinator.env_loader.load_from_environment() {
            config = coordinator.merge_configs(&config, &env_config);
        }
        
        // Validate final configuration
        if let Err(e) = validation::ConfigValidator::validate(&config) {
            eprintln!("Warning: Configuration validation failed: {}. Using system defaults.", e);
            config = coordinator.analyzer.system_optimized_config();
        }
        
        config
    }

    /// Create configuration from preset with system optimization
    pub fn create_from_preset(preset: ConfigPreset) -> QuantumMCTSConfig {
        let coordinator = Self::global();
        
        match FluentConfigBuilder::preset(preset).build() {
            Ok(config) => {
                // Verify system compatibility
                if let Err(e) = coordinator.analyzer.is_compatible(&config) {
                    eprintln!("Warning: Preset incompatible with system: {}. Using optimized config.", e);
                    coordinator.analyzer.system_optimized_config()
                } else {
                    config
                }
            },
            Err(e) => {
                eprintln!("Warning: Preset configuration invalid: {}. Using system defaults.", e);
                coordinator.analyzer.system_optimized_config()
            }
        }
    }

    /// Get system capability summary
    pub fn system_summary(&self) -> SystemCapabilitySummary {
        self.analyzer.capability_summary()
    }

    /// Get performance recommendations for current system
    pub fn performance_recommendations(&self) -> Vec<String> {
        self.analyzer.performance_recommendations()
    }

    /// Validate configuration against current system
    pub fn validate_config(&self, config: &QuantumMCTSConfig) -> Result<(), String> {
        // First validate parameter bounds
        validation::ConfigValidator::validate(config)?;
        
        // Then check system compatibility
        self.analyzer.is_compatible(config)?;
        
        Ok(())
    }

    /// Merge two configurations with intelligent precedence
    fn merge_configs(&self, base: &QuantumMCTSConfig, override_config: &QuantumMCTSConfig) -> QuantumMCTSConfig {
        // Start with a clone of the base config
        let mut merged = base.clone();
        
        // Merge parallelism (with bounds checking)
        if override_config.max_quantum_parallel != base.max_quantum_parallel {
            let cpu_count = self.analyzer.resources().cpu_count;
            if override_config.max_quantum_parallel <= cpu_count * 4 {
                merged.max_quantum_parallel = override_config.max_quantum_parallel;
            }
        }
        
        // Merge other parameters with validation
        if (0.1..=10.0).contains(&override_config.quantum_exploration) {
            merged.quantum_exploration = override_config.quantum_exploration;
        }
        
        if (0.001..=1.0).contains(&override_config.decoherence_threshold) {
            merged.decoherence_threshold = override_config.decoherence_threshold;
        }
        
        if (0.0..=1.0).contains(&override_config.entanglement_strength) {
            merged.entanglement_strength = override_config.entanglement_strength;
        }
        
        if override_config.recursive_iterations > 0 && override_config.recursive_iterations <= 20 {
            merged.recursive_iterations = override_config.recursive_iterations;
        }
        
        if (0.0001..=1.0).contains(&override_config.amplitude_threshold) {
            merged.amplitude_threshold = override_config.amplitude_threshold;
        }
        
        if (0.001..=1.0).contains(&override_config.phase_evolution_rate) {
            merged.phase_evolution_rate = override_config.phase_evolution_rate;
        }
        
        if override_config.simulation_timeout_ms >= 100 && override_config.simulation_timeout_ms <= 3_600_000 {
            merged.simulation_timeout_ms = override_config.simulation_timeout_ms;
        }
        
        if override_config.max_tree_size >= 10 && override_config.max_tree_size <= 10_000_000 {
            merged.max_tree_size = override_config.max_tree_size;
        }
        
        // Boolean flags are merged directly
        merged.enable_error_correction = override_config.enable_error_correction;
        
        if override_config.measurement_precision > 0.0 && override_config.measurement_precision <= 1.0 {
            merged.measurement_precision = override_config.measurement_precision;
        }
        
        merged
    }

    /// Create configuration for specific use case
    pub fn create_for_use_case(use_case: UseCase) -> QuantumMCTSConfig {
        let coordinator = Self::global();
        
        let config = match use_case {
            UseCase::RealTime => {
                FluentConfigBuilder::preset(ConfigPreset::Performance)
                    .performance().speed()
                    .parallelism().conservative()
                    .build_unchecked()
            },
            UseCase::BatchProcessing => {
                FluentConfigBuilder::preset(ConfigPreset::Accuracy)
                    .accuracy().maximum()
                    .parallelism().aggressive()
                    .build_unchecked()
            },
            UseCase::Mobile => {
                FluentConfigBuilder::preset(ConfigPreset::Minimal)
                    .performance().speed()
                    .parallelism().custom(1)
                    .build_unchecked()
            },
            UseCase::HighPerformanceComputing => {
                FluentConfigBuilder::preset(ConfigPreset::Accuracy)
                    .accuracy().maximum()
                    .parallelism().aggressive()
                    .build_unchecked()
            },
            UseCase::Development => {
                FluentConfigBuilder::preset(ConfigPreset::Default)
                    .performance().balanced()
                    .parallelism().cpu_count()
                    .build_unchecked()
            },
            UseCase::Testing => {
                FluentConfigBuilder::preset(ConfigPreset::Minimal)
                    .performance().speed()
                    .parallelism().custom(1)
                    .transform(|mut c| {
                        c.simulation_timeout_ms = 1_000; // Very short for tests
                        c.max_tree_size = 100; // Tiny tree for tests
                        c
                    })
                    .build_unchecked()
            },
        };
        
        // Validate against system
        if let Err(e) = coordinator.validate_config(&config) {
            eprintln!("Warning: Use case config incompatible: {}. Using system defaults.", e);
            coordinator.analyzer.system_optimized_config()
        } else {
            config
        }
    }
}

impl Default for ConfigCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common use cases for configuration optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseCase {
    RealTime,
    BatchProcessing,
    Mobile,
    HighPerformanceComputing,
    Development,
    Testing,
}

/// Configuration factory functions for common scenarios
pub mod factory {
    use super::*;

    /// Create default configuration with system optimization
    pub fn default() -> QuantumMCTSConfig {
        ConfigCoordinator::create_optimal_config()
    }

    /// Create performance-optimized configuration
    pub fn performance() -> QuantumMCTSConfig {
        ConfigCoordinator::create_from_preset(ConfigPreset::Performance)
    }

    /// Create accuracy-optimized configuration  
    pub fn accuracy() -> QuantumMCTSConfig {
        ConfigCoordinator::create_from_preset(ConfigPreset::Accuracy)
    }

    /// Create system-optimized configuration
    pub fn system_optimized() -> QuantumMCTSConfig {
        ConfigCoordinator::create_from_preset(ConfigPreset::SystemOptimized)
    }

    /// Create minimal configuration for constrained environments
    pub fn minimal() -> QuantumMCTSConfig {
        ConfigCoordinator::create_from_preset(ConfigPreset::Minimal)
    }

    /// Create configuration for real-time applications
    pub fn real_time() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::RealTime)
    }

    /// Create configuration for batch processing
    pub fn batch_processing() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::BatchProcessing)
    }

    /// Create configuration for mobile/embedded systems
    pub fn mobile() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::Mobile)
    }

    /// Create configuration for HPC environments
    pub fn hpc() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::HighPerformanceComputing)
    }

    /// Create configuration for development/debugging
    pub fn development() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::Development)
    }

    /// Create configuration optimized for testing
    pub fn testing() -> QuantumMCTSConfig {
        ConfigCoordinator::create_for_use_case(UseCase::Testing)
    }
}

/// Quick access functions for immediate use
pub fn default_config() -> QuantumMCTSConfig {
    factory::default()
}

pub fn performance_config() -> QuantumMCTSConfig {
    factory::performance()
}

pub fn accuracy_config() -> QuantumMCTSConfig {
    factory::accuracy()
}

pub fn system_config() -> QuantumMCTSConfig {
    factory::system_optimized()
}

/// Get system information and recommendations
pub fn system_info() -> SystemCapabilitySummary {
    ConfigCoordinator::global().system_summary()
}

/// Validate a configuration for current system
pub fn validate(config: &QuantumMCTSConfig) -> Result<(), String> {
    ConfigCoordinator::global().validate_config(config)
}
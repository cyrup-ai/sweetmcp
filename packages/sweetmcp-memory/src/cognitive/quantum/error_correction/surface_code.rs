//! Surface code coordination facade
//!
//! This module provides a unified, ergonomic interface for surface code quantum error correction
//! by delegating to specialized submodules with blazing-fast performance and zero allocation optimizations.

// Re-export all surface code functionality from submodules
pub use self::surface_code::{
    // Core types
    SurfaceCode, SurfaceCodeCoordinator, SurfaceCodeConfig, PerformanceReport, CoordinatorMetrics,
    
    // Syndrome detection
    QubitPosition, PauliType, PauliOperator, StabilizerType, StabilizerGenerator,
    SurfaceCodeSyndrome, SyndromeDetector, SyndromeDetectionConfig, SyndromeDetectionMetrics,
    
    // Error correction
    LogicalError, LogicalErrorType, SurfaceCodeCorrection, CorrectionAlgorithm,
    ErrorChain, ChainType, SurfaceCodeCorrector, CorrectionMetrics, CorrectionConfig,
    
    // Layout management
    BoundaryType, SurfaceCodeLayout, QubitType, LayoutMetrics,
    SurfaceCodeLayoutBuilder, OptimizationTarget, LayoutConstraints,
    
    // Logical operations
    LogicalOperatorType, LogicalOperator, LogicalOperators, LogicalOperatorMetrics,
    LogicalOperationsEngine, OperationMetrics, LogicalOperationConfig,
};

// Import the coordination module
mod surface_code;

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::HashMap;

/// Create surface code with distance and open boundaries (convenience function)
#[inline]
pub fn create_surface_code(distance: usize) -> CognitiveResult<SurfaceCode> {
    SurfaceCode::new(distance, BoundaryType::Open)
}

/// Create surface code with distance and specified boundary type (convenience function)
#[inline]
pub fn create_surface_code_with_boundary(
    distance: usize,
    boundary_type: BoundaryType,
) -> CognitiveResult<SurfaceCode> {
    SurfaceCode::new(distance, boundary_type)
}

/// Create surface code coordinator with default configuration (convenience function)
#[inline]
pub fn create_coordinator(distance: usize) -> CognitiveResult<SurfaceCodeCoordinator> {
    SurfaceCodeCoordinator::new(distance, BoundaryType::Open)
}

/// Create surface code coordinator with custom configuration (convenience function)
#[inline]
pub fn create_coordinator_with_config(
    distance: usize,
    boundary_type: BoundaryType,
    config: SurfaceCodeConfig,
) -> CognitiveResult<SurfaceCodeCoordinator> {
    SurfaceCodeCoordinator::with_config(distance, boundary_type, config)
}

/// Perform complete error correction on error pattern (convenience function)
#[inline]
pub fn correct_errors(
    coordinator: &mut SurfaceCodeCoordinator,
    error_pattern: &HashMap<QubitPosition, PauliType>,
    round: usize,
) -> CognitiveResult<SurfaceCodeCorrection> {
    coordinator.error_correction_cycle(error_pattern, round)
}

/// Apply logical X operation (convenience function)
#[inline]
pub fn apply_logical_x(
    coordinator: &mut SurfaceCodeCoordinator,
    logical_qubit_index: usize,
    state: &mut HashMap<QubitPosition, Complex64>,
) -> CognitiveResult<()> {
    coordinator.apply_logical_operation(LogicalOperatorType::X, logical_qubit_index, state)
}

/// Apply logical Z operation (convenience function)
#[inline]
pub fn apply_logical_z(
    coordinator: &mut SurfaceCodeCoordinator,
    logical_qubit_index: usize,
    state: &mut HashMap<QubitPosition, Complex64>,
) -> CognitiveResult<()> {
    coordinator.apply_logical_operation(LogicalOperatorType::Z, logical_qubit_index, state)
}

/// Measure logical X observable (convenience function)
#[inline]
pub fn measure_logical_x(
    coordinator: &mut SurfaceCodeCoordinator,
    logical_qubit_index: usize,
    state: &HashMap<QubitPosition, Complex64>,
) -> CognitiveResult<f64> {
    coordinator.measure_logical_observable(LogicalOperatorType::X, logical_qubit_index, state)
}

/// Measure logical Z observable (convenience function)
#[inline]
pub fn measure_logical_z(
    coordinator: &mut SurfaceCodeCoordinator,
    logical_qubit_index: usize,
    state: &HashMap<QubitPosition, Complex64>,
) -> CognitiveResult<f64> {
    coordinator.measure_logical_observable(LogicalOperatorType::Z, logical_qubit_index, state)
}

/// Calculate optimal surface code parameters for target error rate (convenience function)
#[inline]
pub fn calculate_optimal_parameters(target_error_rate: f64) -> (usize, BoundaryType) {
    // Simple heuristic for parameter selection
    let distance = if target_error_rate < 0.001 {
        9
    } else if target_error_rate < 0.01 {
        7
    } else if target_error_rate < 0.05 {
        5
    } else {
        3
    };
    
    // Use open boundaries for simplicity
    (distance, BoundaryType::Open)
}

/// Validate surface code configuration (convenience function)
#[inline]
pub fn validate_surface_code(surface_code: &SurfaceCode) -> CognitiveResult<bool> {
    surface_code.validate()
}

/// Get surface code performance report (convenience function)
#[inline]
pub fn get_performance_report(coordinator: &SurfaceCodeCoordinator) -> PerformanceReport {
    coordinator.get_performance_report()
}

/// Create performance-optimized surface code coordinator (convenience function)
#[inline]
pub fn create_high_performance_coordinator(
    distance: usize,
    boundary_type: BoundaryType,
) -> CognitiveResult<SurfaceCodeCoordinator> {
    let config = SurfaceCodeConfig::performance_optimized();
    SurfaceCodeCoordinator::with_config(distance, boundary_type, config)
}

/// Surface code factory for creating optimized instances
pub struct SurfaceCodeFactory;

impl SurfaceCodeFactory {
    /// Create surface code optimized for low error rates
    #[inline]
    pub fn create_low_error_rate_code(target_error_rate: f64) -> CognitiveResult<SurfaceCodeCoordinator> {
        let (distance, boundary_type) = calculate_optimal_parameters(target_error_rate);
        create_high_performance_coordinator(distance, boundary_type)
    }
    
    /// Create surface code optimized for high throughput
    #[inline]
    pub fn create_high_throughput_code(distance: usize) -> CognitiveResult<SurfaceCodeCoordinator> {
        let mut config = SurfaceCodeConfig::performance_optimized();
        config.syndrome_detection.enable_parallel_processing = true;
        config.syndrome_detection.batch_size = 256;
        config.logical_operations.enable_parallel_operations = true;
        
        SurfaceCodeCoordinator::with_config(distance, BoundaryType::Open, config)
    }
    
    /// Create surface code optimized for memory efficiency
    #[inline]
    pub fn create_memory_efficient_code(distance: usize) -> CognitiveResult<SurfaceCodeCoordinator> {
        let mut config = SurfaceCodeConfig::new();
        config.syndrome_detection.enable_caching = false;
        config.logical_operations.enable_caching = false;
        config.logical_operations.max_cache_size = 100;
        
        SurfaceCodeCoordinator::with_config(distance, BoundaryType::Open, config)
    }
    
    /// Create surface code with custom layout
    #[inline]
    pub fn create_with_custom_layout(
        layout: SurfaceCodeLayout,
        distance: usize,
    ) -> CognitiveResult<SurfaceCodeCoordinator> {
        let surface_code = SurfaceCode::with_layout(layout, distance)?;
        
        let syndrome_detector = SyndromeDetector::new(
            surface_code.x_stabilizers.clone(),
            surface_code.z_stabilizers.clone(),
        );
        
        let error_corrector = SurfaceCodeCorrector::new(
            distance,
            surface_code.qubit_layout.dimensions,
        );
        
        let logical_engine = LogicalOperationsEngine::new(surface_code.qubit_layout.clone())?;
        
        // Create coordinator manually since we have custom surface code
        Ok(SurfaceCodeCoordinator {
            surface_code,
            syndrome_detector,
            error_corrector,
            logical_engine,
            metrics: CoordinatorMetrics::new(),
            config: SurfaceCodeConfig::default(),
        })
    }
}

/// Surface code utilities
pub struct SurfaceCodeUtils;

impl SurfaceCodeUtils {
    /// Calculate theoretical error correction threshold
    #[inline]
    pub fn theoretical_threshold(boundary_type: BoundaryType) -> f64 {
        match boundary_type {
            BoundaryType::Open => 0.109,
            BoundaryType::Periodic => 0.103,
            BoundaryType::Twisted => 0.097,
        }
    }
    
    /// Estimate physical qubits needed for given logical qubits and distance
    #[inline]
    pub fn estimate_physical_qubits(logical_qubits: usize, distance: usize) -> usize {
        // Rough estimate: each logical qubit needs ~distance^2 physical qubits
        logical_qubits * distance * distance
    }
    
    /// Calculate code rate (logical qubits / physical qubits)
    #[inline]
    pub fn calculate_code_rate(surface_code: &SurfaceCode) -> f64 {
        let (n, k, _) = surface_code.code_parameters();
        if n > 0 {
            k as f64 / n as f64
        } else {
            0.0
        }
    }
    
    /// Estimate error correction time complexity
    #[inline]
    pub fn estimate_time_complexity(algorithm: CorrectionAlgorithm, num_syndromes: usize) -> String {
        match algorithm {
            CorrectionAlgorithm::MinimumWeightPerfectMatching => {
                format!("O({}³) ≈ {}", num_syndromes, num_syndromes.pow(3))
            }
            CorrectionAlgorithm::UnionFind => {
                let log_n = (num_syndromes as f64).log2().ceil() as usize;
                format!("O({} log {}) ≈ {}", num_syndromes, num_syndromes, num_syndromes * log_n)
            }
            CorrectionAlgorithm::BeliefPropagation => {
                format!("O({}²) ≈ {}", num_syndromes, num_syndromes.pow(2))
            }
            CorrectionAlgorithm::NeuralNetwork => {
                format!("O({}) ≈ {}", num_syndromes, num_syndromes)
            }
            CorrectionAlgorithm::LookupTable => {
                "O(1) ≈ 1".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_code_creation() {
        let surface_code = create_surface_code(3).expect("Failed to create surface code");
        assert_eq!(surface_code.distance, 3);
        assert!(surface_code.validate().expect("Validation failed"));
    }

    #[test]
    fn test_coordinator_creation() {
        let coordinator = create_coordinator(3).expect("Failed to create coordinator");
        assert_eq!(coordinator.get_surface_code().distance, 3);
    }

    #[test]
    fn test_parameter_calculation() {
        let (distance, boundary_type) = calculate_optimal_parameters(0.001);
        assert_eq!(distance, 9);
        assert_eq!(boundary_type, BoundaryType::Open);
    }

    #[test]
    fn test_factory_methods() {
        let coordinator = SurfaceCodeFactory::create_low_error_rate_code(0.001)
            .expect("Failed to create low error rate code");
        assert!(coordinator.get_surface_code().distance >= 7);
        
        let coordinator = SurfaceCodeFactory::create_high_throughput_code(5)
            .expect("Failed to create high throughput code");
        assert_eq!(coordinator.get_surface_code().distance, 5);
    }

    #[test]
    fn test_utils() {
        let threshold = SurfaceCodeUtils::theoretical_threshold(BoundaryType::Open);
        assert!((threshold - 0.109).abs() < 1e-6);
        
        let qubits = SurfaceCodeUtils::estimate_physical_qubits(1, 3);
        assert_eq!(qubits, 9);
    }
}

/// Re-export macros for ergonomic usage
pub use surface_code::{
    surface_code, surface_code_coordinator,
    apply_logical_x, apply_logical_z,
    measure_logical_x, measure_logical_z,
};
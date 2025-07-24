//! Surface code coordination module
//!
//! This module provides a unified interface for all surface code functionality,
//! integrating syndrome detection, correction algorithms, layout management,
//! and logical operations with blazing-fast performance and zero allocation optimizations.

pub mod syndrome_detection;
pub mod correction_algorithms;
pub mod layout_management;
pub mod logical_operations;

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::HashMap;
use std::time::Instant;

// Re-export key types for ergonomic API
pub use syndrome_detection::{
    QubitPosition, PauliType, PauliOperator, StabilizerType, StabilizerGenerator,
    SurfaceCodeSyndrome, SyndromeDetector, SyndromeDetectionConfig, SyndromeDetectionMetrics,
};
pub use correction_algorithms::{
    LogicalError, LogicalErrorType, SurfaceCodeCorrection, CorrectionAlgorithm,
    ErrorChain, ChainType, SurfaceCodeCorrector, CorrectionMetrics, CorrectionConfig,
};
pub use layout_management::{
    BoundaryType, SurfaceCodeLayout, QubitType, LayoutMetrics,
    SurfaceCodeLayoutBuilder, OptimizationTarget, LayoutConstraints,
};
pub use logical_operations::{
    LogicalOperatorType, LogicalOperator, LogicalOperators, LogicalOperatorMetrics,
    LogicalOperationsEngine, OperationMetrics, LogicalOperationConfig,
};

/// Comprehensive surface code implementation with optimized error correction
#[derive(Debug, Clone)]
pub struct SurfaceCode {
    /// Code distance (determines error correction capability)
    pub distance: usize,
    /// Physical qubit layout in 2D grid
    pub qubit_layout: SurfaceCodeLayout,
    /// X-type stabilizer generators
    pub x_stabilizers: Vec<StabilizerGenerator>,
    /// Z-type stabilizer generators  
    pub z_stabilizers: Vec<StabilizerGenerator>,
    /// Logical operators
    pub logical_operators: LogicalOperators,
    /// Error threshold for this code
    pub error_threshold: f64,
}

impl SurfaceCode {
    /// Create new surface code with specified distance and boundary type
    #[inline]
    pub fn new(distance: usize, boundary_type: BoundaryType) -> CognitiveResult<Self> {
        // Calculate optimal dimensions for given distance
        let dimensions = Self::calculate_optimal_dimensions(distance, boundary_type);
        
        // Create layout
        let qubit_layout = SurfaceCodeLayout::new(dimensions, boundary_type)?;
        
        // Generate stabilizers
        let (x_stabilizers, z_stabilizers) = qubit_layout.generate_stabilizers()?;
        
        // Create logical operations engine and extract operators
        let logical_engine = LogicalOperationsEngine::new(qubit_layout.clone())?;
        let logical_operators = logical_engine.get_logical_operators().clone();
        
        // Calculate error threshold based on boundary type
        let error_threshold = Self::calculate_error_threshold(boundary_type, distance);
        
        Ok(Self {
            distance,
            qubit_layout,
            x_stabilizers,
            z_stabilizers,
            logical_operators,
            error_threshold,
        })
    }

    /// Create surface code with custom layout
    #[inline]
    pub fn with_layout(layout: SurfaceCodeLayout, distance: usize) -> CognitiveResult<Self> {
        let (x_stabilizers, z_stabilizers) = layout.generate_stabilizers()?;
        let logical_engine = LogicalOperationsEngine::new(layout.clone())?;
        let logical_operators = logical_engine.get_logical_operators().clone();
        let error_threshold = Self::calculate_error_threshold(layout.boundary_type, distance);
        
        Ok(Self {
            distance,
            qubit_layout: layout,
            x_stabilizers,
            z_stabilizers,
            logical_operators,
            error_threshold,
        })
    }

    /// Calculate optimal dimensions for given distance and boundary type
    #[inline]
    fn calculate_optimal_dimensions(distance: usize, boundary_type: BoundaryType) -> (usize, usize) {
        match boundary_type {
            BoundaryType::Open => {
                // For open boundaries, use square grid slightly larger than distance
                let size = distance * 2 + 1;
                (size, size)
            }
            BoundaryType::Periodic => {
                // For periodic boundaries, use distance as grid size
                (distance, distance)
            }
            BoundaryType::Twisted => {
                // For twisted boundaries, use similar to periodic
                (distance, distance)
            }
        }
    }

    /// Calculate error threshold based on boundary type and distance
    #[inline]
    fn calculate_error_threshold(boundary_type: BoundaryType, distance: usize) -> f64 {
        let base_threshold = match boundary_type {
            BoundaryType::Open => 0.109, // Approximate threshold for surface code
            BoundaryType::Periodic => 0.103,
            BoundaryType::Twisted => 0.097,
        };
        
        // Adjust threshold based on distance (larger distance = higher threshold)
        let distance_factor = 1.0 + (distance as f64 - 3.0) * 0.01;
        base_threshold * distance_factor.max(1.0)
    }

    /// Get total number of physical qubits
    #[inline]
    pub fn total_qubits(&self) -> usize {
        self.qubit_layout.get_metrics().total_qubits
    }

    /// Get number of logical qubits
    #[inline]
    pub fn logical_qubit_count(&self) -> usize {
        self.logical_operators.logical_qubit_count
    }

    /// Get code parameters as tuple (n, k, d) where:
    /// n = number of physical qubits
    /// k = number of logical qubits  
    /// d = code distance
    #[inline]
    pub fn code_parameters(&self) -> (usize, usize, usize) {
        (
            self.total_qubits(),
            self.logical_qubit_count(),
            self.distance,
        )
    }

    /// Check if code can correct given number of errors
    #[inline]
    pub fn can_correct_errors(&self, num_errors: usize) -> bool {
        num_errors <= self.distance / 2
    }

    /// Get error correction capability (maximum correctable errors)
    #[inline]
    pub fn error_correction_capability(&self) -> usize {
        self.distance / 2
    }

    /// Check if error rate is below threshold
    #[inline]
    pub fn is_below_threshold(&self, error_rate: f64) -> bool {
        error_rate < self.error_threshold
    }

    /// Validate surface code structure
    #[inline]
    pub fn validate(&self) -> CognitiveResult<bool> {
        // Validate layout
        if !self.qubit_layout.validate()? {
            return Ok(false);
        }
        
        // Validate stabilizers
        for stabilizer in &self.x_stabilizers {
            if !stabilizer.is_valid() {
                return Ok(false);
            }
        }
        
        for stabilizer in &self.z_stabilizers {
            if !stabilizer.is_valid() {
                return Ok(false);
            }
        }
        
        // Validate logical operators
        if !self.logical_operators.validate()? {
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Get surface code information as string
    #[inline]
    pub fn info_string(&self) -> String {
        let (n, k, d) = self.code_parameters();
        format!(
            "Surface Code [[{}, {}, {}]] with {} boundary, threshold: {:.3}",
            n, k, d,
            self.qubit_layout.boundary_type.as_str(),
            self.error_threshold
        )
    }
}

/// High-level surface code coordinator providing ergonomic APIs
pub struct SurfaceCodeCoordinator {
    /// Surface code instance
    surface_code: SurfaceCode,
    /// Syndrome detector
    syndrome_detector: SyndromeDetector,
    /// Error corrector
    error_corrector: SurfaceCodeCorrector,
    /// Logical operations engine
    logical_engine: LogicalOperationsEngine,
    /// Coordinator metrics
    metrics: CoordinatorMetrics,
    /// Configuration
    config: SurfaceCodeConfig,
}

impl SurfaceCodeCoordinator {
    /// Create new surface code coordinator
    #[inline]
    pub fn new(distance: usize, boundary_type: BoundaryType) -> CognitiveResult<Self> {
        let surface_code = SurfaceCode::new(distance, boundary_type)?;
        
        let syndrome_detector = SyndromeDetector::new(
            surface_code.x_stabilizers.clone(),
            surface_code.z_stabilizers.clone(),
        );
        
        let error_corrector = SurfaceCodeCorrector::new(
            distance,
            surface_code.qubit_layout.dimensions,
        );
        
        let logical_engine = LogicalOperationsEngine::new(surface_code.qubit_layout.clone())?;
        
        Ok(Self {
            surface_code,
            syndrome_detector,
            error_corrector,
            logical_engine,
            metrics: CoordinatorMetrics::new(),
            config: SurfaceCodeConfig::default(),
        })
    }

    /// Create coordinator with custom configuration
    #[inline]
    pub fn with_config(
        distance: usize,
        boundary_type: BoundaryType,
        config: SurfaceCodeConfig,
    ) -> CognitiveResult<Self> {
        let mut coordinator = Self::new(distance, boundary_type)?;
        coordinator.config = config;
        
        // Apply configuration to subcomponents
        coordinator.syndrome_detector.update_config(config.syndrome_detection);
        coordinator.error_corrector.update_config(config.error_correction);
        coordinator.logical_engine.update_config(config.logical_operations);
        
        Ok(coordinator)
    }

    /// Perform complete error correction cycle
    #[inline]
    pub fn error_correction_cycle(
        &mut self,
        error_pattern: &HashMap<QubitPosition, PauliType>,
        round: usize,
    ) -> CognitiveResult<SurfaceCodeCorrection> {
        let start_time = Instant::now();
        
        // Detect syndrome
        let syndrome = self.syndrome_detector.detect_syndrome(error_pattern, round)?;
        
        // Correct errors
        let correction = self.error_corrector.correct_errors(&syndrome)?;
        
        let cycle_time = start_time.elapsed();
        self.metrics.record_correction_cycle(
            cycle_time,
            syndrome.weight(),
            correction.weight(),
            correction.is_successful(),
        );
        
        Ok(correction)
    }

    /// Apply logical operation
    #[inline]
    pub fn apply_logical_operation(
        &mut self,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
        state: &mut HashMap<QubitPosition, Complex64>,
    ) -> CognitiveResult<()> {
        self.logical_engine.apply_logical_operation(operator_type, logical_qubit_index, state)
    }

    /// Measure logical observable
    #[inline]
    pub fn measure_logical_observable(
        &mut self,
        operator_type: LogicalOperatorType,
        logical_qubit_index: usize,
        state: &HashMap<QubitPosition, Complex64>,
    ) -> CognitiveResult<f64> {
        self.logical_engine.measure_logical_observable(operator_type, logical_qubit_index, state)
    }

    /// Get surface code instance
    #[inline]
    pub fn get_surface_code(&self) -> &SurfaceCode {
        &self.surface_code
    }

    /// Get coordinator metrics
    #[inline]
    pub fn get_metrics(&self) -> &CoordinatorMetrics {
        &self.metrics
    }

    /// Get comprehensive performance report
    #[inline]
    pub fn get_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            surface_code_info: self.surface_code.info_string(),
            coordinator_metrics: self.metrics.clone(),
            syndrome_metrics: self.syndrome_detector.get_metrics().clone(),
            correction_metrics: self.error_corrector.get_metrics().clone(),
            logical_metrics: self.logical_engine.get_metrics().clone(),
            layout_metrics: self.surface_code.qubit_layout.get_metrics().clone(),
            logical_operator_metrics: self.surface_code.logical_operators.get_metrics().clone(),
        }
    }

    /// Update configuration
    #[inline]
    pub fn update_config(&mut self, config: SurfaceCodeConfig) {
        self.config = config;
        self.syndrome_detector.update_config(config.syndrome_detection);
        self.error_corrector.update_config(config.error_correction);
        self.logical_engine.update_config(config.logical_operations);
    }

    /// Validate entire surface code system
    #[inline]
    pub fn validate(&self) -> CognitiveResult<bool> {
        self.surface_code.validate()
    }
}

/// Coordinator performance metrics
#[derive(Debug, Clone)]
pub struct CoordinatorMetrics {
    /// Total correction cycles performed
    pub total_correction_cycles: u64,
    /// Successful correction cycles
    pub successful_correction_cycles: u64,
    /// Total cycle time
    pub total_cycle_time_ms: u64,
    /// Average syndrome weight
    pub average_syndrome_weight: f64,
    /// Average correction weight
    pub average_correction_weight: f64,
    /// Creation time
    pub creation_time: Instant,
}

impl CoordinatorMetrics {
    /// Create new metrics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_correction_cycles: 0,
            successful_correction_cycles: 0,
            total_cycle_time_ms: 0,
            average_syndrome_weight: 0.0,
            average_correction_weight: 0.0,
            creation_time: Instant::now(),
        }
    }

    /// Record correction cycle
    #[inline]
    pub fn record_correction_cycle(
        &mut self,
        cycle_time: std::time::Duration,
        syndrome_weight: usize,
        correction_weight: usize,
        successful: bool,
    ) {
        self.total_correction_cycles += 1;
        self.total_cycle_time_ms += cycle_time.as_millis() as u64;
        
        if successful {
            self.successful_correction_cycles += 1;
        }
        
        // Update averages with exponential moving average
        let alpha = 0.1;
        self.average_syndrome_weight = alpha * syndrome_weight as f64 + 
            (1.0 - alpha) * self.average_syndrome_weight;
        self.average_correction_weight = alpha * correction_weight as f64 + 
            (1.0 - alpha) * self.average_correction_weight;
    }

    /// Get success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.total_correction_cycles > 0 {
            self.successful_correction_cycles as f64 / self.total_correction_cycles as f64
        } else {
            0.0
        }
    }

    /// Get average cycle time
    #[inline]
    pub fn average_cycle_time_ms(&self) -> f64 {
        if self.total_correction_cycles > 0 {
            self.total_cycle_time_ms as f64 / self.total_correction_cycles as f64
        } else {
            0.0
        }
    }

    /// Get cycles per second
    #[inline]
    pub fn cycles_per_second(&self) -> f64 {
        let age_seconds = self.creation_time.elapsed().as_secs_f64();
        if age_seconds > 0.0 {
            self.total_correction_cycles as f64 / age_seconds
        } else {
            0.0
        }
    }
}

impl Default for CoordinatorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive surface code configuration
#[derive(Debug, Clone)]
pub struct SurfaceCodeConfig {
    /// Syndrome detection configuration
    pub syndrome_detection: SyndromeDetectionConfig,
    /// Error correction configuration
    pub error_correction: CorrectionConfig,
    /// Logical operations configuration
    pub logical_operations: LogicalOperationConfig,
    /// Enable performance optimizations
    pub enable_optimizations: bool,
    /// Enable detailed logging
    pub enable_logging: bool,
}

impl SurfaceCodeConfig {
    /// Create new configuration
    #[inline]
    pub fn new() -> Self {
        Self {
            syndrome_detection: SyndromeDetectionConfig::new(),
            error_correction: CorrectionConfig::new(),
            logical_operations: LogicalOperationConfig::new(),
            enable_optimizations: false,
            enable_logging: false,
        }
    }

    /// Create performance-optimized configuration
    #[inline]
    pub fn performance_optimized() -> Self {
        Self {
            syndrome_detection: SyndromeDetectionConfig::performance_optimized(),
            error_correction: CorrectionConfig::new(),
            logical_operations: LogicalOperationConfig::performance_optimized(),
            enable_optimizations: true,
            enable_logging: false,
        }
    }
}

impl Default for SurfaceCodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Surface code information
    pub surface_code_info: String,
    /// Coordinator metrics
    pub coordinator_metrics: CoordinatorMetrics,
    /// Syndrome detection metrics
    pub syndrome_metrics: SyndromeDetectionMetrics,
    /// Error correction metrics
    pub correction_metrics: CorrectionMetrics,
    /// Logical operation metrics
    pub logical_metrics: OperationMetrics,
    /// Layout metrics
    pub layout_metrics: LayoutMetrics,
    /// Logical operator metrics
    pub logical_operator_metrics: LogicalOperatorMetrics,
}

impl PerformanceReport {
    /// Get overall performance score
    #[inline]
    pub fn overall_performance_score(&self) -> f64 {
        let success_score = self.coordinator_metrics.success_rate();
        let speed_score = 1.0 / (1.0 + self.coordinator_metrics.average_cycle_time_ms() / 1000.0);
        let efficiency_score = self.layout_metrics.quality_score();
        
        (success_score * 0.5 + speed_score * 0.3 + efficiency_score * 0.2).clamp(0.0, 1.0)
    }

    /// Generate summary string
    #[inline]
    pub fn summary(&self) -> String {
        format!(
            "{}\nSuccess Rate: {:.2}%, Avg Cycle Time: {:.2}ms, Performance Score: {:.3}",
            self.surface_code_info,
            self.coordinator_metrics.success_rate() * 100.0,
            self.coordinator_metrics.average_cycle_time_ms(),
            self.overall_performance_score()
        )
    }
}

/// Ergonomic macros for surface code operations
#[macro_export]
macro_rules! surface_code {
    ($distance:expr) => {
        SurfaceCode::new($distance, BoundaryType::Open)
    };
    ($distance:expr, $boundary:expr) => {
        SurfaceCode::new($distance, $boundary)
    };
}

#[macro_export]
macro_rules! surface_code_coordinator {
    ($distance:expr) => {
        SurfaceCodeCoordinator::new($distance, BoundaryType::Open)
    };
    ($distance:expr, $boundary:expr) => {
        SurfaceCodeCoordinator::new($distance, $boundary)
    };
    ($distance:expr, $boundary:expr, $config:expr) => {
        SurfaceCodeCoordinator::with_config($distance, $boundary, $config)
    };
}

#[macro_export]
macro_rules! apply_logical_x {
    ($coordinator:expr, $qubit:expr, $state:expr) => {
        $coordinator.apply_logical_operation(LogicalOperatorType::X, $qubit, $state)
    };
}

#[macro_export]
macro_rules! apply_logical_z {
    ($coordinator:expr, $qubit:expr, $state:expr) => {
        $coordinator.apply_logical_operation(LogicalOperatorType::Z, $qubit, $state)
    };
}

#[macro_export]
macro_rules! measure_logical_x {
    ($coordinator:expr, $qubit:expr, $state:expr) => {
        $coordinator.measure_logical_observable(LogicalOperatorType::X, $qubit, $state)
    };
}

#[macro_export]
macro_rules! measure_logical_z {
    ($coordinator:expr, $qubit:expr, $state:expr) => {
        $coordinator.measure_logical_observable(LogicalOperatorType::Z, $qubit, $state)
    };
}
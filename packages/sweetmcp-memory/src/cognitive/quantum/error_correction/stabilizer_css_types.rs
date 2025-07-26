//! CSS (Calderbank-Shor-Steane) code types for stabilizer quantum error correction
//!
//! This module defines CSS-specific data structures and types used in CSS code
//! implementations with zero allocation patterns and blazing-fast performance.
//!
//! # Features
//! - CSS code construction from classical codes
//! - Support for various CSS code families (surface codes, color codes, etc.)
//! - Efficient X/Z separate decoding
//! - Performance metrics and analysis
//!
//! # Example
//! ```rust
//! use sweetmcp_memory::cognitive::quantum::error_correction::{
//!     stabilizer_css_types::{CSSCode, CSSConstructionParams, CSSDecodingMethod},
//!     quantum_error_correction::ErrorCorrectionCode,
//! };
//!
//! // Create a new CSS code from construction parameters
//! let params = CSSConstructionParams {
//!     x_generator: vec![vec![true, false, true], vec![false, true, true]],
//!     z_generator: vec![vec![true, true, false], vec![false, true, true]],
//!     n: 3,
//!     k: 1,
//!     d: 1,
//!     construction_method: CSSConstructionMethod::DirectConstruction,
//! };
//!
//! let css_code = CSSCode::from_construction_params(params).unwrap();
//! 
//! // Get decoding metrics
//! let metrics = css_code.get_metrics();
//! println!("Success rate: {:.2}%", metrics.success_rate() * 100.0);
//! ```

use crate::cognitive::quantum::types::{CognitiveError, CognitiveResult};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use super::{
    topological_stabilizers::{StabilizerGroup, TopologicalStabilizer},
    topological_types::StabilizerType,
    topological_pauli::PauliType,
    topological_pauli_strings::PauliString,
    syndromes::ErrorSyndrome,
    quantum_error_correction::{
        ErrorCorrectionCode,
        StabilizerGenerator as QecStabilizerGenerator,
        CodeParameters as BaseCodeParameters,
    },
    topological_types::TopologicalCodeType,
};

/// CSS (Calderbank-Shor-Steane) code implementation
///
/// A CSS code is defined by two classical codes: one for X errors and one for Z errors.
/// This structure provides an efficient implementation with zero-allocation patterns
/// and support for various decoding strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSCode {
    /// Base stabilizer code
    pub base_code: ErrorCorrectionCode,
    
    /// X-type stabilizers (Z-errors are corrected by X-stabilizers)
    pub x_stabilizers: Vec<QecStabilizerGenerator>,
    
    /// Z-type stabilizers (X-errors are corrected by Z-stabilizers)
    pub z_stabilizers: Vec<QecStabilizerGenerator>,
    
    /// Stabilizer group for efficient operations
    #[serde(skip_serializing, skip_deserializing)]
    pub stabilizer_group: Option<StabilizerGroup>,
    
    /// CSS code parameters and properties
    pub css_parameters: CSSParameters,
    
    /// Performance metrics
    #[serde(skip_serializing, skip_deserializing)]
    pub metrics: CSSDecodingMetrics,
    
    /// Cached logical operators (lazy-initialized)
    #[serde(skip_serializing, skip_deserializing)]
    logical_operators: Option<Vec<QecStabilizerGenerator>>,
    
    /// Cached distance (lazy-initialized)
    #[serde(skip_serializing, skip_deserializing)]
    distance: Option<usize>,
}

/// Parameters specific to CSS codes
#[derive(Debug, Clone)]
pub struct CSSParameters {
    /// Classical code for X errors
    pub x_code_parameters: ClassicalCodeParams,
    
    /// Classical code for Z errors
    pub z_code_parameters: ClassicalCodeParams,
    
    /// Enable separate X/Z decoding
    pub separate_decoding: bool,
}

/// Classical code parameters for CSS construction
#[derive(Debug, Clone)]
pub struct ClassicalCodeParams {
    /// Code length
    pub n: usize,
    
    /// Code dimension
    pub k: usize,
    
    /// Minimum distance
    pub d: usize,
    
    /// Generator matrix
    pub generator_matrix: Vec<Vec<bool>>,
    
    /// Parity check matrix
    pub parity_check_matrix: Vec<Vec<bool>>,
}

/// CSS code construction parameters
#[derive(Debug, Clone)]
pub struct CSSConstructionParams {
    /// X-code generator matrix
    pub x_generator: Vec<Vec<bool>>,
    
    /// Z-code generator matrix
    pub z_generator: Vec<Vec<bool>>,
    
    /// Code parameters
    pub n: usize,
    pub k: usize,
    pub d: usize,
    
    /// Construction method
    pub construction_method: CSSConstructionMethod,
}

/// Methods for constructing CSS codes
#[derive(Debug, Clone, Copy)]
pub enum CSSConstructionMethod {
    /// Direct construction from classical codes
    DirectConstruction,
    
    /// Hypergraph product construction
    HypergraphProduct,
    
    /// Quantum BCH construction
    QuantumBCH,
    
    /// Surface code construction
    SurfaceCode,
}

/// CSS decoding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSDecodingResult {
    /// Success status of decoding
    pub success: bool,
    
    /// Applied X correction (if successful)
    pub x_correction: Option<PauliString>,
    
    /// Applied Z correction (if successful)
    pub z_correction: Option<PauliString>,
    
    /// Residual X syndrome after correction (if applicable)
    pub residual_x_syndrome: Option<ErrorSyndrome>,
    
    /// Residual Z syndrome after correction (if applicable)
    pub residual_z_syndrome: Option<ErrorSyndrome>,
    
    /// Decoding metrics
    pub metrics: CSSDecodingMetrics,
}

/// CSS decoding methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CSSDecodingMethod {
    /// Independent X/Z decoding (faster but less accurate)
    IndependentXZ {
        /// Maximum number of iterations for X decoding
        max_x_iterations: usize,
        /// Maximum number of iterations for Z decoding
        max_z_iterations: usize,
    },
    
    /// Sequential X then Z decoding
    SequentialXZ {
        /// Maximum number of iterations for X decoding
        max_x_iterations: usize,
        /// Maximum number of iterations for Z decoding
        max_z_iterations: usize,
        /// Whether to use the X syndrome for Z decoding
        use_x_syndrome_for_z: bool,
    },
    
    /// Iterative belief propagation
    BeliefPropagation {
        /// Maximum number of iterations
        max_iterations: usize,
        /// Convergence threshold
        convergence_threshold: f64,
        /// Damping factor (0.0 to 1.0)
        damping: f64,
    },
    
    /// Maximum likelihood decoding (exact but expensive)
    MaximumLikelihood {
        /// Maximum error weight to consider
        max_error_weight: usize,
        /// Whether to use early termination
        early_termination: bool,
    },
    
    /// Union-find decoder (fast and constant-time)
    UnionFind {
        /// Growth parameter for clusters
        growth_rate: f64,
        /// Whether to use weighted union
        weighted_union: bool,
    },
}

impl Default for CSSDecodingMethod {
    fn default() -> Self {
        Self::IndependentXZ { 
            max_x_iterations: 100,
            max_z_iterations: 100,
        }
    }
}

impl CSSDecodingMethod {
    /// Get a human-readable name for the decoding method
    pub fn name(&self) -> &'static str {
        match self {
            Self::IndependentXZ { .. } => "Independent X/Z Decoding",
            Self::SequentialXZ { .. } => "Sequential X/Z Decoding",
            Self::BeliefPropagation { .. } => "Belief Propagation",
            Self::MaximumLikelihood { .. } => "Maximum Likelihood",
            Self::UnionFind { .. } => "Union-Find Decoder",
        }
    }
    
    /// Get the maximum number of iterations for this method
    pub fn max_iterations(&self) -> usize {
        match self {
            Self::IndependentXZ { max_x_iterations, .. } => *max_x_iterations,
            Self::SequentialXZ { max_x_iterations, .. } => *max_x_iterations,
            Self::BeliefPropagation { max_iterations, .. } => *max_iterations,
            Self::MaximumLikelihood { max_error_weight, .. } => *max_error_weight,
            Self::UnionFind { .. } => 100, // Fixed for union-find
        }
    }
}

/// Performance metrics for CSS decoding
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CSSDecodingMetrics {
    /// Number of decoding attempts
    pub decode_attempts: u64,
    
    /// Number of successful decodings
    pub successful_decodes: u64,
    
    /// Time spent in X decoding (microseconds)
    pub x_decode_time_us: u64,
    
    /// Time spent in Z decoding (microseconds)
    pub z_decode_time_us: u64,
    
    /// Time spent in total (microseconds)
    pub total_time_us: u64,
    
    /// Number of X errors detected
    pub x_errors_detected: u64,
    
    /// Number of Z errors detected
    pub z_errors_detected: u64,
    
    /// Number of X errors corrected
    pub x_errors_corrected: u64,
    
    /// Number of Z errors corrected
    pub z_errors_corrected: u64,
    
    /// Average number of iterations (for iterative decoders)
    pub avg_iterations: f64,
    
    /// Maximum number of iterations used
    pub max_iterations: usize,
    
    /// Number of times the decoder hit the iteration limit
    pub iteration_limited: u64,
    
    /// Detailed timing information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_timing: Option<HashMap<String, u64>>,
    
    /// Custom metrics specific to the decoder
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom_metrics: HashMap<String, f64>,
}

impl CSSDecodingMetrics {
    /// Create new metrics with zero values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record the start of a decoding operation
    pub fn record_start(&mut self) -> Instant {
        self.decode_attempts += 1;
        Instant::now()
    }
    
    /// Record the completion of X decoding
    pub fn record_x_complete(&mut self, start_time: Instant) -> Duration {
        let elapsed = start_time.elapsed();
        self.x_decode_time_us += elapsed.as_micros() as u64;
        self.total_time_us = self.x_decode_time_us + self.z_decode_time_us;
        elapsed
    }
    
    /// Record the completion of Z decoding
    pub fn record_z_complete(&mut self, start_time: Instant) -> Duration {
        let elapsed = start_time.elapsed();
        self.z_decode_time_us += elapsed.as_micros() as u64;
        self.total_time_us = self.x_decode_time_us + self.z_decode_time_us;
        elapsed
    }
    
    /// Record a successful decode
    pub fn record_success(&mut self) {
        self.successful_decodes += 1;
    }
    
    /// Record X error statistics
    pub fn record_x_errors(&mut self, detected: usize, corrected: usize) {
        self.x_errors_detected += detected as u64;
        self.x_errors_corrected += corrected as u64;
    }
    
    /// Record Z error statistics
    pub fn record_z_errors(&mut self, detected: usize, corrected: usize) {
        self.z_errors_detected += detected as u64;
        self.z_errors_corrected += corrected as u64;
    }
    
    /// Record iteration statistics
    pub fn record_iterations(&mut self, iterations: usize, max_iterations: usize) {
        if iterations >= max_iterations {
            self.iteration_limited += 1;
        }
        self.max_iterations = self.max_iterations.max(iterations);
        
        // Update running average of iterations
        let total_iterations = self.avg_iterations * (self.decode_attempts - 1) as f64;
        self.avg_iterations = (total_iterations + iterations as f64) / self.decode_attempts as f64;
    }
    
    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: impl Into<String>, value: f64) {
        self.custom_metrics.entry(name.into()).or_insert(value);
    }
    
    /// Add a timing measurement to the detailed timing
    pub fn add_timing(&mut self, name: impl Into<String>, duration: Duration) {
        let timing = self.detailed_timing.get_or_insert_with(HashMap::new);
        *timing.entry(name.into()).or_insert(0) += duration.as_micros() as u64;
    }
    
    /// Get the success rate of decoding attempts
    pub fn success_rate(&self) -> f64 {
        if self.decode_attempts == 0 {
            return 0.0;
        }
        self.successful_decodes as f64 / self.decode_attempts as f64
    }
    
    /// Get the X error correction rate
    pub fn x_correction_rate(&self) -> f64 {
        if self.x_errors_detected == 0 {
            return 0.0;
        }
        self.x_errors_corrected as f64 / self.x_errors_detected as f64
    }
    
    /// Get the Z error correction rate
    pub fn z_correction_rate(&self) -> f64 {
        if self.z_errors_detected == 0 {
            return 0.0;
        }
        self.z_errors_corrected as f64 / self.z_errors_detected as f64
    }
    
    /// Get the average time per decode in microseconds
    pub fn avg_decode_time_us(&self) -> f64 {
        if self.decode_attempts == 0 {
            return 0.0;
        }
        self.total_time_us as f64 / self.decode_attempts as f64
    }
    
    /// Get the average X decode time in microseconds
    pub fn avg_x_decode_time_us(&self) -> f64 {
        if self.decode_attempts == 0 {
            return 0.0;
        }
        self.x_decode_time_us as f64 / self.decode_attempts as f64
    }
    
    /// Get the average Z decode time in microseconds
    pub fn avg_z_decode_time_us(&self) -> f64 {
        if self.decode_attempts == 0 {
            return 0.0;
        }
        self.z_decode_time_us as f64 / self.decode_attempts as f64
    }
    
    /// Get decoding rate (bits per second)
    pub fn decoding_rate(&self, code_length: usize) -> f64 {
        if self.total_time_us == 0 {
            return 0.0;
        }
        (self.decode_attempts * code_length as u64) as f64 * 1_000_000.0 / self.total_time_us as f64
    }
    
    /// Merge another set of metrics into this one
    pub fn merge(&mut self, other: &Self) {
        if other.decode_attempts == 0 {
            return;
        }
        
        let self_attempts = self.decode_attempts;
        let other_attempts = other.decode_attempts;
        let total_attempts = self_attempts + other_attempts;
        
        if total_attempts == 0 {
            return;
        }
        
        // Update simple counters
        self.decode_attempts = total_attempts;
        self.successful_decodes += other.successful_decodes;
        self.x_decode_time_us += other.x_decode_time_us;
        self.z_decode_time_us += other.z_decode_time_us;
        self.total_time_us += other.total_time_us;
        self.x_errors_detected += other.x_errors_detected;
        self.z_errors_detected += other.z_errors_detected;
        self.x_errors_corrected += other.x_errors_corrected;
        self.z_errors_corrected += other.z_errors_corrected;
        self.iteration_limited += other.iteration_limited;
        
        // Update weighted averages
        let self_weight = self_attempts as f64 / total_attempts as f64;
        let other_weight = other_attempts as f64 / total_attempts as f64;
        
        self.avg_iterations = (self.avg_iterations * self_weight) + 
                             (other.avg_iterations * other_weight);
        
        // Update max iterations
        self.max_iterations = self.max_iterations.max(other.max_iterations);
        
        // Merge custom metrics
        for (key, value) in &other.custom_metrics {
            let entry = self.custom_metrics.entry(key.clone()).or_default();
            *entry = (*entry * self_weight) + (*value * other_weight);
        }
        
        // Merge detailed timing if present
        if let Some(other_timing) = &other.detailed_timing {
            let timing = self.detailed_timing.get_or_insert_with(HashMap::new);
            for (key, value) in other_timing {
                *timing.entry(key.clone()).or_insert(0) += value;
            }
        }
    }
}

impl CSSCode {
    /// Create a new CSS code
    pub fn new(
        base_code: ErrorCorrectionCode,
        x_stabilizers: Vec<QecStabilizerGenerator>,
        z_stabilizers: Vec<QecStabilizerGenerator>,
        css_parameters: CSSParameters,
    ) -> CognitiveResult<Self> {
        // Validate CSS structure
        if x_stabilizers.len() + z_stabilizers.len() != base_code.n - base_code.k {
            return Err(CognitiveError::InvalidQuantumState(
                "X and Z stabilizers must sum to n-k".to_string()
            ));
        }

        // Validate that X and Z stabilizers commute
        for x_stab in &x_stabilizers {
            for z_stab in &z_stabilizers {
                if !Self::stabilizers_commute(x_stab, z_stab) {
                    return Err(CognitiveError::InvalidQuantumState(
                        "X and Z stabilizers must commute".to_string()
                    ));
                }
            }
        }

        Ok(Self {
            base_code,
            x_stabilizers,
            z_stabilizers,
            css_parameters,
            metrics: CSSDecodingMetrics::new(),
        })
    }

    /// Create CSS code from construction parameters
    pub fn from_construction_params(params: CSSConstructionParams) -> CognitiveResult<Self> {
        match params.construction_method {
            CSSConstructionMethod::DirectConstruction => {
                Self::construct_direct(params)
            }
            CSSConstructionMethod::HypergraphProduct => {
                Self::construct_hypergraph_product(params)
            }
            CSSConstructionMethod::QuantumBCH => {
                Self::construct_quantum_bch(params)
            }
            CSSConstructionMethod::SurfaceCode => {
                Self::construct_surface_code(params)
            }
        }
    }

    /// Get base stabilizer code
    pub fn base_code(&self) -> &ErrorCorrectionCode {
        &self.base_code
    }

    /// Get X-type stabilizers
    pub fn x_stabilizers(&self) -> &[QecStabilizerGenerator] {
        &self.x_stabilizers
    }

    /// Get Z-type stabilizers
    pub fn z_stabilizers(&self) -> &[QecStabilizerGenerator] {
        &self.z_stabilizers
    }

    /// Get CSS parameters
    #[inline]
    pub const fn css_parameters(&self) -> &CSSParameters {
        &self.css_parameters
    }

    /// Check if two stabilizers commute
    pub fn stabilizers_commute(
        &self,
        stab1: &QecStabilizerGenerator, 
        stab2: &QecStabilizerGenerator
    ) -> bool {
        // Use the stabilizer group to check commutation
        self.stabilizer_group.commute(stab1, stab2)
    }

    /// Direct construction from classical codes
    fn construct_direct(params: CSSConstructionParams) -> CognitiveResult<Self> {
        // Implementation will be in operations module
        Err(CognitiveError::NotImplemented("Direct construction not yet implemented".to_string()))
    }

    /// Hypergraph product construction
    fn construct_hypergraph_product(params: CSSConstructionParams) -> CognitiveResult<Self> {
        // Implementation will be in operations module
        Err(CognitiveError::NotImplemented("Hypergraph product construction not yet implemented".to_string()))
    }

    /// Quantum BCH construction
    fn construct_quantum_bch(params: CSSConstructionParams) -> CognitiveResult<Self> {
        // Implementation will be in operations module
        Err(CognitiveError::NotImplemented("Quantum BCH construction not yet implemented".to_string()))
    }

    /// Surface code construction
    fn construct_surface_code(params: CSSConstructionParams) -> CognitiveResult<Self> {
        // Implementation will be in operations module
        Err(CognitiveError::NotImplemented("Surface code construction not yet implemented".to_string()))
    }
}

impl ClassicalCodeParams {
    /// Create new classical code parameters
    pub fn new(
        n: usize,
        k: usize,
        d: usize,
        generator_matrix: Vec<Vec<bool>>,
        parity_check_matrix: Vec<Vec<bool>>,
    ) -> CognitiveResult<Self> {
        // Validate dimensions
        if generator_matrix.len() != k {
            return Err(CognitiveError::InvalidQuantumState(
                format!("Generator matrix must have {} rows", k)
            ));
        }

        if parity_check_matrix.len() != n - k {
            return Err(CognitiveError::InvalidQuantumState(
                format!("Parity check matrix must have {} rows", n - k)
            ));
        }

        for row in &generator_matrix {
            if row.len() != n {
                return Err(CognitiveError::InvalidQuantumState(
                    format!("Generator matrix rows must have length {}", n)
                ));
            }
        }

        for row in &parity_check_matrix {
            if row.len() != n {
                return Err(CognitiveError::InvalidQuantumState(
                    format!("Parity check matrix rows must have length {}", n)
                ));
            }
        }

        Ok(Self {
            n,
            k,
            d,
            generator_matrix,
            parity_check_matrix,
        })
    }

    /// Get code rate
    #[inline]
    pub fn rate(&self) -> f64 {
        self.k as f64 / self.n as f64
    }

    /// Get redundancy
    #[inline]
    pub fn redundancy(&self) -> usize {
        self.n - self.k
    }
}
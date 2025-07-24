//! Core data structures for stabilizer quantum error correction
//!
//! This module defines the fundamental types and structures used in stabilizer
//! code implementations, including stabilizer generators, logical operators,
//! error patterns, and CSS code structures with zero allocation patterns.

use crate::cognitive::quantum::{
    Complex64,
    types::{CognitiveError, CognitiveResult},
};
use std::collections::{HashMap, HashSet};
use smallvec::SmallVec;

/// Stabilizer code implementation with optimized operations
#[derive(Debug, Clone)]
pub struct StabilizerCode {
    /// Number of physical qubits
    pub n: usize,
    
    /// Number of logical qubits
    pub k: usize,
    
    /// Code distance
    pub d: usize,
    
    /// Stabilizer generators
    pub stabilizers: Vec<StabilizerGenerator>,
    
    /// Logical X operators
    pub logical_x: Vec<LogicalOperator>,
    
    /// Logical Z operators
    pub logical_z: Vec<LogicalOperator>,
    
    /// Syndrome lookup table for fast decoding
    pub syndrome_table: HashMap<Vec<bool>, ErrorPattern>,
    
    /// Code parameters
    pub parameters: StabilizerCodeParameters,
}

/// Parameters for stabilizer code configuration
#[derive(Debug, Clone)]
pub struct StabilizerCodeParameters {
    /// Error correction threshold
    pub error_threshold: f64,
    
    /// Maximum syndrome weight for correction
    pub max_syndrome_weight: usize,
    
    /// Enable fast syndrome lookup
    pub fast_lookup: bool,
    
    /// Decoder type
    pub decoder_type: DecoderType,
}

/// Types of decoders for stabilizer codes
#[derive(Debug, Clone, Copy)]
pub enum DecoderType {
    /// Table lookup decoder
    TableLookup,
    
    /// Syndrome-based decoder
    SyndromeBased,
    
    /// Maximum likelihood decoder
    MaximumLikelihood,
    
    /// Belief propagation decoder
    BeliefPropagation,
}

/// Stabilizer generator with optimized representation
#[derive(Debug, Clone)]
pub struct StabilizerGenerator {
    /// Generator identifier
    pub id: String,
    
    /// Pauli operators in this generator
    pub paulis: SmallVec<[PauliOp; 8]>,
    
    /// Generator weight (number of non-identity operators)
    pub weight: usize,
    
    /// Commutation relations with other generators
    pub commutation_matrix: Vec<bool>,
}

/// Pauli operator with position and type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PauliOp {
    /// Qubit index
    pub qubit: usize,
    
    /// Pauli type
    pub pauli: PauliMatrix,
    
    /// Phase factor (0, 1, 2, 3 for +1, +i, -1, -i)
    pub phase: u8,
}

/// Pauli matrix types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PauliMatrix {
    I, // Identity
    X, // Pauli-X
    Y, // Pauli-Y  
    Z, // Pauli-Z
}

/// Logical operator representation
#[derive(Debug, Clone)]
pub struct LogicalOperator {
    /// Operator identifier
    pub id: String,
    
    /// Pauli string
    pub paulis: SmallVec<[PauliOp; 16]>,
    
    /// Operator type
    pub operator_type: LogicalOpType,
    
    /// Logical qubit index
    pub logical_qubit: usize,
}

/// Type of logical operator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOpType {
    X,
    Z,
}

/// Error pattern for syndrome table lookup
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    /// Error locations
    pub error_qubits: SmallVec<[usize; 4]>,
    
    /// Error types at each location
    pub error_types: SmallVec<[PauliMatrix; 4]>,
    
    /// Pattern probability
    pub probability: f64,
    
    /// Correction operations
    pub corrections: SmallVec<[PauliOp; 4]>,
}

/// CSS (Calderbank-Shor-Steane) code implementation
#[derive(Debug, Clone)]
pub struct CSSCode {
    /// Base stabilizer code
    pub base_code: StabilizerCode,
    
    /// X-type stabilizers
    pub x_stabilizers: Vec<StabilizerGenerator>,
    
    /// Z-type stabilizers
    pub z_stabilizers: Vec<StabilizerGenerator>,
    
    /// CSS code properties
    pub css_parameters: CSSParameters,
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

/// Syndrome extraction result
#[derive(Debug, Clone)]
pub struct SyndromeResult {
    /// Syndrome vector
    pub syndrome: Vec<bool>,
    
    /// Syndrome weight
    pub weight: usize,
    
    /// Extraction timestamp
    pub timestamp: std::time::Instant,
    
    /// Measurement errors detected
    pub measurement_errors: Vec<usize>,
}

impl StabilizerCode {
    /// Create a new stabilizer code with validation
    pub fn new(
        n: usize,
        k: usize,
        d: usize,
        stabilizers: Vec<StabilizerGenerator>,
        logical_x: Vec<LogicalOperator>,
        logical_z: Vec<LogicalOperator>,
    ) -> CognitiveResult<Self> {
        // Validate code parameters
        if stabilizers.len() != n - k {
            return Err(CognitiveError::InvalidParameter(
                format!("Expected {} stabilizers for [[{}, {}, {}]] code", n - k, n, k, d)
            ));
        }

        if logical_x.len() != k || logical_z.len() != k {
            return Err(CognitiveError::InvalidParameter(
                format!("Expected {} logical X and Z operators", k)
            ));
        }

        let parameters = StabilizerCodeParameters {
            error_threshold: 0.01,
            max_syndrome_weight: n / 2,
            fast_lookup: true,
            decoder_type: DecoderType::SyndromeBased,
        };

        let mut code = Self {
            n,
            k,
            d,
            stabilizers,
            logical_x,
            logical_z,
            syndrome_table: HashMap::new(),
            parameters,
        };

        // Build syndrome table if fast lookup is enabled
        if code.parameters.fast_lookup {
            code.build_syndrome_table()?;
        }

        Ok(code)
    }

    /// Build syndrome lookup table for fast decoding
    fn build_syndrome_table(&mut self) -> CognitiveResult<()> {
        // Implementation will be in operations module
        Ok(())
    }

    /// Get code parameters
    #[inline]
    pub const fn parameters(&self) -> (usize, usize, usize) {
        (self.n, self.k, self.d)
    }

    /// Get number of stabilizers
    #[inline]
    pub fn num_stabilizers(&self) -> usize {
        self.stabilizers.len()
    }

    /// Get stabilizer by index
    #[inline]
    pub fn stabilizer(&self, index: usize) -> Option<&StabilizerGenerator> {
        self.stabilizers.get(index)
    }

    /// Get logical X operator by index
    #[inline]
    pub fn logical_x(&self, index: usize) -> Option<&LogicalOperator> {
        self.logical_x.get(index)
    }

    /// Get logical Z operator by index
    #[inline]
    pub fn logical_z(&self, index: usize) -> Option<&LogicalOperator> {
        self.logical_z.get(index)
    }
}

impl CSSCode {
    /// Create a new CSS code
    pub fn new(
        base_code: StabilizerCode,
        x_stabilizers: Vec<StabilizerGenerator>,
        z_stabilizers: Vec<StabilizerGenerator>,
        css_parameters: CSSParameters,
    ) -> CognitiveResult<Self> {
        // Validate CSS structure
        if x_stabilizers.len() + z_stabilizers.len() != base_code.n - base_code.k {
            return Err(CognitiveError::InvalidParameter(
                "X and Z stabilizers must sum to n-k".to_string()
            ));
        }

        Ok(Self {
            base_code,
            x_stabilizers,
            z_stabilizers,
            css_parameters,
        })
    }

    /// Get base stabilizer code
    #[inline]
    pub const fn base_code(&self) -> &StabilizerCode {
        &self.base_code
    }

    /// Get X-type stabilizers
    #[inline]
    pub fn x_stabilizers(&self) -> &[StabilizerGenerator] {
        &self.x_stabilizers
    }

    /// Get Z-type stabilizers
    #[inline]
    pub fn z_stabilizers(&self) -> &[StabilizerGenerator] {
        &self.z_stabilizers
    }
}
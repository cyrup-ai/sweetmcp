//! Core data structures for stabilizer quantum error correction
//!
//! This module defines the fundamental types and structures used in stabilizer
//! code implementations with zero allocation patterns and blazing-fast performance.

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

impl PauliOp {
    /// Create a new Pauli operator
    #[inline]
    pub const fn new(qubit: usize, pauli: PauliMatrix, phase: u8) -> Self {
        Self { qubit, pauli, phase }
    }

    /// Create identity operator
    #[inline]
    pub const fn identity(qubit: usize) -> Self {
        Self::new(qubit, PauliMatrix::I, 0)
    }

    /// Create X operator
    #[inline]
    pub const fn x(qubit: usize) -> Self {
        Self::new(qubit, PauliMatrix::X, 0)
    }

    /// Create Y operator
    #[inline]
    pub const fn y(qubit: usize) -> Self {
        Self::new(qubit, PauliMatrix::Y, 0)
    }

    /// Create Z operator
    #[inline]
    pub const fn z(qubit: usize) -> Self {
        Self::new(qubit, PauliMatrix::Z, 0)
    }

    /// Check if operator is identity
    #[inline]
    pub const fn is_identity(&self) -> bool {
        matches!(self.pauli, PauliMatrix::I)
    }

    /// Get operator weight (0 for identity, 1 for non-identity)
    #[inline]
    pub const fn weight(&self) -> usize {
        if self.is_identity() { 0 } else { 1 }
    }
}

impl PauliMatrix {
    /// Check if matrix commutes with another
    #[inline]
    pub const fn commutes_with(self, other: Self) -> bool {
        match (self, other) {
            (PauliMatrix::I, _) | (_, PauliMatrix::I) => true,
            (PauliMatrix::X, PauliMatrix::X) | 
            (PauliMatrix::Y, PauliMatrix::Y) | 
            (PauliMatrix::Z, PauliMatrix::Z) => true,
            _ => false,
        }
    }

    /// Get matrix multiplication result
    #[inline]
    pub const fn multiply(self, other: Self) -> (Self, u8) {
        match (self, other) {
            (PauliMatrix::I, other) => (other, 0),
            (this, PauliMatrix::I) => (this, 0),
            (PauliMatrix::X, PauliMatrix::X) => (PauliMatrix::I, 0),
            (PauliMatrix::Y, PauliMatrix::Y) => (PauliMatrix::I, 0),
            (PauliMatrix::Z, PauliMatrix::Z) => (PauliMatrix::I, 0),
            (PauliMatrix::X, PauliMatrix::Y) => (PauliMatrix::Z, 1),
            (PauliMatrix::Y, PauliMatrix::X) => (PauliMatrix::Z, 3),
            (PauliMatrix::Y, PauliMatrix::Z) => (PauliMatrix::X, 1),
            (PauliMatrix::Z, PauliMatrix::Y) => (PauliMatrix::X, 3),
            (PauliMatrix::Z, PauliMatrix::X) => (PauliMatrix::Y, 1),
            (PauliMatrix::X, PauliMatrix::Z) => (PauliMatrix::Y, 3),
        }
    }
}
//! Topological Pauli operators and stabilizer types
//!
//! This module provides Pauli operator implementations and stabilizer types for topological codes
//! with zero-allocation patterns and blazing-fast performance.
//!
//! # Features
//! - Pauli operators (X, Y, Z, I) with efficient matrix representations
//! - Topological Pauli operators with support for color codes and surface codes
//! - Stabilizer types and logical operator definitions
//! - Commutation and multiplication operations

use std::fmt;
use std::ops::{Mul, MulAssign};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use smallvec::SmallVec;
use num_traits::{One, Zero};

use crate::cognitive::quantum::Complex64;
use super::topological_pauli_strings::PauliString;

/// Error types for Pauli operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum PauliError {
    /// Invalid qubit index
    #[error("Invalid qubit index: {0}")]
    InvalidQubitIndex(usize),
    
    /// Incompatible operator dimensions
    #[error("Incompatible operator dimensions: {0}")]
    IncompatibleDimensions(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// Result type for Pauli operations
pub type PauliResult<T> = Result<T, PauliError>;

/// Pauli operators (I, X, Y, Z) with their matrix representations and properties.
/// 
/// The Pauli operators form a group under matrix multiplication and are fundamental
/// building blocks for quantum error correction codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PauliType {
    /// Identity operator (does nothing)
    I,
    /// Pauli-X (bit-flip) operator
    X,
    /// Pauli-Y (bit+phase flip) operator
    Y,
    /// Pauli-Z (phase-flip) operator
    Z,
}

impl PauliType {
    /// Get the matrix representation (as 4 complex numbers for 2x2 matrix)
    /// 
    /// Returns the 2x2 matrix representation of the Pauli operator as a flat array
    /// in row-major order: [a, b, c, d] represents the matrix:
    /// ```
    /// [a b]
    /// [c d]
    /// ```
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// let x_matrix = PauliType::X.matrix();
    /// assert_eq!(x_matrix[0], Complex64::new(0.0, 0.0));
    /// assert_eq!(x_matrix[1], Complex64::new(1.0, 0.0));
    /// assert_eq!(x_matrix[2], Complex64::new(1.0, 0.0));
    /// assert_eq!(x_matrix[3], Complex64::new(0.0, 0.0));
    /// ```
    pub fn matrix(&self) -> [Complex64; 4] {
        match self {
            PauliType::I => [
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
            ],
            PauliType::X => [
                Complex64::new(0.0, 0.0), Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
            ],
            PauliType::Y => [
                Complex64::new(0.0, 0.0), Complex64::new(0.0, -1.0),
                Complex64::new(0.0, 1.0), Complex64::new(0.0, 0.0),
            ],
            PauliType::Z => [
                Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0),
                Complex64::new(0.0, 0.0), Complex64::new(-1.0, 0.0),
            ],
        }
    }

    /// Check if this Pauli operator commutes with another Pauli operator.
    /// 
    /// Two Pauli operators commute if they are the same or if one of them is the identity.
    /// Different non-identity Pauli operators anticommute.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// assert!(PauliType::X.commutes_with(&PauliType::X));  // Same operator
    /// assert!(PauliType::I.commutes_with(&PauliType::Z));  // I commutes with everything
    /// assert!(!PauliType::X.commutes_with(&PauliType::Z)); // X and Z anticommute
    /// ```
    pub fn commutes_with(&self, other: &PauliType) -> bool {
        // I commutes with everything
        if *self == PauliType::I || *other == PauliType::I {
            return true;
        }
        // Same operators commute
        if self == other {
            return true;
        }
        // Different non-I Paulis anticommute
        false
    }

    /// Multiply two Pauli operators and return the result with phase.
    /// 
    /// Returns a tuple of (phase, result) where phase is a complex number
    /// and result is the resulting Pauli operator.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// // X * Y = iZ
    /// let (phase, result) = PauliType::X.multiply(&PauliType::Y);
    /// assert_eq!(result, PauliType::Z);
    /// assert!((phase - Complex64::new(0.0, 1.0)).norm() < 1e-10);
    /// 
    /// // Y * X = -iZ
    /// let (phase, result) = PauliType::Y.multiply(&PauliType::X);
    /// assert_eq!(result, PauliType::Z);
    /// assert!((phase - Complex64::new(0.0, -1.0)).norm() < 1e-10);
    /// ```
    pub fn multiply(&self, other: &PauliType) -> (Complex64, PauliType) {
        match (self, other) {
            // Identity cases
            (PauliType::I, _) => (Complex64::new(1.0, 0.0), *other),
            (_, PauliType::I) => (Complex64::new(1.0, 0.0), *self),
            
            // Same operators square to identity
            (a, b) if a == b => (Complex64::new(1.0, 0.0), PauliType::I),
            
            // Cyclic permutations with +i phase
            (PauliType::X, PauliType::Y) => (Complex64::new(0.0, 1.0), PauliType::Z),
            (PauliType::Y, PauliType::Z) => (Complex64::new(0.0, 1.0), PauliType::X),
            (PauliType::Z, PauliType::X) => (Complex64::new(0.0, 1.0), PauliType::Y),
            
            // Reverse cyclic permutations with -i phase
            (PauliType::Y, PauliType::X) => (Complex64::new(0.0, -1.0), PauliType::Z),
            (PauliType::Z, PauliType::Y) => (Complex64::new(0.0, -1.0), PauliType::X),
            (PauliType::X, PauliType::Z) => (Complex64::new(0.0, -1.0), PauliType::Y),
            
            _ => unreachable!("All Pauli multiplications should be covered"),
        }
    }

    /// Get the conjugate transpose (Hermitian conjugate)
    /// 
    /// For Pauli operators, this is just the operator itself since they are Hermitian.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// assert_eq!(PauliType::X.dagger(), PauliType::X);
    /// assert_eq!(PauliType::Y.dagger(), PauliType::Y);
    /// assert_eq!(PauliType::Z.dagger(), PauliType::Z);
    /// ```
    pub const fn dagger(&self) -> PauliType {
        // All Pauli matrices are Hermitian, so the conjugate transpose is the same
        *self
    }

    /// Get the eigenvalues of the Pauli operator
    /// 
    /// Returns the two eigenvalues (1 and -1) for non-identity Pauli operators,
    /// or (1, 1) for the identity operator.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// // Non-identity Pauli operators have eigenvalues +1 and -1
    /// let (e1, e2) = PauliType::Z.eigenvalues();
    /// assert_eq!(e1, Complex64::new(1.0, 0.0));
    /// assert_eq!(e2, Complex64::new(-1.0, 0.0));
    /// 
    /// // Identity has eigenvalues (1, 1)
    /// let (e1, e2) = PauliType::I.eigenvalues();
    /// assert_eq!(e1, Complex64::new(1.0, 0.0));
    /// assert_eq!(e2, Complex64::new(1.0, 0.0));
    /// ```
    pub fn eigenvalues(&self) -> [Complex64; 2] {
        match self {
            PauliType::I => [
                Complex64::new(1.0, 0.0),
                Complex64::new(1.0, 0.0),
            ],
            _ => [
                Complex64::new(1.0, 0.0),
                Complex64::new(-1.0, 0.0),
            ],
        }
    }

    /// Check if the operator is the identity
    /// 
    /// Returns `true` if this is the identity operator (I), which leaves quantum states unchanged.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// assert!(PauliType::I.is_identity());
    /// assert!(!PauliType::X.is_identity());
    /// assert!(!PauliType::Y.is_identity());
    /// assert!(!PauliType::Z.is_identity());
    /// ```
    pub const fn is_identity(&self) -> bool {
        matches!(self, PauliType::I)
    }

    /// Get the trace of the matrix representation
    /// 
    /// The trace is the sum of the diagonal elements of the matrix.
    /// For 2x2 Pauli matrices, the trace is 2 for the identity matrix
    /// and 0 for all other Pauli matrices.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::PauliType;
    /// 
    /// // Identity has trace 2 (for 2x2 matrix)
    /// assert_eq!(PauliType::I.trace(), Complex64::new(2.0, 0.0));
    /// 
    /// // All other Pauli matrices have trace 0
    /// assert_eq!(PauliType::X.trace(), Complex64::new(0.0, 0.0));
    /// assert_eq!(PauliType::Y.trace(), Complex64::new(0.0, 0.0));
    /// assert_eq!(PauliType::Z.trace(), Complex64::new(0.0, 0.0));
    /// ```
    pub fn trace(&self) -> Complex64 {
        match self {
            PauliType::I => Complex64::new(2.0, 0.0),  // 2 for 2x2 identity
            _ => Complex64::new(0.0, 0.0),
        }
    }
}

/// A Pauli operator in the context of a topological code
///
/// Represents a Pauli operator (X, Y, Z) acting on a specific edge
/// of a topological code lattice, with an associated complex coefficient.
/// This is a fundamental building block for describing quantum operations
/// in topological quantum error correction codes like surface codes and color codes.
///
/// # Fields
/// - `edge_id`: The ID of the edge in the lattice where this operator acts
/// - `pauli_type`: The type of Pauli operator (X, Y, or Z)
/// - `coefficient`: Complex coefficient (typically ±1 or ±i) for the operator
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologicalPauli {
    /// Edge ID in the lattice where this operator acts
    pub edge_id: usize,
    
    /// Type of Pauli operator (X, Y, or Z)
    pub pauli_type: PauliType,
    
    /// Complex coefficient (typically ±1 or ±i)
    pub coefficient: Complex64,
}

impl TopologicalPauli {
    /// Create a new topological Pauli operator
    /// 
    /// # Arguments
    /// * `edge_id` - The ID of the edge in the lattice where this operator acts
    /// * `pauli_type` - The type of Pauli operator (X, Y, or Z)
    /// * `coefficient` - Complex coefficient (typically ±1 or ±i)
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{PauliType, TopologicalPauli};
    /// 
    /// // Create a Pauli-X operator on edge 42 with coefficient 1.0
    /// let op = TopologicalPauli::new(42, PauliType::X, Complex64::new(1.0, 0.0));
    /// assert_eq!(op.edge_id, 42);
    /// assert_eq!(op.pauli_type, PauliType::X);
    /// ```
    pub fn new(edge_id: usize, pauli_type: PauliType, coefficient: Complex64) -> Self {
        Self {
            edge_id,
            pauli_type,
            coefficient,
        }
    }

    /// Create an identity operator on the specified edge
    /// 
    /// The identity operator leaves quantum states unchanged.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{PauliType, TopologicalPauli};
    /// 
    /// let id = TopologicalPauli::identity(0);
    /// assert!(id.pauli_type.is_identity());
    /// assert_eq!(id.edge_id, 0);
    /// ```
    pub fn identity(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::I, Complex64::new(1.0, 0.0))
    }

    /// Create an X operator on the specified edge
    /// 
    /// The Pauli-X operator flips the state of a qubit (|0⟩ ↔ |1⟩).
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{PauliType, TopologicalPauli};
    /// 
    /// let x = TopologicalPauli::x(0);
    /// assert_eq!(x.pauli_type, PauliType::X);
    /// assert_eq!(x.edge_id, 0);
    /// ```
    pub fn x(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::X, Complex64::new(1.0, 0.0))
    }

    /// Create a Y operator on the specified edge
    /// 
    /// The Pauli-Y operator flips the qubit state and applies a phase of i.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{PauliType, TopologicalPauli};
    /// 
    /// let y = TopologicalPauli::y(1);
    /// assert_eq!(y.pauli_type, PauliType::Y);
    /// assert_eq!(y.edge_id, 1);
    /// ```
    pub fn y(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::Y, Complex64::new(1.0, 0.0))
    }

    /// Create a Z operator on the specified edge
    /// 
    /// The Pauli-Z operator applies a phase flip (|1⟩ → -|1⟩).
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{PauliType, TopologicalPauli};
    /// 
    /// let z = TopologicalPauli::z(2);
    /// assert_eq!(z.pauli_type, PauliType::Z);
    /// assert_eq!(z.edge_id, 2);
    /// ```
    pub fn z(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::Z, Complex64::new(1.0, 0.0))
    }

    /// Check if this operator commutes with another operator
    /// 
    /// Two operators commute if they act on different edges or if their
    /// Pauli types commute when acting on the same edge.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{TopologicalPauli, PauliType};
    /// 
    /// let x0 = TopologicalPauli::x(0);
    /// let z0 = TopologicalPauli::z(0);
    /// let x1 = TopologicalPauli::x(1);
    /// 
    /// // X and Z anticommute on same edge
    /// assert!(!x0.commutes_with(&z0));
    /// 
    /// // Operators on different edges always commute
    /// assert!(x0.commutes_with(&x1));
    /// ```
    pub fn commutes_with(&self, other: &TopologicalPauli) -> bool {
        if self.edge_id != other.edge_id {
            return true; // Operators on different edges commute
        }
        self.pauli_type.commutes_with(&other.pauli_type)
    }

    /// Multiply this operator with another topological Pauli operator
    /// 
    /// Returns `None` if the operators act on different edges.
    /// The result includes the appropriate phase factor from the multiplication.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{TopologicalPauli, PauliType};
    /// 
    /// let x = TopologicalPauli::x(0);
    /// let y = TopologicalPauli::y(0);
    /// let product = x.multiply(&y).unwrap();
    /// 
    /// // X * Y = iZ
    /// assert_eq!(product.pauli_type, PauliType::Z);
    /// assert!((product.coefficient - Complex64::new(0.0, 1.0)).norm() < 1e-10);
    /// 
    /// // Operators on different edges cannot be multiplied
    /// let x1 = TopologicalPauli::x(1);
    /// assert!(y.multiply(&x1).is_none());
    /// ```
    pub fn multiply(&self, other: &TopologicalPauli) -> Option<TopologicalPauli> {
        if self.edge_id != other.edge_id {
            return None; // Can only multiply operators on the same edge
        }
        
        let (coeff, pauli_type) = self.pauli_type.multiply(&other.pauli_type);
        let coefficient = self.coefficient * other.coefficient * coeff;
        
        Some(TopologicalPauli {
            edge_id: self.edge_id,
            pauli_type,
            coefficient,
        })
    }

    /// Compute the Hermitian conjugate (adjoint) of this operator
    /// 
    /// For Pauli operators, this is equivalent to complex conjugating the coefficient
    /// since the Pauli matrices are Hermitian.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{TopologicalPauli, PauliType};
    /// 
    /// // Create an operator with a complex coefficient
    /// let op = TopologicalPauli::new(0, PauliType::X, Complex64::new(1.0, 1.0));
    /// let adj = op.dagger();
    /// 
    /// // The adjoint should have the complex conjugate of the coefficient
    /// assert_eq!(adj.coefficient, Complex64::new(1.0, -1.0));
    /// assert_eq!(adj.pauli_type, PauliType::X);
    /// assert_eq!(adj.edge_id, 0);
    /// ```
    pub fn dagger(&self) -> TopologicalPauli {
        // For Pauli matrices, the conjugate transpose is the same as the original
        // except we need to complex conjugate the coefficient
        Self {
            edge_id: self.edge_id,
            pauli_type: self.pauli_type,
            coefficient: self.coefficient.conj(),
        }
    }

    /// Check if this operator is effectively the identity
    /// 
    /// Returns `true` if the Pauli type is I or if the coefficient is zero.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::Complex64;
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{TopologicalPauli, PauliType};
    /// 
    /// // Identity operator is always identity
    /// let id = TopologicalPauli::identity(0);
    /// assert!(id.is_identity());
    /// 
    /// // Zero coefficient makes any operator effectively identity
    /// let zero_op = TopologicalPauli::new(0, PauliType::X, Complex64::new(0.0, 0.0));
    /// assert!(zero_op.is_identity());
    /// 
    /// // Non-identity Pauli with non-zero coefficient is not identity
    /// let x = TopologicalPauli::x(0);
    /// assert!(!x.is_identity());
    /// ```
    pub fn is_identity(&self) -> bool {
        self.pauli_type == PauliType::I || self.coefficient.norm_sqr() < 1e-10
    }

    /// Get the weight of this operator (number of non-identity terms)
    /// 
    /// For a single Pauli operator, this is 1 if it's not identity, 0 otherwise.
    /// 
    /// # Examples
    /// ```
    /// use sweetmcp_memory::cognitive::quantum::error_correction::{TopologicalPauli, PauliType};
    /// 
    /// assert_eq!(TopologicalPauli::x(0).weight(), 1);
    /// assert_eq!(TopologicalPauli::identity(0).weight(), 0);
    /// ```
    pub fn weight(&self) -> usize {
        if self.is_identity() { 0 } else { 1 }
    }
}

/// Types of topological stabilizers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopologicalStabilizerType {
    /// Vertex stabilizer (star operator)
    Vertex,
    
    /// Plaquette stabilizer
    Plaquette,
    
    /// Color code stabilizer
    Color(ColorType),
}

impl TopologicalStabilizerType {
    /// Check if stabilizer is X-type
    pub fn is_x_type(&self) -> bool {
        matches!(self, TopologicalStabilizerType::Vertex)
    }

    /// Check if stabilizer is Z-type
    pub fn is_z_type(&self) -> bool {
        matches!(self, TopologicalStabilizerType::Plaquette)
    }

    /// Get the typical Pauli type for this stabilizer
    pub fn typical_pauli(&self) -> PauliType {
        match self {
            TopologicalStabilizerType::Vertex => PauliType::X,
            TopologicalStabilizerType::Plaquette => PauliType::Z,
            TopologicalStabilizerType::Color(_) => PauliType::X, // Varies by color
        }
    }

    /// Check if stabilizer commutes with another type
    pub fn commutes_with(&self, other: &TopologicalStabilizerType) -> bool {
        match (self, other) {
            // Same type always commutes
            (a, b) if a == b => true,
            // X and Z type stabilizers commute if they don't share edges
            (TopologicalStabilizerType::Vertex, TopologicalStabilizerType::Plaquette) |
            (TopologicalStabilizerType::Plaquette, TopologicalStabilizerType::Vertex) => true,
            // Color code stabilizers have more complex commutation relations
            _ => false,
        }
    }
}

/// Color types for color codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorType {
    Red,
    Green,
    Blue,
}

impl ColorType {
    /// Get all color types
    pub fn all() -> [ColorType; 3] {
        [ColorType::Red, ColorType::Green, ColorType::Blue]
    }

    /// Get the other two colors
    pub fn others(&self) -> [ColorType; 2] {
        match self {
            ColorType::Red => [ColorType::Green, ColorType::Blue],
            ColorType::Green => [ColorType::Red, ColorType::Blue],
            ColorType::Blue => [ColorType::Red, ColorType::Green],
        }
    }

    /// Get RGB values for visualization
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            ColorType::Red => (255, 0, 0),
            ColorType::Green => (0, 255, 0),
            ColorType::Blue => (0, 0, 255),
        }
    }

    /// Get the Pauli type associated with this color in color codes
    pub fn pauli_type(&self) -> PauliType {
        match self {
            ColorType::Red => PauliType::X,
            ColorType::Green => PauliType::Y,
            ColorType::Blue => PauliType::Z,
        }
    }

    /// Get the complementary color combination
    pub fn complement(&self) -> [ColorType; 2] {
        self.others()
    }
}

/// Logical operator types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOperatorType {
    X, Z,
}

impl LogicalOperatorType {
    /// Check if operator commutes with another
    pub fn commutes_with(&self, other: &LogicalOperatorType) -> bool {
        self == other
    }

    /// Get the anticommuting operator
    pub fn anticommuting(&self) -> LogicalOperatorType {
        match self {
            LogicalOperatorType::X => LogicalOperatorType::Z,
            LogicalOperatorType::Z => LogicalOperatorType::X,
        }
    }

    /// Get the corresponding Pauli type
    pub fn to_pauli(&self) -> PauliType {
        match self {
            LogicalOperatorType::X => PauliType::X,
            LogicalOperatorType::Z => PauliType::Z,
        }
    }

    /// Check if this is an X-type logical operator
    pub fn is_x_type(&self) -> bool {
        matches!(self, LogicalOperatorType::X)
    }

    /// Check if this is a Z-type logical operator
    pub fn is_z_type(&self) -> bool {
        matches!(self, LogicalOperatorType::Z)
    }
}
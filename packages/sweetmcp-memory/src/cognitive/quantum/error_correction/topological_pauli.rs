//! Topological Pauli operators and stabilizer types
//!
//! This module provides Pauli operator implementations and stabilizer types for topological codes
//! with zero-allocation patterns and blazing-fast performance.

use crate::cognitive::quantum::Complex64;
use smallvec::SmallVec;

pub use super::topological_pauli_strings::PauliString;

/// Pauli types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PauliType {
    I, X, Y, Z,
}

impl PauliType {
    /// Get the matrix representation (as 4 complex numbers for 2x2 matrix)
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

    /// Check if Pauli operator commutes with another
    pub fn commutes_with(&self, other: &PauliType) -> bool {
        match (self, other) {
            (PauliType::I, _) | (_, PauliType::I) => true,
            (PauliType::X, PauliType::X) | (PauliType::Y, PauliType::Y) | (PauliType::Z, PauliType::Z) => true,
            _ => false,
        }
    }

    /// Get the result of multiplying two Pauli operators
    pub fn multiply(&self, other: &PauliType) -> (Complex64, PauliType) {
        match (self, other) {
            (PauliType::I, p) | (p, PauliType::I) => (Complex64::new(1.0, 0.0), *p),
            (PauliType::X, PauliType::X) | (PauliType::Y, PauliType::Y) | (PauliType::Z, PauliType::Z) => {
                (Complex64::new(1.0, 0.0), PauliType::I)
            },
            (PauliType::X, PauliType::Y) => (Complex64::new(0.0, 1.0), PauliType::Z),
            (PauliType::Y, PauliType::X) => (Complex64::new(0.0, -1.0), PauliType::Z),
            (PauliType::Y, PauliType::Z) => (Complex64::new(0.0, 1.0), PauliType::X),
            (PauliType::Z, PauliType::Y) => (Complex64::new(0.0, -1.0), PauliType::X),
            (PauliType::Z, PauliType::X) => (Complex64::new(0.0, 1.0), PauliType::Y),
            (PauliType::X, PauliType::Z) => (Complex64::new(0.0, -1.0), PauliType::Y),
        }
    }

    /// Get the conjugate transpose (Hermitian conjugate)
    pub fn dagger(&self) -> PauliType {
        *self // All Pauli matrices are Hermitian
    }

    /// Get eigenvalues
    pub fn eigenvalues(&self) -> [Complex64; 2] {
        match self {
            PauliType::I => [Complex64::new(1.0, 0.0), Complex64::new(1.0, 0.0)],
            PauliType::X | PauliType::Y | PauliType::Z => [
                Complex64::new(1.0, 0.0), 
                Complex64::new(-1.0, 0.0)
            ],
        }
    }

    /// Check if operator is identity
    pub fn is_identity(&self) -> bool {
        matches!(self, PauliType::I)
    }

    /// Get the trace of the matrix
    pub fn trace(&self) -> Complex64 {
        match self {
            PauliType::I => Complex64::new(2.0, 0.0),
            PauliType::X | PauliType::Y | PauliType::Z => Complex64::new(0.0, 0.0),
        }
    }
}

/// Pauli operator in topological context
#[derive(Debug, Clone)]
pub struct TopologicalPauli {
    /// Edge ID where operator acts
    pub edge_id: usize,
    
    /// Pauli type
    pub pauli_type: PauliType,
    
    /// Coefficient
    pub coefficient: Complex64,
}

impl TopologicalPauli {
    /// Create new topological Pauli operator
    pub fn new(edge_id: usize, pauli_type: PauliType, coefficient: Complex64) -> Self {
        Self {
            edge_id,
            pauli_type,
            coefficient,
        }
    }

    /// Create identity operator on edge
    pub fn identity(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::I, Complex64::new(1.0, 0.0))
    }

    /// Create X operator on edge
    pub fn x(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::X, Complex64::new(1.0, 0.0))
    }

    /// Create Y operator on edge
    pub fn y(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::Y, Complex64::new(1.0, 0.0))
    }

    /// Create Z operator on edge
    pub fn z(edge_id: usize) -> Self {
        Self::new(edge_id, PauliType::Z, Complex64::new(1.0, 0.0))
    }

    /// Check if operator commutes with another
    pub fn commutes_with(&self, other: &TopologicalPauli) -> bool {
        if self.edge_id != other.edge_id {
            true // Operators on different edges always commute
        } else {
            self.pauli_type.commutes_with(&other.pauli_type)
        }
    }

    /// Multiply with another topological Pauli
    pub fn multiply(&self, other: &TopologicalPauli) -> Option<TopologicalPauli> {
        if self.edge_id != other.edge_id {
            return None; // Cannot multiply operators on different edges
        }

        let (phase, result_pauli) = self.pauli_type.multiply(&other.pauli_type);
        let result_coefficient = self.coefficient * other.coefficient * phase;

        Some(TopologicalPauli::new(
            self.edge_id,
            result_pauli,
            result_coefficient,
        ))
    }

    /// Get the Hermitian conjugate
    pub fn dagger(&self) -> TopologicalPauli {
        TopologicalPauli::new(
            self.edge_id,
            self.pauli_type.dagger(),
            self.coefficient.conj(),
        )
    }

    /// Check if operator is effectively identity (coefficient is zero or Pauli is I)
    pub fn is_identity(&self) -> bool {
        self.coefficient.norm() < 1e-10 || self.pauli_type.is_identity()
    }

    /// Get the weight (number of non-identity terms)
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
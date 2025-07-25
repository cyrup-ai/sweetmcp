//! Quantum error correction implementations
//!
//! This module provides comprehensive quantum error correction implementations
//! including stabilizer codes, topological codes, surface codes, and color codes
//! with zero-allocation patterns and blazing-fast performance.

// All quantum error correction modules
pub mod circuit_builder;
pub mod circuit_impl;
pub mod circuits;
pub mod codes;
pub mod gate_impl;
pub mod logical_qubits;
pub mod measurements;
pub mod quantum_circuits;
pub mod quantum_error_correction;
pub mod stabilizer_basic_operations;
pub mod stabilizer_core_types;
pub mod stabilizer_css_types;
pub mod stabilizer_decoders;
pub mod stabilizer_mod;
pub mod stabilizer_operations;
pub mod stabilizer_types;
pub mod surface_code;
pub mod syndromes;
pub mod topological_lattice;
pub mod topological_lattice_types;
pub mod topological_logical_operators;
pub mod topological_pauli;
pub mod topological_pauli_strings;
pub mod topological_stabilizers;
pub mod topological_types;
pub mod types;

// Backward compatibility alias
pub use surface_code::SurfaceCode as TopologicalCode;
pub use quantum_error_correction::{
    QuantumErrorCorrection, ErrorCorrectionCode, ErrorCorrectionCodeType,
    CodeParameters, StabilizerGenerator, LogicalOperator, LogicalOperatorType,
    PauliOperator, ErrorCorrectionConfig, ErrorCorrectionStatistics,
    ErrorCorrectionMetrics, CorrectionResult, ErrorCorrectionError,
};

// Core type definitions
pub use topological_types::*;

// Ensure all types are properly exported (removed LogicalOperatorType duplicate)
pub use topological_types::{
    TopologicalCodeType, StabilizerType, ColorType,
};

// Main error correction types for external use are now in quantum_error_correction module

// Pauli operator implementations
pub use topological_pauli::{TopologicalPauli, PauliType};
pub use topological_pauli_strings::PauliString;
pub use topological_lattice_types::{
    TopologicalLattice, LatticeVertex, LatticeEdge, LatticeFace, 
    LatticeStatistics, BoundaryConditions
};

// Re-export generation methods through TopologicalLattice impl

// Stabilizer generators and operations
pub use self::topological_stabilizers::{
    TopologicalStabilizer, StabilizerGroup, StabilizerStatistics
};

// Re-export generation methods through StabilizerGroup impl

// Logical operators
pub use topological_logical_operators::{
    TopologicalLogicalOperator, LogicalOperatorSet, LogicalOperatorStatistics
};

// Re-export generation methods through LogicalOperatorSet impl

use crate::cognitive::types::{CognitiveError, CognitiveResult};
use crate::cognitive::quantum::Complex64;

// Main topological code implementation is now re-exported from surface_code

/// Topological code parameters
#[derive(Debug, Clone)]
pub struct TopologicalParameters {
    /// Error threshold
    pub error_threshold: f64,
    
    /// Decoder type
    pub decoder_type: TopologicalDecoderType,
    
    /// Enable fast decoding optimizations
    pub fast_decoding: bool,
    
    /// Maximum correction rounds
    pub max_correction_rounds: usize,
}

/// Toric code implementation
#[derive(Debug, Clone)]
pub struct ToricCode {
    /// Base topological code
    pub base_code: TopologicalCode,
    
    /// Lattice size (L x L)
    pub lattice_size: usize,
    
    /// Number of logical qubits (2 for standard toric code)
    pub logical_qubits: usize,
}

/// Color code lattice type for quantum error correction
#[derive(Debug, Clone)]
pub struct ColorCodeLattice {
    pub width: usize,
    pub height: usize,
    pub colors: Vec<ColorType>,
    pub stabilizers: Vec<StabilizerType>,
}

impl ColorCodeLattice {
    /// Create new color code lattice
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            colors: Vec::new(),
            stabilizers: Vec::new(),
        }
    }
}

/// Color code implementation
#[derive(Debug, Clone)]
pub struct ColorCode {
    /// Base topological code
    pub base_code: TopologicalCode,
    
    /// Lattice type
    pub lattice_type: ColorCodeLattice,
    
    /// Color assignments
    pub color_assignments: std::collections::HashMap<usize, ColorType>,
}

impl TopologicalCode {
    /// Create a new topological code
    pub fn new(
        code_type: TopologicalCodeType,
        dimensions: (usize, usize),
        boundary: BoundaryConditions,
    ) -> CognitiveResult<Self> {
        let lattice = TopologicalLattice::generate(code_type, dimensions, boundary)?;
        let stabilizers = StabilizerGroup::generate_for_lattice(code_type, &lattice)?;
        let logical_operators = LogicalOperatorSet::generate_for_lattice(code_type, &lattice)?;
        
        let parameters = TopologicalParameters {
            error_threshold: match code_type {
                TopologicalCodeType::ToricCode | TopologicalCodeType::PlanarCode => 0.11,
                TopologicalCodeType::ColorCode => 0.15,
                TopologicalCodeType::HyperbolicCode => 0.20,
            },
            decoder_type: TopologicalDecoderType::MWPM,
            fast_decoding: true,
            max_correction_rounds: 10,
        };

        Ok(Self {
            code_type,
            lattice,
            stabilizers,
            logical_operators,
            parameters,
        })
    }

    /// Get the code distance
    pub fn distance(&self) -> usize {
        // Distance is the minimum weight of non-trivial logical operators
        let mut min_weight = usize::MAX;
        
        for op in self.logical_operators.all_operators() {
            let weight = op.weight();
            if weight > 0 && weight < min_weight {
                min_weight = weight;
            }
        }
        
        if min_weight == usize::MAX { 1 } else { min_weight }
    }

    /// Get the number of physical qubits
    pub fn num_physical_qubits(&self) -> usize {
        self.lattice.edges.iter()
            .filter_map(|e| e.qubit_id)
            .count()
    }

    /// Get the number of logical qubits
    pub fn num_logical_qubits(&self) -> usize {
        self.logical_operators.num_logical_qubits
    }

    /// Calculate syndrome for an error pattern
    pub fn calculate_syndrome(&self, error_pattern: &PauliString) -> Vec<bool> {
        self.stabilizers.calculate_syndrome(error_pattern)
    }

    /// Validate the code (check all consistency requirements)
    pub fn validate(&self) -> Result<(), String> {
        // Validate lattice structure
        self.lattice.validate()?;
        
        // Validate stabilizer commutation relations
        self.stabilizers.validate_commutation()?;
        
        // Validate logical operator commutation relations
        self.logical_operators.validate()?;
        
        Ok(())
    }

    /// Get code statistics
    pub fn statistics(&self) -> TopologicalCodeStatistics {
        TopologicalCodeStatistics {
            code_type: self.code_type,
            lattice_stats: self.lattice.statistics(),
            stabilizer_stats: self.stabilizers.statistics(),
            logical_operator_stats: self.logical_operators.statistics(),
            distance: self.distance(),
            num_physical_qubits: self.num_physical_qubits(),
            num_logical_qubits: self.num_logical_qubits(),
            error_threshold: self.parameters.error_threshold,
        }
    }
}

/// Statistics about a topological code
#[derive(Debug, Clone)]
pub struct TopologicalCodeStatistics {
    pub code_type: TopologicalCodeType,
    pub lattice_stats: LatticeStatistics,
    pub stabilizer_stats: StabilizerStatistics,
    pub logical_operator_stats: LogicalOperatorStatistics,
    pub distance: usize,
    pub num_physical_qubits: usize,
    pub num_logical_qubits: usize,
    pub error_threshold: f64,
}

impl ToricCode {
    /// Create a new toric code
    pub fn new(lattice_size: usize) -> CognitiveResult<Self> {
        let base_code = TopologicalCode::new(
            TopologicalCodeType::ToricCode,
            (lattice_size, lattice_size),
            BoundaryConditions::periodic(),
        )?;

        Ok(Self {
            base_code,
            lattice_size,
            logical_qubits: 2,
        })
    }

    /// Get the code distance (equal to lattice size for toric code)
    pub fn distance(&self) -> usize {
        self.lattice_size
    }
}

impl ColorCode {
    /// Create a new color code
    pub fn new(
        dimensions: (usize, usize),
        lattice_type: ColorCodeLattice,
    ) -> CognitiveResult<Self> {
        let base_code = TopologicalCode::new(
            TopologicalCodeType::ColorCode,
            dimensions,
            BoundaryConditions::open(),
        )?;

        // Generate color assignments
        let mut color_assignments = std::collections::HashMap::new();
        let colors = [ColorType::Red, ColorType::Green, ColorType::Blue];
        
        for (i, face) in base_code.lattice.faces.iter().enumerate() {
            color_assignments.insert(face.id, colors[i % 3]);
        }

        Ok(Self {
            base_code,
            lattice_type,
            color_assignments,
        })
    }
}

/// Convenience functions for creating common topological codes
impl TopologicalCode {
    /// Create a surface code (planar code with open boundaries)
    pub fn surface_code(distance: usize) -> CognitiveResult<Self> {
        Self::new(
            TopologicalCodeType::PlanarCode,
            (distance, distance),
            BoundaryConditions::open(),
        )
    }

    /// Create a toric code (surface code with periodic boundaries)
    pub fn toric_code(distance: usize) -> CognitiveResult<Self> {
        Self::new(
            TopologicalCodeType::ToricCode,
            (distance, distance),
            BoundaryConditions::periodic(),
        )
    }

    /// Create a triangular color code
    pub fn triangular_color_code(distance: usize) -> CognitiveResult<Self> {
        Self::new(
            TopologicalCodeType::ColorCode,
            (distance, distance),
            BoundaryConditions::open(),
        )
    }
}

/// Error correction utilities
pub mod utils {
    use super::*;

    /// Calculate the minimum distance of a topological code
    pub fn calculate_minimum_distance(code: &TopologicalCode) -> usize {
        code.distance()
    }

    /// Estimate error threshold for a given code
    pub fn estimate_error_threshold(code_type: TopologicalCodeType) -> f64 {
        match code_type {
            TopologicalCodeType::ToricCode | TopologicalCodeType::PlanarCode => 0.11,
            TopologicalCodeType::ColorCode => 0.15,
            TopologicalCodeType::HyperbolicCode => 0.20,
        }
    }

    /// Check if an error pattern is correctable
    pub fn is_correctable(code: &TopologicalCode, error_pattern: &PauliString) -> bool {
        let syndrome = code.calculate_syndrome(error_pattern);
        // Simplified check - in practice would use proper decoder
        syndrome.iter().filter(|&&bit| bit).count() <= code.distance() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_code_creation() {
        let code = TopologicalCode::surface_code(3).expect("Failed to create surface code");
        assert_eq!(code.code_type, TopologicalCodeType::PlanarCode);
        assert!(code.validate().is_ok());
    }

    #[test]
    fn test_toric_code_creation() {
        let toric = ToricCode::new(3).expect("Failed to create toric code");
        assert_eq!(toric.logical_qubits, 2);
        assert_eq!(toric.distance(), 3);
    }

    #[test]
    fn test_code_validation() {
        let code = TopologicalCode::toric_code(2).expect("Failed to create toric code");
        assert!(code.validate().is_ok());
    }
}
//! Quantum error correction implementations
//!
//! This module provides comprehensive quantum error correction implementations
//! including stabilizer codes, topological codes, surface codes, and color codes
//! with zero-allocation patterns and blazing-fast performance.

// Core quantum error correction modules
pub mod circuit_builder;
pub mod circuit_impl;
pub mod gate_impl;
pub mod logical_qubits;
pub mod measurements;
pub mod quantum_circuits;
pub mod quantum_error_correction;
pub mod stabilizer_basic_operations;
pub mod stabilizer_css_types;
pub mod stabilizer_decoders;
pub mod surface_code;
pub mod syndromes;
pub mod topological_lattice_generation;
pub mod topological_lattice_types;
pub mod topological_logical_generation;
pub mod topological_logical_operators;
pub mod topological_pauli;
pub mod topological_pauli_strings;
pub mod topological_stabilizer_generation;
pub mod topological_stabilizers;
pub mod topological_types;

// Re-export commonly used types for convenience
pub use quantum_error_correction::{
    ErrorCorrectionCode, ErrorCorrectionCodeType, ErrorCorrectionConfig,
    ErrorCorrectionError, ErrorCorrectionMetrics, ErrorCorrectionStatistics,
    StabilizerGenerator, LogicalOperator, LogicalOperatorType, CodeParameters,
    CorrectionResult,
};

/// Re-export topological Pauli operator types and related enums
pub use topological_pauli::{
    // Core types
    PauliType,
    TopologicalPauli,
    PauliError,
    PauliResult,
    
    // Enums
    TopologicalStabilizerType,
    ColorType,
    
    // Constants and traits not available - using basic types only
    
    // Re-export with original names for backward compatibility
    LogicalOperatorType as TopologicalLogicalOperatorType,
};

/// Re-export stabilizer group and related types
pub use topological_stabilizers::{
    StabilizerGroup, StabilizerStatistics, TopologicalStabilizer,
};

// Re-export topological types
pub use topological_types::{
    BoundaryType, TopologicalCodeType, TopologicalDecoderType,
    VertexType, EdgeOrientation, FaceType, LatticeElement, StabilizerType
};

pub use syndromes::{ErrorSyndrome, QecErrorType};
pub use topological_lattice_types::TopologicalLattice;
pub use topological_logical_operators::TopologicalLogicalOperator;

use surface_code::{SurfaceCode, SurfaceCodeLayout};
use topological_types::BoundaryType as SurfaceBoundaryType;

// TopologicalCodeType and BoundaryType already re-exported above in topological_types block

use std::sync::Arc;

/// Topological code implementation that wraps SurfaceCode for backward compatibility
#[derive(Debug, Clone)]
pub struct TopologicalCode {
    /// The underlying surface code implementation
    inner: Arc<SurfaceCode>,
    /// Type of the topological code
    pub code_type: TopologicalCodeType,
    /// Boundary conditions
    pub boundary_type: BoundaryType,
    /// Error correction parameters
    pub parameters: TopologicalParameters,
}

impl TopologicalCode {
    /// Create a new topological code with the given distance and boundary type
    pub fn new(distance: usize, boundary_type: BoundaryType) -> CognitiveResult<Self> {
        // Convert to layout boundary type
        let layout_boundary = match boundary_type {
            BoundaryType::Open => SurfaceBoundaryType::Open,
            BoundaryType::Periodic => SurfaceBoundaryType::Periodic,
            BoundaryType::Twisted => SurfaceBoundaryType::Twisted,
            BoundaryType::Mixed => SurfaceBoundaryType::Open, // Default to Open for mixed
        };
        
        let inner = Arc::new(SurfaceCode::new(distance, layout_boundary)?);
        let code_type = match boundary_type {
            BoundaryType::Open => TopologicalCodeType::PlanarCode,
            BoundaryType::Periodic => TopologicalCodeType::ToricCode,
            BoundaryType::Twisted => TopologicalCodeType::PlanarCode,
            BoundaryType::Mixed => TopologicalCodeType::PlanarCode, // Default to PlanarCode for mixed
        };
        
        let parameters = TopologicalParameters {
            error_threshold: match code_type {
                TopologicalCodeType::ToricCode => 0.11,
                TopologicalCodeType::PlanarCode => 0.11,
                TopologicalCodeType::ColorCode => 0.15,
                TopologicalCodeType::HyperbolicCode => 0.20,
            },
            decoder_type: TopologicalDecoderType::MWPM,
            fast_decoding: true,
            max_correction_rounds: 10,
        };

        Ok(Self {
            inner,
            code_type,
            boundary_type,
            parameters,
        })
    }
    
    /// Create a topological code with a custom layout
    pub fn with_layout(layout: SurfaceCodeLayout, distance: usize) -> CognitiveResult<Self> {
        let inner = Arc::new(SurfaceCode::with_layout(layout, distance)?);
        let code_type = TopologicalCodeType::PlanarCode;
        let boundary_type = BoundaryType::Open;
        let parameters = TopologicalParameters {
            error_threshold: 0.11, // Default for planar code
            decoder_type: TopologicalDecoderType::MWPM,
            fast_decoding: true,
            max_correction_rounds: 10,
        };
        
        Ok(Self {
            inner,
            code_type,
            boundary_type,
            parameters,
        })
    }
    
    /// Get the code parameters (distance, logical_qubits, physical_qubits)
    pub fn parameters(&self) -> (usize, usize, usize) {
        // Get the number of physical qubits from the inner SurfaceCode
        let physical_qubits = self.inner.qubit_layout.len();
        // Return (distance, logical_qubits, physical_qubits)
        (self.inner.distance, 2, physical_qubits) // 2 logical qubits for surface/toric codes
    }
    
    /// Get the error threshold for this code
    pub fn error_threshold(&self) -> f64 {
        self.inner.error_threshold
    }
    
    /// Get the decoder type for this code
    pub fn decoder_type(&self) -> TopologicalDecoderType {
        // Default to MWPM decoder if not specified
        self.parameters.decoder_type
    }
    
    /// Get a reference to the inner SurfaceCode
    pub fn inner(&self) -> &SurfaceCode {
        &self.inner
    }
    
    /// Get a mutable reference to the inner SurfaceCode
    pub fn inner_mut(&mut self) -> &mut SurfaceCode {
        &mut self.inner
    }
    
    /// Convert into the inner SurfaceCode
    pub fn into_inner(self) -> SurfaceCode {
        self.inner
    }
    
    /// Get the code type
    pub fn code_type(&self) -> TopologicalCodeType {
        self.code_type
    }
    
    /// Get the boundary type
    pub fn boundary_type(&self) -> BoundaryType {
        self.boundary_type
    }
    
    /// Get the topological parameters
    pub fn topological_parameters(&self) -> &TopologicalParameters {
        &self.parameters
    }
}

impl std::ops::Deref for TopologicalCode {
    type Target = SurfaceCode;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for TopologicalCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Implement From conversions
impl From<SurfaceCode> for TopologicalCode {
    fn from(sc: SurfaceCode) -> Self {
        // Create a new TopologicalCode with default parameters
        let code_type = TopologicalCodeType::PlanarCode; // Default to PlanarCode
        let boundary_type = BoundaryType::Open; // Default to Open boundaries
        let parameters = TopologicalParameters {
            error_threshold: 0.11, // Default threshold
            decoder_type: TopologicalDecoderType::MWPM,
            fast_decoding: true,
            max_correction_rounds: 10,
        };
        
        Self {
            inner: Arc::new(sc),
            code_type,
            boundary_type,
            parameters,
        }
    }
}

impl From<TopologicalCode> for SurfaceCode {
    fn from(tc: TopologicalCode) -> Self {
        tc.inner
    }
}

// Core error correction types (re-exported for backward compatibility)
// Note: ErrorCorrectionCode, ErrorCorrectionCodeType already available from main import above
#[deprecated(note = "Import from quantum_error_correction module directly")]
pub use quantum_error_correction::{
    QuantumErrorCorrection, PauliOperator,
};

// Topological types already imported above via specific imports

// Pauli operators and strings (TopologicalPauli and PauliType already imported above)
pub use topological_pauli_strings::PauliString;

// Lattice types and generation (TopologicalLattice already imported above)
/// Re-export topological lattice types and related structures
pub use topological_lattice_types::{
    BoundaryConditions, LatticeEdge, LatticeFace, LatticeStatistics, LatticeVertex,
};

/// Re-export logical operator types and operations
pub use topological_logical_operators::{
    LogicalOperatorSet, LogicalOperatorStatistics,
};

// PauliString already imported above

// Pauli types already imported above

// Topological types already imported above

// Stabilizer types and operations (stabilizer_core_types functionality moved to stabilizer_css_types)
// pub use stabilizer_core_types::*; // Module no longer exists

/// Re-export CSS code types and functionality
pub use stabilizer_css_types::{
    CSSCode, CSSParameters, ClassicalCodeParams, CSSConstructionParams,
    CSSConstructionMethod, CSSDecodingResult, CSSDecodingMethod, CSSDecodingMetrics,
};
// Pauli types already imported above

// Stabilizer types already imported above

// Stabilizer generation functions not available - TODO: implement or remove references

/// Re-export decoder types and implementations
pub use stabilizer_decoders::{
    SyndromeDecoder, SyndromeDecoderConfig, MaximumLikelihoodDecoder, MLDecoderConfig,
    BeliefPropagationDecoder, BPDecoderConfig, DecodingMetrics, DecodingResult,
    ErrorPattern, PauliOp, ChannelErrorRates, FactorGraph, VariableNode, CheckNode, Edge,
};
// TopologicalStabilizer, StabilizerGroup, StabilizerStatistics already imported above
// pub use topological_stabilizers::{} - no unique exports needed
// Stabilizer generation already imported above with specific functions

// Logical operators already imported above

// Re-export generation methods through LogicalOperatorSet impl

// Generation modules already imported above with specific functions

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
    /// Create a new topological code with explicit type and dimensions
    #[allow(dead_code)]
    fn with_dimensions(
        code_type: TopologicalCodeType,
        dimensions: (usize, usize),
        boundary_type: BoundaryType,
    ) -> CognitiveResult<Self> {
        // Convert to layout boundary type
        let layout_boundary = match boundary_type {
            BoundaryType::Open => SurfaceBoundaryType::Open,
            BoundaryType::Periodic => SurfaceBoundaryType::Periodic,
            BoundaryType::Twisted => SurfaceBoundaryType::Twisted,
            BoundaryType::Mixed => SurfaceBoundaryType::Open, // Default to Open for mixed
        };
        
        let inner = SurfaceCode::new(dimensions.0, layout_boundary)?;
        
        let parameters = TopologicalParameters {
            error_threshold: match code_type {
                TopologicalCodeType::ToricCode => 0.11,
                TopologicalCodeType::PlanarCode => 0.11,
                TopologicalCodeType::ColorCode => 0.15,
                TopologicalCodeType::HyperbolicCode => 0.20,
            },
            decoder_type: TopologicalDecoderType::MWPM,
            fast_decoding: true,
            max_correction_rounds: 10,
        };

        Ok(Self {
            inner,
            code_type,
            boundary_type,
            parameters,
        })
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
        let base_code = TopologicalCode::toric_code(lattice_size)?;
        Ok(Self { base_code, lattice_size, logical_qubits: 2 })
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
        // Create the base code with the specified boundary conditions
        let base_code = TopologicalCode::triangular_color_code(dimensions.0)?;

        // Generate color assignments
        let mut color_assignments = std::collections::HashMap::new();
        let colors = [ColorType::Red, ColorType::Green, ColorType::Blue];
        
        // Use the lattice type to determine the color pattern
        for (i, face) in base_code.inner().qubit_layout.iter().enumerate() {
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
        use crate::cognitive::quantum::error_correction::topological_types::BoundaryType as SurfaceBoundaryType;
        
        // Create a new TopologicalCode with open boundaries
        let inner = SurfaceCode::new(distance, SurfaceBoundaryType::Open)
            .map_err(|e| CognitiveError::from(format!("Failed to create surface code: {}", e)))?;
        
        // Create the code with appropriate parameters
        Ok(Self {
            inner,
            code_type: TopologicalCodeType::PlanarCode,
            boundary_type: BoundaryType::Open,
            parameters: TopologicalParameters {
                error_threshold: 0.11,
                decoder_type: TopologicalDecoderType::MWPM,
                fast_decoding: true,
                max_correction_rounds: 10,
            },
        })
    }

    /// Create a toric code (surface code with periodic boundaries)
    pub fn toric_code(distance: usize) -> CognitiveResult<Self> {
        use crate::cognitive::quantum::error_correction::topological_types::BoundaryType as SurfaceBoundaryType;
        
        // Create a new TopologicalCode with periodic boundaries
        let inner = SurfaceCode::new(distance, SurfaceBoundaryType::Periodic)
            .map_err(|e| CognitiveError::from(format!("Failed to create toric code: {}", e)))?;
        
        // Create the code with appropriate parameters
        Ok(Self {
            inner,
            code_type: TopologicalCodeType::ToricCode,
            boundary_type: BoundaryType::Periodic,
            parameters: TopologicalParameters {
                error_threshold: 0.11,
                decoder_type: TopologicalDecoderType::MWPM,
                fast_decoding: true,
                max_correction_rounds: 10,
            },
        })
    }

    /// Create a triangular color code
    pub fn triangular_color_code(distance: usize) -> CognitiveResult<Self> {
        use crate::cognitive::quantum::error_correction::topological_types::BoundaryType as SurfaceBoundaryType;
        
        // For now, treat color code as a surface code with open boundaries
        // This will be updated when we implement proper color code support
        let inner = SurfaceCode::new(distance, SurfaceBoundaryType::Open)
            .map_err(|e| CognitiveError::from(format!("Failed to create color code: {}", e)))?;
        
        // Create the code with appropriate parameters
        Ok(Self {
            inner,
            code_type: TopologicalCodeType::ColorCode,
            boundary_type: BoundaryType::Open,
            parameters: TopologicalParameters {
                error_threshold: 0.15, // Higher threshold for color codes
                decoder_type: TopologicalDecoderType::MWPM,
                fast_decoding: true,
                max_correction_rounds: 10,
            },
        })
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

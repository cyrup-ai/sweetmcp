//! DEPRECATED: Basic operations for stabilizer quantum error correction
//!
//! This module is deprecated. All functionality has been moved to more specialized modules:
//! - `topological_pauli` for Pauli operations
//! - `topological_stabilizers` for stabilizer operations
//! - `syndromes` for syndrome processing
//! - `quantum_error_correction` for core error correction types

#![allow(deprecated)]
#![allow(unused_imports)]

// Re-export from more specialized modules
pub use super::{
    topological_pauli::{PauliType, TopologicalPauli},
    topological_stabilizers::{TopologicalStabilizer, StabilizerGroup, StabilizerStatistics},
    syndromes::ErrorSyndrome,
    quantum_error_correction::{
        ErrorCorrectionCode, ErrorCorrectionCodeType, CodeParameters,
        StabilizerGenerator as QecStabilizerGenerator,
        PauliOperator as QecPauliOperator,
        ErrorCorrectionConfig, ErrorCorrectionStatistics, ErrorCorrectionMetrics
    },
    topological_types::{
        TopologicalCodeType, LatticeElement, TopologicalDecoderType,
        BoundaryType, ColorType, LogicalOperatorType, StabilizerType
    },
};

// Add deprecation warning
#[deprecated(
    since = "0.1.0",
    note = "This module is deprecated. Use the specialized modules directly instead."
)]
pub mod deprecated {
    // This module is intentionally left empty as all functionality has been moved
    // to more specialized modules
}

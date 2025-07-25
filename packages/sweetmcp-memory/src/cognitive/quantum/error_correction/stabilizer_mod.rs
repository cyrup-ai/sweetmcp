//! Stabilizer quantum error correction module integration
//!
//! This module provides ergonomic re-exports and integration for all stabilizer
//! code components with zero allocation patterns and blazing-fast performance.

// Core types and structures
pub use super::stabilizer_core_types::{
    StabilizerCode, StabilizerCodeParameters, DecoderType,
    StabilizerGenerator, PauliOp, PauliMatrix, LogicalOperator, LogicalOpType,
    ErrorPattern, SyndromeResult,
};

// CSS code types
pub use super::stabilizer_css_types::{
    CSSCode, CSSParameters, ClassicalCodeParams, CSSConstructionParams,
    CSSConstructionMethod, CSSDecodingResult, CSSDecodingMethod, CSSDecodingMetrics,
};

// Basic operations
pub use super::stabilizer_basic_operations::*;

use crate::cognitive::quantum::types::{CognitiveError, CognitiveResult};
use smallvec::SmallVec;

/// Stabilizer code builder for ergonomic construction
pub struct StabilizerCodeBuilder {
    n: Option<usize>,
    k: Option<usize>,
    d: Option<usize>,
    stabilizers: Vec<StabilizerGenerator>,
    logical_x: Vec<LogicalOperator>,
    logical_z: Vec<LogicalOperator>,
    parameters: Option<StabilizerCodeParameters>,
}

/// CSS code builder for ergonomic construction
pub struct CSSCodeBuilder {
    base_code: Option<StabilizerCode>,
    x_stabilizers: Vec<StabilizerGenerator>,
    z_stabilizers: Vec<StabilizerGenerator>,
    css_parameters: Option<CSSParameters>,
}

/// Convenience functions for common stabilizer codes
pub struct CommonCodes;

impl StabilizerCodeBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            n: None,
            k: None,
            d: None,
            stabilizers: Vec::new(),
            logical_x: Vec::new(),
            logical_z: Vec::new(),
            parameters: None,
        }
    }

    /// Set code parameters
    pub fn parameters(mut self, n: usize, k: usize, d: usize) -> Self {
        self.n = Some(n);
        self.k = Some(k);
        self.d = Some(d);
        self
    }

    /// Add stabilizer generator
    pub fn stabilizer(mut self, generator: StabilizerGenerator) -> Self {
        self.stabilizers.push(generator);
        self
    }

    /// Add multiple stabilizer generators
    pub fn stabilizers(mut self, generators: Vec<StabilizerGenerator>) -> Self {
        self.stabilizers.extend(generators);
        self
    }

    /// Add logical X operator
    pub fn logical_x(mut self, operator: LogicalOperator) -> Self {
        self.logical_x.push(operator);
        self
    }

    /// Add logical Z operator
    pub fn logical_z(mut self, operator: LogicalOperator) -> Self {
        self.logical_z.push(operator);
        self
    }

    /// Set code parameters
    pub fn code_parameters(mut self, parameters: StabilizerCodeParameters) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Build the stabilizer code
    pub fn build(self) -> CognitiveResult<StabilizerCode> {
        let n = self.n.ok_or_else(|| CognitiveError::InvalidQuantumState("Missing n parameter".to_string()))?;
        let k = self.k.ok_or_else(|| CognitiveError::InvalidQuantumState("Missing k parameter".to_string()))?;
        let d = self.d.ok_or_else(|| CognitiveError::InvalidQuantumState("Missing d parameter".to_string()))?;

        StabilizerCode::new(n, k, d, self.stabilizers, self.logical_x, self.logical_z)
    }
}

impl CSSCodeBuilder {
    /// Create new CSS builder
    pub fn new() -> Self {
        Self {
            base_code: None,
            x_stabilizers: Vec::new(),
            z_stabilizers: Vec::new(),
            css_parameters: None,
        }
    }

    /// Set base stabilizer code
    pub fn base_code(mut self, code: StabilizerCode) -> Self {
        self.base_code = Some(code);
        self
    }

    /// Add X-type stabilizer
    pub fn x_stabilizer(mut self, generator: StabilizerGenerator) -> Self {
        self.x_stabilizers.push(generator);
        self
    }

    /// Add Z-type stabilizer
    pub fn z_stabilizer(mut self, generator: StabilizerGenerator) -> Self {
        self.z_stabilizers.push(generator);
        self
    }

    /// Set CSS parameters
    pub fn css_parameters(mut self, parameters: CSSParameters) -> Self {
        self.css_parameters = Some(parameters);
        self
    }

    /// Build the CSS code
    pub fn build(self) -> CognitiveResult<CSSCode> {
        let base_code = self.base_code.ok_or_else(|| {
            CognitiveError::InvalidQuantumState("Missing base code".to_string())
        })?;

        let css_parameters = self.css_parameters.ok_or_else(|| {
            CognitiveError::InvalidQuantumState("Missing CSS parameters".to_string())
        })?;

        CSSCode::new(base_code, self.x_stabilizers, self.z_stabilizers, css_parameters)
    }
}

impl CommonCodes {
    /// Create 5-qubit perfect code
    pub fn five_qubit_code() -> CognitiveResult<StabilizerCode> {
        let stabilizers = vec![
            StabilizerGenerator::new(
                "S1".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(0), PauliOp::z(1), PauliOp::z(2), PauliOp::x(3)
                ])
            ),
            StabilizerGenerator::new(
                "S2".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(1), PauliOp::z(2), PauliOp::z(3), PauliOp::x(4)
                ])
            ),
            StabilizerGenerator::new(
                "S3".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(0), PauliOp::x(2), PauliOp::z(3), PauliOp::z(4)
                ])
            ),
            StabilizerGenerator::new(
                "S4".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::z(0), PauliOp::x(1), PauliOp::x(3), PauliOp::z(4)
                ])
            ),
        ];

        let logical_x = vec![
            LogicalOperator {
                id: "X_L".to_string(),
                paulis: SmallVec::from_slice(&[
                    PauliOp::x(0), PauliOp::x(1), PauliOp::x(2), PauliOp::x(3), PauliOp::x(4)
                ]),
                operator_type: LogicalOpType::X,
                logical_qubit: 0,
            }
        ];

        let logical_z = vec![
            LogicalOperator {
                id: "Z_L".to_string(),
                paulis: SmallVec::from_slice(&[
                    PauliOp::z(0), PauliOp::z(1), PauliOp::z(2), PauliOp::z(3), PauliOp::z(4)
                ]),
                operator_type: LogicalOpType::Z,
                logical_qubit: 0,
            }
        ];

        StabilizerCode::new(5, 1, 3, stabilizers, logical_x, logical_z)
    }

    /// Create 7-qubit Steane code
    pub fn steane_code() -> CognitiveResult<StabilizerCode> {
        let stabilizers = vec![
            StabilizerGenerator::new(
                "S1".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(0), PauliOp::x(2), PauliOp::x(4), PauliOp::x(6)
                ])
            ),
            StabilizerGenerator::new(
                "S2".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(1), PauliOp::x(2), PauliOp::x(5), PauliOp::x(6)
                ])
            ),
            StabilizerGenerator::new(
                "S3".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::x(3), PauliOp::x(4), PauliOp::x(5), PauliOp::x(6)
                ])
            ),
            StabilizerGenerator::new(
                "S4".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::z(0), PauliOp::z(2), PauliOp::z(4), PauliOp::z(6)
                ])
            ),
            StabilizerGenerator::new(
                "S5".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::z(1), PauliOp::z(2), PauliOp::z(5), PauliOp::z(6)
                ])
            ),
            StabilizerGenerator::new(
                "S6".to_string(),
                SmallVec::from_slice(&[
                    PauliOp::z(3), PauliOp::z(4), PauliOp::z(5), PauliOp::z(6)
                ])
            ),
        ];

        let logical_x = vec![
            LogicalOperator {
                id: "X_L".to_string(),
                paulis: SmallVec::from_slice(&[
                    PauliOp::x(0), PauliOp::x(1), PauliOp::x(2), PauliOp::x(3),
                    PauliOp::x(4), PauliOp::x(5), PauliOp::x(6)
                ]),
                operator_type: LogicalOpType::X,
                logical_qubit: 0,
            }
        ];

        let logical_z = vec![
            LogicalOperator {
                id: "Z_L".to_string(),
                paulis: SmallVec::from_slice(&[
                    PauliOp::z(0), PauliOp::z(1), PauliOp::z(2), PauliOp::z(3),
                    PauliOp::z(4), PauliOp::z(5), PauliOp::z(6)
                ]),
                operator_type: LogicalOpType::Z,
                logical_qubit: 0,
            }
        ];

        StabilizerCode::new(7, 1, 3, stabilizers, logical_x, logical_z)
    }
}

impl Default for StabilizerCodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CSSCodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
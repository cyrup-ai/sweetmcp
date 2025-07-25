//! Core types for quantum error correction
//!
//! This module re-exports all error correction types for convenient access
//! while maintaining clear separation of concerns across submodules.

pub use crate::cognitive::quantum::error_correction::codes::{ErrorCorrectionCode, PauliOperator};
pub use crate::cognitive::quantum::error_correction::logical_qubits::LogicalQubit;
pub use crate::cognitive::quantum::error_correction::quantum_circuits::{QuantumCircuit, QuantumGate};
pub use crate::cognitive::quantum::error_correction::syndromes::{ErrorSyndrome, QecErrorType};
pub use crate::cognitive::quantum::error_correction::measurements::QuantumMeasurementResult;
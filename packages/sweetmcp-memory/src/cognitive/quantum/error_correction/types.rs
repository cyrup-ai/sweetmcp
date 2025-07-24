//! Core types for quantum error correction
//!
//! This module re-exports all error correction types for convenient access
//! while maintaining clear separation of concerns across submodules.

mod codes;
mod logical_qubits;
mod quantum_circuits;
mod syndromes;
mod measurements;

pub use codes::{ErrorCorrectionCode, PauliOperator};
pub use logical_qubits::LogicalQubit;
pub use quantum_circuits::{QuantumCircuit, QuantumGate};
pub use syndromes::{ErrorSyndrome, ErrorType};
pub use measurements::QuantumMeasurementResult;
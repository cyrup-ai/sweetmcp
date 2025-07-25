//! Error syndrome definitions for quantum error correction
//!
//! This module provides error syndrome structures with correction
//! operations and error type classifications.

use super::gate_impl::QuantumGate;

/// Error syndrome information with correction operations
#[derive(Debug, Clone)]
pub struct ErrorSyndrome {
    pub syndrome_bits: Vec<bool>,
    pub error_location: Vec<usize>,
    pub error_type: QecErrorType,
    pub correction_operation: Vec<QuantumGate>,
}

impl ErrorSyndrome {
    /// Create a new error syndrome
    pub fn new(syndrome_bits: Vec<bool>, error_type: QecErrorType) -> Self {
        Self {
            syndrome_bits,
            error_location: Vec::new(),
            error_type,
            correction_operation: Vec::new(),
        }
    }

    /// Check if syndrome indicates no error
    pub fn is_no_error(&self) -> bool {
        self.syndrome_bits.iter().all(|&b| !b)
    }

    /// Get syndrome weight (number of triggered stabilizers)
    pub fn weight(&self) -> usize {
        self.syndrome_bits.iter().filter(|&&b| b).count()
    }

    /// Add correction operation
    pub fn add_correction(&mut self, gate: QuantumGate) {
        self.correction_operation.push(gate);
    }

    /// Set error location
    pub fn set_error_location(&mut self, location: Vec<usize>) {
        self.error_location = location;
    }
}

/// Types of quantum errors with associated probabilities
#[derive(Debug, Clone, PartialEq)]
pub enum QecErrorType {
    /// Bit flip error (X error)
    BitFlip,
    /// Phase flip error (Z error)  
    PhaseFlip,
    /// Combined bit and phase flip (Y error)
    BitPhaseFlip,
    /// Depolarizing error
    Depolarizing,
    /// Amplitude damping error
    AmplitudeDamping,
    /// Phase damping error
    PhaseDamping,
    /// No error
    NoError,
}

impl QecErrorType {
    /// Get correction gate for single-qubit error
    pub fn correction_gate(&self, qubit: usize) -> Option<QuantumGate> {
        match self {
            QecErrorType::BitFlip => Some(QuantumGate::PauliX { target: qubit }),
            QecErrorType::PhaseFlip => Some(QuantumGate::PauliZ { target: qubit }),
            QecErrorType::BitPhaseFlip => Some(QuantumGate::PauliY { target: qubit }),
            _ => None, // Complex errors need multiple gates
        }
    }

    /// Get error name as string
    pub fn name(&self) -> &'static str {
        match self {
            QecErrorType::BitFlip => "bit_flip",
            QecErrorType::PhaseFlip => "phase_flip",
            QecErrorType::BitPhaseFlip => "bit_phase_flip",
            QecErrorType::Depolarizing => "depolarizing",
            QecErrorType::AmplitudeDamping => "amplitude_damping",
            QecErrorType::PhaseDamping => "phase_damping",
            QecErrorType::NoError => "no_error",
        }
    }

    /// Check if error is correctable by single Pauli
    pub fn is_pauli_correctable(&self) -> bool {
        matches!(self, QecErrorType::BitFlip | QecErrorType::PhaseFlip | QecErrorType::BitPhaseFlip)
    }
}
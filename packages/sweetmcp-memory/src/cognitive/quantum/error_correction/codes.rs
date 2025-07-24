//! Error correction code definitions
//!
//! This module provides fundamental error correction code structures
//! with stabilizer generators and encoding efficiency calculations.

use crate::cognitive::quantum::Complex64;

/// Error correction code definition with optimized structures
#[derive(Debug, Clone)]
pub struct ErrorCorrectionCode {
    pub name: String,
    pub code_distance: usize,
    pub logical_qubits: usize,
    pub physical_qubits: usize,
    pub threshold_error_rate: f64,
    pub stabilizer_generators: Vec<PauliOperator>,
}

impl ErrorCorrectionCode {
    /// Create a new error correction code
    pub fn new(
        name: &str,
        code_distance: usize,
        logical_qubits: usize,
        physical_qubits: usize,
        threshold_error_rate: f64,
    ) -> Self {
        Self {
            name: name.to_string(),
            code_distance,
            logical_qubits,
            physical_qubits,
            threshold_error_rate,
            stabilizer_generators: Vec::new(),
        }
    }

    /// Add stabilizer generator
    pub fn add_stabilizer(&mut self, pauli_string: &str, coefficient: Complex64) {
        self.stabilizer_generators.push(PauliOperator {
            pauli_string: pauli_string.to_string(),
            coefficient,
        });
    }

    /// Get encoding efficiency (logical/physical ratio)
    pub fn encoding_efficiency(&self) -> f64 {
        self.logical_qubits as f64 / self.physical_qubits as f64
    }

    /// Check if error rate is below threshold
    pub fn can_correct_error_rate(&self, error_rate: f64) -> bool {
        error_rate < self.threshold_error_rate
    }

    /// Get maximum correctable errors
    pub fn max_correctable_errors(&self) -> usize {
        (self.code_distance - 1) / 2
    }
}

/// Pauli operator for stabilizer codes with optimized representation
#[derive(Debug, Clone)]
pub struct PauliOperator {
    pub pauli_string: String, // e.g., "XYZII"
    pub coefficient: Complex64,
}

impl PauliOperator {
    /// Create a new Pauli operator
    pub fn new(pauli_string: &str, coefficient: Complex64) -> Self {
        Self {
            pauli_string: pauli_string.to_string(),
            coefficient,
        }
    }

    /// Create identity operator
    pub fn identity(length: usize) -> Self {
        Self {
            pauli_string: "I".repeat(length),
            coefficient: Complex64::new(1.0, 0.0),
        }
    }

    /// Get operator length (number of qubits)
    pub fn length(&self) -> usize {
        self.pauli_string.len()
    }

    /// Check if operator is identity
    pub fn is_identity(&self) -> bool {
        self.pauli_string.chars().all(|c| c == 'I')
    }

    /// Get weight (number of non-identity operators)
    pub fn weight(&self) -> usize {
        self.pauli_string.chars().filter(|&c| c != 'I').count()
    }

    /// Commutes with another Pauli operator
    pub fn commutes_with(&self, other: &PauliOperator) -> bool {
        if self.length() != other.length() {
            return false;
        }

        let mut anticommuting_pairs = 0;
        for (a, b) in self.pauli_string.chars().zip(other.pauli_string.chars()) {
            if (a == 'X' && b == 'Y') || (a == 'Y' && b == 'X') ||
               (a == 'Y' && b == 'Z') || (a == 'Z' && b == 'Y') ||
               (a == 'Z' && b == 'X') || (a == 'X' && b == 'Z') {
                anticommuting_pairs += 1;
            }
        }

        anticommuting_pairs % 2 == 0
    }
}
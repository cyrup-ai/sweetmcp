//! Logical qubit definitions for error correction
//!
//! This module provides logical qubit structures with physical
//! qubit mappings and error syndrome management.

use super::types::ErrorSyndrome;
use super::quantum_circuits::QuantumCircuit;

/// Logical qubit encoded in physical qubits with circuit definitions
#[derive(Debug, Clone)]
pub struct LogicalQubit {
    pub physical_qubit_indices: Vec<usize>,
    pub encoding_circuit: QuantumCircuit,
    pub decoding_circuit: QuantumCircuit,
    pub error_syndromes: Vec<ErrorSyndrome>,
}

impl LogicalQubit {
    /// Create a new logical qubit
    pub fn new(physical_qubit_indices: Vec<usize>) -> Self {
        Self {
            physical_qubit_indices,
            encoding_circuit: QuantumCircuit::new(0),
            decoding_circuit: QuantumCircuit::new(0),
            error_syndromes: Vec::new(),
        }
    }

    /// Get number of physical qubits
    pub fn physical_qubit_count(&self) -> usize {
        self.physical_qubit_indices.len()
    }

    /// Check if qubit index is used
    pub fn uses_qubit(&self, qubit_index: usize) -> bool {
        self.physical_qubit_indices.contains(&qubit_index)
    }

    /// Add error syndrome
    pub fn add_syndrome(&mut self, syndrome: ErrorSyndrome) {
        self.error_syndromes.push(syndrome);
    }

    /// Find syndrome by pattern
    pub fn find_syndrome(&self, syndrome_bits: &[bool]) -> Option<&ErrorSyndrome> {
        self.error_syndromes.iter().find(|s| s.syndrome_bits == syndrome_bits)
    }
}
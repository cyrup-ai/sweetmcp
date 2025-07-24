//! Quantum circuit definitions for error correction
//!
//! This module provides quantum circuit and gate structures
//! optimized for error correction operations.

/// Quantum circuit representation with optimized gate storage
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    pub gates: Vec<QuantumGate>,
    pub qubit_count: usize,
}

impl QuantumCircuit {
    /// Create a new quantum circuit
    pub fn new(qubit_count: usize) -> Self {
        Self {
            gates: Vec::new(),
            qubit_count,
        }
    }

    /// Add a gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) {
        self.gates.push(gate);
    }

    /// Get circuit depth
    pub fn depth(&self) -> usize {
        // Simplified depth calculation - in practice would track dependencies
        self.gates.len()
    }

    /// Get gate count
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Check if circuit is empty
    pub fn is_empty(&self) -> bool {
        self.gates.is_empty()
    }

    /// Clear all gates
    pub fn clear(&mut self) {
        self.gates.clear();
    }
}

/// Quantum gate types for error correction circuits
#[derive(Debug, Clone)]
pub enum QuantumGate {
    /// Pauli-X gate
    PauliX { target: usize },
    /// Pauli-Y gate
    PauliY { target: usize },
    /// Pauli-Z gate
    PauliZ { target: usize },
    /// Hadamard gate
    Hadamard { target: usize },
    /// CNOT gate
    CNOT { control: usize, target: usize },
    /// Controlled-Z gate
    CZ { control: usize, target: usize },
    /// Phase gate
    Phase { target: usize, angle: f64 },
    /// Measurement
    Measure { target: usize, classical_bit: usize },
}

impl QuantumGate {
    /// Get qubits affected by this gate
    pub fn affected_qubits(&self) -> Vec<usize> {
        match self {
            QuantumGate::PauliX { target } |
            QuantumGate::PauliY { target } |
            QuantumGate::PauliZ { target } |
            QuantumGate::Hadamard { target } |
            QuantumGate::Phase { target, .. } |
            QuantumGate::Measure { target, .. } => vec![*target],
            QuantumGate::CNOT { control, target } |
            QuantumGate::CZ { control, target } => vec![*control, *target],
        }
    }

    /// Check if gate is a measurement
    pub fn is_measurement(&self) -> bool {
        matches!(self, QuantumGate::Measure { .. })
    }

    /// Check if gate is single qubit
    pub fn is_single_qubit(&self) -> bool {
        self.affected_qubits().len() == 1 && !self.is_measurement()
    }

    /// Check if gate is two qubit
    pub fn is_two_qubit(&self) -> bool {
        self.affected_qubits().len() == 2
    }
}
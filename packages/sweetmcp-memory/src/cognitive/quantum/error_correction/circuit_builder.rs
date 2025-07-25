//! Circuit builder for creating error correction circuits
//!
//! This module provides a fluent builder pattern for constructing
//! quantum circuits with common error correction operations.

use super::quantum_circuits::QuantumCircuit;
use super::gate_impl::QuantumGate;

/// Circuit builder for creating common error correction circuits
pub struct CircuitBuilder {
    circuit: QuantumCircuit,
}

impl CircuitBuilder {
    /// Create a new circuit builder
    pub fn new(qubit_count: usize) -> Self {
        Self {
            circuit: QuantumCircuit::new(qubit_count),
        }
    }

    /// Add Pauli-X gate
    pub fn x(mut self, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::PauliX { target });
        self
    }

    /// Add Pauli-Y gate
    pub fn y(mut self, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::PauliY { target });
        self
    }

    /// Add Pauli-Z gate
    pub fn z(mut self, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::PauliZ { target });
        self
    }

    /// Add Hadamard gate
    pub fn h(mut self, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::Hadamard { target });
        self
    }

    /// Add CNOT gate
    pub fn cnot(mut self, control: usize, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::CNOT { control, target });
        self
    }

    /// Add CZ gate
    pub fn cz(mut self, control: usize, target: usize) -> Self {
        self.circuit.add_gate(QuantumGate::CZ { control, target });
        self
    }

    /// Add measurement
    pub fn measure(mut self, target: usize, classical_bit: usize) -> Self {
        self.circuit.add_gate(QuantumGate::Measure { target, classical_bit });
        self
    }

    /// Build the circuit
    pub fn build(self) -> QuantumCircuit {
        self.circuit
    }
}
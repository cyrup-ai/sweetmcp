//! Quantum gate implementation for error correction
//!
//! This module provides quantum gate types and operations
//! optimized for error correction circuits.

/// Quantum gate types for error correction circuits
#[derive(Debug, Clone)]
pub enum QuantumGate {
    /// Pauli-X gate (bit flip)
    PauliX { target: usize },
    /// Pauli-Y gate (bit and phase flip)
    PauliY { target: usize },
    /// Pauli-Z gate (phase flip)
    PauliZ { target: usize },
    /// Hadamard gate (creates superposition)
    Hadamard { target: usize },
    /// CNOT gate (controlled-X)
    CNOT { control: usize, target: usize },
    /// Controlled-Z gate
    CZ { control: usize, target: usize },
    /// Phase gate with arbitrary angle
    Phase { target: usize, angle: f64 },
    /// S gate (phase π/2)
    S { target: usize },
    /// T gate (phase π/4)  
    T { target: usize },
    /// Measurement in computational basis
    Measure { target: usize, classical_bit: usize },
    /// Reset to |0⟩ state
    Reset { target: usize },
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
            QuantumGate::S { target } |
            QuantumGate::T { target } |
            QuantumGate::Measure { target, .. } |
            QuantumGate::Reset { target } => vec![*target],
            QuantumGate::CNOT { control, target } |
            QuantumGate::CZ { control, target } => vec![*control, *target],
        }
    }

    /// Check if gate affects specific qubit
    pub fn affects_qubit(&self, qubit: usize) -> bool {
        self.affected_qubits().contains(&qubit)
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

    /// Check if gate is a Pauli gate
    pub fn is_pauli(&self) -> bool {
        matches!(self, QuantumGate::PauliX { .. } | QuantumGate::PauliY { .. } | QuantumGate::PauliZ { .. })
    }

    /// Check if gate is a Clifford gate
    pub fn is_clifford(&self) -> bool {
        matches!(self, 
            QuantumGate::PauliX { .. } | 
            QuantumGate::PauliY { .. } | 
            QuantumGate::PauliZ { .. } |
            QuantumGate::Hadamard { .. } |
            QuantumGate::S { .. } |
            QuantumGate::CNOT { .. } |
            QuantumGate::CZ { .. }
        )
    }

    /// Get gate type as string
    pub fn gate_type(&self) -> &'static str {
        match self {
            QuantumGate::PauliX { .. } => "X",
            QuantumGate::PauliY { .. } => "Y",
            QuantumGate::PauliZ { .. } => "Z",
            QuantumGate::Hadamard { .. } => "H",
            QuantumGate::CNOT { .. } => "CNOT",
            QuantumGate::CZ { .. } => "CZ",
            QuantumGate::Phase { .. } => "Phase",
            QuantumGate::S { .. } => "S",
            QuantumGate::T { .. } => "T",
            QuantumGate::Measure { .. } => "Measure",
            QuantumGate::Reset { .. } => "Reset",
        }
    }

    /// Get inverse of the gate
    pub fn inverse(&self) -> Self {
        match self {
            QuantumGate::PauliX { target } => QuantumGate::PauliX { target: *target },
            QuantumGate::PauliY { target } => QuantumGate::PauliY { target: *target },
            QuantumGate::PauliZ { target } => QuantumGate::PauliZ { target: *target },
            QuantumGate::Hadamard { target } => QuantumGate::Hadamard { target: *target },
            QuantumGate::CNOT { control, target } => QuantumGate::CNOT { control: *control, target: *target },
            QuantumGate::CZ { control, target } => QuantumGate::CZ { control: *control, target: *target },
            QuantumGate::Phase { target, angle } => QuantumGate::Phase { target: *target, angle: -angle },
            QuantumGate::S { target } => QuantumGate::Phase { target: *target, angle: -std::f64::consts::PI / 2.0 },
            QuantumGate::T { target } => QuantumGate::Phase { target: *target, angle: -std::f64::consts::PI / 4.0 },
            QuantumGate::Measure { target, classical_bit } => QuantumGate::Measure { target: *target, classical_bit: *classical_bit },
            QuantumGate::Reset { target } => QuantumGate::Reset { target: *target },
        }
    }

    /// Check if gate commutes with another gate
    pub fn commutes_with(&self, other: &QuantumGate) -> bool {
        let self_qubits = self.affected_qubits();
        let other_qubits = other.affected_qubits();
        
        // Gates on different qubits always commute
        if self_qubits.iter().all(|q| !other_qubits.contains(q)) {
            return true;
        }

        // Same gates on same qubits commute
        if std::mem::discriminant(self) == std::mem::discriminant(other) && self_qubits == other_qubits {
            return true;
        }

        // Specific commutation rules for common gates
        match (self, other) {
            (QuantumGate::PauliZ { target: t1 }, QuantumGate::CNOT { control: c2, target: _ }) |
            (QuantumGate::CNOT { control: c2, target: _ }, QuantumGate::PauliZ { target: t1 }) => t1 == c2,
            (QuantumGate::PauliX { target: t1 }, QuantumGate::CNOT { control: _, target: t2 }) |
            (QuantumGate::CNOT { control: _, target: t2 }, QuantumGate::PauliX { target: t1 }) => t1 == t2,
            _ => false, // Conservative approach for other cases
        }
    }

    /// Get gate cost (arbitrary units for optimization)
    pub fn cost(&self) -> f64 {
        match self {
            QuantumGate::PauliX { .. } | QuantumGate::PauliY { .. } | QuantumGate::PauliZ { .. } => 1.0,
            QuantumGate::Hadamard { .. } | QuantumGate::S { .. } | QuantumGate::T { .. } => 1.0,
            QuantumGate::Phase { .. } => 1.0,
            QuantumGate::CNOT { .. } | QuantumGate::CZ { .. } => 2.0, // Two-qubit gates are more expensive
            QuantumGate::Measure { .. } => 5.0, // Measurements are expensive
            QuantumGate::Reset { .. } => 3.0,
        }
    }
}
//! Quantum circuits and gates for error correction
//!
//! This module coordinates circuit-related functionality with
//! clear separation between implementation, gates, and builders.

pub use crate::cognitive::quantum::error_correction::circuit_impl::QuantumCircuit;
pub use crate::cognitive::quantum::error_correction::gate_impl::QuantumGate;
pub use crate::cognitive::quantum::error_correction::circuit_builder::CircuitBuilder;
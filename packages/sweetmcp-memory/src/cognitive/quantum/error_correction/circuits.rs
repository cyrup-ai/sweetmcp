//! Quantum circuits and gates for error correction
//!
//! This module coordinates circuit-related functionality with
//! clear separation between implementation, gates, and builders.

mod circuit_impl;
mod gate_impl;
mod circuit_builder;

pub use circuit_impl::QuantumCircuit;
pub use gate_impl::QuantumGate;
pub use circuit_builder::CircuitBuilder;
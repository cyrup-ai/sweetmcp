//! Production-grade quantum-inspired routing system
//!
//! This module implements quantum-inspired algorithms for cognitive routing,
//! including superposition states, entanglement networks, and quantum measurement.

pub mod complex;
pub mod config;
pub mod entanglement;
pub mod error_correction;
pub mod hardware;
pub mod mcts_integration;
pub mod measurement;
pub mod metrics;
pub mod ml_decoder;
pub mod recursive_improvement;
pub mod router;
pub mod state;
pub mod types;

pub use complex::Complex64;
pub use config::{QuantumOrchestrationConfig, RecursiveState};
pub use entanglement::{EntanglementGraph, EntanglementLink};
pub use error_correction::{ErrorCorrectionCode, QuantumErrorCorrection};
pub use hardware::{QuantumConfig, QuantumHardwareBackend};
pub use mcts_integration::QuantumOrchestrator;
pub use recursive_improvement::RecursiveImprovement;
pub use measurement::{BasisType, MeasurementBasis, MeasurementOperator};
pub use metrics::QuantumMetrics;
pub use ml_decoder::{MLDecoder, MLModelType, QuantumLayer};
pub use router::QuantumRouter;
pub use state::{PhaseEvolution, SuperpositionState, TimeDependentTerm};
pub use types::*;

// Tests are located in the tests/ directory

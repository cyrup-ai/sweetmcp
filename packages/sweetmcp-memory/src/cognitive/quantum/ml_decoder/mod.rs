//! ML decoder module decomposition
//!
//! This module provides the decomposed ML decoder functionality split into
//! logical modules for better maintainability and adherence to the 300-line limit.

pub mod core;
pub mod training;
pub mod inference;
pub mod decoding;
pub mod optimizers;
pub mod gradients;
pub mod quantum_ops;
pub mod config;

// Re-export key types and functions for backward compatibility
pub use core::{
    MLDecoder, MLModelType, QuantumLayer, ParameterizedGate, ParameterizedGateType,
    EntanglingStructure, InferenceEngine, OptimizationBackend, GradientMethod,
    HardwareAcceleration, TrainingData
};

pub use training::{
    TrainingConfig, TrainingReport, CrossValidationReport, HyperparameterGrid,
    HyperparameterTrial, HyperparameterResult, EarlyStoppingConfig,
    LearningRateScheduler, MetricsTracker
};

pub use inference::{
    ModelMetrics, ComplexityMetrics, ValidationResult
};
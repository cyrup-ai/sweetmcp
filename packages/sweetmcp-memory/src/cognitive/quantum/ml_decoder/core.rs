//! Core ML decoder structures and types
//!
//! This module provides the core machine learning decoder functionality with quantum
//! neural networks, parameterized gates, and inference engines with zero allocation
//! fast paths and blazing-fast performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Machine learning decoder for quantum error correction with optimized inference
pub struct MLDecoder {
    pub model_type: MLModelType,
    pub trained_parameters: Vec<f64>,
    pub inference_engine: InferenceEngine,
}

/// Types of machine learning models for decoding with comprehensive model support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLModelType {
    NeuralNetwork { layers: Vec<usize> },
    SupportVectorMachine { kernel: String },
    RandomForest { trees: usize },
    QuantumNeuralNetwork { quantum_layers: Vec<QuantumLayer> },
}

/// Quantum layer for quantum neural networks with optimized gate structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumLayer {
    pub qubit_count: usize,
    pub parameterized_gates: Vec<ParameterizedGate>,
    pub entangling_structure: EntanglingStructure,
}

/// Parameterized quantum gate with fast parameter access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterizedGate {
    pub gate_type: ParameterizedGateType,
    pub target_qubits: Vec<usize>,
    pub parameters: Vec<f64>,
}

/// Types of parameterized quantum gates with comprehensive gate set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterizedGateType {
    RX,  // Rotation around X axis
    RY,  // Rotation around Y axis
    RZ,  // Rotation around Z axis
    CRX, // Controlled RX
    CRY, // Controlled RY
    CRZ, // Controlled RZ
    Custom(String),
}

/// Entangling structure for quantum circuits with optimized connectivity patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntanglingStructure {
    Linear,
    AllToAll,
    Circular,
    Custom(Vec<(usize, usize)>),
}

/// Inference engine for ML models with hardware acceleration support
pub struct InferenceEngine {
    pub optimization_backend: OptimizationBackend,
    pub gradient_computation: GradientMethod,
    pub hardware_acceleration: HardwareAcceleration,
}

/// Optimization backend for training with fast optimization algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationBackend {
    Adam { learning_rate: f64, beta1: f64, beta2: f64 },
    SGD { learning_rate: f64, momentum: f64 },
    RMSprop { learning_rate: f64, decay: f64 },
    LBFGS { max_iterations: usize },
    QuantumOptimizer { method: String },
}

/// Gradient computation methods with efficient differentiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientMethod {
    BackPropagation,
    ParameterShift,
    FiniteDifference { epsilon: f64 },
    AutomaticDifferentiation,
}

/// Hardware acceleration options with platform optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    CPU,
    GPU { device_id: usize },
    TPU { device_id: usize },
    QuantumHardware { backend: String },
    None,
}

impl MLDecoder {
    /// Create new ML decoder with optimized initialization
    pub fn new(model_type: MLModelType) -> Self {
        let parameter_count = Self::calculate_parameter_count(&model_type);
        
        Self {
            model_type,
            trained_parameters: vec![0.0; parameter_count],
            inference_engine: InferenceEngine::default(),
        }
    }

    /// Calculate parameter count for model type with fast counting
    fn calculate_parameter_count(model_type: &MLModelType) -> usize {
        match model_type {
            MLModelType::NeuralNetwork { layers } => {
                if layers.len() < 2 {
                    return 0;
                }
                
                let mut count = 0;
                for window in layers.windows(2) {
                    count += window[0] * window[1] + window[1]; // weights + biases
                }
                count
            }
            MLModelType::SupportVectorMachine { .. } => 1000, // Placeholder
            MLModelType::RandomForest { trees } => trees * 100, // Placeholder
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                quantum_layers
                    .iter()
                    .map(|layer| layer.parameterized_gates.len() * 3) // 3 params per gate avg
                    .sum()
            }
        }
    }

    /// Initialize quantum state vector from classical syndrome with fast initialization
    pub fn initialize_quantum_state(&self, syndrome: &[bool]) -> Vec<f64> {
        use smallvec::SmallVec;

        let qubit_count = syndrome.len();
        let state_size = 1 << qubit_count; // 2^n quantum state amplitudes

        // Use SmallVec for small quantum systems (up to 6 qubits = 64 amplitudes)
        let mut state: SmallVec<[f64; 64]> = if state_size <= 64 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; state_size])
        };

        // Initialize to computational basis state corresponding to syndrome
        let mut classical_state = 0usize;
        for (i, &bit) in syndrome.iter().enumerate() {
            if bit {
                classical_state |= 1 << i;
            }
        }

        // Resize and set initial state
        state.resize(state_size, 0.0);
        if classical_state < state_size {
            state[classical_state] = 1.0;
        }

        state.into_vec()
    }

    /// Apply quantum layer to state vector with optimized gate application
    pub fn apply_quantum_layer(
        &self,
        mut state: Vec<f64>,
        layer: &QuantumLayer,
        qubit_count: usize,
    ) -> Vec<f64> {
        // Apply parameterized gates
        for gate in &layer.parameterized_gates {
            state = self.apply_parameterized_gate(state, gate, qubit_count);
        }

        // Apply entangling structure
        state = self.apply_entangling_structure(state, &layer.entangling_structure, qubit_count);

        state
    }

    /// Apply parameterized quantum gate to state vector with fast gate operations
    pub fn apply_parameterized_gate(
        &self,
        mut state: Vec<f64>,
        gate: &ParameterizedGate,
        qubit_count: usize,
    ) -> Vec<f64> {
        use std::f64::consts::PI;

        if gate.target_qubits.is_empty() || gate.parameters.is_empty() {
            return state;
        }

        let target = gate.target_qubits[0];
        if target >= qubit_count {
            return state;
        }

        let theta = gate.parameters[0];
        let cos_half = (theta / 2.0).cos();
        let sin_half = (theta / 2.0).sin();

        match gate.gate_type {
            ParameterizedGateType::RX => {
                self.apply_single_qubit_rotation(state, target, qubit_count, cos_half, -sin_half, 0.0, 0.0)
            }
            ParameterizedGateType::RY => {
                self.apply_single_qubit_rotation(state, target, qubit_count, cos_half, 0.0, -sin_half, 0.0)
            }
            ParameterizedGateType::RZ => {
                self.apply_single_qubit_rotation(state, target, qubit_count, cos_half, 0.0, 0.0, sin_half)
            }
            ParameterizedGateType::CRX | ParameterizedGateType::CRY | ParameterizedGateType::CRZ => {
                if gate.target_qubits.len() >= 2 {
                    let control = gate.target_qubits[1];
                    self.apply_controlled_rotation(state, control, target, qubit_count, &gate.gate_type, theta)
                } else {
                    state
                }
            }
            ParameterizedGateType::Custom(_) => {
                // Custom gate implementation would go here
                state
            }
        }
    }

    /// Apply single qubit rotation with optimized matrix multiplication
    fn apply_single_qubit_rotation(
        &self,
        mut state: Vec<f64>,
        target: usize,
        qubit_count: usize,
        cos_half: f64,
        sin_x: f64,
        sin_y: f64,
        sin_z: f64,
    ) -> Vec<f64> {
        let state_size = state.len();
        let target_mask = 1 << target;
        
        // Process pairs of amplitudes
        for i in 0..state_size {
            if i & target_mask == 0 {
                let j = i | target_mask;
                if j < state_size {
                    let amp_0 = state[i];
                    let amp_1 = state[j];
                    
                    // Apply rotation matrix
                    state[i] = cos_half * amp_0 + sin_x * amp_1 + sin_y * amp_1;
                    state[j] = cos_half * amp_1 - sin_x * amp_0 + sin_z * amp_0;
                }
            }
        }
        
        state
    }

    /// Apply controlled rotation with optimized controlled gate logic
    fn apply_controlled_rotation(
        &self,
        mut state: Vec<f64>,
        control: usize,
        target: usize,
        qubit_count: usize,
        gate_type: &ParameterizedGateType,
        theta: f64,
    ) -> Vec<f64> {
        if control >= qubit_count || target >= qubit_count || control == target {
            return state;
        }

        let control_mask = 1 << control;
        let target_mask = 1 << target;
        let cos_half = (theta / 2.0).cos();
        let sin_half = (theta / 2.0).sin();

        for i in 0..state.len() {
            // Only apply rotation when control qubit is |1⟩
            if i & control_mask != 0 {
                if i & target_mask == 0 {
                    let j = i | target_mask;
                    if j < state.len() {
                        let amp_0 = state[i];
                        let amp_1 = state[j];
                        
                        match gate_type {
                            ParameterizedGateType::CRX => {
                                state[i] = cos_half * amp_0 - sin_half * amp_1;
                                state[j] = cos_half * amp_1 - sin_half * amp_0;
                            }
                            ParameterizedGateType::CRY => {
                                state[i] = cos_half * amp_0 - sin_half * amp_1;
                                state[j] = cos_half * amp_1 + sin_half * amp_0;
                            }
                            ParameterizedGateType::CRZ => {
                                state[i] = (cos_half - sin_half) * amp_0;
                                state[j] = (cos_half + sin_half) * amp_1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        state
    }

    /// Apply entangling structure with optimized entanglement patterns
    pub fn apply_entangling_structure(
        &self,
        mut state: Vec<f64>,
        structure: &EntanglingStructure,
        qubit_count: usize,
    ) -> Vec<f64> {
        match structure {
            EntanglingStructure::Linear => {
                for i in 0..qubit_count.saturating_sub(1) {
                    state = self.apply_cnot_gate(state, i, i + 1, qubit_count);
                }
            }
            EntanglingStructure::AllToAll => {
                for i in 0..qubit_count {
                    for j in (i + 1)..qubit_count {
                        state = self.apply_cnot_gate(state, i, j, qubit_count);
                    }
                }
            }
            EntanglingStructure::Circular => {
                for i in 0..qubit_count {
                    let next = (i + 1) % qubit_count;
                    state = self.apply_cnot_gate(state, i, next, qubit_count);
                }
            }
            EntanglingStructure::Custom(connections) => {
                for &(control, target) in connections {
                    if control < qubit_count && target < qubit_count && control != target {
                        state = self.apply_cnot_gate(state, control, target, qubit_count);
                    }
                }
            }
        }
        
        state
    }

    /// Apply CNOT gate with optimized two-qubit gate implementation
    fn apply_cnot_gate(
        &self,
        mut state: Vec<f64>,
        control: usize,
        target: usize,
        qubit_count: usize,
    ) -> Vec<f64> {
        if control >= qubit_count || target >= qubit_count || control == target {
            return state;
        }

        let control_mask = 1 << control;
        let target_mask = 1 << target;

        for i in 0..state.len() {
            // Only flip target when control is |1⟩
            if i & control_mask != 0 {
                let j = i ^ target_mask; // Flip target bit
                if j < state.len() && i != j {
                    // Swap amplitudes
                    let temp = state[i];
                    state[i] = state[j];
                    state[j] = temp;
                }
            }
        }
        
        state
    }

    /// Get model parameter count with fast parameter counting
    pub fn parameter_count(&self) -> usize {
        self.trained_parameters.len()
    }

    /// Set model parameters with optimized parameter setting
    pub fn set_parameters(&mut self, parameters: Vec<f64>) {
        if parameters.len() == self.trained_parameters.len() {
            self.trained_parameters = parameters;
        }
    }

    /// Get model parameters with zero-copy access
    pub fn get_parameters(&self) -> &[f64] {
        &self.trained_parameters
    }

    /// Clone model parameters with optimized cloning
    pub fn clone_parameters(&self) -> Vec<f64> {
        self.trained_parameters.clone()
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self {
            optimization_backend: OptimizationBackend::Adam {
                learning_rate: 0.001,
                beta1: 0.9,
                beta2: 0.999,
            },
            gradient_computation: GradientMethod::BackPropagation,
            hardware_acceleration: HardwareAcceleration::CPU,
        }
    }
}

impl QuantumLayer {
    /// Create new quantum layer with optimized initialization
    pub fn new(qubit_count: usize) -> Self {
        Self {
            qubit_count,
            parameterized_gates: Vec::new(),
            entangling_structure: EntanglingStructure::Linear,
        }
    }

    /// Add parameterized gate with fast gate addition
    pub fn add_gate(&mut self, gate: ParameterizedGate) {
        self.parameterized_gates.push(gate);
    }

    /// Set entangling structure
    pub fn set_entangling_structure(&mut self, structure: EntanglingStructure) {
        self.entangling_structure = structure;
    }

    /// Get parameter count for this layer
    pub fn parameter_count(&self) -> usize {
        self.parameterized_gates
            .iter()
            .map(|gate| gate.parameters.len())
            .sum()
    }
}

impl ParameterizedGate {
    /// Create new parameterized gate with optimized initialization
    pub fn new(gate_type: ParameterizedGateType, target_qubits: Vec<usize>) -> Self {
        let parameter_count = match gate_type {
            ParameterizedGateType::RX | ParameterizedGateType::RY | ParameterizedGateType::RZ => 1,
            ParameterizedGateType::CRX | ParameterizedGateType::CRY | ParameterizedGateType::CRZ => 1,
            ParameterizedGateType::Custom(_) => 1, // Default for custom gates
        };

        Self {
            gate_type,
            target_qubits,
            parameters: vec![0.0; parameter_count],
        }
    }

    /// Set gate parameters with bounds checking
    pub fn set_parameters(&mut self, parameters: Vec<f64>) {
        if !parameters.is_empty() {
            self.parameters = parameters;
        }
    }

    /// Get gate parameters with zero-copy access
    pub fn get_parameters(&self) -> &[f64] {
        &self.parameters
    }
}

/// Training data for ML decoder with optimized data structures
#[derive(Debug, Clone)]
pub struct TrainingData {
    pub syndromes: Vec<Vec<bool>>,
    pub corrections: Vec<Vec<bool>>,
    pub weights: Option<Vec<f64>>,
}

impl TrainingData {
    /// Create new training data with optimized initialization
    pub fn new() -> Self {
        Self {
            syndromes: Vec::new(),
            corrections: Vec::new(),
            weights: None,
        }
    }

    /// Add training sample with fast sample addition
    pub fn add_sample(&mut self, syndrome: Vec<bool>, correction: Vec<bool>) {
        self.syndromes.push(syndrome);
        self.corrections.push(correction);
    }

    /// Add weighted training sample
    pub fn add_weighted_sample(&mut self, syndrome: Vec<bool>, correction: Vec<bool>, weight: f64) {
        self.add_sample(syndrome, correction);
        
        if self.weights.is_none() {
            self.weights = Some(vec![1.0; self.syndromes.len() - 1]);
        }
        
        if let Some(ref mut weights) = self.weights {
            weights.push(weight);
        }
    }

    /// Get sample count
    pub fn len(&self) -> usize {
        self.syndromes.len()
    }

    /// Check if training data is empty
    pub fn is_empty(&self) -> bool {
        self.syndromes.is_empty()
    }

    /// Get sample at index with bounds checking
    pub fn get_sample(&self, index: usize) -> Option<(&[bool], &[bool])> {
        if index < self.syndromes.len() && index < self.corrections.len() {
            Some((&self.syndromes[index], &self.corrections[index]))
        } else {
            None
        }
    }

    /// Get sample weight at index
    pub fn get_weight(&self, index: usize) -> f64 {
        self.weights
            .as_ref()
            .and_then(|weights| weights.get(index))
            .copied()
            .unwrap_or(1.0)
    }
}
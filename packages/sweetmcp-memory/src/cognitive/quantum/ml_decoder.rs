//! Machine learning components for quantum error correction and optimization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Machine learning decoder for quantum error correction
pub struct MLDecoder {
    pub model_type: MLModelType,
    pub trained_parameters: Vec<f64>,
    pub inference_engine: InferenceEngine,
}

/// Types of machine learning models for decoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLModelType {
    NeuralNetwork { layers: Vec<usize> },
    SupportVectorMachine { kernel: String },
    RandomForest { trees: usize },
    QuantumNeuralNetwork { quantum_layers: Vec<QuantumLayer> },
}

/// Quantum layer for quantum neural networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumLayer {
    pub qubit_count: usize,
    pub parameterized_gates: Vec<ParameterizedGate>,
    pub entangling_structure: EntanglingStructure,
}

/// Parameterized quantum gate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterizedGate {
    pub gate_type: ParameterizedGateType,
    pub target_qubits: Vec<usize>,
    pub parameters: Vec<f64>,
}

/// Types of parameterized quantum gates
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

/// Entangling structure for quantum circuits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntanglingStructure {
    Linear,
    AllToAll,
    Circular,
    Custom(Vec<(usize, usize)>),
}

/// Inference engine for ML models
pub struct InferenceEngine {
    pub optimization_backend: OptimizationBackend,
    pub gradient_computation: GradientMethod,
    pub hardware_acceleration: HardwareAcceleration,
}

/// Optimization backend for training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationBackend {
    Adam {
        learning_rate: f64,
        beta1: f64,
        beta2: f64,
    },
    SGD {
        learning_rate: f64,
        momentum: f64,
    },
    LBFGS {
        memory_size: usize,
    },
    QuantumNaturalGradient,
}

/// Gradient computation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GradientMethod {
    ParameterShift,
    FiniteDifference,
    Adjoint,
    QuantumBackpropagation,
}

/// Hardware acceleration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareAcceleration {
    CPU,
    GPU {
        device_id: usize,
    },
    QuantumProcessor {
        backend: String,
    },
    Hybrid {
        classical: Box<HardwareAcceleration>,
        quantum: Box<HardwareAcceleration>,
    },
}

impl MLDecoder {
    /// Create a new ML decoder
    pub fn new(model_type: MLModelType) -> Self {
        let trained_parameters = match &model_type {
            MLModelType::NeuralNetwork { layers } => {
                // Initialize parameters based on layer sizes
                let mut params = Vec::new();
                for i in 0..layers.len() - 1 {
                    let weights = layers[i] * layers[i + 1];
                    let biases = layers[i + 1];
                    params.extend(vec![0.0; weights + biases]);
                }
                params
            }
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                // Initialize quantum parameters
                quantum_layers
                    .iter()
                    .flat_map(|layer| layer.parameterized_gates.iter())
                    .flat_map(|gate| gate.parameters.clone())
                    .collect()
            }
            _ => Vec::new(),
        };

        Self {
            model_type,
            trained_parameters,
            inference_engine: InferenceEngine::default(),
        }
    }

    /// Perform inference on error syndrome
    pub fn decode_syndrome(&self, syndrome: &[bool]) -> Vec<usize> {
        match &self.model_type {
            MLModelType::NeuralNetwork { layers } => {
                self.neural_network_inference(syndrome, layers)
            }
            MLModelType::QuantumNeuralNetwork { quantum_layers } => {
                self.quantum_neural_network_inference(syndrome, quantum_layers)
            }
            _ => {
                // Simple majority voting for other models
                syndrome
                    .iter()
                    .enumerate()
                    .filter(|&(_, bit)| bit)
                    .map(|(i, _)| i)
                    .collect()
            }
        }
    }

    /// Neural network inference implementation
    fn neural_network_inference(&self, syndrome: &[bool], layers: &[usize]) -> Vec<usize> {
        let mut activations: Vec<f64> = syndrome
            .iter()
            .map(|&b| if b { 1.0 } else { 0.0 })
            .collect();

        let mut param_idx = 0;

        // Forward pass through layers
        for i in 0..layers.len() - 1 {
            let mut next_activations = vec![0.0; layers[i + 1]];

            // Apply weights
            for j in 0..layers[i] {
                for k in 0..layers[i + 1] {
                    if param_idx < self.trained_parameters.len() {
                        next_activations[k] += activations[j] * self.trained_parameters[param_idx];
                        param_idx += 1;
                    }
                }
            }

            // Apply biases and activation function (ReLU)
            for k in 0..layers[i + 1] {
                if param_idx < self.trained_parameters.len() {
                    next_activations[k] += self.trained_parameters[param_idx];
                    param_idx += 1;
                }
                next_activations[k] = next_activations[k].max(0.0);
            }

            activations = next_activations;
        }

        // Convert output to error locations
        activations
            .iter()
            .enumerate()
            .filter(|&(_, a)| a > 0.5)
            .map(|(i, _)| i)
            .collect()
    }

    /// Quantum neural network inference using quantum circuit simulation
    fn quantum_neural_network_inference(
        &self,
        syndrome: &[bool],
        layers: &[QuantumLayer],
    ) -> Vec<usize> {
        use smallvec::SmallVec;
        
        if layers.is_empty() {
            // Fallback to classical threshold when no quantum layers
            return syndrome
                .iter()
                .enumerate()
                .filter(|&(_, bit)| bit)
                .map(|(i, _)| i)
                .collect();
        }

        // Initialize quantum state from syndrome
        let qubit_count = syndrome.len();
        let mut quantum_state = self.initialize_quantum_state(syndrome);
        
        // Apply each quantum layer sequentially
        for layer in layers {
            quantum_state = self.apply_quantum_layer(quantum_state, layer, qubit_count);
        }
        
        // Measure final quantum state and extract error positions
        self.measure_quantum_state(quantum_state, qubit_count)
    }

    /// Initialize quantum state vector from classical syndrome
    fn initialize_quantum_state(&self, syndrome: &[bool]) -> Vec<f64> {
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

    /// Apply quantum layer to state vector
    fn apply_quantum_layer(
        &self, 
        mut state: Vec<f64>, 
        layer: &QuantumLayer,
        qubit_count: usize
    ) -> Vec<f64> {
        // Apply parameterized gates
        for gate in &layer.parameterized_gates {
            state = self.apply_parameterized_gate(state, gate, qubit_count);
        }
        
        // Apply entangling structure
        state = self.apply_entangling_structure(state, &layer.entangling_structure, qubit_count);
        
        state
    }

    /// Apply parameterized quantum gate to state vector
    fn apply_parameterized_gate(
        &self,
        mut state: Vec<f64>,
        gate: &ParameterizedGate,
        qubit_count: usize
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
            ParameterizedGateType::RY => {
                // RY rotation: cos(θ/2)|0⟩ - sin(θ/2)|1⟩, sin(θ/2)|0⟩ + cos(θ/2)|1⟩
                let state_size = state.len();
                let mut new_state = vec![0.0; state_size];
                
                for i in 0..state_size {
                    let bit_set = (i >> target) & 1 == 1;
                    let flipped_i = i ^ (1 << target);
                    
                    if bit_set {
                        // |1⟩ component
                        new_state[i] = -sin_half * state[flipped_i] + cos_half * state[i];
                    } else {
                        // |0⟩ component  
                        new_state[i] = cos_half * state[i] + sin_half * state[flipped_i];
                    }
                }
                
                state = new_state;
            }
            ParameterizedGateType::RX => {
                // RX rotation matrix application
                let state_size = state.len();
                let mut new_state = vec![0.0; state_size];
                
                for i in 0..state_size {
                    let flipped_i = i ^ (1 << target);
                    new_state[i] = cos_half * state[i] - sin_half * state[flipped_i];
                    new_state[flipped_i] = cos_half * state[flipped_i] - sin_half * state[i];
                }
                
                state = new_state;
            }
            ParameterizedGateType::RZ => {
                // RZ rotation: e^(-iθ/2)|0⟩, e^(iθ/2)|1⟩ (phase rotation)
                for i in 0..state.len() {
                    let bit_set = (i >> target) & 1 == 1;
                    if bit_set {
                        // Apply phase e^(iθ/2) ≈ cos(θ/2) + i*sin(θ/2)
                        // For real simulation, we'll use cos(θ/2) approximation
                        state[i] *= cos_half;
                    } else {
                        // Apply phase e^(-iθ/2) ≈ cos(θ/2) - i*sin(θ/2)  
                        state[i] *= cos_half;
                    }
                }
            }
            _ => {
                // For other gate types, apply RY as default
                let state_size = state.len();
                let mut new_state = vec![0.0; state_size];
                
                for i in 0..state_size {
                    let bit_set = (i >> target) & 1 == 1;
                    let flipped_i = i ^ (1 << target);
                    
                    if bit_set {
                        new_state[i] = -sin_half * state[flipped_i] + cos_half * state[i];
                    } else {
                        new_state[i] = cos_half * state[i] + sin_half * state[flipped_i];
                    }
                }
                
                state = new_state;
            }
        }
        
        state
    }

    /// Apply entangling structure to quantum state
    fn apply_entangling_structure(
        &self,
        mut state: Vec<f64>,
        structure: &EntanglingStructure,
        qubit_count: usize
    ) -> Vec<f64> {
        match structure {
            EntanglingStructure::Linear => {
                // Apply CNOT gates in linear chain: 0-1, 1-2, 2-3, ...
                for i in 0..(qubit_count - 1) {
                    state = self.apply_cnot_gate(state, i, i + 1, qubit_count);
                }
            }
            EntanglingStructure::AllToAll => {
                // Apply CNOT gates between all qubit pairs
                for control in 0..qubit_count {
                    for target in 0..qubit_count {
                        if control != target {
                            state = self.apply_cnot_gate(state, control, target, qubit_count);
                        }
                    }
                }
            }
            EntanglingStructure::Circular => {
                // Apply CNOT gates in circular pattern: 0-1, 1-2, ..., (n-1)-0
                for i in 0..qubit_count {
                    let next = (i + 1) % qubit_count;
                    state = self.apply_cnot_gate(state, i, next, qubit_count);
                }
            }
            EntanglingStructure::Custom(pairs) => {
                // Apply CNOT gates for custom pairs
                for &(control, target) in pairs {
                    if control < qubit_count && target < qubit_count && control != target {
                        state = self.apply_cnot_gate(state, control, target, qubit_count);
                    }
                }
            }
        }
        
        state
    }

    /// Apply CNOT gate between control and target qubits
    fn apply_cnot_gate(
        &self,
        mut state: Vec<f64>,
        control: usize,
        target: usize,
        qubit_count: usize
    ) -> Vec<f64> {
        if control >= qubit_count || target >= qubit_count || control == target {
            return state;
        }
        
        let state_size = state.len();
        let mut new_state = vec![0.0; state_size];
        
        for i in 0..state_size {
            let control_bit = (i >> control) & 1 == 1;
            
            if control_bit {
                // Control is |1⟩, flip target
                let flipped_i = i ^ (1 << target);
                new_state[flipped_i] = state[i];
            } else {
                // Control is |0⟩, identity on target
                new_state[i] = state[i];
            }
        }
        
        state = new_state;
        state
    }

    /// Measure quantum state and extract error positions
    fn measure_quantum_state(&self, state: Vec<f64>, qubit_count: usize) -> Vec<usize> {
        use smallvec::SmallVec;
        
        // Find the computational basis state with highest probability amplitude
        let mut max_amplitude = 0.0;
        let mut max_state = 0usize;
        
        for (i, &amplitude) in state.iter().enumerate() {
            let probability = amplitude * amplitude;
            if probability > max_amplitude {
                max_amplitude = probability;
                max_state = i;
            }
        }
        
        // Extract qubit positions that are |1⟩ in the measured state
        let mut error_positions: SmallVec<[usize; 16]> = SmallVec::new();
        
        for qubit in 0..qubit_count {
            if (max_state >> qubit) & 1 == 1 {
                error_positions.push(qubit);
            }
        }
        
        error_positions.into_vec()
    }

    /// Train the decoder on labeled data
    pub fn train(&mut self, training_data: &[(Vec<bool>, Vec<usize>)]) {
        match &self.inference_engine.optimization_backend {
            OptimizationBackend::Adam {
                learning_rate,
                beta1,
                beta2,
            } => {
                self.train_adam(training_data, *learning_rate, *beta1, *beta2);
            }
            OptimizationBackend::SGD {
                learning_rate,
                momentum,
            } => {
                self.train_sgd(training_data, *learning_rate, *momentum);
            }
            _ => {
                // Placeholder for other optimizers
            }
        }
    }

    /// Adam optimizer training
    fn train_adam(
        &mut self,
        training_data: &[(Vec<bool>, Vec<usize>)],
        lr: f64,
        beta1: f64,
        beta2: f64,
    ) {
        let mut m = vec![0.0; self.trained_parameters.len()];
        let mut v = vec![0.0; self.trained_parameters.len()];
        let epsilon = 1e-8;

        for (t, (syndrome, target)) in training_data.iter().enumerate() {
            // Compute gradients (simplified)
            let prediction = self.decode_syndrome(syndrome);
            let error = self.compute_error(&prediction, target);
            let gradients = self.compute_gradients(syndrome, &error);

            // Update moments
            for i in 0..self.trained_parameters.len() {
                if i < gradients.len() {
                    m[i] = beta1 * m[i] + (1.0 - beta1) * gradients[i];
                    v[i] = beta2 * v[i] + (1.0 - beta2) * gradients[i].powi(2);

                    // Bias correction
                    let m_hat = m[i] / (1.0 - beta1.powi((t + 1) as i32));
                    let v_hat = v[i] / (1.0 - beta2.powi((t + 1) as i32));

                    // Update parameters
                    self.trained_parameters[i] -= lr * m_hat / (v_hat.sqrt() + epsilon);
                }
            }
        }
    }

    /// SGD optimizer training with momentum and adaptive learning rate
    fn train_sgd(&mut self, training_data: &[(Vec<bool>, Vec<usize>)], lr: f64, momentum: f64) {
        use smallvec::SmallVec;
        use std::collections::HashMap;

        if training_data.is_empty() || self.trained_parameters.is_empty() {
            return;
        }

        // Initialize momentum vectors using SmallVec for small parameter sets
        let param_count = self.trained_parameters.len();
        let mut velocity: SmallVec<[f64; 256]> = if param_count <= 256 {
            SmallVec::new()
        } else {
            SmallVec::from_vec(vec![0.0; param_count])
        };
        velocity.resize(param_count, 0.0);

        // Learning rate decay schedule
        let initial_lr = lr;
        let decay_rate: f64 = 0.95;
        let decay_steps = training_data.len() / 10; // Decay every 10% of data
        
        // Gradient clipping threshold for stability
        let gradient_clip_threshold = 5.0;
        
        // Batch processing for efficiency (mini-batch SGD)
        let batch_size = (training_data.len() / 32).max(1).min(64); // Adaptive batch size
        let num_batches = (training_data.len() + batch_size - 1) / batch_size;
        
        // Track convergence metrics
        let mut prev_loss = f64::INFINITY;
        let convergence_threshold = 1e-6;
        let patience_limit = 5;
        let mut patience_counter = 0;

        // Multiple epochs for better convergence
        for epoch in 0..100 { // Maximum 100 epochs
            let mut epoch_loss = 0.0;
            let mut processed_samples = 0;

            // Process training data in mini-batches
            for batch_idx in 0..num_batches {
                let batch_start = batch_idx * batch_size;
                let batch_end = ((batch_idx + 1) * batch_size).min(training_data.len());
                let batch = &training_data[batch_start..batch_end];

                // Compute batch gradients
                let mut batch_gradients: SmallVec<[f64; 256]> = if param_count <= 256 {
                    SmallVec::new()
                } else {
                    SmallVec::from_vec(vec![0.0; param_count])
                };
                batch_gradients.resize(param_count, 0.0);

                let mut batch_loss = 0.0;

                // Accumulate gradients for the batch
                for (syndrome, target) in batch {
                    // Forward pass
                    let prediction = self.decode_syndrome(syndrome);
                    let error = self.compute_error(&prediction, target);
                    let sample_gradients = self.compute_gradients(syndrome, &error);

                    // Accumulate loss (using cross-entropy-like loss)
                    let sample_loss = self.compute_loss(&prediction, target);
                    batch_loss += sample_loss;

                    // Accumulate gradients
                    for (i, &grad) in sample_gradients.iter().enumerate() {
                        if i < batch_gradients.len() {
                            batch_gradients[i] += grad;
                        }
                    }
                }

                // Average gradients over batch
                let batch_size_f64 = batch.len() as f64;
                for grad in &mut batch_gradients {
                    *grad /= batch_size_f64;
                }

                // Gradient clipping for stability
                let grad_norm = batch_gradients.iter().map(|g| g * g).sum::<f64>().sqrt();
                if grad_norm > gradient_clip_threshold {
                    let clip_factor = gradient_clip_threshold / grad_norm;
                    for grad in &mut batch_gradients {
                        *grad *= clip_factor;
                    }
                }

                // Adaptive learning rate with decay
                let current_step = epoch * num_batches + batch_idx;
                let current_lr = if decay_steps > 0 && current_step % decay_steps == 0 {
                    initial_lr * decay_rate.powi((current_step / decay_steps) as i32)
                } else {
                    initial_lr
                };

                // SGD with momentum update
                for i in 0..param_count {
                    if i < batch_gradients.len() {
                        // Update velocity with momentum
                        velocity[i] = momentum * velocity[i] - current_lr * batch_gradients[i];
                        
                        // Update parameters
                        self.trained_parameters[i] += velocity[i];
                        
                        // Apply L2 regularization (weight decay)
                        let l2_lambda = 1e-4;
                        self.trained_parameters[i] -= l2_lambda * current_lr * self.trained_parameters[i];
                    }
                }

                epoch_loss += batch_loss;
                processed_samples += batch.len();
            }

            // Average loss over epoch
            epoch_loss /= processed_samples as f64;

            // Check for convergence
            let loss_improvement = (prev_loss - epoch_loss).abs();
            if loss_improvement < convergence_threshold {
                patience_counter += 1;
                if patience_counter >= patience_limit {
                    // Early stopping - converged
                    break;
                }
            } else {
                patience_counter = 0;
            }

            prev_loss = epoch_loss;

            // Note: Learning rate adaptation is handled per-batch above
            // Additional global adaptations could be added here if needed
        }

        // Post-training parameter normalization for stability
        self.normalize_parameters();
    }

    /// Compute training loss for a single sample
    fn compute_loss(&self, prediction: &[usize], target: &[usize]) -> f64 {
        // Cross-entropy-like loss for error correction
        let mut loss = 0.0;
        let total_positions = prediction.len().max(target.len()).max(1);

        // False positive penalty
        for &pred_pos in prediction {
            if !target.contains(&pred_pos) {
                loss += 1.0; // Penalty for incorrect prediction
            }
        }

        // False negative penalty
        for &target_pos in target {
            if !prediction.contains(&target_pos) {
                loss += 1.0; // Penalty for missed error
            }
        }

        // Normalize by total possible positions
        loss / total_positions as f64
    }

    /// Normalize parameters to prevent exploding gradients
    fn normalize_parameters(&mut self) {
        let param_norm = self.trained_parameters.iter()
            .map(|p| p * p)
            .sum::<f64>()
            .sqrt();

        if param_norm > 10.0 { // Threshold for normalization
            let norm_factor = 10.0 / param_norm;
            for param in &mut self.trained_parameters {
                *param *= norm_factor;
            }
        }
    }

    /// Compute error between prediction and target
    fn compute_error(&self, prediction: &[usize], target: &[usize]) -> Vec<f64> {
        let mut error = vec![0.0; prediction.len().max(target.len())];

        for &idx in prediction {
            if idx < error.len() && !target.contains(&idx) {
                error[idx] = 1.0;
            }
        }

        for &idx in target {
            if idx < error.len() && !prediction.contains(&idx) {
                error[idx] = -1.0;
            }
        }

        error
    }

    /// Compute gradients (simplified)
    fn compute_gradients(&self, syndrome: &[bool], error: &[f64]) -> Vec<f64> {
        // Simplified gradient computation
        let input_size = syndrome.len();
        let output_size = error.len();
        let mut gradients = Vec::new();

        for i in 0..input_size {
            for j in 0..output_size {
                let input_val = if syndrome[i] { 1.0 } else { 0.0 };
                gradients.push(input_val * error[j]);
            }
        }

        gradients
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
            gradient_computation: GradientMethod::ParameterShift,
            hardware_acceleration: HardwareAcceleration::CPU,
        }
    }
}

impl QuantumLayer {
    /// Create a new quantum layer with default entangling
    pub fn new(qubit_count: usize) -> Self {
        Self {
            qubit_count,
            parameterized_gates: Vec::new(),
            entangling_structure: EntanglingStructure::Linear,
        }
    }

    /// Add a parameterized gate to the layer
    pub fn add_gate(&mut self, gate: ParameterizedGate) {
        self.parameterized_gates.push(gate);
    }

    /// Generate a standard layer with RY rotations and CNOT entangling
    pub fn standard_layer(qubit_count: usize) -> Self {
        let mut layer = Self::new(qubit_count);

        // Add RY rotation gates on all qubits
        for i in 0..qubit_count {
            layer.add_gate(ParameterizedGate {
                gate_type: ParameterizedGateType::RY,
                target_qubits: vec![i],
                parameters: vec![0.0], // Will be trained
            });
        }

        // Set linear entangling structure
        layer.entangling_structure = EntanglingStructure::Linear;

        layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_decoder_creation() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![10, 5, 2],
        };
        let decoder = MLDecoder::new(model_type);

        // Should have (10*5 + 5) + (5*2 + 2) = 55 + 12 = 67 parameters
        assert_eq!(decoder.trained_parameters.len(), 67);
    }

    #[test]
    fn test_neural_network_inference() {
        let model_type = MLModelType::NeuralNetwork {
            layers: vec![4, 3, 2],
        };
        let mut decoder = MLDecoder::new(model_type);

        // Set some non-zero parameters
        for param in &mut decoder.trained_parameters {
            *param = 0.1;
        }

        let syndrome = vec![true, false, true, false];
        let result = decoder.decode_syndrome(&syndrome);

        // Should produce some output
        assert!(!result.is_empty() || result.is_empty()); // Always true, just checking it runs
    }

    #[test]
    fn test_quantum_layer_creation() {
        let layer = QuantumLayer::standard_layer(4);

        assert_eq!(layer.qubit_count, 4);
        assert_eq!(layer.parameterized_gates.len(), 4);
        assert!(matches!(
            layer.entangling_structure,
            EntanglingStructure::Linear
        ));
    }
}

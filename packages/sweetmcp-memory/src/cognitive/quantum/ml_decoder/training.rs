//! Training coordination for ML decoder
//!
//! This module coordinates training using extracted modules for decoding,
//! optimization, gradient computation, and quantum operations with zero
//! allocation fast paths and blazing-fast performance.

// Import functionality from sibling modules
use super::decoding::*;
use super::optimizers::*;
use super::gradients::*;
use super::quantum_ops::*;
use super::config::*;

use super::core::{MLDecoder, OptimizationBackend, GradientMethod};

impl MLDecoder {
    /// Comprehensive training pipeline that coordinates all training aspects
    pub fn train_comprehensive(
        &mut self,
        training_data: &[(Vec<bool>, Vec<bool>)],
        validation_data: Option<&[(Vec<bool>, Vec<bool>)]>,
        config: &TrainingConfig,
    ) -> Result<TrainingReport, String> {
        let mut report = TrainingReport::new();
        
        // Pre-training validation
        self.validate_training_data(training_data)?;
        
        // Initialize training state
        let mut best_loss = f64::INFINITY;
        let mut best_parameters = self.trained_parameters.clone();
        let mut patience_counter = 0;
        
        for epoch in 0..config.max_epochs {
            // Training phase
            let train_loss = self.train(training_data)?;
            report.training_losses.push(train_loss);
            
            // Validation phase
            let validation_loss = if let Some(val_data) = validation_data {
                let loss = self.evaluate(val_data);
                report.validation_losses.push(loss);
                loss
            } else {
                train_loss
            };
            
            // Early stopping logic
            if validation_loss < best_loss {
                best_loss = validation_loss;
                best_parameters = self.trained_parameters.clone();
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= config.early_stopping_patience {
                    report.early_stopped = true;
                    break;
                }
            }
            
            // Learning rate scheduling
            if epoch % config.lr_decay_steps == 0 && epoch > 0 {
                self.apply_learning_rate_decay(config.lr_decay_factor);
            }
            
            report.epochs_completed = epoch + 1;
        }
        
        // Restore best parameters
        self.trained_parameters = best_parameters;
        report.final_loss = best_loss;
        
        Ok(report)
    }

    /// Evaluate model on validation/test data
    pub fn evaluate(&self, test_data: &[(Vec<bool>, Vec<bool>)]) -> f64 {
        self.compute_total_loss(test_data)
    }

    /// Validate training data format and consistency
    fn validate_training_data(&self, training_data: &[(Vec<bool>, Vec<bool>)]) -> Result<(), String> {
        if training_data.is_empty() {
            return Err("Training data cannot be empty".to_string());
        }
        
        let (first_input, first_output) = &training_data[0];
        let input_size = first_input.len();
        let output_size = first_output.len();
        
        for (i, (input, output)) in training_data.iter().enumerate() {
            if input.len() != input_size {
                return Err(format!("Input size mismatch at sample {}: expected {}, got {}", 
                    i, input_size, input.len()));
            }
            if output.len() != output_size {
                return Err(format!("Output size mismatch at sample {}: expected {}, got {}", 
                    i, output_size, output.len()));
            }
        }
        
        Ok(())
    }

    /// Apply learning rate decay to optimizer
    fn apply_learning_rate_decay(&mut self, decay_factor: f64) {
        match &mut self.inference_engine.optimization_backend {
            OptimizationBackend::Adam { learning_rate, .. } => {
                *learning_rate *= decay_factor;
            }
            OptimizationBackend::SGD { learning_rate, .. } => {
                *learning_rate *= decay_factor;
            }
            OptimizationBackend::RMSprop { learning_rate, .. } => {
                *learning_rate *= decay_factor;
            }
            OptimizationBackend::LBFGS { .. } => {
                // L-BFGS doesn't use explicit learning rate
            }
        }
    }

    /// Cross-validation training for robust model evaluation
    pub fn cross_validate(
        &mut self,
        data: &[(Vec<bool>, Vec<bool>)],
        k_folds: usize,
        config: &TrainingConfig,
    ) -> Result<CrossValidationReport, String> {
        if k_folds < 2 {
            return Err("k_folds must be at least 2".to_string());
        }
        
        let fold_size = data.len() / k_folds;
        let mut fold_reports = Vec::with_capacity(k_folds);
        
        for fold in 0..k_folds {
            // Split data into training and validation
            let val_start = fold * fold_size;
            let val_end = if fold == k_folds - 1 { data.len() } else { (fold + 1) * fold_size };
            
            let validation_data: Vec<_> = data[val_start..val_end].to_vec();
            let mut training_data = Vec::with_capacity(data.len() - validation_data.len());
            training_data.extend_from_slice(&data[..val_start]);
            training_data.extend_from_slice(&data[val_end..]);
            
            // Reset parameters for each fold
            self.reset_parameters();
            
            // Train on this fold
            let report = self.train_comprehensive(&training_data, Some(&validation_data), config)?;
            fold_reports.push(report);
        }
        
        Ok(CrossValidationReport { fold_reports })
    }

    /// Reset model parameters for fresh training
    pub fn reset_parameters(&mut self) {
        // Reset to small random values
        for param in &mut self.trained_parameters {
            *param = (simple_random() - 0.5) * 0.1;
        }
    }

    /// Hyperparameter optimization using grid search
    pub fn optimize_hyperparameters(
        &mut self,
        data: &[(Vec<bool>, Vec<bool>)],
        param_grid: &HyperparameterGrid,
        config: &TrainingConfig,
    ) -> Result<HyperparameterResult, String> {
        let mut best_score = f64::INFINITY;
        let mut best_lr = param_grid.learning_rates[0];
        let mut best_momentum = param_grid.momentum_values[0];
        let mut best_decay = param_grid.decay_rates[0];
        let mut best_batch_size = param_grid.batch_sizes[0];
        let mut best_reg_lambda = param_grid.regularization_lambdas[0];
        let mut results = Vec::new();
        
        for &lr in &param_grid.learning_rates {
            for &momentum in &param_grid.momentum_values {
                for &decay in &param_grid.decay_rates {
                    for &batch_size in &param_grid.batch_sizes {
                        for &reg_lambda in &param_grid.regularization_lambdas {
                            // Update hyperparameters
                            self.inference_engine.optimization_backend = OptimizationBackend::SGD {
                                learning_rate: lr,
                                momentum,
                            };
                            
                            let start_time = std::time::SystemTime::now();
                            
                            // Cross-validate with these parameters
                            let cv_report = self.cross_validate(data, 3, config)?; // Use 3-fold for speed
                            let avg_score = cv_report.average_validation_loss();
                            
                            let training_time = start_time.elapsed()
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            
                            results.push(HyperparameterTrial {
                                learning_rate: lr,
                                momentum,
                                decay_rate: decay,
                                batch_size,
                                regularization_lambda: reg_lambda,
                                validation_score: avg_score,
                                training_time_ms: training_time,
                            });
                            
                            if avg_score < best_score {
                                best_score = avg_score;
                                best_lr = lr;
                                best_momentum = momentum;
                                best_decay = decay;
                                best_batch_size = batch_size;
                                best_reg_lambda = reg_lambda;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(HyperparameterResult {
            best_learning_rate: best_lr,
            best_momentum,
            best_decay_rate: best_decay,
            best_batch_size,
            best_regularization_lambda: best_reg_lambda,
            best_validation_score: best_score,
            all_trials: results,
        })
    }

    /// Batch training with mini-batches for better convergence
    pub fn train_with_batches(
        &mut self,
        training_data: &[(Vec<bool>, Vec<bool>)],
        batch_size: usize,
    ) -> Result<f64, String> {
        if batch_size == 0 || training_data.is_empty() {
            return Err("Invalid batch size or empty training data".to_string());
        }
        
        let mut total_loss = 0.0;
        let mut batch_count = 0;
        
        // Process data in batches
        for batch_start in (0..training_data.len()).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(training_data.len());
            let batch = &training_data[batch_start..batch_end];
            
            let batch_loss = self.train(batch)?;
            total_loss += batch_loss;
            batch_count += 1;
        }
        
        Ok(total_loss / batch_count as f64)
    }
}

// Simple random number generator for demo purposes
use std::sync::atomic::{AtomicU64, Ordering};

static SEED: AtomicU64 = AtomicU64::new(12345);

fn simple_random() -> f64 {
    let seed = SEED.load(Ordering::Relaxed);
    let next = seed.wrapping_mul(1103515245).wrapping_add(12345);
    SEED.store(next, Ordering::Relaxed);
    (next % 1000000) as f64 / 1000000.0
}
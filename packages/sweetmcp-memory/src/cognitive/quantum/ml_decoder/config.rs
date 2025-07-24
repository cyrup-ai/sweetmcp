//! Configuration and reporting structures for ML decoder training
//!
//! This module provides training configuration, reporting structures, and
//! hyperparameter optimization support with zero allocation fast paths.

/// Training configuration parameters
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub max_epochs: usize,
    pub early_stopping_patience: usize,
    pub lr_decay_steps: usize,
    pub lr_decay_factor: f64,
    pub gradient_clip_norm: f64,
    pub regularization_lambda: f64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            max_epochs: 100,
            early_stopping_patience: 10,
            lr_decay_steps: 25,
            lr_decay_factor: 0.9,
            gradient_clip_norm: 10.0,
            regularization_lambda: 1e-4,
        }
    }
}

/// Training report with metrics and statistics
#[derive(Debug, Clone)]
pub struct TrainingReport {
    pub training_losses: Vec<f64>,
    pub validation_losses: Vec<f64>,
    pub epochs_completed: usize,
    pub final_loss: f64,
    pub early_stopped: bool,
}

impl TrainingReport {
    pub fn new() -> Self {
        Self {
            training_losses: Vec::new(),
            validation_losses: Vec::new(),
            epochs_completed: 0,
            final_loss: f64::INFINITY,
            early_stopped: false,
        }
    }
}

/// Cross-validation report
#[derive(Debug, Clone)]
pub struct CrossValidationReport {
    pub fold_reports: Vec<TrainingReport>,
}

impl CrossValidationReport {
    pub fn average_validation_loss(&self) -> f64 {
        let sum: f64 = self.fold_reports.iter().map(|r| r.final_loss).sum();
        sum / self.fold_reports.len() as f64
    }

    pub fn standard_deviation(&self) -> f64 {
        let mean = self.average_validation_loss();
        let variance: f64 = self.fold_reports
            .iter()
            .map(|r| (r.final_loss - mean).powi(2))
            .sum::<f64>() / self.fold_reports.len() as f64;
        variance.sqrt()
    }

    pub fn confidence_interval(&self, confidence_level: f64) -> (f64, f64) {
        let mean = self.average_validation_loss();
        let std_dev = self.standard_deviation();
        let n = self.fold_reports.len() as f64;
        
        // Approximate t-value for common confidence levels
        let t_value = match confidence_level {
            0.95 => 1.96,
            0.99 => 2.576,
            _ => 1.96, // Default to 95%
        };
        
        let margin = t_value * std_dev / n.sqrt();
        (mean - margin, mean + margin)
    }
}

/// Hyperparameter grid for optimization
#[derive(Debug, Clone)]
pub struct HyperparameterGrid {
    pub learning_rates: Vec<f64>,
    pub momentum_values: Vec<f64>,
    pub decay_rates: Vec<f64>,
    pub batch_sizes: Vec<usize>,
    pub regularization_lambdas: Vec<f64>,
}

impl Default for HyperparameterGrid {
    fn default() -> Self {
        Self {
            learning_rates: vec![0.001, 0.01, 0.1],
            momentum_values: vec![0.0, 0.9, 0.99],
            decay_rates: vec![0.9, 0.95, 0.99],
            batch_sizes: vec![16, 32, 64],
            regularization_lambdas: vec![1e-6, 1e-4, 1e-2],
        }
    }
}

/// Single hyperparameter trial result
#[derive(Debug, Clone)]
pub struct HyperparameterTrial {
    pub learning_rate: f64,
    pub momentum: f64,
    pub decay_rate: f64,
    pub batch_size: usize,
    pub regularization_lambda: f64,
    pub validation_score: f64,
    pub training_time_ms: u64,
}

/// Hyperparameter optimization result
#[derive(Debug, Clone)]
pub struct HyperparameterResult {
    pub best_learning_rate: f64,
    pub best_momentum: f64,
    pub best_decay_rate: f64,
    pub best_batch_size: usize,
    pub best_regularization_lambda: f64,
    pub best_validation_score: f64,
    pub all_trials: Vec<HyperparameterTrial>,
}

impl HyperparameterResult {
    pub fn get_top_n_trials(&self, n: usize) -> Vec<&HyperparameterTrial> {
        let mut sorted_trials: Vec<_> = self.all_trials.iter().collect();
        sorted_trials.sort_by(|a, b| a.validation_score.partial_cmp(&b.validation_score).unwrap_or(std::cmp::Ordering::Equal));
        sorted_trials.into_iter().take(n).collect()
    }

    pub fn get_pareto_front(&self) -> Vec<&HyperparameterTrial> {
        let mut pareto_trials = Vec::new();
        
        for trial in &self.all_trials {
            let mut is_dominated = false;
            
            for other in &self.all_trials {
                if other.validation_score <= trial.validation_score && 
                   other.training_time_ms <= trial.training_time_ms &&
                   (other.validation_score < trial.validation_score || other.training_time_ms < trial.training_time_ms) {
                    is_dominated = true;
                    break;
                }
            }
            
            if !is_dominated {
                pareto_trials.push(trial);
            }
        }
        
        pareto_trials
    }
}

/// Early stopping configuration
#[derive(Debug, Clone)]
pub struct EarlyStoppingConfig {
    pub patience: usize,
    pub min_delta: f64,
    pub restore_best_weights: bool,
    pub monitor_metric: String,
}

impl Default for EarlyStoppingConfig {
    fn default() -> Self {
        Self {
            patience: 10,
            min_delta: 1e-6,
            restore_best_weights: true,
            monitor_metric: "validation_loss".to_string(),
        }
    }
}

/// Learning rate scheduler configuration
#[derive(Debug, Clone)]
pub enum LearningRateScheduler {
    StepDecay { step_size: usize, gamma: f64 },
    ExponentialDecay { gamma: f64 },
    CosineAnnealing { t_max: usize, eta_min: f64 },
    ReduceOnPlateau { factor: f64, patience: usize, threshold: f64 },
}

impl Default for LearningRateScheduler {
    fn default() -> Self {
        Self::StepDecay { step_size: 25, gamma: 0.9 }
    }
}

/// Training metrics tracker
#[derive(Debug, Clone)]
pub struct MetricsTracker {
    pub train_losses: Vec<f64>,
    pub val_losses: Vec<f64>,
    pub train_accuracies: Vec<f64>,
    pub val_accuracies: Vec<f64>,
    pub learning_rates: Vec<f64>,
    pub epoch_times: Vec<u64>,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            train_losses: Vec::new(),
            val_losses: Vec::new(),
            train_accuracies: Vec::new(),
            val_accuracies: Vec::new(),
            learning_rates: Vec::new(),
            epoch_times: Vec::new(),
        }
    }

    pub fn record_epoch(&mut self, train_loss: f64, val_loss: f64, lr: f64, epoch_time: u64) {
        self.train_losses.push(train_loss);
        self.val_losses.push(val_loss);
        self.learning_rates.push(lr);
        self.epoch_times.push(epoch_time);
    }

    pub fn get_best_epoch(&self) -> Option<usize> {
        self.val_losses
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
    }

    pub fn moving_average(&self, values: &[f64], window: usize) -> Vec<f64> {
        if window == 0 || values.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(values.len());
        
        for i in 0..values.len() {
            let start = if i >= window - 1 { i - window + 1 } else { 0 };
            let end = i + 1;
            let sum: f64 = values[start..end].iter().sum();
            let avg = sum / (end - start) as f64;
            result.push(avg);
        }
        
        result
    }
}
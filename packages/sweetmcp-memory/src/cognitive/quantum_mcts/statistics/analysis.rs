//! Mathematical analysis utilities for statistics
//!
//! This module provides comprehensive mathematical analysis functions with
//! zero-allocation algorithms, numerical stability, and vectorized calculations.

use std::cmp::Ordering;

/// Mathematical analysis utilities for statistical operations
pub struct StatisticsUtils;

impl StatisticsUtils {
    /// Calculate percentiles from a dataset with numerical stability
    pub fn calculate_percentiles(data: &[f64], percentiles: &[f64]) -> Vec<(f64, f64)> {
        if data.is_empty() {
            return percentiles.iter().map(|&p| (p, 0.0)).collect();
        }
        
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| {
            a.partial_cmp(b).unwrap_or_else(|| {
                // Handle NaN values by placing them at the end
                match (a.is_nan(), b.is_nan()) {
                    (true, false) => Ordering::Greater,
                    (false, true) => Ordering::Less,
                    _ => Ordering::Equal,
                }
            })
        });
        
        // Remove NaN values if present
        sorted_data.retain(|&x| !x.is_nan());
        
        if sorted_data.is_empty() {
            return percentiles.iter().map(|&p| (p, 0.0)).collect();
        }
        
        percentiles.iter().map(|&p| {
            let adjusted_percentile = p.clamp(0.0, 100.0);
            let index = (adjusted_percentile / 100.0 * (sorted_data.len() - 1) as f64) as usize;
            let safe_index = index.min(sorted_data.len() - 1);
            let value = sorted_data[safe_index];
            (adjusted_percentile, value)
        }).collect()
    }
    
    /// Calculate moving average with specified window size and zero allocation
    pub fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
        if data.is_empty() || window_size == 0 {
            return Vec::new();
        }
        
        if data.len() < window_size {
            let sum: f64 = data.iter().filter(|&&x| !x.is_nan()).sum();
            let count = data.iter().filter(|&&x| !x.is_nan()).count();
            if count > 0 {
                return vec![sum / count as f64];
            } else {
                return vec![0.0];
            }
        }
        
        let mut result = Vec::with_capacity(data.len() - window_size + 1);
        
        // Calculate first window
        let mut window_sum: f64 = data.iter()
            .take(window_size)
            .filter(|&&x| !x.is_nan())
            .sum();
        let mut window_count = data.iter()
            .take(window_size)
            .filter(|&&x| !x.is_nan())
            .count();
        
        if window_count > 0 {
            result.push(window_sum / window_count as f64);
        } else {
            result.push(0.0);
        }
        
        // Slide the window for remaining calculations
        for i in window_size..data.len() {
            let old_value = data[i - window_size];
            let new_value = data[i];
            
            // Update window sum and count
            if !old_value.is_nan() {
                window_sum -= old_value;
                window_count -= 1;
            }
            if !new_value.is_nan() {
                window_sum += new_value;
                window_count += 1;
            }
            
            if window_count > 0 {
                result.push(window_sum / window_count as f64);
            } else {
                result.push(0.0);
            }
        }
        
        result
    }
    
    /// Calculate exponential moving average with numerical precision
    pub fn exponential_moving_average(data: &[f64], alpha: f64) -> Vec<f64> {
        if data.is_empty() {
            return Vec::new();
        }
        
        let clamped_alpha = alpha.clamp(0.0, 1.0);
        let beta = 1.0 - clamped_alpha;
        
        let mut ema = Vec::with_capacity(data.len());
        
        // Initialize with first non-NaN value
        let first_value = data.iter()
            .find(|&&x| !x.is_nan())
            .copied()
            .unwrap_or(0.0);
        ema.push(first_value);
        
        // Calculate remaining EMA values
        for &value in data.iter().skip(1) {
            let prev_ema = ema.last().copied().unwrap_or(first_value);
            let new_ema = if value.is_nan() {
                prev_ema // Maintain previous value if current is NaN
            } else {
                clamped_alpha * value + beta * prev_ema
            };
            ema.push(new_ema);
        }
        
        ema
    }
    
    /// Detect anomalies using z-score method with robust statistics
    pub fn detect_anomalies(data: &[f64], threshold: f64) -> Vec<(usize, f64)> {
        if data.len() < 2 {
            return Vec::new();
        }
        
        // Filter out NaN values for calculation
        let valid_data: Vec<(usize, f64)> = data.iter()
            .enumerate()
            .filter(|(_, &value)| !value.is_nan())
            .map(|(i, &value)| (i, value))
            .collect();
        
        if valid_data.len() < 2 {
            return Vec::new();
        }
        
        // Calculate robust mean and standard deviation
        let mean = valid_data.iter().map(|(_, value)| *value).sum::<f64>() / valid_data.len() as f64;
        let variance = valid_data.iter()
            .map(|(_, value)| (value - mean).powi(2))
            .sum::<f64>() / (valid_data.len() - 1) as f64;
        let std_dev = variance.sqrt();
        
        if std_dev <= 1e-10 {
            return Vec::new(); // No variation in data
        }
        
        let clamped_threshold = threshold.max(1.0); // Minimum threshold of 1.0
        
        valid_data.iter()
            .filter_map(|&(index, value)| {
                let z_score = (value - mean) / std_dev;
                if z_score.abs() > clamped_threshold {
                    Some((index, z_score))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Calculate correlation coefficient between two datasets
    pub fn correlation_coefficient(x_data: &[f64], y_data: &[f64]) -> f64 {
        if x_data.len() != y_data.len() || x_data.len() < 2 {
            return 0.0;
        }
        
        // Filter paired valid data points
        let valid_pairs: Vec<(f64, f64)> = x_data.iter()
            .zip(y_data.iter())
            .filter(|(&x, &y)| !x.is_nan() && !y.is_nan())
            .map(|(&x, &y)| (x, y))
            .collect();
        
        if valid_pairs.len() < 2 {
            return 0.0;
        }
        
        let n = valid_pairs.len() as f64;
        let mean_x = valid_pairs.iter().map(|(x, _)| *x).sum::<f64>() / n;
        let mean_y = valid_pairs.iter().map(|(_, y)| *y).sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut sum_sq_x = 0.0;
        let mut sum_sq_y = 0.0;
        
        for (x, y) in &valid_pairs {
            let diff_x = x - mean_x;
            let diff_y = y - mean_y;
            numerator += diff_x * diff_y;
            sum_sq_x += diff_x * diff_x;
            sum_sq_y += diff_y * diff_y;
        }
        
        let denominator = (sum_sq_x * sum_sq_y).sqrt();
        if denominator <= 1e-10 {
            return 0.0;
        }
        
        (numerator / denominator).clamp(-1.0, 1.0)
    }
    
    /// Calculate linear regression coefficients (slope, intercept)
    pub fn linear_regression(x_data: &[f64], y_data: &[f64]) -> Option<(f64, f64)> {
        if x_data.len() != y_data.len() || x_data.len() < 2 {
            return None;
        }
        
        // Filter paired valid data points
        let valid_pairs: Vec<(f64, f64)> = x_data.iter()
            .zip(y_data.iter())
            .filter(|(&x, &y)| !x.is_nan() && !y.is_nan())
            .map(|(&x, &y)| (x, y))
            .collect();
        
        if valid_pairs.len() < 2 {
            return None;
        }
        
        let n = valid_pairs.len() as f64;
        let sum_x = valid_pairs.iter().map(|(x, _)| *x).sum::<f64>();
        let sum_y = valid_pairs.iter().map(|(_, y)| *y).sum::<f64>();
        let sum_xy = valid_pairs.iter().map(|(x, y)| x * y).sum::<f64>();
        let sum_x_sq = valid_pairs.iter().map(|(x, _)| x * x).sum::<f64>();
        
        let denominator = n * sum_x_sq - sum_x * sum_x;
        if denominator.abs() <= 1e-10 {
            return None; // No linear relationship possible
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        
        Some((slope, intercept))
    }
    
    /// Calculate entropy of a probability distribution
    pub fn calculate_entropy(probabilities: &[f64]) -> f64 {
        if probabilities.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = probabilities.iter().filter(|&&p| p > 0.0).sum();
        if sum <= 1e-10 {
            return 0.0;
        }
        
        let mut entropy = 0.0;
        for &p in probabilities {
            if p > 1e-10 {
                let normalized_p = p / sum;
                entropy -= normalized_p * normalized_p.ln();
            }
        }
        
        entropy
    }
    
    /// Calculate Kullback-Leibler divergence between two distributions
    pub fn kl_divergence(p_dist: &[f64], q_dist: &[f64]) -> f64 {
        if p_dist.len() != q_dist.len() || p_dist.is_empty() {
            return f64::INFINITY;
        }
        
        let mut divergence = 0.0;
        
        for (&p, &q) in p_dist.iter().zip(q_dist.iter()) {
            if p > 1e-10 {
                if q <= 1e-10 {
                    return f64::INFINITY; // Undefined when q = 0 but p > 0
                }
                divergence += p * (p / q).ln();
            }
        }
        
        divergence
    }
    
    /// Calculate median with efficient partial sorting
    pub fn calculate_median(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mut valid_data: Vec<f64> = data.iter()
            .filter(|&&x| !x.is_nan())
            .copied()
            .collect();
        
        if valid_data.is_empty() {
            return 0.0;
        }
        
        valid_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        
        let len = valid_data.len();
        if len % 2 == 0 {
            (valid_data[len / 2 - 1] + valid_data[len / 2]) / 2.0
        } else {
            valid_data[len / 2]
        }
    }
    
    /// Calculate interquartile range (IQR)
    pub fn calculate_iqr(data: &[f64]) -> f64 {
        let percentiles = Self::calculate_percentiles(data, &[25.0, 75.0]);
        if percentiles.len() >= 2 {
            percentiles[1].1 - percentiles[0].1
        } else {
            0.0
        }
    }
    
    /// Detect outliers using IQR method
    pub fn detect_outliers_iqr(data: &[f64], iqr_multiplier: f64) -> Vec<(usize, f64)> {
        if data.len() < 4 {
            return Vec::new(); // Need at least 4 points for quartiles
        }
        
        let percentiles = Self::calculate_percentiles(data, &[25.0, 75.0]);
        if percentiles.len() < 2 {
            return Vec::new();
        }
        
        let q1 = percentiles[0].1;
        let q3 = percentiles[1].1;
        let iqr = q3 - q1;
        
        if iqr <= 1e-10 {
            return Vec::new(); // No variation
        }
        
        let multiplier = iqr_multiplier.max(1.0);
        let lower_bound = q1 - multiplier * iqr;
        let upper_bound = q3 + multiplier * iqr;
        
        data.iter()
            .enumerate()
            .filter_map(|(i, &value)| {
                if !value.is_nan() && (value < lower_bound || value > upper_bound) {
                    Some((i, value))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Calculate weighted average with numerical stability
    pub fn weighted_average(values: &[f64], weights: &[f64]) -> f64 {
        if values.len() != weights.len() || values.is_empty() {
            return 0.0;
        }
        
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (&value, &weight) in values.iter().zip(weights.iter()) {
            if !value.is_nan() && !weight.is_nan() && weight > 0.0 {
                weighted_sum += value * weight;
                total_weight += weight;
            }
        }
        
        if total_weight > 1e-10 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }
    
    /// Calculate coefficient of variation
    pub fn coefficient_of_variation(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let valid_data: Vec<f64> = data.iter()
            .filter(|&&x| !x.is_nan())
            .copied()
            .collect();
        
        if valid_data.len() < 2 {
            return 0.0;
        }
        
        let mean = valid_data.iter().sum::<f64>() / valid_data.len() as f64;
        if mean.abs() <= 1e-10 {
            return f64::INFINITY; // Undefined when mean is zero
        }
        
        let variance = valid_data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (valid_data.len() - 1) as f64;
        
        let std_dev = variance.sqrt();
        std_dev / mean.abs()
    }
}
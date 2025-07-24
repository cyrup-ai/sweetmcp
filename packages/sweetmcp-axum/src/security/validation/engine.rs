//! Validation engine with comprehensive rule-based validation
//!
//! This module provides the core validation engine that orchestrates all validation
//! rules with zero allocation patterns, blazing-fast performance, and comprehensive
//! caching for production environments.

use crate::security::validation::core::*;
use crate::security::validation::rules::*;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

/// Validation engine with comprehensive rule-based validation
pub struct ValidationEngine {
    /// Validation rules
    rules: Vec<Arc<dyn ValidationRule>>,
    /// Configuration
    config: ValidationConfig,
    /// Validation cache for performance
    cache: DashMap<u64, ValidationResult>,
    /// Timeout duration
    timeout_duration: Duration,
    /// Atomic counters for performance
    total_validations: AtomicU64,
    successful_validations: AtomicU64,
    failed_validations: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl ValidationEngine {
    /// Create new validation engine with optimized initialization
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            config: ValidationConfig::default(),
            cache: DashMap::new(),
            timeout_duration: Duration::from_millis(1000),
            total_validations: AtomicU64::new(0),
            successful_validations: AtomicU64::new(0),
            failed_validations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        };

        // Add default rules
        engine.add_rule(Arc::new(EmailValidationRule));
        engine.add_rule(Arc::new(UrlValidationRule));
        engine.add_rule(Arc::new(PathTraversalValidationRule));
        engine.add_rule(Arc::new(SqlInjectionValidationRule));
        engine.add_rule(Arc::new(XssValidationRule));
        engine.add_rule(Arc::new(LengthValidationRule::new(1, MAX_INPUT_SIZE)));
        engine.add_rule(Arc::new(CharacterSetValidationRule::alphanumeric_with_spaces()));
        engine.add_rule(Arc::new(NumericValidationRule::decimal()));

        engine
    }

    /// Create engine with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        let mut engine = Self::new();
        engine.config = config;
        engine.timeout_duration = Duration::from_millis(engine.config.validation_timeout_ms);
        engine
    }

    /// Add validation rule
    pub fn add_rule(&mut self, rule: Arc<dyn ValidationRule>) {
        self.rules.push(rule);
        // Sort rules by priority (highest first)
        self.rules.sort_by_key(|rule| std::cmp::Reverse(rule.priority()));
    }

    /// Remove validation rule by name
    pub fn remove_rule(&mut self, rule_name: &str) {
        self.rules.retain(|rule| rule.rule_name() != rule_name);
    }

    /// Validate input with comprehensive checking
    pub async fn validate(
        &self,
        input: &str,
        validation_type: ValidationType,
    ) -> Result<ValidationResult, ValidationError> {
        let start_time = SystemTime::now();
        self.total_validations.fetch_add(1, Ordering::Relaxed);

        // Check input length
        if input.len() > self.config.max_input_length {
            self.failed_validations.fetch_add(1, Ordering::Relaxed);
            return Err(ValidationError::new(
                "INPUT_TOO_LONG",
                "Input exceeds maximum length",
                ValidationSeverity::High,
                "length",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "VALIDATION_ERROR",
                    "Validation failed",
                    ValidationSeverity::High,
                    "engine",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Check cache first for performance
        if self.config.enable_caching {
            let cache_key = self.compute_cache_key(input, validation_type);
            if let Some(cached_result) = self.cache.get(&cache_key) {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(cached_result.clone());
            }
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }

        // Perform validation with timeout
        let validation_future = self.validate_internal(input, validation_type);
        let result = match timeout(self.timeout_duration, validation_future).await {
            Ok(result) => result,
            Err(_) => {
                self.failed_validations.fetch_add(1, Ordering::Relaxed);
                return Err(ValidationError::new(
                    "VALIDATION_TIMEOUT",
                    "Validation timeout exceeded",
                    ValidationSeverity::High,
                    "timeout",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "VALIDATION_ERROR",
                        "Validation failed",
                        ValidationSeverity::High,
                        "engine",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        };

        // Update metrics
        let duration = start_time.elapsed().map(|d| d.as_micros() as u64).unwrap_or(0);
        let mut final_result = result?;
        final_result.duration_us = duration;
        final_result.rules_checked = self.rules.len() as u32;

        if final_result.is_valid() {
            self.successful_validations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_validations.fetch_add(1, Ordering::Relaxed);
        }

        // Cache result for future use
        if self.config.enable_caching {
            let cache_key = self.compute_cache_key(input, validation_type);
            self.cache.insert(cache_key, final_result.clone());
        }

        Ok(final_result)
    }

    /// Internal validation logic
    async fn validate_internal(
        &self,
        input: &str,
        validation_type: ValidationType,
    ) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();

        // Apply all relevant rules
        for rule in &self.rules {
            if rule.applies_to(validation_type) {
                if let Err(error) = rule.validate(input) {
                    if result.add_error(error).is_err() {
                        // Maximum errors reached
                        break;
                    }
                }
            }
        }

        Ok(result)
    }

    /// Compute cache key for input and validation type
    fn compute_cache_key(&self, input: &str, validation_type: ValidationType) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        validation_type.hash(&mut hasher);
        hasher.finish()
    }

    /// Validate email address
    pub async fn validate_email(&self, email: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(email, ValidationType::Email).await
    }

    /// Validate URL
    pub async fn validate_url(&self, url: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(url, ValidationType::Url).await
    }

    /// Validate file path
    pub async fn validate_path(&self, path: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(path, ValidationType::Path).await
    }

    /// Validate for SQL injection
    pub async fn validate_sql_safe(&self, input: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(input, ValidationType::SqlInjection).await
    }

    /// Validate for XSS
    pub async fn validate_xss_safe(&self, input: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(input, ValidationType::Xss).await
    }

    /// Validate JSON
    pub async fn validate_json(&self, json: &str) -> Result<ValidationResult, ValidationError> {
        let mut result = self.validate(json, ValidationType::Json).await?;

        // Additional JSON parsing validation
        if serde_json::from_str::<serde_json::Value>(json).is_err() {
            if let Some(error) = ValidationError::new(
                "JSON_PARSE_ERROR",
                "Invalid JSON format",
                ValidationSeverity::High,
                "json",
                json,
            ) {
                let _ = result.add_error(error);
            }
        }

        Ok(result)
    }

    /// Validate numeric input
    pub async fn validate_numeric(&self, input: &str) -> Result<ValidationResult, ValidationError> {
        self.validate(input, ValidationType::Numeric).await
    }

    /// Validate with custom rules
    pub async fn validate_custom(
        &self,
        input: &str,
        custom_rules: &[Arc<dyn ValidationRule>],
    ) -> Result<ValidationResult, ValidationError> {
        let start_time = SystemTime::now();
        self.total_validations.fetch_add(1, Ordering::Relaxed);

        let mut result = ValidationResult::new();

        // Apply custom rules
        for rule in custom_rules {
            if let Err(error) = rule.validate(input) {
                if result.add_error(error).is_err() {
                    // Maximum errors reached
                    break;
                }
            }
        }

        // Update metrics
        let duration = start_time.elapsed().map(|d| d.as_micros() as u64).unwrap_or(0);
        result.duration_us = duration;
        result.rules_checked = custom_rules.len() as u32;

        if result.is_valid() {
            self.successful_validations.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_validations.fetch_add(1, Ordering::Relaxed);
        }

        Ok(result)
    }

    /// Get validation metrics
    pub fn get_metrics(&self) -> ValidationMetrics {
        ValidationMetrics {
            total_validations: self.total_validations.load(Ordering::Relaxed),
            successful_validations: self.successful_validations.load(Ordering::Relaxed),
            failed_validations: self.failed_validations.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_size: self.cache.len() as u64,
            rule_count: self.rules.len() as u32,
        }
    }

    /// Clear validation cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Set validation timeout
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Get configuration
    pub fn get_config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ValidationConfig) {
        self.config = config;
        self.timeout_duration = Duration::from_millis(self.config.validation_timeout_ms);
    }

    /// Get rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Get rule names
    pub fn rule_names(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.rule_name()).collect()
    }

    /// Check if rule exists
    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.rules.iter().any(|rule| rule.rule_name() == rule_name)
    }

    /// Enable or disable caching
    pub fn set_caching_enabled(&mut self, enabled: bool) {
        self.config.enable_caching = enabled;
        if !enabled {
            self.clear_cache();
        }
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl_seconds: u64) {
        self.config.cache_ttl_seconds = ttl_seconds;
    }

    /// Set maximum input length
    pub fn set_max_input_length(&mut self, max_length: usize) {
        self.config.max_input_length = max_length;
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation metrics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct ValidationMetrics {
    /// Total validations performed
    pub total_validations: u64,
    /// Successful validations
    pub successful_validations: u64,
    /// Failed validations
    pub failed_validations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Cache size
    pub cache_size: u64,
    /// Number of rules
    pub rule_count: u32,
}

impl ValidationMetrics {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_validations == 0 {
            0.0
        } else {
            (self.successful_validations as f64 / self.total_validations as f64) * 100.0
        }
    }

    /// Calculate cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_ops = self.cache_hits + self.cache_misses;
        if total_cache_ops == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_cache_ops as f64) * 100.0
        }
    }

    /// Check if validation performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.success_rate() >= 95.0 && self.cache_hit_rate() >= 80.0
    }

    /// Get performance score (0.0 to 1.0)
    pub fn performance_score(&self) -> f64 {
        let success_weight = 0.7;
        let cache_weight = 0.3;

        let success_score = self.success_rate() / 100.0;
        let cache_score = self.cache_hit_rate() / 100.0;

        (success_score * success_weight + cache_score * cache_weight).min(1.0)
    }

    /// Check if metrics indicate good performance
    pub fn is_performance_good(&self) -> bool {
        self.performance_score() >= 0.9
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }

    /// Get cache miss rate as percentage
    pub fn cache_miss_rate(&self) -> f64 {
        100.0 - self.cache_hit_rate()
    }
}
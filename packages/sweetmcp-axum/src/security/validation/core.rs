//! Core validation types and structures
//!
//! This module provides the core validation framework types with zero allocation
//! patterns, blazing-fast performance, and comprehensive validation support
//! for production environments.

use arrayvec::{ArrayString, ArrayVec};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of validation errors to track without heap allocation
pub const MAX_VALIDATION_ERRORS: usize = 32;

/// Maximum size for validation error messages
pub const MAX_ERROR_MESSAGE_SIZE: usize = 256;

/// Maximum size for validation cache keys
pub const MAX_CACHE_KEY_SIZE: usize = 128;

/// Maximum size for input strings to validate
pub const MAX_INPUT_SIZE: usize = 1024;

/// Maximum number of validation rules per validator
pub const MAX_VALIDATION_RULES: usize = 64;

/// Default padding for cache-line alignment
pub fn default_validation_padding() -> [u8; 64] {
    [0; 64]
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Critical security violation
    Critical,
    /// High severity issue
    High,
    /// Medium severity issue
    Medium,
    /// Low severity issue
    Low,
    /// Informational message
    Info,
}

impl ValidationSeverity {
    /// Get numeric weight for severity
    pub fn weight(&self) -> u32 {
        match self {
            ValidationSeverity::Critical => 5,
            ValidationSeverity::High => 4,
            ValidationSeverity::Medium => 3,
            ValidationSeverity::Low => 2,
            ValidationSeverity::Info => 1,
        }
    }

    /// Check if severity requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, ValidationSeverity::Critical | ValidationSeverity::High)
    }
}

/// Validation error with zero allocation storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code for programmatic handling
    pub error_code: ArrayString<64>,
    /// Human-readable error message
    pub message: ArrayString<MAX_ERROR_MESSAGE_SIZE>,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Validation context
    pub context: ArrayString<64>,
    /// Input that caused the error (truncated if too long)
    pub input: ArrayString<MAX_INPUT_SIZE>,
    /// Timestamp when error occurred
    pub timestamp: u64,
    /// Thread ID where error occurred
    pub thread_id: u64,
}

impl ValidationError {
    /// Create new validation error with optimized initialization
    pub fn new(
        error_code: &str,
        message: &str,
        severity: ValidationSeverity,
        context: &str,
        input: &str,
    ) -> Option<Self> {
        let error_code_str = ArrayString::from(error_code).ok()?;
        let message_str = ArrayString::from(message).ok()?;
        let context_str = ArrayString::from(context).ok()?;
        let input_str = ArrayString::from(input).ok()?;

        Some(Self {
            error_code: error_code_str,
            message: message_str,
            severity,
            context: context_str,
            input: input_str,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            thread_id: std::thread::current().id().as_u64().get(),
        })
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(self.severity, ValidationSeverity::Critical)
    }

    /// Check if error requires immediate action
    pub fn requires_immediate_action(&self) -> bool {
        self.severity.requires_immediate_action()
    }

    /// Get priority score for sorting
    pub fn priority_score(&self) -> u32 {
        self.severity.weight()
    }
}

/// Validation result with zero allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation errors found (stack-allocated)
    pub errors: ArrayVec<ValidationError, MAX_VALIDATION_ERRORS>,
    /// Overall validation status
    pub is_valid: bool,
    /// Validation duration in microseconds
    pub duration_us: u64,
    /// Number of rules checked
    pub rules_checked: u32,
    /// Validation timestamp
    pub timestamp: u64,
    /// Cache padding for performance
    _padding: [u8; 64],
}

impl ValidationResult {
    /// Create new validation result with optimized initialization
    pub fn new() -> Self {
        Self {
            errors: ArrayVec::new(),
            is_valid: true,
            duration_us: 0,
            rules_checked: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            _padding: default_validation_padding(),
        }
    }

    /// Add validation error with capacity checking
    pub fn add_error(&mut self, error: ValidationError) -> Result<(), ValidationError> {
        if self.errors.try_push(error).is_err() {
            // If we can't add more errors, create a capacity error
            if let Some(capacity_error) = ValidationError::new(
                "VALIDATION_CAPACITY_EXCEEDED",
                "Maximum validation errors exceeded",
                ValidationSeverity::Critical,
                "capacity",
                "too_many_errors",
            ) {
                return Err(capacity_error);
            }
        }
        self.is_valid = false;
        Ok(())
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }

    /// Get error count by severity
    pub fn count_by_severity(&self, severity: ValidationSeverity) -> usize {
        self.errors.iter().filter(|e| e.severity == severity).count()
    }

    /// Get critical error count
    pub fn critical_error_count(&self) -> usize {
        self.count_by_severity(ValidationSeverity::Critical)
    }

    /// Get high severity error count
    pub fn high_error_count(&self) -> usize {
        self.count_by_severity(ValidationSeverity::High)
    }

    /// Check if any critical errors exist
    pub fn has_critical_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_critical())
    }

    /// Get highest severity error
    pub fn highest_severity(&self) -> Option<ValidationSeverity> {
        self.errors.iter().map(|e| e.severity).max_by_key(|s| s.weight())
    }

    /// Get most critical error
    pub fn most_critical_error(&self) -> Option<&ValidationError> {
        self.errors.iter().max_by_key(|e| e.priority_score())
    }

    /// Sort errors by priority
    pub fn sort_by_priority(&mut self) {
        self.errors.sort_by_key(|e| std::cmp::Reverse(e.priority_score()));
    }

    /// Get total error weight for scoring
    pub fn total_weight(&self) -> u32 {
        self.errors.iter().map(|e| e.severity.weight()).sum()
    }

    /// Check if errors require immediate action
    pub fn requires_immediate_action(&self) -> bool {
        self.errors.iter().any(|e| e.requires_immediate_action())
    }

    /// Get validation summary
    pub fn get_summary(&self) -> ValidationSummary {
        ValidationSummary {
            is_valid: self.is_valid(),
            total_errors: self.errors.len(),
            critical_errors: self.critical_error_count(),
            high_errors: self.high_error_count(),
            medium_errors: self.count_by_severity(ValidationSeverity::Medium),
            low_errors: self.count_by_severity(ValidationSeverity::Low),
            info_errors: self.count_by_severity(ValidationSeverity::Info),
            duration_us: self.duration_us,
            rules_checked: self.rules_checked,
            requires_action: self.requires_immediate_action(),
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Overall validation status
    pub is_valid: bool,
    /// Total number of errors
    pub total_errors: usize,
    /// Critical severity errors
    pub critical_errors: usize,
    /// High severity errors
    pub high_errors: usize,
    /// Medium severity errors
    pub medium_errors: usize,
    /// Low severity errors
    pub low_errors: usize,
    /// Info level errors
    pub info_errors: usize,
    /// Validation duration in microseconds
    pub duration_us: u64,
    /// Number of rules checked
    pub rules_checked: u32,
    /// Whether immediate action is required
    pub requires_action: bool,
}

impl ValidationSummary {
    /// Check if validation is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_valid && self.critical_errors == 0 && self.high_errors == 0
    }

    /// Get health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        if self.total_errors == 0 {
            return 1.0;
        }

        let mut score = 1.0;
        score -= self.critical_errors as f64 * 0.3;
        score -= self.high_errors as f64 * 0.2;
        score -= self.medium_errors as f64 * 0.1;
        score -= self.low_errors as f64 * 0.05;
        score -= self.info_errors as f64 * 0.01;

        score.max(0.0).min(1.0)
    }
}

/// Validation input types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationType {
    /// Email address validation
    Email,
    /// URL validation
    Url,
    /// Path validation (file system paths)
    Path,
    /// SQL injection prevention
    SqlInjection,
    /// XSS prevention
    Xss,
    /// JSON validation
    Json,
    /// XML validation
    Xml,
    /// Regular expression validation
    Regex,
    /// Length validation
    Length,
    /// Character set validation
    CharacterSet,
    /// Numeric validation
    Numeric,
    /// Date/time validation
    DateTime,
    /// IP address validation
    IpAddress,
    /// Domain name validation
    Domain,
    /// Custom validation
    Custom,
}

impl ValidationType {
    /// Get validation type name
    pub fn name(&self) -> &'static str {
        match self {
            ValidationType::Email => "email",
            ValidationType::Url => "url",
            ValidationType::Path => "path",
            ValidationType::SqlInjection => "sql_injection",
            ValidationType::Xss => "xss",
            ValidationType::Json => "json",
            ValidationType::Xml => "xml",
            ValidationType::Regex => "regex",
            ValidationType::Length => "length",
            ValidationType::CharacterSet => "character_set",
            ValidationType::Numeric => "numeric",
            ValidationType::DateTime => "datetime",
            ValidationType::IpAddress => "ip_address",
            ValidationType::Domain => "domain",
            ValidationType::Custom => "custom",
        }
    }

    /// Get default severity for validation type
    pub fn default_severity(&self) -> ValidationSeverity {
        match self {
            ValidationType::SqlInjection | ValidationType::Xss | ValidationType::Path => {
                ValidationSeverity::Critical
            }
            ValidationType::Email | ValidationType::Url | ValidationType::IpAddress => {
                ValidationSeverity::High
            }
            ValidationType::Json | ValidationType::Xml | ValidationType::Domain => {
                ValidationSeverity::Medium
            }
            ValidationType::Length | ValidationType::CharacterSet | ValidationType::Numeric => {
                ValidationSeverity::Low
            }
            ValidationType::DateTime | ValidationType::Regex | ValidationType::Custom => {
                ValidationSeverity::Medium
            }
        }
    }
}

/// Validation configuration with optimized defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable email validation
    pub enable_email_validation: bool,
    /// Enable URL validation
    pub enable_url_validation: bool,
    /// Enable path validation
    pub enable_path_validation: bool,
    /// Enable SQL injection prevention
    pub enable_sql_injection_prevention: bool,
    /// Enable XSS prevention
    pub enable_xss_prevention: bool,
    /// Maximum validation timeout in milliseconds
    pub validation_timeout_ms: u64,
    /// Maximum input length to validate
    pub max_input_length: usize,
    /// Enable validation caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable SIMD acceleration where available
    pub enable_simd_acceleration: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_email_validation: true,
            enable_url_validation: true,
            enable_path_validation: true,
            enable_sql_injection_prevention: true,
            enable_xss_prevention: true,
            validation_timeout_ms: 1000,
            max_input_length: MAX_INPUT_SIZE,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            enable_simd_acceleration: true,
        }
    }
}
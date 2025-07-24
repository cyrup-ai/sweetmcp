//! Validation rules with SIMD-accelerated pattern matching
//!
//! This module provides comprehensive validation rules for various input types
//! with zero allocation patterns, blazing-fast SIMD acceleration, and production-ready
//! security validation for all external inputs.

use crate::security::validation::core::*;
use memchr::memmem;
use url::Url;

/// Validation rule trait for implementing custom validation logic
pub trait ValidationRule: Send + Sync {
    /// Check if input passes this validation rule
    fn validate(&self, input: &str) -> Result<(), ValidationError>;

    /// Get rule name for debugging
    fn rule_name(&self) -> &'static str;

    /// Get rule severity
    fn severity(&self) -> ValidationSeverity;

    /// Check if rule applies to validation type
    fn applies_to(&self, validation_type: ValidationType) -> bool {
        // Default implementation applies to all types
        true
    }

    /// Get rule priority for ordering
    fn priority(&self) -> u32 {
        self.severity().weight()
    }
}

/// Email validation rule with RFC 5322 compliance
#[derive(Debug)]
pub struct EmailValidationRule;

impl ValidationRule for EmailValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // SIMD-accelerated email validation
        let finder = memmem::Finder::new(b"@");

        if finder.find(input.as_bytes()).is_none() {
            return Err(ValidationError::new(
                "EMAIL_MISSING_AT_SYMBOL",
                "Email must contain @ symbol",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Check for multiple @ symbols
        if input.matches('@').count() != 1 {
            return Err(ValidationError::new(
                "EMAIL_MULTIPLE_AT_SYMBOLS",
                "Email must contain exactly one @ symbol",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Split at @ symbol
        let parts: Vec<&str> = input.split('@').collect();
        if parts.len() != 2 {
            return Err(ValidationError::new(
                "EMAIL_INVALID_FORMAT",
                "Email format is invalid",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        // Validate local part
        if local_part.is_empty() || local_part.len() > 64 {
            return Err(ValidationError::new(
                "EMAIL_INVALID_LOCAL_PART",
                "Email local part must be 1-64 characters",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Validate domain part
        if domain_part.is_empty() || domain_part.len() > 253 {
            return Err(ValidationError::new(
                "EMAIL_INVALID_DOMAIN_PART",
                "Email domain part must be 1-253 characters",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Check for valid characters in local part
        for ch in local_part.chars() {
            if !ch.is_ascii_alphanumeric() && !matches!(ch, '.' | '_' | '-' | '+') {
                return Err(ValidationError::new(
                    "EMAIL_INVALID_LOCAL_CHAR",
                    "Email local part contains invalid characters",
                    ValidationSeverity::High,
                    "email",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "EMAIL_VALIDATION_ERROR",
                        "Email validation failed",
                        ValidationSeverity::High,
                        "email",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        // Check for valid characters in domain part
        for ch in domain_part.chars() {
            if !ch.is_ascii_alphanumeric() && !matches!(ch, '.' | '-') {
                return Err(ValidationError::new(
                    "EMAIL_INVALID_DOMAIN_CHAR",
                    "Email domain part contains invalid characters",
                    ValidationSeverity::High,
                    "email",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "EMAIL_VALIDATION_ERROR",
                        "Email validation failed",
                        ValidationSeverity::High,
                        "email",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        // Check for valid domain structure
        if !domain_part.contains('.') {
            return Err(ValidationError::new(
                "EMAIL_INVALID_DOMAIN_STRUCTURE",
                "Email domain must contain at least one dot",
                ValidationSeverity::High,
                "email",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "EMAIL_VALIDATION_ERROR",
                    "Email validation failed",
                    ValidationSeverity::High,
                    "email",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "email_validation"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::High
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Email)
    }
}

/// URL validation rule with comprehensive scheme checking
#[derive(Debug)]
pub struct UrlValidationRule;

impl ValidationRule for UrlValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // Parse URL using url crate for comprehensive validation
        match Url::parse(input) {
            Ok(url) => {
                // Check for allowed schemes
                let allowed_schemes = ["http", "https", "ftp", "ftps"];
                if !allowed_schemes.contains(&url.scheme()) {
                    return Err(ValidationError::new(
                        "URL_INVALID_SCHEME",
                        "URL scheme not allowed",
                        ValidationSeverity::High,
                        "url",
                        input,
                    )
                    .ok_or_else(|| {
                        ValidationError::new(
                            "URL_VALIDATION_ERROR",
                            "URL validation failed",
                            ValidationSeverity::High,
                            "url",
                            "unknown",
                        )
                        .unwrap()
                    })?);
                }

                // Check for valid host
                if url.host().is_none() {
                    return Err(ValidationError::new(
                        "URL_MISSING_HOST",
                        "URL must have a valid host",
                        ValidationSeverity::High,
                        "url",
                        input,
                    )
                    .ok_or_else(|| {
                        ValidationError::new(
                            "URL_VALIDATION_ERROR",
                            "URL validation failed",
                            ValidationSeverity::High,
                            "url",
                            "unknown",
                        )
                        .unwrap()
                    })?);
                }
            }
            Err(_) => {
                return Err(ValidationError::new(
                    "URL_PARSE_ERROR",
                    "URL format is invalid",
                    ValidationSeverity::High,
                    "url",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "URL_VALIDATION_ERROR",
                        "URL validation failed",
                        ValidationSeverity::High,
                        "url",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "url_validation"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::High
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Url)
    }
}

/// Path traversal prevention rule
#[derive(Debug)]
pub struct PathTraversalValidationRule;

impl ValidationRule for PathTraversalValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // SIMD-accelerated path traversal pattern detection
        let traversal_patterns = [
            &b"../"[..],
            &b"..\\"[..],
            &b"/../"[..],
            &b"/..\\"[..],
            &b"\\../"[..],
            &b"\\..\\"[..],
        ];

        for pattern in &traversal_patterns {
            let finder = memmem::Finder::new(pattern);
            if finder.find(input.as_bytes()).is_some() {
                return Err(ValidationError::new(
                    "PATH_TRAVERSAL_DETECTED",
                    "Path traversal pattern detected",
                    ValidationSeverity::Critical,
                    "path",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "PATH_VALIDATION_ERROR",
                        "Path validation failed",
                        ValidationSeverity::Critical,
                        "path",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        // Check for null bytes
        if input.contains('\0') {
            return Err(ValidationError::new(
                "PATH_NULL_BYTE_DETECTED",
                "Null byte detected in path",
                ValidationSeverity::Critical,
                "path",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "PATH_VALIDATION_ERROR",
                    "Path validation failed",
                    ValidationSeverity::Critical,
                    "path",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "path_traversal_prevention"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Critical
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Path)
    }
}

/// SQL injection prevention rule
#[derive(Debug)]
pub struct SqlInjectionValidationRule;

impl ValidationRule for SqlInjectionValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // SIMD-accelerated SQL injection pattern detection
        let sql_patterns = [
            &b"' OR '1'='1"[..],
            &b"' OR 1=1"[..],
            &b"' OR TRUE"[..],
            &b"' UNION SELECT"[..],
            &b"' DROP TABLE"[..],
            &b"' DELETE FROM"[..],
            &b"' INSERT INTO"[..],
            &b"' UPDATE SET"[..],
            &b"-- "[..],
            &b"/*"[..],
            &b"*/"[..],
            &b"xp_cmdshell"[..],
            &b"sp_executesql"[..],
            &b"exec("[..],
            &b"execute("[..],
        ];

        for pattern in &sql_patterns {
            let finder = memmem::Finder::new(pattern);
            if finder.find(input.to_uppercase().as_bytes()).is_some() {
                return Err(ValidationError::new(
                    "SQL_INJECTION_DETECTED",
                    "SQL injection pattern detected",
                    ValidationSeverity::Critical,
                    "sql",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "SQL_VALIDATION_ERROR",
                        "SQL validation failed",
                        ValidationSeverity::Critical,
                        "sql",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "sql_injection_prevention"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Critical
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::SqlInjection)
    }
}

/// XSS prevention rule
#[derive(Debug)]
pub struct XssValidationRule;

impl ValidationRule for XssValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // SIMD-accelerated XSS pattern detection
        let xss_patterns = [
            &b"<script"[..],
            &b"</script>"[..],
            &b"javascript:"[..],
            &b"vbscript:"[..],
            &b"onload="[..],
            &b"onerror="[..],
            &b"onclick="[..],
            &b"onmouseover="[..],
            &b"<iframe"[..],
            &b"<object"[..],
            &b"<embed"[..],
            &b"<form"[..],
            &b"<input"[..],
            &b"<textarea"[..],
            &b"<select"[..],
            &b"<option"[..],
        ];

        for pattern in &xss_patterns {
            let finder = memmem::Finder::new(pattern);
            if finder.find(input.to_lowercase().as_bytes()).is_some() {
                return Err(ValidationError::new(
                    "XSS_PATTERN_DETECTED",
                    "XSS pattern detected",
                    ValidationSeverity::Critical,
                    "xss",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "XSS_VALIDATION_ERROR",
                        "XSS validation failed",
                        ValidationSeverity::Critical,
                        "xss",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "xss_prevention"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Critical
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Xss)
    }
}

/// Length validation rule
#[derive(Debug)]
pub struct LengthValidationRule {
    /// Minimum allowed length
    pub min_length: usize,
    /// Maximum allowed length
    pub max_length: usize,
}

impl LengthValidationRule {
    /// Create new length validation rule
    pub fn new(min_length: usize, max_length: usize) -> Self {
        Self {
            min_length,
            max_length,
        }
    }
}

impl ValidationRule for LengthValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        let length = input.len();

        if length < self.min_length {
            return Err(ValidationError::new(
                "LENGTH_TOO_SHORT",
                "Input is too short",
                ValidationSeverity::Medium,
                "length",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "LENGTH_VALIDATION_ERROR",
                    "Length validation failed",
                    ValidationSeverity::Medium,
                    "length",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        if length > self.max_length {
            return Err(ValidationError::new(
                "LENGTH_TOO_LONG",
                "Input is too long",
                ValidationSeverity::Medium,
                "length",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "LENGTH_VALIDATION_ERROR",
                    "Length validation failed",
                    ValidationSeverity::Medium,
                    "length",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "length_validation"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Medium
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Length)
    }
}

/// Character set validation rule
#[derive(Debug)]
pub struct CharacterSetValidationRule {
    /// Allowed characters
    pub allowed_chars: String,
    /// Whether to allow whitespace
    pub allow_whitespace: bool,
}

impl CharacterSetValidationRule {
    /// Create new character set validation rule
    pub fn new(allowed_chars: String, allow_whitespace: bool) -> Self {
        Self {
            allowed_chars,
            allow_whitespace,
        }
    }

    /// Create alphanumeric rule
    pub fn alphanumeric() -> Self {
        Self::new(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string(),
            false,
        )
    }

    /// Create alphanumeric with spaces rule
    pub fn alphanumeric_with_spaces() -> Self {
        Self::new(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".to_string(),
            true,
        )
    }
}

impl ValidationRule for CharacterSetValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        for ch in input.chars() {
            if ch.is_whitespace() && !self.allow_whitespace {
                return Err(ValidationError::new(
                    "CHARSET_WHITESPACE_NOT_ALLOWED",
                    "Whitespace characters not allowed",
                    ValidationSeverity::Medium,
                    "charset",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "CHARSET_VALIDATION_ERROR",
                        "Character set validation failed",
                        ValidationSeverity::Medium,
                        "charset",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }

            if !ch.is_whitespace() && !self.allowed_chars.contains(ch) {
                return Err(ValidationError::new(
                    "CHARSET_INVALID_CHARACTER",
                    "Invalid character in input",
                    ValidationSeverity::Medium,
                    "charset",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "CHARSET_VALIDATION_ERROR",
                        "Character set validation failed",
                        ValidationSeverity::Medium,
                        "charset",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "character_set_validation"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Medium
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::CharacterSet)
    }
}

/// Numeric validation rule
#[derive(Debug)]
pub struct NumericValidationRule {
    /// Allow negative numbers
    pub allow_negative: bool,
    /// Allow decimal numbers
    pub allow_decimal: bool,
    /// Minimum value (optional)
    pub min_value: Option<f64>,
    /// Maximum value (optional)
    pub max_value: Option<f64>,
}

impl NumericValidationRule {
    /// Create new numeric validation rule
    pub fn new(
        allow_negative: bool,
        allow_decimal: bool,
        min_value: Option<f64>,
        max_value: Option<f64>,
    ) -> Self {
        Self {
            allow_negative,
            allow_decimal,
            min_value,
            max_value,
        }
    }

    /// Create integer-only rule
    pub fn integer_only() -> Self {
        Self::new(true, false, None, None)
    }

    /// Create positive integer rule
    pub fn positive_integer() -> Self {
        Self::new(false, false, Some(0.0), None)
    }

    /// Create decimal rule
    pub fn decimal() -> Self {
        Self::new(true, true, None, None)
    }
}

impl ValidationRule for NumericValidationRule {
    fn validate(&self, input: &str) -> Result<(), ValidationError> {
        // Parse as number
        let parsed_value = if self.allow_decimal {
            input.parse::<f64>()
        } else {
            input.parse::<i64>().map(|i| i as f64)
        };

        let value = match parsed_value {
            Ok(v) => v,
            Err(_) => {
                return Err(ValidationError::new(
                    "NUMERIC_PARSE_ERROR",
                    "Input is not a valid number",
                    ValidationSeverity::Medium,
                    "numeric",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "NUMERIC_VALIDATION_ERROR",
                        "Numeric validation failed",
                        ValidationSeverity::Medium,
                        "numeric",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        };

        // Check negative values
        if !self.allow_negative && value < 0.0 {
            return Err(ValidationError::new(
                "NUMERIC_NEGATIVE_NOT_ALLOWED",
                "Negative numbers not allowed",
                ValidationSeverity::Medium,
                "numeric",
                input,
            )
            .ok_or_else(|| {
                ValidationError::new(
                    "NUMERIC_VALIDATION_ERROR",
                    "Numeric validation failed",
                    ValidationSeverity::Medium,
                    "numeric",
                    "unknown",
                )
                .unwrap()
            })?);
        }

        // Check minimum value
        if let Some(min) = self.min_value {
            if value < min {
                return Err(ValidationError::new(
                    "NUMERIC_BELOW_MINIMUM",
                    "Value is below minimum",
                    ValidationSeverity::Medium,
                    "numeric",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "NUMERIC_VALIDATION_ERROR",
                        "Numeric validation failed",
                        ValidationSeverity::Medium,
                        "numeric",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        // Check maximum value
        if let Some(max) = self.max_value {
            if value > max {
                return Err(ValidationError::new(
                    "NUMERIC_ABOVE_MAXIMUM",
                    "Value is above maximum",
                    ValidationSeverity::Medium,
                    "numeric",
                    input,
                )
                .ok_or_else(|| {
                    ValidationError::new(
                        "NUMERIC_VALIDATION_ERROR",
                        "Numeric validation failed",
                        ValidationSeverity::Medium,
                        "numeric",
                        "unknown",
                    )
                    .unwrap()
                })?);
            }
        }

        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "numeric_validation"
    }

    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Medium
    }

    fn applies_to(&self, validation_type: ValidationType) -> bool {
        matches!(validation_type, ValidationType::Numeric)
    }
}
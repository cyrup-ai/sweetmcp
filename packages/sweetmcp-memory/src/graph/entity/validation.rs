//! Entity validation system
//!
//! This module provides validation framework for entities with
//! rule-based validation and performance optimizations.

use super::core::Entity;
pub use super::types::AttributeType;
use super::validation_rules::{RequiredAttributeRule, AttributeTypeRule, AttributeRangeRule, AttributeLengthRule};
use crate::graph::graph_db::{GraphError, Result};

/// Validation rule trait for entities
pub trait ValidationRule: Send + Sync {
    /// Validate an entity
    fn validate(&self, entity: &dyn Entity) -> Result<()>;

    /// Get the rule name
    fn name(&self) -> &str;

    /// Clone the validation rule
    fn clone_rule(&self) -> Box<dyn ValidationRule>;
}


/// Entity validator that manages and executes validation rules
pub struct EntityValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
}

impl EntityValidator {
    /// Create a new entity validator
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a validation rule
    pub fn add_rule<R: ValidationRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// Add a required attribute rule
    pub fn require_attribute(&mut self, attribute: &str) {
        self.add_rule(RequiredAttributeRule::new(attribute));
    }

    /// Add an attribute type rule
    pub fn require_attribute_type(&mut self, attribute: &str, attr_type: AttributeType) {
        self.add_rule(AttributeTypeRule::new(attribute, attr_type));
    }

    /// Add an attribute range rule
    pub fn require_attribute_range(&mut self, attribute: &str, min: Option<f64>, max: Option<f64>) {
        self.add_rule(AttributeRangeRule::new(attribute, min, max));
    }

    /// Add an attribute length rule
    pub fn require_attribute_length(&mut self, attribute: &str, min: Option<usize>, max: Option<usize>) {
        self.add_rule(AttributeLengthRule::new(attribute, min, max));
    }

    /// Validate an entity against all rules with early termination
    pub fn validate(&self, entity: &dyn Entity) -> Result<()> {
        for rule in &self.rules {
            rule.validate(entity)?;
        }
        Ok(())
    }

    /// Get all rule names
    pub fn rule_names(&self) -> Vec<&str> {
        self.rules.iter().map(|rule| rule.name()).collect()
    }

    /// Get rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Check if validator has any rules
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Clear all rules
    pub fn clear(&mut self) {
        self.rules.clear();
    }

    /// Remove rule by name
    pub fn remove_rule(&mut self, name: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|rule| rule.name() != name);
        self.rules.len() < initial_len
    }

    /// Validate and collect all errors (doesn't stop at first error)
    pub fn validate_all_errors(&self, entity: &dyn Entity) -> Vec<GraphError> {
        let mut errors = Vec::new();
        for rule in &self.rules {
            if let Err(error) = rule.validate(entity) {
                errors.push(error);
            }
        }
        errors
    }

    /// Check if entity passes validation (boolean result)
    pub fn is_valid(&self, entity: &dyn Entity) -> bool {
        self.validate(entity).is_ok()
    }

    /// Get validation summary for an entity
    pub fn validation_summary(&self, entity: &dyn Entity) -> ValidationSummary {
        let mut passed_rules = Vec::new();
        let mut failed_rules = Vec::new();

        for rule in &self.rules {
            match rule.validate(entity) {
                Ok(_) => passed_rules.push(rule.name().to_string()),
                Err(error) => failed_rules.push(ValidationFailure {
                    rule_name: rule.name().to_string(),
                    error_message: error.to_string(),
                }),
            }
        }

        ValidationSummary {
            total_rules: self.rules.len(),
            passed_rules,
            failed_rules,
        }
    }
}

impl Default for EntityValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EntityValidator {
    fn clone(&self) -> Self {
        let mut cloned = Self::new();
        for rule in &self.rules {
            // Note: This will panic for CustomValidationRule
            match std::panic::catch_unwind(|| rule.clone_rule()) {
                Ok(cloned_rule) => cloned.rules.push(cloned_rule),
                Err(_) => {
                    // Skip rules that cannot be cloned (like CustomValidationRule)
                    continue;
                }
            }
        }
        cloned
    }
}

/// Validation failure information
#[derive(Debug, Clone)]
pub struct ValidationFailure {
    pub rule_name: String,
    pub error_message: String,
}

/// Validation summary for an entity
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_rules: usize,
    pub passed_rules: Vec<String>,
    pub failed_rules: Vec<ValidationFailure>,
}

impl ValidationSummary {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.failed_rules.is_empty()
    }

    /// Get pass rate as percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_rules == 0 {
            return 100.0;
        }
        (self.passed_rules.len() as f64 / self.total_rules as f64) * 100.0
    }

    /// Get number of failed rules
    pub fn failure_count(&self) -> usize {
        self.failed_rules.len()
    }

    /// Get number of passed rules
    pub fn success_count(&self) -> usize {
        self.passed_rules.len()
    }

    /// Get first error message if any
    pub fn first_error(&self) -> Option<&str> {
        self.failed_rules.first().map(|f| f.error_message.as_str())
    }

    /// Get all error messages
    pub fn all_errors(&self) -> Vec<&str> {
        self.failed_rules.iter().map(|f| f.error_message.as_str()).collect()
    }
}

/// Builder for creating entity validators
pub struct ValidatorBuilder {
    validator: EntityValidator,
}

impl ValidatorBuilder {
    /// Create a new validator builder
    pub fn new() -> Self {
        Self {
            validator: EntityValidator::new(),
        }
    }

    /// Add required attribute
    pub fn require(mut self, attribute: &str) -> Self {
        self.validator.require_attribute(attribute);
        self
    }

    /// Add typed attribute requirement
    pub fn require_type(mut self, attribute: &str, attr_type: AttributeType) -> Self {
        self.validator.require_attribute_type(attribute, attr_type);
        self
    }

    /// Add range requirement
    pub fn require_range(mut self, attribute: &str, min: Option<f64>, max: Option<f64>) -> Self {
        self.validator.require_attribute_range(attribute, min, max);
        self
    }

    /// Add length requirement
    pub fn require_length(mut self, attribute: &str, min: Option<usize>, max: Option<usize>) -> Self {
        self.validator.require_attribute_length(attribute, min, max);
        self
    }

    /// Build the validator
    pub fn build(self) -> EntityValidator {
        self.validator
    }
}

impl Default for ValidatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
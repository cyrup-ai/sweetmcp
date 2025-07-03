//! Data validation for migrations

use serde::{Deserialize, Serialize};

use crate::migration::{MigrationError, Result};

/// Data validator
pub struct DataValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
}

impl DataValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate data
    pub fn validate<T: Serialize>(&self, data: &T) -> Result<()> {
        let value = serde_json::to_value(data)?;

        for rule in &self.rules {
            rule.validate(&value)?;
        }

        Ok(())
    }
}

impl Default for DataValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rule trait
pub trait ValidationRule: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &serde_json::Value) -> Result<()>;

    /// Get rule name
    fn name(&self) -> &str;
}

/// Required field validation
pub struct RequiredField {
    field_name: String,
}

impl RequiredField {
    pub fn new(field_name: String) -> Self {
        Self { field_name }
    }
}

impl ValidationRule for RequiredField {
    fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let serde_json::Value::Object(map) = value {
            if !map.contains_key(&self.field_name) {
                return Err(MigrationError::ValidationFailed(format!(
                    "Required field '{}' is missing",
                    self.field_name
                )));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "required_field"
    }
}

/// Type validation
pub struct TypeValidation {
    field_name: String,
    expected_type: ValueType,
}

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl TypeValidation {
    pub fn new(field_name: String, expected_type: ValueType) -> Self {
        Self {
            field_name,
            expected_type,
        }
    }
}

impl ValidationRule for TypeValidation {
    fn validate(&self, value: &serde_json::Value) -> Result<()> {
        if let serde_json::Value::Object(map) = value {
            if let Some(field_value) = map.get(&self.field_name) {
                let matches = match self.expected_type {
                    ValueType::String => field_value.is_string(),
                    ValueType::Number => field_value.is_number(),
                    ValueType::Boolean => field_value.is_boolean(),
                    ValueType::Array => field_value.is_array(),
                    ValueType::Object => field_value.is_object(),
                };

                if !matches {
                    return Err(MigrationError::ValidationFailed(format!(
                        "Field '{}' has incorrect type",
                        self.field_name
                    )));
                }
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "type_validation"
    }
}

/// Schema validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidator {
    /// Schema definition
    pub schema: serde_json::Value,
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }

    /// Validate against schema
    pub fn validate(&self, _data: &serde_json::Value) -> Result<()> {
        // Simplified schema validation
        // In production, would use jsonschema crate
        Ok(())
    }
}

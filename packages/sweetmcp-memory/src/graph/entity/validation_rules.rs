//! Specific validation rule implementations
//!
//! This module provides concrete validation rule implementations for
//! common validation scenarios with optimized validation logic.

use super::core::Entity;
use super::validation::{ValidationRule, AttributeType};
use crate::graph::graph_db::{GraphError, Result};
use surrealdb::sql::Value;

/// Required attribute validation rule
pub struct RequiredAttributeRule {
    /// Rule name
    name: String,

    /// Required attribute name
    attribute: String,
}

impl RequiredAttributeRule {
    /// Create a new required attribute rule
    pub fn new(attribute: &str) -> Self {
        Self {
            name: format!("RequiredAttribute:{}", attribute),
            attribute: attribute.to_string(),
        }
    }
}

impl ValidationRule for RequiredAttributeRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if entity.get_attribute(&self.attribute).is_none() {
            return Err(GraphError::ValidationError(format!(
                "Required attribute '{}' is missing",
                self.attribute
            )));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(RequiredAttributeRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
        })
    }
}

/// Attribute type validation rule
pub struct AttributeTypeRule {
    /// Rule name
    name: String,

    /// Attribute name
    attribute: String,

    /// Expected type
    expected_type: AttributeType,
}

impl AttributeTypeRule {
    /// Create a new attribute type rule
    pub fn new(attribute: &str, expected_type: AttributeType) -> Self {
        Self {
            name: format!("AttributeType:{}:{:?}", attribute, expected_type),
            attribute: attribute.to_string(),
            expected_type,
        }
    }
}

impl ValidationRule for AttributeTypeRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if let Some(value) = entity.get_attribute(&self.attribute) {
            let matches = match self.expected_type {
                AttributeType::String => matches!(value, Value::Strand(_)),
                AttributeType::Number => matches!(value, Value::Number(_)),
                AttributeType::Boolean => matches!(value, Value::Bool(_)),
                AttributeType::Array => matches!(value, Value::Array(_)),
                AttributeType::Object => matches!(value, Value::Object(_)),
            };

            if !matches {
                return Err(GraphError::ValidationError(format!(
                    "Attribute '{}' has incorrect type, expected {:?}",
                    self.attribute, self.expected_type
                )));
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(AttributeTypeRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
            expected_type: self.expected_type,
        })
    }
}

/// Attribute range validation rule for numeric values
pub struct AttributeRangeRule {
    name: String,
    attribute: String,
    min_value: Option<f64>,
    max_value: Option<f64>,
}

impl AttributeRangeRule {
    /// Create a new range validation rule
    pub fn new(attribute: &str, min_value: Option<f64>, max_value: Option<f64>) -> Self {
        Self {
            name: format!("AttributeRange:{}:{}:{}", 
                attribute, 
                min_value.map_or("None".to_string(), |v| v.to_string()),
                max_value.map_or("None".to_string(), |v| v.to_string())
            ),
            attribute: attribute.to_string(),
            min_value,
            max_value,
        }
    }
}

impl ValidationRule for AttributeRangeRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if let Some(value) = entity.get_attribute(&self.attribute) {
            if let Value::Number(num) = value {
                let val = num.as_float();
                
                if let Some(min) = self.min_value {
                    if val < min {
                        return Err(GraphError::ValidationError(format!(
                            "Attribute '{}' value {} is below minimum {}",
                            self.attribute, val, min
                        )));
                    }
                }
                
                if let Some(max) = self.max_value {
                    if val > max {
                        return Err(GraphError::ValidationError(format!(
                            "Attribute '{}' value {} is above maximum {}",
                            self.attribute, val, max
                        )));
                    }
                }
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(AttributeRangeRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
            min_value: self.min_value,
            max_value: self.max_value,
        })
    }
}

/// Custom validation rule with closure
pub struct CustomValidationRule {
    /// Rule name
    name: String,

    /// Validation function
    validator: Box<dyn Fn(&dyn Entity) -> Result<()> + Send + Sync>,
}

impl CustomValidationRule {
    /// Create a new custom validation rule
    pub fn new(
        name: &str,
        validator: Box<dyn Fn(&dyn Entity) -> Result<()> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            validator,
        }
    }
}

impl ValidationRule for CustomValidationRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        (self.validator)(entity)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        // Note: Custom validation rules cannot be cloned due to closure constraints
        panic!("CustomValidationRule cannot be cloned - use multiple instances instead")
    }
}

/// String length validation rule
pub struct AttributeLengthRule {
    name: String,
    attribute: String,
    min_length: Option<usize>,
    max_length: Option<usize>,
}

impl AttributeLengthRule {
    /// Create a new string length validation rule
    pub fn new(attribute: &str, min_length: Option<usize>, max_length: Option<usize>) -> Self {
        Self {
            name: format!("AttributeLength:{}:{}:{}", 
                attribute,
                min_length.map_or("None".to_string(), |v| v.to_string()),
                max_length.map_or("None".to_string(), |v| v.to_string())
            ),
            attribute: attribute.to_string(),
            min_length,
            max_length,
        }
    }
}

impl ValidationRule for AttributeLengthRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if let Some(value) = entity.get_attribute(&self.attribute) {
            if let Value::Strand(string_val) = value {
                let len = string_val.as_str().len();
                
                if let Some(min) = self.min_length {
                    if len < min {
                        return Err(GraphError::ValidationError(format!(
                            "Attribute '{}' length {} is below minimum {}",
                            self.attribute, len, min
                        )));
                    }
                }
                
                if let Some(max) = self.max_length {
                    if len > max {
                        return Err(GraphError::ValidationError(format!(
                            "Attribute '{}' length {} is above maximum {}",
                            self.attribute, len, max
                        )));
                    }
                }
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(AttributeLengthRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
            min_length: self.min_length,
            max_length: self.max_length,
        })
    }
}

/// Pattern matching validation rule for string attributes
pub struct AttributePatternRule {
    name: String,
    attribute: String,
    pattern: String,
}

impl AttributePatternRule {
    /// Create a new pattern validation rule
    pub fn new(attribute: &str, pattern: &str) -> Self {
        Self {
            name: format!("AttributePattern:{}:{}", attribute, pattern),
            attribute: attribute.to_string(),
            pattern: pattern.to_string(),
        }
    }
}

impl ValidationRule for AttributePatternRule {
    fn validate(&self, entity: &dyn Entity) -> Result<()> {
        if let Some(value) = entity.get_attribute(&self.attribute) {
            if let Value::Strand(string_val) = value {
                let text = string_val.as_str();
                
                // Simple pattern matching (would use regex crate in production)
                let matches = if self.pattern.contains('*') {
                    let parts: Vec<&str> = self.pattern.split('*').collect();
                    if parts.len() == 2 {
                        text.starts_with(parts[0]) && text.ends_with(parts[1])
                    } else {
                        text.contains(&self.pattern.replace('*', ""))
                    }
                } else {
                    text.contains(&self.pattern)
                };
                
                if !matches {
                    return Err(GraphError::ValidationError(format!(
                        "Attribute '{}' value '{}' does not match pattern '{}'",
                        self.attribute, text, self.pattern
                    )));
                }
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_rule(&self) -> Box<dyn ValidationRule> {
        Box::new(AttributePatternRule {
            name: self.name.clone(),
            attribute: self.attribute.clone(),
            pattern: self.pattern.clone(),
        })
    }
}
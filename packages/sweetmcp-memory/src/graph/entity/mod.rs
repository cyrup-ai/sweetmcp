//! Entity module for graph database operations
//!
//! This module provides comprehensive entity management functionality
//! decomposed into focused submodules for maintainability.

pub mod base_entity;
pub mod builder;
pub mod comparison;
pub mod conversion;
pub mod core;
pub mod extended;
pub mod queries;
pub mod relationships;
pub mod types;
pub mod validation;
pub mod validation_rules;

// Re-export key types for backward compatibility
pub use core::{Entity, BaseEntity, EntityFuture};
pub use builder::EntityBuilder;
pub use comparison::{EntityPattern, IdMatchType};
pub use extended::{ExtendedEntity, EntityMetadata};
pub use validation::{EntityValidator, ValidationRule, AttributeType, ValidationSummary, ValidatorBuilder};
pub use validation_rules::{
    RequiredAttributeRule, AttributeTypeRule, AttributeRangeRule, 
    AttributeLengthRule, AttributePatternRule, CustomValidationRule
};
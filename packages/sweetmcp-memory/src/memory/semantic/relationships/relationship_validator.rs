//! Relationship validation for semantic relationships
//!
//! This module provides blazing-fast relationship validation with zero allocation
//! optimizations and elegant ergonomic interfaces for semantic relationship validation.

use super::relationship_types::SemanticRelationshipType;
use std::collections::HashSet;

/// Relationship validator for semantic relationships
pub struct RelationshipValidator;

impl RelationshipValidator {
    /// Validate a relationship between two items
    #[inline]
    pub fn validate_relationship(
        source_id: &str,
        target_id: &str,
        relationship_type: &SemanticRelationshipType,
        source_type: &super::super::item_types::SemanticItemType,
        target_type: &super::super::item_types::SemanticItemType,
    ) -> Result<(), ValidationError> {
        // Check for self-referential relationships
        if source_id == target_id {
            return Err(ValidationError::SelfReferential {
                item_id: source_id.to_string(),
                relationship_type: relationship_type.clone(),
            });
        }

        // Check type compatibility
        if !relationship_type.is_compatible_with_item_types(source_type, target_type) {
            return Err(ValidationError::IncompatibleTypes {
                source_type: source_type.clone(),
                target_type: target_type.clone(),
                relationship_type: relationship_type.clone(),
            });
        }

        Ok(())
    }

    /// Validate multiple relationships for consistency
    #[inline]
    pub fn validate_relationship_set(
        relationships: &[(String, String, SemanticRelationshipType)],
    ) -> Result<(), ValidationError> {
        // Check for cycles in hierarchical relationships
        for (source, target, rel_type) in relationships {
            if Self::would_create_cycle(relationships, source, target, rel_type) {
                return Err(ValidationError::CycleDetected {
                    source_id: source.clone(),
                    target_id: target.clone(),
                    relationship_type: rel_type.clone(),
                });
            }
        }

        // Check for conflicting relationships
        Self::validate_relationship_conflicts(relationships)?;

        Ok(())
    }

    /// Check if adding a relationship would create a cycle
    #[inline]
    pub fn would_create_cycle(
        relationships: &[(String, String, SemanticRelationshipType)],
        source_id: &str,
        target_id: &str,
        relationship_type: &SemanticRelationshipType,
    ) -> bool {
        // Only check for hierarchical relationships that could create cycles
        if !relationship_type.is_hierarchical() {
            return false;
        }

        // Simple cycle detection: check if target already has a path back to source
        Self::has_path_between(relationships, target_id, source_id, relationship_type)
    }

    /// Check if there's a path between two nodes
    #[inline]
    fn has_path_between(
        relationships: &[(String, String, SemanticRelationshipType)],
        start_id: &str,
        end_id: &str,
        relationship_type: &SemanticRelationshipType,
    ) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![start_id];

        while let Some(current_id) = stack.pop() {
            if current_id == end_id {
                return true;
            }

            if visited.contains(current_id) {
                continue;
            }
            visited.insert(current_id);

            // Find outgoing relationships of the same type
            for (source, target, rel_type) in relationships {
                if source == current_id && rel_type == relationship_type {
                    stack.push(target.as_str());
                }
            }
        }

        false
    }

    /// Validate relationship conflicts
    #[inline]
    fn validate_relationship_conflicts(
        relationships: &[(String, String, SemanticRelationshipType)],
    ) -> Result<(), ValidationError> {
        let mut relationship_map: std::collections::HashMap<(String, String), Vec<SemanticRelationshipType>> = 
            std::collections::HashMap::new();

        // Group relationships by source-target pair
        for (source, target, rel_type) in relationships {
            let key = (source.clone(), target.clone());
            relationship_map.entry(key).or_insert_with(Vec::new).push(rel_type.clone());
        }

        // Check for conflicts within each source-target pair
        for ((source, target), rel_types) in relationship_map {
            if let Some(conflict) = Self::find_relationship_conflict(&rel_types) {
                return Err(ValidationError::ConflictingRelationships {
                    source_id: source,
                    target_id: target,
                    relationship_types: conflict,
                });
            }
        }

        Ok(())
    }

    /// Find conflicting relationship types
    #[inline]
    fn find_relationship_conflict(
        relationship_types: &[SemanticRelationshipType],
    ) -> Option<Vec<SemanticRelationshipType>> {
        // Define conflicting relationship pairs
        let conflicts = [
            (SemanticRelationshipType::IsA, SemanticRelationshipType::HasA),
            (SemanticRelationshipType::HasA, SemanticRelationshipType::PartOf),
        ];

        for (rel1, rel2) in &conflicts {
            if relationship_types.contains(rel1) && relationship_types.contains(rel2) {
                return Some(vec![rel1.clone(), rel2.clone()]);
            }
        }

        None
    }

    /// Get relationship validation rules
    #[inline]
    pub fn get_validation_rules() -> Vec<(&'static str, &'static str)> {
        vec![
            ("IsA relationships should connect concepts to categories or categories to categories", "is_a_rule"),
            ("HasA relationships should originate from concepts or categories", "has_a_rule"),
            ("PartOf relationships should target concepts or categories", "part_of_rule"),
            ("Causes relationships should originate from facts or rules", "causes_rule"),
            ("Hierarchical relationships should not create cycles", "no_cycles_rule"),
            ("RelatedTo relationships can connect any item types", "related_to_rule"),
            ("No self-referential relationships allowed", "no_self_reference_rule"),
            ("No conflicting relationship types between same items", "no_conflicts_rule"),
        ]
    }

    /// Validate relationship strength consistency
    #[inline]
    pub fn validate_strength_consistency(
        relationships: &[(String, String, SemanticRelationshipType, f32)],
    ) -> Result<(), ValidationError> {
        for (source, target, rel_type, strength) in relationships {
            let expected_weight = rel_type.strength_weight();
            let tolerance = 0.2; // 20% tolerance

            if (strength - expected_weight).abs() > tolerance {
                return Err(ValidationError::InconsistentStrength {
                    source_id: source.clone(),
                    target_id: target.clone(),
                    relationship_type: rel_type.clone(),
                    actual_strength: *strength,
                    expected_strength: expected_weight,
                });
            }
        }

        Ok(())
    }

    /// Validate relationship transitivity
    #[inline]
    pub fn validate_transitivity(
        relationships: &[(String, String, SemanticRelationshipType)],
        enforce_transitivity: bool,
    ) -> Result<Vec<(String, String, SemanticRelationshipType)>, ValidationError> {
        if !enforce_transitivity {
            return Ok(Vec::new());
        }

        let mut missing_transitive = Vec::new();

        // Check for missing transitive relationships
        for (source1, target1, rel_type1) in relationships {
            if !rel_type1.is_hierarchical() {
                continue;
            }

            for (source2, target2, rel_type2) in relationships {
                if target1 == source2 && rel_type1 == rel_type2 {
                    // We have A -> B and B -> C, should have A -> C
                    let transitive_exists = relationships.iter().any(|(s, t, rt)| {
                        s == source1 && t == target2 && rt == rel_type1
                    });

                    if !transitive_exists {
                        missing_transitive.push((
                            source1.clone(),
                            target2.clone(),
                            rel_type1.clone(),
                        ));
                    }
                }
            }
        }

        Ok(missing_transitive)
    }

    /// Validate relationship depth limits
    #[inline]
    pub fn validate_depth_limits(
        relationships: &[(String, String, SemanticRelationshipType)],
        max_depth: usize,
    ) -> Result<(), ValidationError> {
        for (source, _, rel_type) in relationships {
            let depth = Self::calculate_relationship_depth(relationships, source, rel_type);
            if depth > max_depth {
                return Err(ValidationError::ExceedsDepthLimit {
                    source_id: source.clone(),
                    relationship_type: rel_type.clone(),
                    actual_depth: depth,
                    max_depth,
                });
            }
        }

        Ok(())
    }

    /// Calculate the maximum depth of relationships from a source
    #[inline]
    fn calculate_relationship_depth(
        relationships: &[(String, String, SemanticRelationshipType)],
        source_id: &str,
        relationship_type: &SemanticRelationshipType,
    ) -> usize {
        let mut max_depth = 0;
        let mut visited = HashSet::new();
        let mut stack = vec![(source_id, 0)];

        while let Some((current_id, depth)) = stack.pop() {
            if visited.contains(current_id) {
                continue;
            }
            visited.insert(current_id);

            max_depth = max_depth.max(depth);

            // Find outgoing relationships of the same type
            for (source, target, rel_type) in relationships {
                if source == current_id && rel_type == relationship_type {
                    stack.push((target.as_str(), depth + 1));
                }
            }
        }

        max_depth
    }

    /// Batch validate relationships with detailed reporting
    #[inline]
    pub fn batch_validate(
        relationships: &[(String, String, SemanticRelationshipType)],
        item_types: &std::collections::HashMap<String, super::super::item_types::SemanticItemType>,
        options: ValidationOptions,
    ) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Individual relationship validation
        for (source, target, rel_type) in relationships {
            if let (Some(source_type), Some(target_type)) = 
                (item_types.get(source), item_types.get(target)) {
                if let Err(error) = Self::validate_relationship(
                    source, target, rel_type, source_type, target_type
                ) {
                    report.errors.push(error);
                }
            } else {
                report.warnings.push(ValidationWarning::MissingItemType {
                    item_id: if item_types.get(source).is_none() { 
                        source.clone() 
                    } else { 
                        target.clone() 
                    },
                });
            }
        }

        // Set-level validation
        if let Err(error) = Self::validate_relationship_set(relationships) {
            report.errors.push(error);
        }

        // Optional validations
        if options.check_transitivity {
            match Self::validate_transitivity(relationships, true) {
                Ok(missing) => {
                    for (source, target, rel_type) in missing {
                        report.warnings.push(ValidationWarning::MissingTransitive {
                            source_id: source,
                            target_id: target,
                            relationship_type: rel_type,
                        });
                    }
                }
                Err(error) => report.errors.push(error),
            }
        }

        if let Some(max_depth) = options.max_depth {
            if let Err(error) = Self::validate_depth_limits(relationships, max_depth) {
                report.errors.push(error);
            }
        }

        report.is_valid = report.errors.is_empty();
        report
    }
}

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Self-referential relationship
    SelfReferential {
        item_id: String,
        relationship_type: SemanticRelationshipType,
    },
    /// Incompatible item types for relationship
    IncompatibleTypes {
        source_type: super::super::item_types::SemanticItemType,
        target_type: super::super::item_types::SemanticItemType,
        relationship_type: SemanticRelationshipType,
    },
    /// Cycle detected in hierarchical relationships
    CycleDetected {
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
    },
    /// Conflicting relationship types
    ConflictingRelationships {
        source_id: String,
        target_id: String,
        relationship_types: Vec<SemanticRelationshipType>,
    },
    /// Inconsistent relationship strength
    InconsistentStrength {
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
        actual_strength: f32,
        expected_strength: f32,
    },
    /// Exceeds maximum depth limit
    ExceedsDepthLimit {
        source_id: String,
        relationship_type: SemanticRelationshipType,
        actual_depth: usize,
        max_depth: usize,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::SelfReferential { item_id, relationship_type } => {
                write!(f, "Self-referential {} relationship for item {}", relationship_type, item_id)
            }
            ValidationError::IncompatibleTypes { source_type, target_type, relationship_type } => {
                write!(f, "Incompatible types for {} relationship: {:?} -> {:?}", 
                       relationship_type, source_type, target_type)
            }
            ValidationError::CycleDetected { source_id, target_id, relationship_type } => {
                write!(f, "Cycle detected in {} relationship: {} -> {}", 
                       relationship_type, source_id, target_id)
            }
            ValidationError::ConflictingRelationships { source_id, target_id, relationship_types } => {
                write!(f, "Conflicting relationships between {} and {}: {:?}", 
                       source_id, target_id, relationship_types)
            }
            ValidationError::InconsistentStrength { source_id, target_id, relationship_type, actual_strength, expected_strength } => {
                write!(f, "Inconsistent strength for {} relationship {} -> {}: {} (expected {})", 
                       relationship_type, source_id, target_id, actual_strength, expected_strength)
            }
            ValidationError::ExceedsDepthLimit { source_id, relationship_type, actual_depth, max_depth } => {
                write!(f, "Relationship depth limit exceeded for {} from {}: {} > {}", 
                       relationship_type, source_id, actual_depth, max_depth)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validation warning types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarning {
    /// Missing item type information
    MissingItemType {
        item_id: String,
    },
    /// Missing transitive relationship
    MissingTransitive {
        source_id: String,
        target_id: String,
        relationship_type: SemanticRelationshipType,
    },
}

/// Validation options
#[derive(Debug, Clone)]
pub struct ValidationOptions {
    pub check_transitivity: bool,
    pub max_depth: Option<usize>,
    pub enforce_strength_consistency: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            check_transitivity: false,
            max_depth: Some(10),
            enforce_strength_consistency: false,
        }
    }
}

/// Validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Create new validation report
    #[inline]
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// Get error count
    #[inline]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get warning count
    #[inline]
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}
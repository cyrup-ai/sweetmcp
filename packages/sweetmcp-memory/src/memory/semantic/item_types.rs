//! Semantic item type definitions and operations
//!
//! This module provides blazing-fast semantic item type operations with zero allocation
//! optimizations and elegant ergonomic interfaces for semantic type management.

use crate::utils::{Result, error::Error};
use serde::{Deserialize, Serialize};

/// Semantic item type enum with optimized string conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticItemType {
    /// Concept (entity, object, idea)
    Concept,
    /// Fact (statement about concepts)
    Fact,
    /// Rule (logical rule or pattern)
    Rule,
    /// Category (classification or grouping)
    Category,
}

impl SemanticItemType {
    /// Convert to string with zero allocation using static strings
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            SemanticItemType::Concept => "concept",
            SemanticItemType::Fact => "fact",
            SemanticItemType::Rule => "rule",
            SemanticItemType::Category => "category",
        }
    }

    /// Convert to string (for compatibility)
    #[inline]
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Convert from string with optimized matching
    #[inline]
    pub fn from_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "concept" => Ok(SemanticItemType::Concept),
            "fact" => Ok(SemanticItemType::Fact),
            "rule" => Ok(SemanticItemType::Rule),
            "category" => Ok(SemanticItemType::Category),
            _ => Err(Error::ConversionError(format!(
                "Invalid semantic item type: {}",
                s
            ))),
        }
    }

    /// Get all semantic item types
    #[inline]
    pub fn all() -> [SemanticItemType; 4] {
        [
            SemanticItemType::Concept,
            SemanticItemType::Fact,
            SemanticItemType::Rule,
            SemanticItemType::Category,
        ]
    }

    /// Get semantic item type description
    #[inline]
    pub fn description(&self) -> &'static str {
        match self {
            SemanticItemType::Concept => "Entity, object, or idea",
            SemanticItemType::Fact => "Statement about concepts",
            SemanticItemType::Rule => "Logical rule or pattern",
            SemanticItemType::Category => "Classification or grouping",
        }
    }

    /// Check if item type is structural (Concept or Category)
    #[inline]
    pub fn is_structural(&self) -> bool {
        matches!(self, SemanticItemType::Concept | SemanticItemType::Category)
    }

    /// Check if item type is informational (Fact or Rule)
    #[inline]
    pub fn is_informational(&self) -> bool {
        matches!(self, SemanticItemType::Fact | SemanticItemType::Rule)
    }

    /// Get default confidence level for item type
    #[inline]
    pub fn default_confidence(&self) -> super::confidence::ConfidenceLevel {
        use super::confidence::ConfidenceLevel;
        match self {
            SemanticItemType::Concept => ConfidenceLevel::High,
            SemanticItemType::Fact => ConfidenceLevel::Medium,
            SemanticItemType::Rule => ConfidenceLevel::High,
            SemanticItemType::Category => ConfidenceLevel::High,
        }
    }

    /// Get priority weight for item type (higher = more important)
    #[inline]
    pub fn priority_weight(&self) -> f32 {
        match self {
            SemanticItemType::Concept => 1.0,
            SemanticItemType::Category => 0.9,
            SemanticItemType::Rule => 0.8,
            SemanticItemType::Fact => 0.7,
        }
    }

    /// Check if item type can have relationships of specified type
    #[inline]
    pub fn can_have_relationship(&self, relationship_type: &super::relationships::SemanticRelationshipType) -> bool {
        use super::relationships::SemanticRelationshipType;
        
        match (self, relationship_type) {
            // Concepts can have all relationship types
            (SemanticItemType::Concept, _) => true,
            
            // Categories can have hierarchical relationships
            (SemanticItemType::Category, SemanticRelationshipType::IsA) => true,
            (SemanticItemType::Category, SemanticRelationshipType::PartOf) => true,
            (SemanticItemType::Category, SemanticRelationshipType::HasA) => true,
            
            // Facts can be related to other facts and concepts
            (SemanticItemType::Fact, SemanticRelationshipType::RelatedTo) => true,
            (SemanticItemType::Fact, SemanticRelationshipType::Causes) => true,
            
            // Rules can cause effects and be related
            (SemanticItemType::Rule, SemanticRelationshipType::Causes) => true,
            (SemanticItemType::Rule, SemanticRelationshipType::RelatedTo) => true,
            
            // Custom relationships are allowed for all types
            (_, SemanticRelationshipType::Custom(_)) => true,
            
            // Default: not allowed
            _ => false,
        }
    }

    /// Get recommended relationship types for this item type
    #[inline]
    pub fn recommended_relationships(&self) -> Vec<super::relationships::SemanticRelationshipType> {
        use super::relationships::SemanticRelationshipType;
        
        match self {
            SemanticItemType::Concept => vec![
                SemanticRelationshipType::IsA,
                SemanticRelationshipType::HasA,
                SemanticRelationshipType::PartOf,
                SemanticRelationshipType::RelatedTo,
            ],
            SemanticItemType::Category => vec![
                SemanticRelationshipType::IsA,
                SemanticRelationshipType::HasA,
                SemanticRelationshipType::PartOf,
            ],
            SemanticItemType::Fact => vec![
                SemanticRelationshipType::RelatedTo,
                SemanticRelationshipType::Causes,
            ],
            SemanticItemType::Rule => vec![
                SemanticRelationshipType::Causes,
                SemanticRelationshipType::RelatedTo,
            ],
        }
    }
}

impl Default for SemanticItemType {
    #[inline]
    fn default() -> Self {
        SemanticItemType::Concept
    }
}

impl std::fmt::Display for SemanticItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SemanticItemType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_string(s)
    }
}

/// Semantic item type classifier for automatic type detection
pub struct SemanticItemTypeClassifier;

impl SemanticItemTypeClassifier {
    /// Classify content to determine semantic item type
    #[inline]
    pub fn classify_content(content: &serde_json::Value) -> SemanticItemType {
        if let Some(content_str) = content.as_str() {
            Self::classify_text(content_str)
        } else if content.is_object() {
            Self::classify_object(content)
        } else {
            SemanticItemType::default()
        }
    }

    /// Classify text content to determine semantic item type
    #[inline]
    fn classify_text(text: &str) -> SemanticItemType {
        let text_lower = text.to_lowercase();
        
        // Rule indicators
        if text_lower.contains("if ") && text_lower.contains("then") ||
           text_lower.contains("when ") && text_lower.contains("do") ||
           text_lower.contains("rule:") ||
           text_lower.starts_with("always ") ||
           text_lower.starts_with("never ") {
            return SemanticItemType::Rule;
        }

        // Fact indicators
        if text_lower.contains(" is ") ||
           text_lower.contains(" are ") ||
           text_lower.contains(" has ") ||
           text_lower.contains(" have ") ||
           text_lower.contains(" was ") ||
           text_lower.contains(" were ") {
            return SemanticItemType::Fact;
        }

        // Category indicators
        if text_lower.contains("category:") ||
           text_lower.contains("type:") ||
           text_lower.contains("class:") ||
           text_lower.ends_with(" category") ||
           text_lower.ends_with(" type") ||
           text_lower.ends_with(" class") {
            return SemanticItemType::Category;
        }

        // Default to concept
        SemanticItemType::Concept
    }

    /// Classify object content to determine semantic item type
    #[inline]
    fn classify_object(obj: &serde_json::Value) -> SemanticItemType {
        if let Some(obj_map) = obj.as_object() {
            // Check for explicit type field
            if let Some(type_value) = obj_map.get("type") {
                if let Some(type_str) = type_value.as_str() {
                    if let Ok(item_type) = SemanticItemType::from_string(type_str) {
                        return item_type;
                    }
                }
            }

            // Check for rule-like structure
            if obj_map.contains_key("condition") && obj_map.contains_key("action") ||
               obj_map.contains_key("if") && obj_map.contains_key("then") {
                return SemanticItemType::Rule;
            }

            // Check for fact-like structure
            if obj_map.contains_key("subject") && obj_map.contains_key("predicate") ||
               obj_map.contains_key("statement") ||
               obj_map.contains_key("assertion") {
                return SemanticItemType::Fact;
            }

            // Check for category-like structure
            if obj_map.contains_key("members") ||
               obj_map.contains_key("subcategories") ||
               obj_map.contains_key("parent_category") {
                return SemanticItemType::Category;
            }
        }

        SemanticItemType::Concept
    }

    /// Get confidence score for classification
    #[inline]
    pub fn classification_confidence(
        content: &serde_json::Value,
        classified_type: &SemanticItemType,
    ) -> super::confidence::ConfidenceLevel {
        use super::confidence::ConfidenceLevel;

        if let Some(content_str) = content.as_str() {
            let indicators = Self::count_type_indicators(content_str, classified_type);
            match indicators {
                0 => ConfidenceLevel::Low,
                1 => ConfidenceLevel::Medium,
                2 => ConfidenceLevel::High,
                _ => ConfidenceLevel::VeryHigh,
            }
        } else if content.is_object() {
            // Object classification is generally more reliable
            if Self::has_explicit_type_field(content) {
                ConfidenceLevel::VeryHigh
            } else {
                ConfidenceLevel::High
            }
        } else {
            ConfidenceLevel::Low
        }
    }

    /// Count type-specific indicators in text
    #[inline]
    fn count_type_indicators(text: &str, item_type: &SemanticItemType) -> usize {
        let text_lower = text.to_lowercase();
        let mut count = 0;

        match item_type {
            SemanticItemType::Rule => {
                if text_lower.contains("if ") { count += 1; }
                if text_lower.contains("then") { count += 1; }
                if text_lower.contains("when ") { count += 1; }
                if text_lower.contains("rule:") { count += 1; }
                if text_lower.starts_with("always ") { count += 1; }
                if text_lower.starts_with("never ") { count += 1; }
            }
            SemanticItemType::Fact => {
                if text_lower.contains(" is ") { count += 1; }
                if text_lower.contains(" are ") { count += 1; }
                if text_lower.contains(" has ") { count += 1; }
                if text_lower.contains(" have ") { count += 1; }
                if text_lower.contains(" was ") { count += 1; }
                if text_lower.contains(" were ") { count += 1; }
            }
            SemanticItemType::Category => {
                if text_lower.contains("category:") { count += 1; }
                if text_lower.contains("type:") { count += 1; }
                if text_lower.contains("class:") { count += 1; }
                if text_lower.ends_with(" category") { count += 1; }
                if text_lower.ends_with(" type") { count += 1; }
                if text_lower.ends_with(" class") { count += 1; }
            }
            SemanticItemType::Concept => {
                // Concepts are default, so any non-specific content gets low score
                count = 0;
            }
        }

        count
    }

    /// Check if object has explicit type field
    #[inline]
    fn has_explicit_type_field(obj: &serde_json::Value) -> bool {
        if let Some(obj_map) = obj.as_object() {
            obj_map.contains_key("type") || 
            obj_map.contains_key("item_type") ||
            obj_map.contains_key("semantic_type")
        } else {
            false
        }
    }
}

/// Semantic item type statistics for analysis
#[derive(Debug, Clone)]
pub struct SemanticItemTypeStatistics {
    pub total_items: usize,
    pub type_distribution: [usize; 4], // Count for each item type
    pub dominant_type: Option<SemanticItemType>,
}

impl SemanticItemTypeStatistics {
    /// Create new statistics
    #[inline]
    pub fn new() -> Self {
        Self {
            total_items: 0,
            type_distribution: [0; 4],
            dominant_type: None,
        }
    }

    /// Calculate statistics from item types
    #[inline]
    pub fn from_types(types: &[SemanticItemType]) -> Self {
        if types.is_empty() {
            return Self::new();
        }

        let mut distribution = [0; 4];
        
        for item_type in types {
            let index = match item_type {
                SemanticItemType::Concept => 0,
                SemanticItemType::Fact => 1,
                SemanticItemType::Rule => 2,
                SemanticItemType::Category => 3,
            };
            distribution[index] += 1;
        }

        let dominant_type = Self::find_dominant_type(&distribution);

        Self {
            total_items: types.len(),
            type_distribution: distribution,
            dominant_type,
        }
    }

    /// Find dominant type from distribution
    #[inline]
    fn find_dominant_type(distribution: &[usize; 4]) -> Option<SemanticItemType> {
        let max_index = distribution
            .iter()
            .enumerate()
            .max_by_key(|&(_, count)| *count)
            .map(|(index, _)| index)?;

        match max_index {
            0 => Some(SemanticItemType::Concept),
            1 => Some(SemanticItemType::Fact),
            2 => Some(SemanticItemType::Rule),
            3 => Some(SemanticItemType::Category),
            _ => None,
        }
    }

    /// Get percentage distribution
    #[inline]
    pub fn percentage_distribution(&self) -> [f32; 4] {
        if self.total_items == 0 {
            return [0.0; 4];
        }

        let mut percentages = [0.0; 4];
        for (i, &count) in self.type_distribution.iter().enumerate() {
            percentages[i] = (count as f32 / self.total_items as f32) * 100.0;
        }
        percentages
    }

    /// Check if distribution is balanced
    #[inline]
    pub fn is_balanced(&self) -> bool {
        if self.total_items == 0 {
            return true;
        }

        let expected_count = self.total_items as f32 / 4.0;
        let max_deviation = self.type_distribution
            .iter()
            .map(|&count| (count as f32 - expected_count).abs())
            .fold(0.0, f32::max);

        max_deviation <= expected_count * 0.5 // Allow 50% deviation
    }

    /// Get diversity score (0.0 to 1.0)
    #[inline]
    pub fn diversity_score(&self) -> f32 {
        if self.total_items == 0 {
            return 0.0;
        }

        let non_zero_types = self.type_distribution.iter().filter(|&&count| count > 0).count();
        non_zero_types as f32 / 4.0
    }
}

impl Default for SemanticItemTypeStatistics {
    fn default() -> Self {
        Self::new()
    }
}
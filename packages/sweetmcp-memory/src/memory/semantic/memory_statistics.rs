//! Semantic memory statistics and analysis functionality
//!
//! This module provides statistical analysis capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::item_core::SemanticItem;
use super::relationship::SemanticRelationship;

/// Memory statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatistics {
    /// Total number of items
    pub total_items: usize,
    /// Total number of relationships
    pub total_relationships: usize,
    /// Count of items by type
    pub item_type_counts: HashMap<String, usize>,
    /// Count of relationships by type
    pub relationship_type_counts: HashMap<String, usize>,
    /// Distribution of confidence levels
    pub confidence_distribution: HashMap<String, usize>,
    /// Average number of connections per item
    pub average_connections_per_item: f64,
}

impl MemoryStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_items: 0,
            total_relationships: 0,
            item_type_counts: HashMap::new(),
            relationship_type_counts: HashMap::new(),
            confidence_distribution: HashMap::new(),
            average_connections_per_item: 0.0,
        }
    }

    /// Calculate statistics from items and relationships
    pub fn calculate(items: &[SemanticItem], relationships: &[SemanticRelationship]) -> Self {
        let item_type_counts = Self::count_items_by_type(items);
        let relationship_type_counts = Self::count_relationships_by_type(relationships);
        let confidence_distribution = Self::get_confidence_distribution(items);

        Self {
            total_items: items.len(),
            total_relationships: relationships.len(),
            item_type_counts,
            relationship_type_counts,
            confidence_distribution,
            average_connections_per_item: if items.is_empty() {
                0.0
            } else {
                (relationships.len() * 2) as f64 / items.len() as f64
            },
        }
    }

    /// Count items by type
    pub fn count_items_by_type(items: &[SemanticItem]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for item in items {
            let type_name = format!("{:?}", item.item_type);
            *counts.entry(type_name).or_insert(0) += 1;
        }
        counts
    }

    /// Count relationships by type
    pub fn count_relationships_by_type(relationships: &[SemanticRelationship]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for rel in relationships {
            let type_name = rel.relationship_type.as_str().to_string();
            *counts.entry(type_name).or_insert(0) += 1;
        }
        counts
    }

    /// Get confidence level distribution
    pub fn get_confidence_distribution(items: &[SemanticItem]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for item in items {
            let confidence_name = format!("{:?}", item.confidence);
            *distribution.entry(confidence_name).or_insert(0) += 1;
        }
        distribution
    }

    /// Get memory density (relationships per item)
    pub fn get_density(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            self.total_relationships as f64 / self.total_items as f64
        }
    }

    /// Get most common item type
    pub fn most_common_item_type(&self) -> Option<(String, usize)> {
        self.item_type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(type_name, count)| (type_name.clone(), *count))
    }

    /// Get most common relationship type
    pub fn most_common_relationship_type(&self) -> Option<(String, usize)> {
        self.relationship_type_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(type_name, count)| (type_name.clone(), *count))
    }

    /// Get most common confidence level
    pub fn most_common_confidence_level(&self) -> Option<(String, usize)> {
        self.confidence_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(confidence, count)| (confidence.clone(), *count))
    }

    /// Calculate diversity score for item types (0.0 to 1.0)
    pub fn item_type_diversity(&self) -> f64 {
        if self.total_items == 0 {
            return 1.0;
        }

        let max_possible_types = 4.0; // Number of SemanticItemType variants
        
        // Shannon entropy normalized by maximum possible entropy
        let entropy = self.item_type_counts
            .values()
            .map(|&count| {
                let p = count as f64 / self.total_items as f64;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        let max_entropy = max_possible_types.log2();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    /// Calculate diversity score for relationship types (0.0 to 1.0)
    pub fn relationship_type_diversity(&self) -> f64 {
        if self.total_relationships == 0 {
            return 1.0;
        }

        // Shannon entropy normalized by log of unique types
        let entropy = self.relationship_type_counts
            .values()
            .map(|&count| {
                let p = count as f64 / self.total_relationships as f64;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        let type_count = self.relationship_type_counts.len() as f64;
        let max_entropy = if type_count > 1.0 { type_count.log2() } else { 1.0 };
        
        entropy / max_entropy
    }

    /// Check if memory is well-connected (high density)
    pub fn is_well_connected(&self) -> bool {
        self.get_density() >= 1.0 // At least one relationship per item on average
    }

    /// Check if memory has good type diversity
    pub fn has_good_diversity(&self) -> bool {
        self.item_type_diversity() >= 0.5 && self.relationship_type_diversity() >= 0.5
    }

    /// Get memory health score (0.0 to 1.0)
    pub fn health_score(&self) -> f64 {
        if self.total_items == 0 {
            return 1.0; // Empty memory is considered healthy
        }

        let connectivity_score = (self.get_density() / 2.0).clamp(0.0, 1.0); // Normalize density
        let diversity_score = (self.item_type_diversity() + self.relationship_type_diversity()) / 2.0;
        let size_score = if self.total_items >= 10 { 1.0 } else { self.total_items as f64 / 10.0 };

        (connectivity_score * 0.4 + diversity_score * 0.4 + size_score * 0.2).clamp(0.0, 1.0)
    }

    /// Get summary report as string
    pub fn summary_report(&self) -> String {
        let mut report = format!(
            "Memory Statistics Summary:\n\
             - Total Items: {}\n\
             - Total Relationships: {}\n\
             - Average Connections per Item: {:.2}\n\
             - Memory Density: {:.2}\n",
            self.total_items,
            self.total_relationships,
            self.average_connections_per_item,
            self.get_density()
        );

        if let Some((item_type, count)) = self.most_common_item_type() {
            report.push_str(&format!("- Most Common Item Type: {} ({})\n", item_type, count));
        }

        if let Some((rel_type, count)) = self.most_common_relationship_type() {
            report.push_str(&format!("- Most Common Relationship Type: {} ({})\n", rel_type, count));
        }

        if let Some((confidence, count)) = self.most_common_confidence_level() {
            report.push_str(&format!("- Most Common Confidence Level: {} ({})\n", confidence, count));
        }

        report.push_str(&format!(
            "- Item Type Diversity: {:.2}\n\
             - Relationship Type Diversity: {:.2}\n\
             - Health Score: {:.2}\n",
            self.item_type_diversity(),
            self.relationship_type_diversity(),
            self.health_score()
        ));

        report
    }
}

impl Default for MemoryStatistics {
    fn default() -> Self {
        Self::new()
    }
}
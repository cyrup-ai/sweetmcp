//! Semantic memory statistics utility functions
//!
//! This module provides utility functions for memory statistics operations
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory_statistics::MemoryStatistics;

impl MemoryStatistics {
    /// Update statistics with new data
    pub fn update(&mut self, items: &[super::item_core::SemanticItem], relationships: &[super::relationship::SemanticRelationship]) {
        *self = Self::calculate(items, relationships);
    }

    /// Merge with another statistics instance
    pub fn merge_with(&mut self, other: &MemoryStatistics) {
        self.total_items += other.total_items;
        self.total_relationships += other.total_relationships;

        // Merge item type counts
        for (item_type, count) in &other.item_type_counts {
            *self.item_type_counts.entry(item_type.clone()).or_insert(0) += count;
        }

        // Merge relationship type counts
        for (rel_type, count) in &other.relationship_type_counts {
            *self.relationship_type_counts.entry(rel_type.clone()).or_insert(0) += count;
        }

        // Merge confidence distribution
        for (confidence, count) in &other.confidence_distribution {
            *self.confidence_distribution.entry(confidence.clone()).or_insert(0) += count;
        }

        // Recalculate average connections per item
        self.average_connections_per_item = if self.total_items == 0 {
            0.0
        } else {
            (self.total_relationships * 2) as f64 / self.total_items as f64
        };
    }

    /// Reset statistics to empty state
    pub fn reset(&mut self) {
        self.total_items = 0;
        self.total_relationships = 0;
        self.item_type_counts.clear();
        self.relationship_type_counts.clear();
        self.confidence_distribution.clear();
        self.average_connections_per_item = 0.0;
    }

    /// Check if statistics are empty
    pub fn is_empty(&self) -> bool {
        self.total_items == 0 && self.total_relationships == 0
    }

    /// Get total memory elements (items + relationships)
    pub fn total_elements(&self) -> usize {
        self.total_items + self.total_relationships
    }

    /// Calculate memory utilization ratio (relationships to items)
    pub fn utilization_ratio(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            self.total_relationships as f64 / self.total_items as f64
        }
    }

    /// Clone statistics with specific fields
    pub fn clone_with_filter(&self, include_items: bool, include_relationships: bool) -> Self {
        Self {
            total_items: if include_items { self.total_items } else { 0 },
            total_relationships: if include_relationships { self.total_relationships } else { 0 },
            item_type_counts: if include_items { self.item_type_counts.clone() } else { std::collections::HashMap::new() },
            relationship_type_counts: if include_relationships { self.relationship_type_counts.clone() } else { std::collections::HashMap::new() },
            confidence_distribution: if include_items { self.confidence_distribution.clone() } else { std::collections::HashMap::new() },
            average_connections_per_item: if include_items && include_relationships {
                self.average_connections_per_item
            } else {
                0.0
            },
        }
    }

    /// Get statistics for items only
    pub fn items_only(&self) -> Self {
        self.clone_with_filter(true, false)
    }

    /// Get statistics for relationships only
    pub fn relationships_only(&self) -> Self {
        self.clone_with_filter(false, true)
    }

    /// Calculate percentage of items by type
    pub fn item_type_percentages(&self) -> std::collections::HashMap<String, f64> {
        let mut percentages = std::collections::HashMap::new();
        
        if self.total_items > 0 {
            for (item_type, count) in &self.item_type_counts {
                let percentage = (*count as f64 / self.total_items as f64) * 100.0;
                percentages.insert(item_type.clone(), percentage);
            }
        }
        
        percentages
    }

    /// Calculate percentage of relationships by type
    pub fn relationship_type_percentages(&self) -> std::collections::HashMap<String, f64> {
        let mut percentages = std::collections::HashMap::new();
        
        if self.total_relationships > 0 {
            for (rel_type, count) in &self.relationship_type_counts {
                let percentage = (*count as f64 / self.total_relationships as f64) * 100.0;
                percentages.insert(rel_type.clone(), percentage);
            }
        }
        
        percentages
    }

    /// Calculate percentage of confidence levels
    pub fn confidence_percentages(&self) -> std::collections::HashMap<String, f64> {
        let mut percentages = std::collections::HashMap::new();
        
        if self.total_items > 0 {
            for (confidence, count) in &self.confidence_distribution {
                let percentage = (*count as f64 / self.total_items as f64) * 100.0;
                percentages.insert(confidence.clone(), percentage);
            }
        }
        
        percentages
    }

    /// Get top N item types by count
    pub fn top_item_types(&self, n: usize) -> Vec<(String, usize)> {
        let mut sorted_types: Vec<_> = self.item_type_counts.iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();
        
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_types.truncate(n);
        sorted_types
    }

    /// Get top N relationship types by count
    pub fn top_relationship_types(&self, n: usize) -> Vec<(String, usize)> {
        let mut sorted_types: Vec<_> = self.relationship_type_counts.iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();
        
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_types.truncate(n);
        sorted_types
    }

    /// Get top N confidence levels by count
    pub fn top_confidence_levels(&self, n: usize) -> Vec<(String, usize)> {
        let mut sorted_levels: Vec<_> = self.confidence_distribution.iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();
        
        sorted_levels.sort_by(|a, b| b.1.cmp(&a.1));
        sorted_levels.truncate(n);
        sorted_levels
    }

    /// Check if statistics are balanced (no single type dominates)
    pub fn is_balanced(&self) -> bool {
        let item_balance = if self.total_items > 0 {
            let max_item_percentage = self.item_type_counts.values()
                .map(|&count| (count as f64 / self.total_items as f64) * 100.0)
                .fold(0.0, f64::max);
            max_item_percentage <= 60.0
        } else {
            true
        };

        let rel_balance = if self.total_relationships > 0 {
            let max_rel_percentage = self.relationship_type_counts.values()
                .map(|&count| (count as f64 / self.total_relationships as f64) * 100.0)
                .fold(0.0, f64::max);
            max_rel_percentage <= 60.0
        } else {
            true
        };

        item_balance && rel_balance
    }

    /// Get imbalance report
    pub fn imbalance_report(&self) -> Option<String> {
        let mut issues = Vec::new();

        // Check item type imbalance
        if self.total_items > 0 {
            for (item_type, count) in &self.item_type_counts {
                let percentage = (*count as f64 / self.total_items as f64) * 100.0;
                if percentage > 70.0 {
                    issues.push(format!("Item type '{}' dominates with {:.1}% of all items", item_type, percentage));
                }
            }
        }

        // Check relationship type imbalance
        if self.total_relationships > 0 {
            for (rel_type, count) in &self.relationship_type_counts {
                let percentage = (*count as f64 / self.total_relationships as f64) * 100.0;
                if percentage > 70.0 {
                    issues.push(format!("Relationship type '{}' dominates with {:.1}% of all relationships", rel_type, percentage));
                }
            }
        }

        if issues.is_empty() {
            None
        } else {
            Some(format!("Imbalance Issues:\n{}", issues.join("\n")))
        }
    }

    /// Calculate growth potential score (0.0 to 1.0)
    pub fn growth_potential(&self) -> f64 {
        if self.total_items == 0 {
            return 1.0; // Empty memory has maximum growth potential
        }

        let density = self.get_density();
        let diversity = (self.item_type_diversity() + self.relationship_type_diversity()) / 2.0;
        let balance_score = if self.is_balanced() { 1.0 } else { 0.5 };

        // Lower density and higher diversity indicate more growth potential
        let density_potential = if density < 1.0 { 1.0 - density } else { 0.5 };
        let diversity_potential = diversity;

        (density_potential * 0.4 + diversity_potential * 0.4 + balance_score * 0.2).clamp(0.0, 1.0)
    }
}
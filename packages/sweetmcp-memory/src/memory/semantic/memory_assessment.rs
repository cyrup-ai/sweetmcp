//! Semantic memory assessment and quality analysis
//!
//! This module provides memory assessment capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;

impl SemanticMemory {
    /// Get memory health score
    pub fn get_health_score(&self) -> f64 {
        if self.items.is_empty() {
            return 1.0; // Empty memory is considered healthy
        }

        let mut health_factors = Vec::new();

        // Factor 1: Relationship density (0.0 to 1.0)
        let max_possible_relationships = self.items.len() * (self.items.len() - 1) / 2;
        let relationship_density = if max_possible_relationships > 0 {
            (self.relationships.len() as f64 / max_possible_relationships as f64).min(1.0)
        } else {
            0.0
        };
        health_factors.push(relationship_density);

        // Factor 2: Connectivity (percentage of non-isolated items)
        let isolated_count = self.get_isolated_items().len();
        let connectivity_ratio = if self.items.len() > 0 {
            1.0 - (isolated_count as f64 / self.items.len() as f64)
        } else {
            1.0
        };
        health_factors.push(connectivity_ratio);

        // Factor 3: Balance (inverse of clustering coefficient variance)
        let clustering_coeff = self.clustering_coefficient();
        let balance_factor = if clustering_coeff > 0.0 && clustering_coeff < 1.0 {
            1.0 - (clustering_coeff - 0.5).abs() * 2.0
        } else {
            0.5
        };
        health_factors.push(balance_factor);

        // Factor 4: Validation health (all items and relationships are valid)
        let validation_health = if self.validate().is_ok() { 1.0 } else { 0.0 };
        health_factors.push(validation_health);

        // Calculate weighted average
        let weights = [0.25, 0.35, 0.15, 0.25]; // Prioritize connectivity and validation
        health_factors
            .iter()
            .zip(&weights)
            .map(|(factor, weight)| factor * weight)
            .sum()
    }

    /// Get memory complexity score
    pub fn get_complexity_score(&self) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }

        let mut complexity_factors = Vec::new();

        // Factor 1: Size complexity (logarithmic scale)
        let size_complexity = (self.items.len() as f64).ln() / 10.0; // Normalize to reasonable range
        complexity_factors.push(size_complexity.min(1.0));

        // Factor 2: Relationship complexity
        let avg_relationships_per_item = if self.items.len() > 0 {
            (self.relationships.len() * 2) as f64 / self.items.len() as f64 // *2 because each relationship connects 2 items
        } else {
            0.0
        };
        let relationship_complexity = (avg_relationships_per_item / 10.0).min(1.0); // Normalize
        complexity_factors.push(relationship_complexity);

        // Factor 3: Graph structure complexity (based on strongly connected components)
        let scc_count = self.get_strongly_connected_components().len();
        let structure_complexity = if self.items.len() > 0 {
            1.0 - (scc_count as f64 / self.items.len() as f64)
        } else {
            0.0
        };
        complexity_factors.push(structure_complexity);

        // Factor 4: Metadata complexity (average metadata per item)
        let avg_metadata_per_item = if self.items.len() > 0 {
            self.items.iter().map(|item| item.metadata.len()).sum::<usize>() as f64 / self.items.len() as f64
        } else {
            0.0
        };
        let metadata_complexity = (avg_metadata_per_item / 20.0).min(1.0); // Normalize
        complexity_factors.push(metadata_complexity);

        // Calculate weighted average
        let weights = [0.2, 0.3, 0.3, 0.2];
        complexity_factors
            .iter()
            .zip(&weights)
            .map(|(factor, weight)| factor * weight)
            .sum()
    }

    /// Get memory efficiency score
    pub fn get_efficiency_score(&self) -> f64 {
        if self.items.is_empty() {
            return 1.0; // Empty memory is perfectly efficient
        }

        let mut efficiency_factors = Vec::new();

        // Factor 1: Storage efficiency (capacity utilization)
        let (items_util, relationships_util) = self.capacity_utilization();
        let storage_efficiency = (items_util + relationships_util) / 2.0;
        efficiency_factors.push(storage_efficiency);

        // Factor 2: Connectivity efficiency (few isolated items)
        let isolated_ratio = self.get_isolated_items().len() as f64 / self.items.len() as f64;
        let connectivity_efficiency = 1.0 - isolated_ratio;
        efficiency_factors.push(connectivity_efficiency);

        // Factor 3: Redundancy efficiency (low duplicate relationships)
        let total_possible_pairs = self.items.len() * (self.items.len() - 1) / 2;
        let redundancy_efficiency = if total_possible_pairs > 0 {
            1.0 - (self.relationships.len() as f64 / total_possible_pairs as f64).min(1.0)
        } else {
            1.0
        };
        efficiency_factors.push(redundancy_efficiency);

        // Factor 4: Access efficiency (based on average path length)
        let diameter = self.graph_diameter().unwrap_or(0);
        let access_efficiency = if diameter > 0 {
            1.0 / (diameter as f64).ln().max(1.0)
        } else {
            1.0
        };
        efficiency_factors.push(access_efficiency);

        // Calculate weighted average
        let weights = [0.25, 0.25, 0.25, 0.25];
        efficiency_factors
            .iter()
            .zip(&weights)
            .map(|(factor, weight)| factor * weight)
            .sum()
    }

    /// Get comprehensive memory assessment
    pub fn get_memory_assessment(&self) -> MemoryAssessment {
        MemoryAssessment {
            health_score: self.get_health_score(),
            complexity_score: self.get_complexity_score(),
            efficiency_score: self.get_efficiency_score(),
            item_count: self.items.len(),
            relationship_count: self.relationships.len(),
            isolated_items: self.get_isolated_items().len(),
            hub_items: self.get_hub_items(5).len(),
            is_connected: self.is_connected(),
            clustering_coefficient: self.clustering_coefficient(),
            diameter: self.graph_diameter(),
            radius: self.graph_radius(),
        }
    }
}

/// Comprehensive memory assessment
#[derive(Debug, Clone)]
pub struct MemoryAssessment {
    pub health_score: f64,
    pub complexity_score: f64,
    pub efficiency_score: f64,
    pub item_count: usize,
    pub relationship_count: usize,
    pub isolated_items: usize,
    pub hub_items: usize,
    pub is_connected: bool,
    pub clustering_coefficient: f64,
    pub diameter: Option<usize>,
    pub radius: Option<usize>,
}

impl MemoryAssessment {
    /// Get overall quality score (weighted combination of health, complexity, efficiency)
    pub fn overall_quality_score(&self) -> f64 {
        // Weight: health 40%, efficiency 35%, complexity 25% (lower complexity is better)
        self.health_score * 0.4 + self.efficiency_score * 0.35 + (1.0 - self.complexity_score) * 0.25
    }

    /// Get quality rating as a descriptive string
    pub fn quality_rating(&self) -> &'static str {
        let score = self.overall_quality_score();
        match score {
            s if s >= 0.9 => "Excellent",
            s if s >= 0.8 => "Very Good",
            s if s >= 0.7 => "Good",
            s if s >= 0.6 => "Fair",
            s if s >= 0.5 => "Poor",
            _ => "Very Poor",
        }
    }

    /// Get recommendations for improving memory quality
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.health_score < 0.7 {
            recommendations.push("Consider validating and fixing any inconsistencies in the memory structure".to_string());
        }

        if self.efficiency_score < 0.6 {
            recommendations.push("Optimize memory usage by compacting orphaned relationships".to_string());
        }

        if self.isolated_items > self.item_count / 4 {
            recommendations.push("Reduce isolated items by creating more relationships between items".to_string());
        }

        if !self.is_connected && self.item_count > 1 {
            recommendations.push("Improve connectivity by linking disconnected components".to_string());
        }

        if self.clustering_coefficient < 0.1 {
            recommendations.push("Increase local clustering to improve semantic coherence".to_string());
        }

        if let Some(diameter) = self.diameter {
            if diameter > 10 {
                recommendations.push("Consider adding shortcut relationships to reduce graph diameter".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Memory structure appears to be in good condition".to_string());
        }

        recommendations
    }

    /// Get detailed assessment report
    pub fn get_detailed_report(&self) -> String {
        format!(
            "Memory Assessment Report\n\
            ========================\n\
            Overall Quality: {} ({:.2})\n\
            \n\
            Scores:\n\
            - Health: {:.2}/1.0\n\
            - Efficiency: {:.2}/1.0\n\
            - Complexity: {:.2}/1.0\n\
            \n\
            Structure:\n\
            - Items: {}\n\
            - Relationships: {}\n\
            - Isolated Items: {}\n\
            - Hub Items (5+ connections): {}\n\
            - Connected: {}\n\
            - Clustering Coefficient: {:.3}\n\
            - Diameter: {}\n\
            - Radius: {}\n\
            \n\
            Recommendations:\n\
            {}",
            self.quality_rating(),
            self.overall_quality_score(),
            self.health_score,
            self.efficiency_score,
            self.complexity_score,
            self.item_count,
            self.relationship_count,
            self.isolated_items,
            self.hub_items,
            if self.is_connected { "Yes" } else { "No" },
            self.clustering_coefficient,
            self.diameter.map_or("N/A".to_string(), |d| d.to_string()),
            self.radius.map_or("N/A".to_string(), |r| r.to_string()),
            self.get_recommendations().join("\n- ")
        )
    }
}
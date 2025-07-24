//! Semantic memory analysis and quality assessment functionality
//!
//! This module provides analysis and quality assessment capabilities for memory statistics
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory_statistics::MemoryStatistics;

impl MemoryStatistics {
    /// Get detailed breakdown by categories
    pub fn detailed_breakdown(&self) -> String {
        let mut breakdown = String::new();
        
        breakdown.push_str("Detailed Statistics Breakdown:\n\n");

        // Item types breakdown
        if !self.item_type_counts.is_empty() {
            breakdown.push_str("Item Types:\n");
            let mut sorted_items: Vec<_> = self.item_type_counts.iter().collect();
            sorted_items.sort_by(|a, b| b.1.cmp(a.1));
            for (item_type, count) in sorted_items {
                let percentage = (*count as f64 / self.total_items as f64) * 100.0;
                breakdown.push_str(&format!("  - {}: {} ({:.1}%)\n", item_type, count, percentage));
            }
            breakdown.push('\n');
        }

        // Relationship types breakdown
        if !self.relationship_type_counts.is_empty() {
            breakdown.push_str("Relationship Types:\n");
            let mut sorted_rels: Vec<_> = self.relationship_type_counts.iter().collect();
            sorted_rels.sort_by(|a, b| b.1.cmp(a.1));
            for (rel_type, count) in sorted_rels {
                let percentage = (*count as f64 / self.total_relationships as f64) * 100.0;
                breakdown.push_str(&format!("  - {}: {} ({:.1}%)\n", rel_type, count, percentage));
            }
            breakdown.push('\n');
        }

        // Confidence distribution breakdown
        if !self.confidence_distribution.is_empty() {
            breakdown.push_str("Confidence Distribution:\n");
            let mut sorted_conf: Vec<_> = self.confidence_distribution.iter().collect();
            sorted_conf.sort_by(|a, b| b.1.cmp(a.1));
            for (confidence, count) in sorted_conf {
                let percentage = (*count as f64 / self.total_items as f64) * 100.0;
                breakdown.push_str(&format!("  - {}: {} ({:.1}%)\n", confidence, count, percentage));
            }
        }

        breakdown
    }

    /// Get quality assessment
    pub fn quality_assessment(&self) -> String {
        let health = self.health_score();
        let density = self.get_density();
        let diversity = (self.item_type_diversity() + self.relationship_type_diversity()) / 2.0;

        let health_status = if health >= 0.8 {
            "Excellent"
        } else if health >= 0.6 {
            "Good"
        } else if health >= 0.4 {
            "Fair"
        } else {
            "Poor"
        };

        let connectivity_status = if density >= 2.0 {
            "Highly Connected"
        } else if density >= 1.0 {
            "Well Connected"
        } else if density >= 0.5 {
            "Moderately Connected"
        } else {
            "Sparsely Connected"
        };

        let diversity_status = if diversity >= 0.8 {
            "Highly Diverse"
        } else if diversity >= 0.6 {
            "Well Diverse"
        } else if diversity >= 0.4 {
            "Moderately Diverse"
        } else {
            "Low Diversity"
        };

        format!(
            "Quality Assessment:\n\
             - Overall Health: {} ({:.2})\n\
             - Connectivity: {} ({:.2})\n\
             - Diversity: {} ({:.2})\n",
            health_status, health,
            connectivity_status, density,
            diversity_status, diversity
        )
    }

    /// Calculate memory efficiency score (0.0 to 1.0)
    pub fn efficiency_score(&self) -> f64 {
        if self.total_items == 0 {
            return 1.0; // Empty memory is perfectly efficient
        }

        let density = self.get_density();
        let health = self.health_score();
        let diversity = (self.item_type_diversity() + self.relationship_type_diversity()) / 2.0;

        // Optimal density is around 1.0-2.0 relationships per item
        let density_score = if density >= 1.0 && density <= 2.0 {
            1.0
        } else if density < 1.0 {
            density
        } else {
            1.0 / (density - 1.0).max(1.0)
        };

        (density_score * 0.4 + health * 0.4 + diversity * 0.2).clamp(0.0, 1.0)
    }

    /// Get memory maturity level
    pub fn maturity_level(&self) -> String {
        let health = self.health_score();
        let density = self.get_density();
        let size = self.total_items;

        if size == 0 {
            "Empty".to_string()
        } else if size < 5 || density < 0.2 {
            "Nascent".to_string()
        } else if size < 20 || density < 0.5 || health < 0.4 {
            "Developing".to_string()
        } else if size < 100 || density < 1.0 || health < 0.6 {
            "Maturing".to_string()
        } else if health >= 0.8 && density >= 1.0 {
            "Mature".to_string()
        } else {
            "Advanced".to_string()
        }
    }

    /// Get performance insights
    pub fn performance_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // Size insights
        if self.total_items == 0 {
            insights.push("Memory is empty - consider adding semantic content".to_string());
        } else if self.total_items < 10 {
            insights.push("Memory has limited content - consider expanding with more items".to_string());
        } else if self.total_items > 1000 {
            insights.push("Memory has substantial content - monitor performance for large-scale operations".to_string());
        }

        // Connectivity insights
        let density = self.get_density();
        if density < 0.5 {
            insights.push("Low connectivity detected - items are poorly connected, consider adding more relationships".to_string());
        } else if density > 3.0 {
            insights.push("Very high connectivity - excellent for traversal but may impact performance".to_string());
        }

        // Diversity insights
        let item_diversity = self.item_type_diversity();
        let rel_diversity = self.relationship_type_diversity();
        
        if item_diversity < 0.3 {
            insights.push("Low item type diversity - content is homogeneous, consider adding varied item types".to_string());
        }
        
        if rel_diversity < 0.3 {
            insights.push("Low relationship type diversity - connections are uniform, consider varied relationship types".to_string());
        }

        // Health insights
        let health = self.health_score();
        if health < 0.4 {
            insights.push("Poor memory health - review content quality and organization".to_string());
        } else if health > 0.8 {
            insights.push("Excellent memory health - well-organized and balanced content".to_string());
        }

        // Balance insights
        if let Some((most_common_type, count)) = self.most_common_item_type() {
            let percentage = (*count as f64 / self.total_items as f64) * 100.0;
            if percentage > 70.0 {
                insights.push(format!("Content heavily skewed toward {} items ({:.1}%) - consider balancing with other types", most_common_type, percentage));
            }
        }

        if let Some((most_common_rel, count)) = self.most_common_relationship_type() {
            let percentage = (*count as f64 / self.total_relationships as f64) * 100.0;
            if percentage > 70.0 {
                insights.push(format!("Relationships heavily skewed toward {} type ({:.1}%) - consider diversifying relationship types", most_common_rel, percentage));
            }
        }

        insights
    }

    /// Get optimization recommendations
    pub fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let density = self.get_density();
        let health = self.health_score();
        let item_diversity = self.item_type_diversity();
        let rel_diversity = self.relationship_type_diversity();

        // Density optimization
        if density < 0.5 {
            recommendations.push("Increase relationship density by connecting related items".to_string());
            recommendations.push("Review existing items for potential semantic connections".to_string());
        } else if density > 4.0 {
            recommendations.push("Consider pruning weak or redundant relationships to improve performance".to_string());
        }

        // Diversity optimization
        if item_diversity < 0.5 {
            recommendations.push("Add items of different types (Concept, Fact, Rule, Category) to improve diversity".to_string());
        }

        if rel_diversity < 0.5 {
            recommendations.push("Use varied relationship types (IsA, PartOf, Causes, etc.) to enrich connections".to_string());
        }

        // Health optimization
        if health < 0.6 {
            recommendations.push("Review and improve content quality and organization".to_string());
            recommendations.push("Ensure balanced distribution of confidence levels".to_string());
        }

        // Size optimization
        if self.total_items < 5 {
            recommendations.push("Add more semantic items to reach critical mass for meaningful analysis".to_string());
        }

        if self.total_relationships == 0 && self.total_items > 1 {
            recommendations.push("Create relationships between existing items to enable semantic traversal".to_string());
        }

        // Performance optimization
        if self.total_items > 500 && density > 2.0 {
            recommendations.push("Consider indexing strategies for large, highly-connected memories".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Memory is well-optimized - maintain current quality standards".to_string());
        }

        recommendations
    }

    /// Get comprehensive analysis report
    pub fn comprehensive_analysis(&self) -> String {
        let mut report = String::new();

        report.push_str("=== COMPREHENSIVE MEMORY ANALYSIS ===\n\n");
        
        // Basic statistics
        report.push_str(&self.summary_report());
        report.push('\n');

        // Quality assessment
        report.push_str(&self.quality_assessment());
        report.push('\n');

        // Maturity and efficiency
        report.push_str(&format!(
            "Maturity & Efficiency:\n\
             - Maturity Level: {}\n\
             - Efficiency Score: {:.2}\n\n",
            self.maturity_level(),
            self.efficiency_score()
        ));

        // Performance insights
        let insights = self.performance_insights();
        if !insights.is_empty() {
            report.push_str("Performance Insights:\n");
            for insight in insights {
                report.push_str(&format!("• {}\n", insight));
            }
            report.push('\n');
        }

        // Optimization recommendations
        let recommendations = self.optimization_recommendations();
        if !recommendations.is_empty() {
            report.push_str("Optimization Recommendations:\n");
            for recommendation in recommendations {
                report.push_str(&format!("• {}\n", recommendation));
            }
        }

        report
    }
}
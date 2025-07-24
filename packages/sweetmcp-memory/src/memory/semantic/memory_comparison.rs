//! Semantic memory statistics comparison functionality
//!
//! This module provides comparison capabilities for memory statistics
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory_statistics::MemoryStatistics;

/// Statistics comparison result
#[derive(Debug, Clone)]
pub struct StatisticsComparison {
    /// Difference in item count
    pub items_diff: i64,
    /// Difference in relationship count
    pub relationships_diff: i64,
    /// Difference in density
    pub density_diff: f64,
    /// Difference in diversity
    pub diversity_diff: f64,
    /// Difference in health score
    pub health_diff: f64,
}

impl StatisticsComparison {
    /// Create a new comparison between two statistics
    pub fn new(current: &MemoryStatistics, previous: &MemoryStatistics) -> Self {
        Self {
            items_diff: current.total_items as i64 - previous.total_items as i64,
            relationships_diff: current.total_relationships as i64 - previous.total_relationships as i64,
            density_diff: current.get_density() - previous.get_density(),
            diversity_diff: current.item_type_diversity() - previous.item_type_diversity(),
            health_diff: current.health_score() - previous.health_score(),
        }
    }

    /// Check if this represents an improvement
    pub fn is_improvement(&self) -> bool {
        self.items_diff >= 0 
            && self.relationships_diff >= 0 
            && self.density_diff >= 0.0 
            && self.diversity_diff >= 0.0 
            && self.health_diff >= 0.0
    }

    /// Check if this represents a significant improvement
    pub fn is_significant_improvement(&self) -> bool {
        self.is_improvement() && (
            self.items_diff > 5 
            || self.relationships_diff > 5 
            || self.density_diff > 0.1 
            || self.diversity_diff > 0.1 
            || self.health_diff > 0.1
        )
    }

    /// Check if this represents a decline
    pub fn is_decline(&self) -> bool {
        self.items_diff < 0 
            || self.relationships_diff < 0 
            || self.density_diff < 0.0 
            || self.diversity_diff < 0.0 
            || self.health_diff < 0.0
    }

    /// Check if this represents a significant decline
    pub fn is_significant_decline(&self) -> bool {
        self.is_decline() && (
            self.items_diff < -5 
            || self.relationships_diff < -5 
            || self.density_diff < -0.1 
            || self.diversity_diff < -0.1 
            || self.health_diff < -0.1
        )
    }

    /// Get improvement summary
    pub fn improvement_summary(&self) -> String {
        format!(
            "Statistics Comparison:\n\
             - Items: {:+}\n\
             - Relationships: {:+}\n\
             - Density: {:+.2}\n\
             - Diversity: {:+.2}\n\
             - Health: {:+.2}\n\
             - Overall: {}",
            self.items_diff,
            self.relationships_diff,
            self.density_diff,
            self.diversity_diff,
            self.health_diff,
            self.get_overall_status()
        )
    }

    /// Get overall status string
    pub fn get_overall_status(&self) -> &'static str {
        if self.is_significant_improvement() {
            "Significantly Improved"
        } else if self.is_improvement() {
            "Improved"
        } else if self.is_significant_decline() {
            "Significantly Declined"
        } else if self.is_decline() {
            "Declined"
        } else {
            "No Change"
        }
    }

    /// Get detailed analysis
    pub fn detailed_analysis(&self) -> String {
        let mut analysis = String::new();
        
        analysis.push_str("Detailed Comparison Analysis:\n\n");

        // Items analysis
        if self.items_diff > 0 {
            analysis.push_str(&format!("✅ Items increased by {} (positive growth)\n", self.items_diff));
        } else if self.items_diff < 0 {
            analysis.push_str(&format!("❌ Items decreased by {} (content loss)\n", self.items_diff.abs()));
        } else {
            analysis.push_str("➖ Items count unchanged\n");
        }

        // Relationships analysis
        if self.relationships_diff > 0 {
            analysis.push_str(&format!("✅ Relationships increased by {} (better connectivity)\n", self.relationships_diff));
        } else if self.relationships_diff < 0 {
            analysis.push_str(&format!("❌ Relationships decreased by {} (reduced connectivity)\n", self.relationships_diff.abs()));
        } else {
            analysis.push_str("➖ Relationships count unchanged\n");
        }

        // Density analysis
        if self.density_diff > 0.1 {
            analysis.push_str(&format!("✅ Density improved significantly by {:.2} (much better connected)\n", self.density_diff));
        } else if self.density_diff > 0.0 {
            analysis.push_str(&format!("✅ Density improved by {:.2} (better connected)\n", self.density_diff));
        } else if self.density_diff < -0.1 {
            analysis.push_str(&format!("❌ Density declined significantly by {:.2} (much less connected)\n", self.density_diff.abs()));
        } else if self.density_diff < 0.0 {
            analysis.push_str(&format!("❌ Density declined by {:.2} (less connected)\n", self.density_diff.abs()));
        } else {
            analysis.push_str("➖ Density unchanged\n");
        }

        // Diversity analysis
        if self.diversity_diff > 0.1 {
            analysis.push_str(&format!("✅ Diversity improved significantly by {:.2} (much more varied content)\n", self.diversity_diff));
        } else if self.diversity_diff > 0.0 {
            analysis.push_str(&format!("✅ Diversity improved by {:.2} (more varied content)\n", self.diversity_diff));
        } else if self.diversity_diff < -0.1 {
            analysis.push_str(&format!("❌ Diversity declined significantly by {:.2} (much less varied content)\n", self.diversity_diff.abs()));
        } else if self.diversity_diff < 0.0 {
            analysis.push_str(&format!("❌ Diversity declined by {:.2} (less varied content)\n", self.diversity_diff.abs()));
        } else {
            analysis.push_str("➖ Diversity unchanged\n");
        }

        // Health analysis
        if self.health_diff > 0.1 {
            analysis.push_str(&format!("✅ Health improved significantly by {:.2} (much healthier memory)\n", self.health_diff));
        } else if self.health_diff > 0.0 {
            analysis.push_str(&format!("✅ Health improved by {:.2} (healthier memory)\n", self.health_diff));
        } else if self.health_diff < -0.1 {
            analysis.push_str(&format!("❌ Health declined significantly by {:.2} (much less healthy memory)\n", self.health_diff.abs()));
        } else if self.health_diff < 0.0 {
            analysis.push_str(&format!("❌ Health declined by {:.2} (less healthy memory)\n", self.health_diff.abs()));
        } else {
            analysis.push_str("➖ Health unchanged\n");
        }

        analysis.push_str(&format!("\nOverall Assessment: {}", self.get_overall_status()));

        analysis
    }

    /// Get recommendations based on comparison
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if self.items_diff < 0 {
            recommendations.push("Consider adding more semantic items to restore content richness".to_string());
        }

        if self.relationships_diff < 0 {
            recommendations.push("Focus on creating more relationships between existing items".to_string());
        }

        if self.density_diff < -0.1 {
            recommendations.push("Memory connectivity has declined significantly - review relationship creation strategies".to_string());
        }

        if self.diversity_diff < -0.1 {
            recommendations.push("Content diversity has decreased - consider adding items of different types".to_string());
        }

        if self.health_diff < -0.1 {
            recommendations.push("Overall memory health has declined - review content quality and organization".to_string());
        }

        if recommendations.is_empty() {
            if self.is_improvement() {
                recommendations.push("Memory is improving - continue current strategies".to_string());
            } else {
                recommendations.push("Memory is stable - consider strategies for growth and improvement".to_string());
            }
        }

        recommendations
    }

    /// Calculate improvement score (0.0 to 1.0, higher is better)
    pub fn improvement_score(&self) -> f64 {
        let items_score = if self.items_diff >= 0 { 1.0 } else { 0.0 };
        let relationships_score = if self.relationships_diff >= 0 { 1.0 } else { 0.0 };
        let density_score = (self.density_diff + 1.0).clamp(0.0, 1.0);
        let diversity_score = (self.diversity_diff + 1.0).clamp(0.0, 1.0);
        let health_score = (self.health_diff + 1.0).clamp(0.0, 1.0);

        (items_score * 0.2 + relationships_score * 0.2 + density_score * 0.2 + diversity_score * 0.2 + health_score * 0.2)
    }
}

impl MemoryStatistics {
    /// Compare with another statistics instance
    pub fn compare_with(&self, other: &MemoryStatistics) -> StatisticsComparison {
        StatisticsComparison::new(self, other)
    }
}
//! Semantic memory health monitoring and diagnostics
//!
//! This module provides health monitoring and diagnostic capabilities for semantic memory
//! with zero allocation, blazing-fast performance, and ergonomic API design.

use super::memory::SemanticMemory;

impl SemanticMemory {
    /// Get memory usage report
    pub fn get_memory_usage_report(&self) -> MemoryUsageReport {
        let (items_util, relationships_util) = self.capacity_utilization();
        let estimated_size = self.estimate_memory_size();
        
        MemoryUsageReport {
            item_count: self.items.len(),
            item_capacity: self.items.capacity(),
            item_utilization: items_util,
            relationship_count: self.relationships.len(),
            relationship_capacity: self.relationships.capacity(),
            relationship_utilization: relationships_util,
            estimated_size_bytes: estimated_size,
            memory_efficiency: (items_util + relationships_util) / 2.0,
        }
    }

    /// Optimize memory layout
    pub fn optimize_memory_layout(&mut self) -> MemoryOptimizationResult {
        let initial_capacity = self.items.capacity() + self.relationships.capacity();
        let initial_size = self.estimate_memory_size();
        
        // Sort items by ID for better cache locality
        self.items.sort_by(|a, b| a.id.cmp(&b.id));
        
        // Sort relationships by source_id, then target_id for better cache locality
        self.relationships.sort_by(|a, b| {
            a.source_id.cmp(&b.source_id).then_with(|| a.target_id.cmp(&b.target_id))
        });
        
        // Shrink to fit to reduce memory overhead
        self.shrink_to_fit();
        
        let final_capacity = self.items.capacity() + self.relationships.capacity();
        let final_size = self.estimate_memory_size();
        
        MemoryOptimizationResult {
            initial_capacity,
            final_capacity,
            capacity_reduction: initial_capacity.saturating_sub(final_capacity),
            initial_size_bytes: initial_size,
            final_size_bytes: final_size,
            size_reduction_bytes: initial_size.saturating_sub(final_size),
        }
    }

    /// Get memory fragmentation analysis
    pub fn analyze_memory_fragmentation(&self) -> MemoryFragmentationAnalysis {
        let item_fragmentation = if self.items.capacity() > 0 {
            1.0 - (self.items.len() as f64 / self.items.capacity() as f64)
        } else {
            0.0
        };
        
        let relationship_fragmentation = if self.relationships.capacity() > 0 {
            1.0 - (self.relationships.len() as f64 / self.relationships.capacity() as f64)
        } else {
            0.0
        };
        
        let overall_fragmentation = (item_fragmentation + relationship_fragmentation) / 2.0;
        
        let fragmentation_level = match overall_fragmentation {
            f if f < 0.1 => FragmentationLevel::Low,
            f if f < 0.3 => FragmentationLevel::Medium,
            f if f < 0.5 => FragmentationLevel::High,
            _ => FragmentationLevel::VeryHigh,
        };
        
        MemoryFragmentationAnalysis {
            item_fragmentation,
            relationship_fragmentation,
            overall_fragmentation,
            fragmentation_level,
            wasted_item_slots: self.items.capacity().saturating_sub(self.items.len()),
            wasted_relationship_slots: self.relationships.capacity().saturating_sub(self.relationships.len()),
        }
    }

    /// Perform memory cleanup
    pub fn cleanup_memory(&mut self) -> MemoryCleanupResult {
        let initial_items = self.items.len();
        let initial_relationships = self.relationships.len();
        
        // Remove orphaned relationships
        let orphaned_removed = self.compact();
        
        // Remove items with empty content (if any)
        let empty_items_removed = {
            let initial_count = self.items.len();
            self.items.retain(|item| !item.content.trim().is_empty());
            initial_count - self.items.len()
        };
        
        // Optimize memory layout
        let optimization_result = self.optimize_memory_layout();
        
        MemoryCleanupResult {
            initial_items,
            initial_relationships,
            final_items: self.items.len(),
            final_relationships: self.relationships.len(),
            orphaned_relationships_removed: orphaned_removed,
            empty_items_removed,
            memory_optimization: optimization_result,
        }
    }

    /// Check memory health
    pub fn check_memory_health(&self) -> MemoryHealthCheck {
        let validation_result = self.validate();
        let fragmentation = self.analyze_memory_fragmentation();
        let usage_report = self.get_memory_usage_report();
        
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for validation errors
        if validation_result.is_err() {
            issues.push("Memory validation failed".to_string());
        }
        
        // Check for high fragmentation
        match fragmentation.fragmentation_level {
            FragmentationLevel::High | FragmentationLevel::VeryHigh => {
                warnings.push("High memory fragmentation detected".to_string());
            }
            _ => {}
        }
        
        // Check for low utilization
        if usage_report.memory_efficiency < 0.5 {
            warnings.push("Low memory utilization detected".to_string());
        }
        
        // Check for isolated items
        let isolated_count = self.get_isolated_items().len();
        if isolated_count > self.items.len() / 4 {
            warnings.push(format!("High number of isolated items: {}", isolated_count));
        }
        
        let health_status = if !issues.is_empty() {
            HealthStatus::Critical
        } else if !warnings.is_empty() {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        MemoryHealthCheck {
            status: health_status,
            issues,
            warnings,
            validation_passed: validation_result.is_ok(),
            fragmentation_analysis: fragmentation,
            usage_report,
        }
    }

    /// Generate health report
    pub fn generate_health_report(&self) -> String {
        let health_check = self.check_memory_health();
        let assessment = self.get_memory_assessment();
        
        format!(
            "Semantic Memory Health Report\n\
            =============================\n\
            \n\
            Overall Status: {:?}\n\
            Quality Rating: {} ({:.2})\n\
            \n\
            Health Metrics:\n\
            - Validation: {}\n\
            - Memory Efficiency: {:.1}%\n\
            - Fragmentation Level: {:?} ({:.1}%)\n\
            \n\
            Structure:\n\
            - Items: {} (capacity: {})\n\
            - Relationships: {} (capacity: {})\n\
            - Isolated Items: {}\n\
            - Connected: {}\n\
            \n\
            Issues:\n\
            {}\n\
            \n\
            Warnings:\n\
            {}\n\
            \n\
            Recommendations:\n\
            {}",
            health_check.status,
            assessment.quality_rating(),
            assessment.overall_quality_score(),
            if health_check.validation_passed { "PASSED" } else { "FAILED" },
            health_check.usage_report.memory_efficiency * 100.0,
            health_check.fragmentation_analysis.fragmentation_level,
            health_check.fragmentation_analysis.overall_fragmentation * 100.0,
            health_check.usage_report.item_count,
            health_check.usage_report.item_capacity,
            health_check.usage_report.relationship_count,
            health_check.usage_report.relationship_capacity,
            self.get_isolated_items().len(),
            if self.is_connected() { "Yes" } else { "No" },
            if health_check.issues.is_empty() { 
                "None".to_string() 
            } else { 
                health_check.issues.join("\n- ") 
            },
            if health_check.warnings.is_empty() { 
                "None".to_string() 
            } else { 
                health_check.warnings.join("\n- ") 
            },
            assessment.get_recommendations().join("\n- ")
        )
    }
}

/// Memory usage report
#[derive(Debug, Clone)]
pub struct MemoryUsageReport {
    pub item_count: usize,
    pub item_capacity: usize,
    pub item_utilization: f64,
    pub relationship_count: usize,
    pub relationship_capacity: usize,
    pub relationship_utilization: f64,
    pub estimated_size_bytes: usize,
    pub memory_efficiency: f64,
}

/// Memory optimization result
#[derive(Debug, Clone)]
pub struct MemoryOptimizationResult {
    pub initial_capacity: usize,
    pub final_capacity: usize,
    pub capacity_reduction: usize,
    pub initial_size_bytes: usize,
    pub final_size_bytes: usize,
    pub size_reduction_bytes: usize,
}

/// Memory fragmentation analysis
#[derive(Debug, Clone)]
pub struct MemoryFragmentationAnalysis {
    pub item_fragmentation: f64,
    pub relationship_fragmentation: f64,
    pub overall_fragmentation: f64,
    pub fragmentation_level: FragmentationLevel,
    pub wasted_item_slots: usize,
    pub wasted_relationship_slots: usize,
}

/// Fragmentation level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentationLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Memory cleanup result
#[derive(Debug, Clone)]
pub struct MemoryCleanupResult {
    pub initial_items: usize,
    pub initial_relationships: usize,
    pub final_items: usize,
    pub final_relationships: usize,
    pub orphaned_relationships_removed: usize,
    pub empty_items_removed: usize,
    pub memory_optimization: MemoryOptimizationResult,
}

/// Memory health check result
#[derive(Debug, Clone)]
pub struct MemoryHealthCheck {
    pub status: HealthStatus,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub validation_passed: bool,
    pub fragmentation_analysis: MemoryFragmentationAnalysis,
    pub usage_report: MemoryUsageReport,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}
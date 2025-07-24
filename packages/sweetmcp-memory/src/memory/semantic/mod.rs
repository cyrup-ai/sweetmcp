//! Semantic memory module coordination
//!
//! This module provides the main coordination layer for semantic memory operations,
//! integrating all submodules with blazing-fast performance and zero allocation optimizations.

pub mod confidence;
pub mod item_types;
pub mod relationships;
pub mod memory_manager_core;
pub mod memory_cleanup;
pub mod memory_optimization;
pub mod semantic_item;
pub mod semantic_relationship;
pub mod coordinator;

// Re-export key types for ergonomic access
pub use confidence::{ConfidenceLevel, ConfidenceCalculator, ConfidenceStatistics};
pub use item_types::{SemanticItemType, SemanticItemTypeClassifier, SemanticItemTypeStatistics};
pub use relationships::{
    SemanticRelationshipType, RelationshipDirection, RelationshipPattern,
    RelationshipStatistics, RelationshipValidator, RelationshipQueryBuilder,
};
pub use memory_manager_core::{
    MemoryStatistics, CleanupConfig, OptimizationStrategy,
};
pub use memory_cleanup::{
    CleanupReport, CleanupStrategy, SemanticMemoryManager,
};
pub use memory_optimization::{
    OptimizationRecommendation, HealthCheckReport, HealthScore, HealthStatus,
    RecommendationType, HealthIssue, IssueSeverity, HealthTrend,
    MemoryOptimizationEngine,
};
pub use semantic_item::{
    SemanticItem, ItemSummary, ArchiveConfig, DeleteConfig, ItemValidationError,
};
pub use semantic_relationship::{
    SemanticRelationship, RelationshipSummary, RelationshipArchiveConfig, 
    RelationshipDeleteConfig, RelationshipValidationError,
};
pub use coordinator::{
    SemanticMemoryCoordinator, ComprehensiveMemoryStatistics,
    SemanticHealthReport,
};
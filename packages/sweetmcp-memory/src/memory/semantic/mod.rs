//! Semantic memory module coordination
//!
//! This module provides the main coordination layer for semantic memory operations,
//! integrating all submodules with blazing-fast performance and zero allocation optimizations.

// All semantic memory modules
pub mod atomic_stats;
pub mod confidence;
pub mod config_types;
pub mod configuration;
pub mod coordinator;
pub mod item;
pub mod item_conversion;
pub mod item_core;
pub mod item_metadata_advanced;
pub mod item_metadata_basic;
pub mod item_metadata_filtering;
pub mod item_metadata_similarity;
pub mod item_metadata_stats;
pub mod item_metadata_validation;
pub mod item_operations;
pub mod item_tags;
pub mod item_types;
pub mod memory;
pub mod memory_analysis;
pub mod memory_assessment;
pub mod memory_batch_operations;
pub mod memory_centrality_analysis;
pub mod memory_cleanup;
pub mod memory_comparison;
pub mod memory_graph_analysis;
pub mod memory_graph_metrics;
pub mod memory_health;
pub mod memory_manager_core;
pub mod memory_multi_field_search;
pub mod memory_operations;
pub mod memory_optimization;
pub mod memory_queries;
pub mod memory_search;
pub mod memory_snapshots;
pub mod memory_statistics;
pub mod memory_stats;
pub mod memory_utilities;
pub mod relationship;
pub mod relationship_types;
pub mod relationships;
pub mod semantic_item;
pub mod semantic_relationship;
pub mod statistics;
pub mod stats_analysis;
pub mod types;

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
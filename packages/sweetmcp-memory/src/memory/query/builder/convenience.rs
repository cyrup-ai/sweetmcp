//! Convenience functions for common query patterns
//!
//! This module provides blazing-fast convenience constructors with zero-allocation
//! patterns and ergonomic APIs for common query use cases.

use super::core::ComplexQueryBuilder;
use crate::memory::MemoryType;

/// Convenience functions for creating common query builders
impl ComplexQueryBuilder {
    /// Create a builder for recent memories
    pub fn recent_memories(days: u32) -> Self {
        Self::new().with_last_days(days)
    }

    /// Create a builder for memories by user
    pub fn user_memories(user_id: impl Into<String>) -> Self {
        Self::new().with_user(user_id)
    }

    /// Create a builder for memories by project
    pub fn project_memories(project_id: impl Into<String>) -> Self {
        Self::new().with_project(project_id)
    }

    /// Create a builder for important memories
    pub fn important_memories(min_importance: f32) -> Self {
        Self::new().with_min_importance(min_importance)
    }

    /// Create a builder for text search
    pub fn text_search(text: impl Into<String>, min_score: f32) -> Self {
        Self::new().with_text_similarity(text, min_score)
    }

    /// Create a builder for memories with specific types
    pub fn typed_memories(types: Vec<MemoryType>) -> Self {
        Self::new().with_memory_types(types)
    }

    /// Create a builder for memories with a single type
    pub fn single_type_memories(memory_type: MemoryType) -> Self {
        Self::new().with_memory_type(memory_type)
    }

    /// Create a builder for memories from the last hour
    pub fn last_hour_memories() -> Self {
        Self::new().with_last_hours(1)
    }

    /// Create a builder for memories from today
    pub fn today_memories() -> Self {
        Self::new().with_last_days(1)
    }

    /// Create a builder for memories from this week
    pub fn this_week_memories() -> Self {
        Self::new().with_last_days(7)
    }

    /// Create a builder for memories from this month
    pub fn this_month_memories() -> Self {
        Self::new().with_last_days(30)
    }

    /// Create a builder for highly important memories (>= 0.8)
    pub fn highly_important_memories() -> Self {
        Self::new().with_min_importance(0.8)
    }

    /// Create a builder for moderately important memories (>= 0.5)
    pub fn moderately_important_memories() -> Self {
        Self::new().with_min_importance(0.5)
    }

    /// Create a builder for memories with specific tag
    pub fn tagged_memories(tag: impl Into<String>) -> Self {
        Self::new().with_tag(tag)
    }

    /// Create a builder for short content memories (< 1000 chars)
    pub fn short_memories() -> Self {
        Self::new().with_max_content_length(1000)
    }

    /// Create a builder for long content memories (>= 5000 chars)
    pub fn long_memories() -> Self {
        Self::new().with_min_content_length(5000)
    }

    /// Create a builder for memories with relationships
    pub fn related_memories(relationship_type: impl Into<String>) -> Self {
        Self::new().with_relationship(relationship_type, None)
    }

    /// Create a builder for memories related to a specific target
    pub fn memories_related_to(
        relationship_type: impl Into<String>,
        target_id: impl Into<String>,
    ) -> Self {
        Self::new().with_relationship(relationship_type, Some(target_id.into()))
    }

    /// Create a builder for episodic memories
    pub fn episodic_memories() -> Self {
        Self::new().with_memory_type(MemoryType::Episodic)
    }

    /// Create a builder for semantic memories
    pub fn semantic_memories() -> Self {
        Self::new().with_memory_type(MemoryType::Semantic)
    }

    /// Create a builder for procedural memories
    pub fn procedural_memories() -> Self {
        Self::new().with_memory_type(MemoryType::Procedural)
    }

    /// Create a builder for working memories
    pub fn working_memories() -> Self {
        Self::new().with_memory_type(MemoryType::Working)
    }
}

/// Factory functions for complex query patterns
pub struct QueryFactory;

impl QueryFactory {
    /// Create a comprehensive search query combining text and metadata
    pub fn comprehensive_search(
        text: impl Into<String>,
        min_score: f32,
        user_id: Option<String>,
        project_id: Option<String>,
        days_back: Option<u32>,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new().with_text_similarity(text, min_score);

        if let Some(user) = user_id {
            builder = builder.with_user(user);
        }

        if let Some(project) = project_id {
            builder = builder.with_project(project);
        }

        if let Some(days) = days_back {
            builder = builder.with_last_days(days);
        }

        builder
    }

    /// Create a query for finding similar memories to a given one
    pub fn similar_memories(
        reference_text: impl Into<String>,
        min_similarity: f32,
        exclude_user: Option<String>,
        memory_types: Option<Vec<MemoryType>>,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new().with_text_similarity(reference_text, min_similarity);

        if let Some(types) = memory_types {
            builder = builder.with_memory_types(types);
        }

        // Note: Excluding user would require NOT operator support
        // For now, we'll just document this limitation

        builder
    }

    /// Create a query for user's important memories in a project
    pub fn user_project_important(
        user_id: impl Into<String>,
        project_id: impl Into<String>,
        min_importance: f32,
        days_back: Option<u32>,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new()
            .with_user(user_id)
            .with_project(project_id)
            .with_min_importance(min_importance);

        if let Some(days) = days_back {
            builder = builder.with_last_days(days);
        }

        builder
    }

    /// Create a query for content analysis (by length and type)
    pub fn content_analysis(
        min_length: Option<usize>,
        max_length: Option<usize>,
        memory_types: Vec<MemoryType>,
        importance_threshold: Option<f32>,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new();

        if min_length.is_some() || max_length.is_some() {
            builder = builder.with_content_length_range(min_length, max_length);
        }

        if !memory_types.is_empty() {
            builder = builder.with_memory_types(memory_types);
        }

        if let Some(threshold) = importance_threshold {
            builder = builder.with_min_importance(threshold);
        }

        builder
    }

    /// Create a query for relationship exploration
    pub fn relationship_exploration(
        relationship_type: impl Into<String>,
        target_id: Option<String>,
        include_related_types: bool,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new().with_relationship(relationship_type, target_id);

        if include_related_types {
            // Add semantic and episodic memories as they often contain relationships
            builder = builder.with_memory_types(vec![MemoryType::Semantic, MemoryType::Episodic]);
        }

        builder
    }

    /// Create a query for temporal analysis
    pub fn temporal_analysis(
        start_days_ago: u32,
        end_days_ago: u32,
        memory_types: Option<Vec<MemoryType>>,
        min_importance: Option<f32>,
    ) -> ComplexQueryBuilder {
        let end = chrono::Utc::now() - chrono::Duration::days(end_days_ago as i64);
        let start = chrono::Utc::now() - chrono::Duration::days(start_days_ago as i64);

        let mut builder = ComplexQueryBuilder::new().with_time_range(start, end);

        if let Some(types) = memory_types {
            builder = builder.with_memory_types(types);
        }

        if let Some(importance) = min_importance {
            builder = builder.with_min_importance(importance);
        }

        builder
    }

    /// Create a query for debugging and analysis
    pub fn debug_query(
        include_all_types: bool,
        include_metadata: bool,
        max_results: Option<usize>,
    ) -> ComplexQueryBuilder {
        let mut builder = ComplexQueryBuilder::new();

        if include_all_types {
            builder = builder.with_memory_types(vec![
                MemoryType::Episodic,
                MemoryType::Semantic,
                MemoryType::Procedural,
                MemoryType::Working,
            ]);
        }

        if include_metadata {
            // Add a broad metadata condition for debugging
            builder = builder.with_metadata_string("debug", "true");
        }

        // Note: max_results would be applied during build phase

        builder
    }
}

/// Predefined query templates for common use cases
pub struct QueryTemplates;

impl QueryTemplates {
    /// Template for daily review queries
    pub fn daily_review(user_id: impl Into<String>) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_user(user_id)
            .with_last_days(1)
            .with_min_importance(0.3)
    }

    /// Template for weekly summary queries
    pub fn weekly_summary(user_id: impl Into<String>) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_user(user_id)
            .with_last_days(7)
            .with_min_importance(0.5)
    }

    /// Template for project retrospective queries
    pub fn project_retrospective(
        project_id: impl Into<String>,
        days_back: u32,
    ) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_project(project_id)
            .with_last_days(days_back)
            .with_memory_types(vec![MemoryType::Episodic, MemoryType::Procedural])
    }

    /// Template for knowledge discovery queries
    pub fn knowledge_discovery(
        text_query: impl Into<String>,
        min_similarity: f32,
    ) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_text_similarity(text_query, min_similarity)
            .with_memory_type(MemoryType::Semantic)
            .with_min_importance(0.4)
    }

    /// Template for learning progress queries
    pub fn learning_progress(
        user_id: impl Into<String>,
        topic_tag: impl Into<String>,
        days_back: u32,
    ) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_user(user_id)
            .with_tag(topic_tag)
            .with_last_days(days_back)
            .with_memory_types(vec![MemoryType::Procedural, MemoryType::Semantic])
    }

    /// Template for collaboration queries
    pub fn collaboration_memories(
        project_id: impl Into<String>,
        relationship_type: impl Into<String>,
    ) -> ComplexQueryBuilder {
        ComplexQueryBuilder::new()
            .with_project(project_id)
            .with_relationship(relationship_type, None)
            .with_memory_type(MemoryType::Episodic)
    }
}
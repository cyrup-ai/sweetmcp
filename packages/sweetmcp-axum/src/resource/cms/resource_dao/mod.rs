//! Resource DAO module with zero allocation patterns
//!
//! This module provides comprehensive resource data access operations for the CMS
//! with zero allocation patterns, blazing-fast performance, and production-ready
//! async operations.

pub mod core;
pub mod streaming;
pub mod operations;

// Re-export core types for ergonomic use
pub use core::{
    ResourceStream, AsyncResource, NodeRow, ResourceDaoError, ResourceQueryBuilder,
    ResourceCacheEntry, ResourceDaoConfig,
};

// Re-export streaming functions
pub use streaming::{
    resources_list_stream, stream_resources_by_type, stream_resources_by_tags,
    stream_resources_by_parent, stream_resources_with_search, stream_paginated_resources,
    stream_sorted_resources, stream_resources_advanced, stream_resources_custom_query,
    stream_resources_realtime, stream_resources_batched, stream_resources_with_retry,
};

// Re-export operations
pub use operations::{
    resource_read_async, find_by_slug, find_by_tags, ResourceDao, CacheStats,
};
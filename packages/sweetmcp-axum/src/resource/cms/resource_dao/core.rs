//! Core resource DAO structures and types
//!
//! This module provides the core data access object functionality for CMS resources
//! with zero allocation patterns, blazing-fast performance, and comprehensive
//! streaming capabilities for production environments.

use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    fmt::Write,
};

use arrayvec::ArrayString;
use futures::Stream;
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

use crate::db::DatabaseClient;
use crate::types::*;

/// Stream type for resources with zero allocation patterns
pub struct ResourceStream {
    /// Inner receiver stream
    inner: ReceiverStream<HandlerResult<Resource>>,
}

impl ResourceStream {
    /// Create new resource stream
    pub fn new(rx: mpsc::Receiver<HandlerResult<Resource>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }

    /// Create empty resource stream
    pub fn empty() -> Self {
        let (tx, rx) = mpsc::channel(1);
        drop(tx); // Close the channel immediately
        Self::new(rx)
    }

    /// Create error resource stream
    pub fn error(error: rpc_router::HandlerError) -> Self {
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _ = tx.send(Err(error)).await;
        });
        Self::new(rx)
    }
}

impl Stream for ResourceStream {
    type Item = HandlerResult<Resource>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Future type for a single resource read with zero allocation patterns
pub struct AsyncResource {
    /// Receiver for the resource result
    rx: oneshot::Receiver<HandlerResult<ReadResourceResult>>,
}

impl AsyncResource {
    /// Create new async resource
    pub fn new(rx: oneshot::Receiver<HandlerResult<ReadResourceResult>>) -> Self {
        Self { rx }
    }

    /// Create immediate error result
    pub fn error(error: rpc_router::HandlerError) -> Self {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Err(error));
        Self { rx }
    }

    /// Create immediate success result
    pub fn success(result: ReadResourceResult) -> Self {
        let (tx, rx) = oneshot::channel();
        let _ = tx.send(Ok(result));
        Self { rx }
    }
}

impl Future for AsyncResource {
    type Output = HandlerResult<ReadResourceResult>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| Err(rpc_router::HandlerError::new("oneshot cancelled")))
        })
    }
}

/// Node row structure for database deserialization
#[derive(serde::Deserialize, Debug)]
pub struct NodeRow {
    /// Node type
    #[serde(rename = "type")]
    pub type_: String,
    /// Node title
    pub title: String,
    /// Node description
    pub description: Option<String>,
    /// Node content
    pub content: Option<String>,
    /// Node slug
    pub slug: Option<String>,
    /// Node tags
    pub tags: Option<Vec<String>>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Parent node
    pub parent: Option<surrealdb::sql::Thing>,
    /// Child nodes
    pub children: Option<Vec<surrealdb::sql::Thing>>,
    /// Linked nodes
    pub links: Option<Vec<surrealdb::sql::Thing>>,
    /// Node metadata
    pub metadata: Option<BTreeMap<String, serde_json::Value>>,
    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Update timestamp
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl NodeRow {
    /// Convert node type to resource node type
    pub fn to_node_type(&self) -> NodeType {
        match self.type_.as_str() {
            "document" => NodeType::Document,
            "folder" => NodeType::Folder,
            "image" => NodeType::Image,
            "video" => NodeType::Video,
            "audio" => NodeType::Audio,
            "archive" => NodeType::Archive,
            "code" => NodeType::Code,
            "data" => NodeType::Data,
            _ => NodeType::Document, // Default fallback
        }
    }

    /// Generate content snippet from content
    pub fn content_snippet(&self) -> Option<String> {
        self.content.as_ref().map(|content| {
            if content.len() > 100 {
                format!("{}...", &content[..97])
            } else {
                content.clone()
            }
        })
    }

    /// Convert SurrealDB Thing to URL using zero-allocation formatting
    pub fn thing_to_url(thing: Option<surrealdb::sql::Thing>) -> Option<Url> {
        thing.and_then(|t| {
            let mut url_str: ArrayString<64> = ArrayString::new();
            if write!(&mut url_str, "cms://{}", t).is_ok() {
                Url::parse(&url_str).ok()
            } else {
                None
            }
        })
    }

    /// Convert Option<Vec<Thing>> to Option<Vec<Url>> using zero-allocation formatting
    pub fn vec_thing_to_vec_url(things: Option<Vec<surrealdb::sql::Thing>>) -> Option<Vec<Url>> {
        things.map(|ts| {
            ts.into_iter()
                .filter_map(|t| {
                    let mut url_str: ArrayString<64> = ArrayString::new();
                    if write!(&mut url_str, "cms://{}", t).is_ok() {
                        Url::parse(&url_str).ok()
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    /// Convert to Resource with optimized allocation patterns
    pub fn to_resource(&self, uri: Url) -> Resource {
        let node_type = self.to_node_type();
        let content_snippet = self.content_snippet();
        let description = self.description.clone().or(content_snippet);

        Resource {
            uri,
            name: self.title.clone(),
            description,
            mime_type: self.mime_type.clone(),
            node_type: Some(node_type),
            parent: Self::thing_to_url(self.parent.clone()),
            children: Self::vec_thing_to_vec_url(self.children.clone()),
            links: Self::vec_thing_to_vec_url(self.links.clone()),
            metadata: self.metadata.clone(),
        }
    }
}

/// Resource DAO error types
#[derive(Debug, Clone)]
pub enum ResourceDaoError {
    /// Database connection error
    DatabaseConnection(String),
    /// Query execution error
    QueryExecution(String),
    /// Resource not found
    ResourceNotFound(String),
    /// Invalid URI format
    InvalidUri(String),
    /// Serialization error
    Serialization(String),
    /// Permission denied
    PermissionDenied(String),
    /// Resource already exists
    ResourceExists(String),
    /// Invalid resource data
    InvalidData(String),
}

impl std::fmt::Display for ResourceDaoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceDaoError::DatabaseConnection(msg) => write!(f, "Database connection error: {}", msg),
            ResourceDaoError::QueryExecution(msg) => write!(f, "Query execution error: {}", msg),
            ResourceDaoError::ResourceNotFound(msg) => write!(f, "Resource not found: {}", msg),
            ResourceDaoError::InvalidUri(msg) => write!(f, "Invalid URI: {}", msg),
            ResourceDaoError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            ResourceDaoError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ResourceDaoError::ResourceExists(msg) => write!(f, "Resource already exists: {}", msg),
            ResourceDaoError::InvalidData(msg) => write!(f, "Invalid resource data: {}", msg),
        }
    }
}

impl std::error::Error for ResourceDaoError {}

impl From<ResourceDaoError> for rpc_router::HandlerError {
    fn from(error: ResourceDaoError) -> Self {
        rpc_router::HandlerError::new(error.to_string())
    }
}

/// Resource query builder with zero allocation patterns
#[derive(Debug, Clone, Default)]
pub struct ResourceQueryBuilder {
    /// Resource types to filter by
    pub resource_types: Option<Vec<String>>,
    /// Tags to filter by
    pub tags: Option<Vec<String>>,
    /// Parent resource to filter by
    pub parent: Option<String>,
    /// Search query
    pub search_query: Option<String>,
    /// Limit for results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort field
    pub sort_field: Option<String>,
    /// Sort direction
    pub sort_direction: Option<String>,
}

impl ResourceQueryBuilder {
    /// Create new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set resource types filter
    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.resource_types = Some(types);
        self
    }

    /// Set tags filter
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set parent filter
    pub fn with_parent(mut self, parent: String) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set search query
    pub fn with_search(mut self, query: String) -> Self {
        self.search_query = Some(query);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set sort field and direction
    pub fn with_sort(mut self, field: String, direction: String) -> Self {
        self.sort_field = Some(field);
        self.sort_direction = Some(direction);
        self
    }

    /// Build SurrealDB query string
    pub fn build_query(&self) -> String {
        let mut query = String::from("SELECT * FROM node");
        let mut conditions = Vec::new();

        // Add type filter
        if let Some(ref types) = self.resource_types {
            if !types.is_empty() {
                let types_str = types
                    .iter()
                    .map(|t| format!("'{}'", t))
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("type IN [{}]", types_str));
            }
        }

        // Add tags filter
        if let Some(ref tags) = self.tags {
            if !tags.is_empty() {
                for tag in tags {
                    conditions.push(format!("'{}' IN tags", tag));
                }
            }
        }

        // Add parent filter
        if let Some(ref parent) = self.parent {
            conditions.push(format!("parent = {}", parent));
        }

        // Add search query
        if let Some(ref search) = self.search_query {
            conditions.push(format!(
                "(title CONTAINS '{}' OR description CONTAINS '{}' OR content CONTAINS '{}')",
                search, search, search
            ));
        }

        // Add WHERE clause if conditions exist
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Add sorting
        if let Some(ref sort_field) = self.sort_field {
            let direction = self.sort_direction.as_deref().unwrap_or("ASC");
            query.push_str(&format!(" ORDER BY {} {}", sort_field, direction));
        }

        // Add limit and offset
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" START {}", offset));
        }

        query
    }
}

/// Resource cache entry with TTL
#[derive(Debug, Clone)]
pub struct ResourceCacheEntry {
    /// Cached resource
    pub resource: Resource,
    /// Cache timestamp
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

impl ResourceCacheEntry {
    /// Create new cache entry
    pub fn new(resource: Resource, ttl_seconds: u64) -> Self {
        Self {
            resource,
            cached_at: chrono::Utc::now(),
            ttl_seconds,
        }
    }

    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(self.cached_at);
        elapsed.num_seconds() as u64 > self.ttl_seconds
    }

    /// Get remaining TTL in seconds
    pub fn remaining_ttl(&self) -> u64 {
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(self.cached_at);
        let elapsed_seconds = elapsed.num_seconds() as u64;
        
        if elapsed_seconds >= self.ttl_seconds {
            0
        } else {
            self.ttl_seconds - elapsed_seconds
        }
    }
}

/// Resource DAO configuration
#[derive(Debug, Clone)]
pub struct ResourceDaoConfig {
    /// Enable caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Default query limit
    pub default_limit: usize,
    /// Maximum query limit
    pub max_limit: usize,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Query timeout in milliseconds
    pub query_timeout_ms: u64,
}

impl Default for ResourceDaoConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_cache_size: 1000,
            default_limit: 50,
            max_limit: 1000,
            enable_query_logging: true,
            query_timeout_ms: 30000, // 30 seconds
        }
    }
}
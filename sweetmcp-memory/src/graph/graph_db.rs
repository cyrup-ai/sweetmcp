//! Graph database abstraction

use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Result type for graph operations
pub type Result<T> = std::result::Result<T, GraphError>;

/// Graph database error types
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Edge not found: {0}")]
    EdgeNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Node ID type
pub type NodeId = String;

/// Node properties
pub type NodeProperties = HashMap<String, serde_json::Value>;

/// A pending node creation operation that can be awaited
pub struct PendingNode {
    rx: oneshot::Receiver<Result<NodeId>>,
}

impl PendingNode {
    pub fn new(rx: oneshot::Receiver<Result<NodeId>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingNode {
    type Output = Result<NodeId>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => {
                Poll::Ready(Err(GraphError::DatabaseError("Channel closed".into())))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending node query operation
pub struct NodeQuery {
    rx: oneshot::Receiver<Result<Option<Node>>>,
}

impl NodeQuery {
    pub fn new(rx: oneshot::Receiver<Result<Option<Node>>>) -> Self {
        Self { rx }
    }
}

impl Future for NodeQuery {
    type Output = Result<Option<Node>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => {
                Poll::Ready(Err(GraphError::DatabaseError("Channel closed".into())))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending node update operation
pub struct NodeUpdate {
    rx: oneshot::Receiver<Result<()>>,
}

impl NodeUpdate {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for NodeUpdate {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => {
                Poll::Ready(Err(GraphError::DatabaseError("Channel closed".into())))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A stream of nodes from a query
pub struct NodeStream {
    rx: tokio::sync::mpsc::Receiver<Result<Node>>,
}

impl NodeStream {
    pub fn new(rx: tokio::sync::mpsc::Receiver<Result<Node>>) -> Self {
        Self { rx }
    }
}

impl Stream for NodeStream {
    type Item = Result<Node>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// Graph database trait - no async methods
pub trait GraphDatabase: Send + Sync + 'static {
    /// Create a node
    fn create_node(&self, properties: NodeProperties) -> PendingNode;

    /// Get a node
    fn get_node(&self, id: &str) -> NodeQuery;

    /// Update a node
    fn update_node(&self, id: &str, properties: NodeProperties) -> NodeUpdate;

    /// Delete a node
    fn delete_node(&self, id: &str) -> NodeUpdate;

    /// Get nodes by type/label
    fn get_nodes_by_type(&self, node_type: &str) -> NodeStream;

    /// Execute a query
    fn query(&self, query: &str, params: Option<GraphQueryOptions>) -> NodeStream;
}

/// Graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node ID
    pub id: NodeId,

    /// Node properties
    pub properties: NodeProperties,

    /// Node labels
    pub labels: Vec<String>,
}

impl Node {
    /// Create a new node
    pub fn new(id: NodeId, label: &str) -> Self {
        Self {
            id,
            properties: HashMap::new(),
            labels: vec![label.to_string()],
        }
    }

    /// Add a property to the node
    pub fn with_property(mut self, key: &str, value: serde_json::Value) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }
}

/// Query options for graph operations
#[derive(Debug, Clone, Default)]
pub struct GraphQueryOptions {
    /// Maximum number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,

    /// Additional filters
    pub filters: HashMap<String, serde_json::Value>,
}

impl GraphQueryOptions {
    /// Create new query options
    pub fn new() -> Self {
        Self {
            limit: None,
            offset: None,
            filters: HashMap::new(),
        }
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

    /// Add filter
    pub fn with_filter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.filters.insert(key.to_string(), value);
        self
    }
}

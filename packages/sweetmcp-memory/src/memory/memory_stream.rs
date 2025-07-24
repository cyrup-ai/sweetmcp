//! Memory stream implementation for streaming memory nodes

use crate::memory::memory_node::MemoryNode;
use crate::utils::error::Result;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// A stream of memory nodes
pub struct MemoryStream {
    rx: mpsc::Receiver<Result<MemoryNode>>,
}

impl MemoryStream {
    /// Create a new MemoryStream from a receiver
    pub fn new(rx: mpsc::Receiver<Result<MemoryNode>>) -> Self {
        Self { rx }
    }
}

impl Stream for MemoryStream {
    type Item = Result<MemoryNode>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

/// A stream of memory relationships
pub struct RelationshipStream {
    rx: mpsc::Receiver<Result<crate::memory::memory_relationship::MemoryRelationship>>,
}

impl RelationshipStream {
    /// Create a new RelationshipStream from a receiver
    pub fn new(rx: mpsc::Receiver<Result<crate::memory::memory_relationship::MemoryRelationship>>) -> Self {
        Self { rx }
    }
}

impl Stream for RelationshipStream {
    type Item = Result<crate::memory::memory_relationship::MemoryRelationship>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
//! Pending operation types for asynchronous memory operations

use crate::memory::memory_node::MemoryNode;
use crate::memory::memory_relationship::MemoryRelationship;
use crate::utils::error::Result;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Represents a pending deletion operation
pub struct PendingDeletion {
    rx: oneshot::Receiver<Result<bool>>,
}

impl PendingDeletion {
    /// Create a new PendingDeletion from a receiver
    pub fn new(rx: oneshot::Receiver<Result<bool>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingDeletion {
    type Output = Result<bool>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::ChannelClosed)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Represents a pending memory node creation operation
pub struct PendingMemory {
    rx: oneshot::Receiver<Result<MemoryNode>>,
}

impl PendingMemory {
    /// Create a new PendingMemory from a receiver
    pub fn new(rx: oneshot::Receiver<Result<MemoryNode>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingMemory {
    type Output = Result<MemoryNode>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::ChannelClosed)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Represents a pending relationship creation operation
pub struct PendingRelationship {
    rx: oneshot::Receiver<Result<MemoryRelationship>>,
}

impl PendingRelationship {
    /// Create a new PendingRelationship from a receiver
    pub fn new(rx: oneshot::Receiver<Result<MemoryRelationship>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRelationship {
    type Output = Result<MemoryRelationship>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(crate::utils::error::Error::ChannelClosed)),
            Poll::Pending => Poll::Pending,
        }
    }
}
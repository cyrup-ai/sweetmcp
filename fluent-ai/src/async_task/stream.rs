//! Async stream with built-in error handling and collection support

use futures::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;

/// Generic async stream wrapper for streaming operations
pub struct AsyncStream<T> {
    receiver: mpsc::UnboundedReceiver<T>,
}

impl<T> AsyncStream<T> {
    /// Create a new AsyncStream from an unbounded receiver
    pub fn new(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self { receiver }
    }
    
    /// Collect all items from the stream into a Vec
    pub async fn collect(mut self) -> Vec<T> {
        let mut items = Vec::new();
        while let Some(item) = self.receiver.recv().await {
            items.push(item);
        }
        items
    }
}

impl<T> Stream for AsyncStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
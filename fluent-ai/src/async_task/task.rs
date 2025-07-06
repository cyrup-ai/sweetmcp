//! Shared async utilities for the Desktop Commander project
//!
//! This module provides reusable async primitives that follow the project's
//! conventions of returning concrete types instead of boxed futures or async fn.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Generic async task wrapper for single operations
///
/// This wraps a oneshot::Receiver and implements Future to provide
/// a concrete return type instead of boxed futures or async fn.
pub struct AsyncTask<T> {
    receiver: oneshot::Receiver<T>,
}

impl<T> AsyncTask<T> {
    /// Create a new AsyncTask from a oneshot receiver
    pub fn new(receiver: oneshot::Receiver<T>) -> Self {
        Self { receiver }
    }
}

impl<T> Future for AsyncTask<T> {
    type Output = Result<T, oneshot::error::RecvError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(Ok(result)),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

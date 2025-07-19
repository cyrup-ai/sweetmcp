use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream; // Removed StreamExt
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

// Restore full re-export
pub use super::super::types::{
    CallToolRequest, CallToolResult, CallToolResultContent, ListToolsRequest, ListToolsResult,
    Tool, ToolCallRequestParams, ToolInputSchema, ToolInputSchemaProperty,
};

/// Stream type for tools
pub struct ToolStream {
    inner: ReceiverStream<HandlerResult<Tool>>,
}

impl ToolStream {
    pub(crate) fn new(rx: mpsc::Receiver<HandlerResult<Tool>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl Stream for ToolStream {
    type Item = HandlerResult<Tool>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// A future for a single tool call execution.
/// This is the domain-specific async result for tool calls.
pub struct ToolCallExecution {
    pub rx: oneshot::Receiver<HandlerResult<CallToolResult>>,
}

impl Future for ToolCallExecution {
    type Output = HandlerResult<CallToolResult>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| Err(rpc_router::HandlerError::new("oneshot cancelled")))
        })
    }
}

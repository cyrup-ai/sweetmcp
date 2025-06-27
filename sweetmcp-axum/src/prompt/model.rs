use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::types::*;

// Stream type for prompts
pub struct PromptStream {
    inner: ReceiverStream<HandlerResult<Prompt>>,
}

impl PromptStream {
    pub(crate) fn new(rx: mpsc::Receiver<HandlerResult<Prompt>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }
}

// Implement Stream trait for PromptStream
impl Stream for PromptStream {
    type Item = HandlerResult<Prompt>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

// Future type for a single prompt get
pub struct PendingPromptResult {
    pub rx: oneshot::Receiver<HandlerResult<PromptResult>>,
}

impl Future for PendingPromptResult {
    type Output = HandlerResult<PromptResult>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| Err(rpc_router::HandlerError::new("oneshot cancelled")))
        })
    }
}

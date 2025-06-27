use futures::Stream;
use rpc_router::HandlerResult;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

// Import types from sibling modules
use super::rpc::{ContextItem, GetContextRequest};

// Stream type for context items (optional streaming API)
#[derive(Debug)] // Added Debug derive
pub struct ContextItemStream {
    inner: ReceiverStream<HandlerResult<ContextItem>>,
}

impl ContextItemStream {
    pub(crate) fn new(rx: mpsc::Receiver<HandlerResult<ContextItem>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl Stream for ContextItemStream {
    type Item = HandlerResult<ContextItem>;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

// Streaming version of context_get_context (optional)
pub fn context_get_context_stream(_request: GetContextRequest) -> ContextItemStream {
    let (_tx, rx) = mpsc::channel(16);

    // TODO: Implement actual context store search
    // tokio::spawn(async move {
    //     let store = CONTEXT_STORE.read().await;
    //     let scopes = request
    //         .scopes
    //         .unwrap_or_else(|| vec!["document".to_string(), "conversation".to_string()]);
    //     let max_results = request.max_results.unwrap_or(5);
    //     let items = store.search(&request.query, &scopes, max_results);
    //     for item in items {
    //         if tx.send(Ok(item)).await.is_err() {
    //             break;
    //         }
    //     }
    // });

    ContextItemStream::new(rx)
}

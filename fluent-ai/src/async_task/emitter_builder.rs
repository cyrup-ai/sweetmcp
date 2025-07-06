//! EmitterBuilder - builds AsyncStream with error handling

use super::AsyncStream;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;

/// Builder that emits AsyncStream after handling Result
pub struct EmitterBuilder<T> {
    inner: Box<dyn EmitterImpl<T>>,
}

/// Hidden trait for implementation
pub trait EmitterImpl<T>: Send {
    fn execute(
        self: Box<Self>
    ) -> Pin<Box<dyn Future<Output = Result<Vec<T>, Box<dyn std::error::Error + Send>>> + Send>>;
}

impl<T: Send + 'static> EmitterBuilder<T> {
    /// Create a new EmitterBuilder
    pub fn new(inner: Box<dyn EmitterImpl<T>>) -> Self {
        Self { inner }
    }
    
    /// Execute with error handling
    pub fn emit<FOk, FErr>(self, on_ok: FOk, on_err: FErr) -> AsyncStream<T>
    where
        FOk: FnOnce(Vec<T>) -> Vec<T> + Send + 'static,
        FErr: FnOnce(Box<dyn std::error::Error + Send>) + Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            match self.inner.execute().await {
                Ok(items) => {
                    for item in on_ok(items) {
                        if tx.send(item).is_err() {
                            break;
                        }
                    }
                }
                Err(e) => on_err(e),
            }
        });
        
        AsyncStream::new(rx)
    }
}

/// Macro for cleaner emit syntax
#[macro_export]
macro_rules! emit {
    ($stream:expr, Ok($items:ident) => $ok:expr, Err($e:ident) => $err:expr) => {
        $stream.emit(|$items| $ok, |$e| { $err })
    };
}
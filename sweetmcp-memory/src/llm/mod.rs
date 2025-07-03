//! LLM integration module for mem0-rs
//!
//! This module provides integration with various LLM providers for
//! memory enhancement, query processing, and natural language understanding.

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::oneshot;

// Re-export types
pub use self::anthropic::AnthropicProvider;
pub use self::openai::OpenAIProvider;

pub mod anthropic;
pub mod openai;
pub mod prompt_templates;

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LLMError>;

/// A pending LLM completion that can be awaited
pub struct PendingCompletion {
    rx: oneshot::Receiver<Result<String>>,
}

impl PendingCompletion {
    pub fn new(rx: oneshot::Receiver<Result<String>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingCompletion {
    type Output = Result<String>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(LLMError::ApiError("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// A pending embedding generation that can be awaited
pub struct PendingEmbedding {
    rx: oneshot::Receiver<Result<Vec<f32>>>,
}

impl PendingEmbedding {
    pub fn new(rx: oneshot::Receiver<Result<Vec<f32>>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingEmbedding {
    type Output = Result<Vec<f32>>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                std::task::Poll::Ready(Err(LLMError::ApiError("Channel closed".to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// LLM provider trait
pub trait LLMProvider: Send + Sync {
    /// Generate a completion for the given prompt
    fn complete(&self, prompt: &str) -> PendingCompletion;

    /// Generate embeddings for the given text
    fn embed(&self, text: &str) -> PendingEmbedding;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// LLM error types
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// LLM completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

/// LLM completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub usage: Usage,
    pub model: String,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

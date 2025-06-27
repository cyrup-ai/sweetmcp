use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream; // Keep Stream
use rpc_router::{HandlerResult, RpcParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

// Stream type for sampling (if you want streaming completions)
pub struct SamplingStream {
    inner: ReceiverStream<HandlerResult<CreateMessageResult>>,
}

impl SamplingStream {
    pub(crate) fn new(rx: mpsc::Receiver<HandlerResult<CreateMessageResult>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl Stream for SamplingStream {
    type Item = HandlerResult<CreateMessageResult>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// A future for a single sampling result.
/// This is the domain-specific async result for sampling.
pub struct AsyncSamplingResult {
    pub rx: oneshot::Receiver<HandlerResult<CompletionUsage>>,
}

// Restore unused impl
impl AsyncSamplingResult {
    pub fn new(rx: oneshot::Receiver<HandlerResult<CompletionUsage>>) -> Self {
        Self { rx }
    }
}

impl Future for AsyncSamplingResult {
    type Output = HandlerResult<CompletionUsage>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| Err(rpc_router::HandlerError::new("oneshot cancelled")))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub role: String,
    pub content: McpMessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessageContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpModelPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<McpModelHint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpModelHint {
    pub name: String,
}

/// Request to create a message using an LLM via the MCP sampling protocol
#[derive(Debug, Deserialize, Serialize, RpcParams, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    /// The conversation history to send to the LLM
    pub messages: Vec<McpMessage>,

    /// Optional system prompt to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Model selection preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<McpModelPreferences>,

    /// Which sources to include context from:
    /// "none", "thisServer", or "allServers"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_context: Option<String>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Controls randomness (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Array of sequences that stop generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Additional provider-specific parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Optional progress tracking
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<crate::types::MetaParams>,
}

/// Result of a sampling/createMessage request
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageResult {
    /// The role of the response (typically "assistant")
    pub role: String,

    /// The content of the message
    pub content: McpMessageContent,

    /// Name of the model used for generation
    pub model: String,

    /// Reason why generation stopped
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<CompletionUsage>,
}

/// Usage statistics for a completion
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionUsage {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

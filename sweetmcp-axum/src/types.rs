use std::collections::HashMap;

// Removed unused db imports
use futures::stream::Stream;
use rpc_router::RpcParams;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::JSONRPC_VERSION;

#[derive(Debug, Deserialize, Serialize, RpcParams, Clone)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: Implementation,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct PromptCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ResourceCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RootCapabilities {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

// --------- resource -------

#[derive(Debug, Deserialize, Serialize, RpcParams, Default, Clone)]
pub struct ListResourcesRequest {
    // Pagination
    pub cursor: Option<String>,
    // Limit number of results
    pub limit: Option<u32>,
    // Offset for pagination
    pub offset: Option<u32>,
    // Filter by category
    pub category: Option<String>,
    // Filter by tags (all tags must match)
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: Url,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Url>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Url>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct ReadResourceRequest {
    pub uri: Url,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaParams>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadResourceResult {
    pub content: ResourceContent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContent {
    pub uri: Url, // The URI of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>, // Optional MIME type
    pub text: Option<String>, // For text resources
    pub blob: Option<String>, // For binary resources (base64 encoded)
}

// --------- prompt -------
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<PromptMessage>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct ListPromptsRequest {
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct GetPromptRequest {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PromptResult {
    pub prompt: Prompt,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptMessageContent,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromptMessageContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

// --------- tool -------

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: ToolInputSchema,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_name: String,
    pub properties: HashMap<String, ToolInputSchemaProperty>,
    pub required: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ToolInputSchemaProperty {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, RpcParams, Debug, Clone)]
pub struct CallToolRequest {
    pub params: ToolCallRequestParams,
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaParams>,
}

#[derive(Deserialize, Serialize, Debug, Clone, RpcParams)]
pub struct ToolCallRequestParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

#[derive(Deserialize, Serialize, RpcParams, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    pub content: Vec<CallToolResultContent>,
    #[serde(default)] // This will default to false if missing
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CallToolResultContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct ListToolsRequest {
    pub cursor: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

// ----- misc ---
#[derive(Deserialize, Serialize)]
pub struct EmptyResult {}

#[derive(Deserialize, Serialize, RpcParams)]
pub struct PingRequest {}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
#[serde(rename_all = "camelCase")]
pub struct CancelledNotification {
    pub request_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetaParams {
    pub progress_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub progress_token: String,
    pub progress: i32,
    pub total: i32,
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct SetLevelRequest {
    pub level: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingResponse {}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct LoggingMessageNotification {
    pub level: String,
    pub logger: String,
    pub data: Value,
}

#[derive(Debug, Deserialize, Serialize, RpcParams)]
pub struct ListRootsRequest {}

#[derive(Debug, Deserialize, Serialize)]
pub struct ListRootsResult {
    pub roots: Vec<Root>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Root {
    pub name: String,
    pub url: String,
}

// JSON-RPC types are defined in router.rs to avoid duplication
// But JsonRpcNotification is still needed here
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

/// A concrete, reusable wrapper for async tasks that hides all async complexity from trait/public interfaces.
///
/// Use this instead of async fn when you are returning a single object and not a stream
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[doc = "A concrete, ergonomic async task wrapper for project-wide use. Implements Future."]
/// A concrete, ergonomic async task wrapper for project-wide use. Implements Future or Stream.
///
/// Use for returning either a single value (via Future) or multiple values (via mpsc Receiver/ReceiverStream) from async APIs without exposing raw futures or streams.
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub enum AsyncTask<T> {
    FutureVariant(Pin<Box<dyn Future<Output = T> + Send + 'static>>),
    StreamVariant(ReceiverStream<T>),
}

impl<T> AsyncTask<T> {
    /// Construct an AsyncTask from a Future (single result)
    pub fn from_future<F>(fut: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        AsyncTask::FutureVariant(Box::pin(fut))
    }

    /// Construct an AsyncTask from a mpsc Receiver (streaming/multi result)
    pub fn from_receiver(receiver: mpsc::Receiver<T>) -> Self {
        AsyncTask::StreamVariant(ReceiverStream::new(receiver))
    }
}

impl<T> Future for AsyncTask<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            AsyncTask::FutureVariant(fut) => fut.as_mut().poll(cx),
            AsyncTask::StreamVariant(_) => panic!("polled as Future, but is a stream variant"),
        }
    }
}

impl<T> Stream for AsyncTask<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            AsyncTask::StreamVariant(s) => unsafe { Pin::new_unchecked(s) }.poll_next(cx),
            AsyncTask::FutureVariant(_) => Poll::Ready(None),
        }
    }
}

pub struct AsyncStream<T> {
    inner: ReceiverStream<T>,
}

impl<T> AsyncStream<T> {
    pub fn new(receiver: mpsc::Receiver<T>) -> Self {
        AsyncStream {
            inner: ReceiverStream::new(receiver),
        }
    }
}

impl<T> Stream for AsyncStream<T> {
    type Item = T;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Safety: we're not moving the inner field
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }.poll_next(cx)
    }
}

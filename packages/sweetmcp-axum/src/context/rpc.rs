use log; // For logging in handlers
use rpc_router::{HandlerResult, RpcParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Use super to access items from parent or sibling modules if needed
// e.g., use super::store::CONTEXT_STORE;

// Import common types used in RPC definitions
use crate::types::MetaParams;

/// Request to retrieve context for an AI response generation
#[derive(Debug, Deserialize, Serialize, RpcParams, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetContextRequest {
    /// Query to search for relevant context
    pub query: String,

    /// Optional scopes to search in (e.g., "documents", "conversations", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,

    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,

    /// Optional progress tracking
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<MetaParams>,
}

/// Result of a context/getContext request
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetContextResult {
    /// List of context items
    pub items: Vec<ContextItem>,

    /// Optional cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// A single item of context
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextItem {
    /// Unique identifier for the context item
    pub id: String,

    /// Source of the context (e.g., "document", "conversation", etc.)
    pub source: String,

    /// Title or name of the context item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Content of the context item
    pub content: ContextContent,

    /// Metadata about the context item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Relevance score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<f32>,
}

/// Content of a context item
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextContent {
    /// Type of content (e.g., "text", "image", etc.)
    #[serde(rename = "type")]
    pub type_: String,

    /// Text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Binary content (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// MIME type of binary content
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Request to subscribe to context updates
#[derive(Debug, Deserialize, Serialize, RpcParams, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeContextRequest {
    /// Scopes to subscribe to
    pub scopes: Vec<String>,
}

/// Result of a context/subscribe request
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeContextResult {
    /// Subscription ID
    pub subscription_id: String,
}

/// Notification for context updates
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextChangedNotification {
    /// Subscription ID
    pub subscription_id: String,

    /// Scope that changed
    pub scope: String,

    /// Type of change (e.g., "added", "updated", "removed")
    pub change_type: String,

    /// Changed context items
    pub items: Vec<ContextItem>,
}

/// Handler for the context/get method
pub async fn context_get(request: GetContextRequest) -> HandlerResult<GetContextResult> {
    log::info!("Received context/get request: {:?}", request);

    // Get memory adapter from global application context
    if let Some(app_context) = crate::context::APPLICATION_CONTEXT.get() {
        let memory_adapter = app_context.memory_adapter();

        // Search for relevant context using the memory system
        match memory_adapter.search_contexts(&request.query).await {
            Ok(search_results) => {
                let mut items = Vec::new();
                let max_results = request.max_results.unwrap_or(10) as usize;

                for (key, value) in search_results.into_iter().take(max_results) {
                    let item = ContextItem {
                        id: key.clone(),
                        source: "memory".to_string(),
                        title: Some(key),
                        content: ContextContent {
                            type_: "text".to_string(),
                            text: Some(value.to_string()),
                            data: None,
                            mime_type: Some("application/json".to_string()),
                        },
                        metadata: Some(value),
                        relevance: None,
                    };
                    items.push(item);
                }

                let result = GetContextResult {
                    items,
                    next_cursor: None,
                };

                return Ok(result);
            }
            Err(e) => {
                log::error!("Error searching context: {}", e);
            }
        }
    }

    // Fallback to empty results if memory system unavailable
    let result = GetContextResult {
        items: vec![],
        next_cursor: None,
    };

    Ok(result)
}

/// Handler for the context/subscribe method
pub async fn context_subscribe(
    request: SubscribeContextRequest,
) -> HandlerResult<SubscribeContextResult> {
    log::info!("Received context/subscribe request: {:?}", request);

    let subscription_id = uuid::Uuid::new_v4().to_string();

    // Store subscription in memory system
    if let Some(app_context) = crate::context::APPLICATION_CONTEXT.get() {
        let memory_adapter = app_context.memory_adapter();

        // Store subscription metadata in memory system
        let subscription_data = serde_json::json!({
            "type": "context_subscription",
            "scopes": request.scopes,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "subscription_id": subscription_id
        });

        if let Err(e) = memory_adapter
            .store_context(
                format!("subscription:{}", subscription_id),
                subscription_data,
            )
            .await
        {
            log::error!("Failed to store subscription in memory system: {}", e);
        }

        // Also add to subscription tracking
        for scope in &request.scopes {
            if let Err(e) = memory_adapter.add_subscription(scope.clone()).await {
                log::error!("Failed to add subscription for scope {}: {}", scope, e);
            }
        }
    }

    log::info!("Created subscription with ID: {}", subscription_id);
    let result = SubscribeContextResult { subscription_id };
    Ok(result)
}

/// Send a context changed notification
pub async fn send_context_changed_notification(
    subscription_id: &str,
    scope: &str,
    change_type: &str,
    items: Vec<ContextItem>,
) -> HandlerResult<()> {
    // In a real implementation, we would send this notification to the client
    // For now, we just log it
    log::info!(
        "Would send context_changed notification: subscription={}, scope={}, change_type={}, items={}",
        subscription_id,
        scope,
        change_type,
        items.len()
    );

    Ok(())
}

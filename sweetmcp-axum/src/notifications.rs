use std::{collections::HashMap, sync::Arc};

use log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, mpsc};

// Import directly from context and type modules
use crate::context::ContextChangedNotification;
use crate::types::{CancelledNotification, JsonRpcNotification};

// Only global notification types/registry should remain here.
// Feature-specific notification types have moved to their respective modules.

/// Progress notification for long-running operations
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressNotification {
    /// Token identifying the operation
    pub progress_token: String,

    /// Current progress value
    pub progress: i32,

    /// Total expected progress
    pub total: i32,

    /// Optional message describing the current progress state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Notification registry to keep track of subscription channels
#[derive(Clone)]
pub struct NotificationRegistry {
    // Token -> Channel sender for progress notifications
    progress_channels: Arc<Mutex<HashMap<String, mpsc::Sender<ProgressNotification>>>>,

    // Subscription ID -> Channel sender for context changed notifications
    context_channels: Arc<Mutex<HashMap<String, mpsc::Sender<ContextChangedNotification>>>>,

    // Global notification sender for broadcasting JSON-RPC notifications
    json_rpc_sender: Arc<Mutex<Option<mpsc::Sender<NotificationPayload>>>>,
}

/// Notification payload for JSON-RPC
#[derive(Debug, Clone)]
pub struct NotificationPayload {
    /// Method name for the notification
    pub method: String,

    /// Parameters for the notification
    pub params: Value,
}

impl NotificationRegistry {
    pub fn new() -> Self {
        Self {
            progress_channels: Arc::new(Mutex::new(HashMap::new())),
            context_channels: Arc::new(Mutex::new(HashMap::new())),
            json_rpc_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Register a global JSON-RPC notification sender
    pub async fn set_json_rpc_sender(&self, sender: mpsc::Sender<NotificationPayload>) {
        let mut lock = self.json_rpc_sender.lock().await;
        *lock = Some(sender);
    }

    /// Register a progress notification channel
    pub async fn register_progress_channel(
        &self,
        token: String,
        sender: mpsc::Sender<ProgressNotification>,
    ) {
        let mut lock = self.progress_channels.lock().await;
        lock.insert(token, sender);
    }

    /// Unregister a progress notification channel
    pub async fn unregister_progress_channel(&self, token: &str) {
        let mut lock = self.progress_channels.lock().await;
        lock.remove(token);
    }

    /// Register a context changed notification channel
    pub async fn register_context_channel(
        &self,
        subscription_id: String,
        sender: mpsc::Sender<ContextChangedNotification>,
    ) {
        let mut lock = self.context_channels.lock().await;
        lock.insert(subscription_id, sender);
    }

    /// Unregister a context changed notification channel
    pub async fn unregister_context_channel(&self, subscription_id: &str) {
        let mut lock = self.context_channels.lock().await;
        lock.remove(subscription_id);
    }

    /// Send a progress notification
    pub async fn send_progress(&self, notification: ProgressNotification) -> bool {
        let lock = self.progress_channels.lock().await;

        if let Some(sender) = lock.get(&notification.progress_token) {
            let send_result = sender.send(notification.clone()).await;

            if send_result.is_err() {
                log::warn!("Failed to send progress notification: {:?}", send_result);
                return false;
            }

            // Also send as JSON-RPC notification
            self.send_json_rpc_notification(
                "$/progress",
                serde_json::to_value(notification).unwrap(),
            )
            .await;

            true
        } else {
            log::warn!(
                "No progress channel found for token: {}",
                notification.progress_token
            );
            false
        }
    }

    /// Send a context changed notification
    pub async fn send_context_changed(&self, notification: ContextChangedNotification) -> bool {
        let lock = self.context_channels.lock().await;

        if let Some(sender) = lock.get(&notification.subscription_id) {
            let send_result = sender.send(notification.clone()).await;

            if send_result.is_err() {
                log::warn!(
                    "Failed to send context changed notification: {:?}",
                    send_result
                );
                return false;
            }

            // Also send as JSON-RPC notification
            self.send_json_rpc_notification(
                "$/context/changed",
                serde_json::to_value(notification).unwrap(),
            )
            .await;

            true
        } else {
            log::warn!(
                "No context channel found for subscription: {}",
                notification.subscription_id
            );
            false
        }
    }

    /// Send a JSON-RPC notification
    pub async fn send_json_rpc_notification(&self, method: &str, params: Value) {
        let lock = self.json_rpc_sender.lock().await;

        if let Some(sender) = &*lock {
            let payload = NotificationPayload {
                method: method.to_string(),
                params,
            };

            if let Err(e) = sender.send(payload).await {
                log::error!("Failed to send JSON-RPC notification: {}", e);
            }
        } else {
            log::warn!("No JSON-RPC notification sender registered");
        }
    }

    /// Send a cancellation notification
    pub async fn send_cancelled(&self, request_id: &str, reason: Option<String>) {
        let notification = CancelledNotification {
            request_id: request_id.to_string(),
            reason,
        };

        self.send_json_rpc_notification("$/cancelled", serde_json::to_value(notification).unwrap())
            .await;
    }

    /// Send an initialized notification
    pub async fn send_initialized(&self) {
        self.send_json_rpc_notification("initialized", Value::Null)
            .await;
    }
}

// Lazy static instance of the notification registry
lazy_static::lazy_static! {
    pub static ref NOTIFICATION_REGISTRY: NotificationRegistry = NotificationRegistry::new();
}

/// Format a notification as a JSON-RPC notification message
pub fn format_notification(method: &str, params: Value) -> String {
    let notification = JsonRpcNotification {
        jsonrpc: crate::JSONRPC_VERSION.to_string(),
        method: method.to_string(),
        params,
    };

    // Handle potential serialization error
    serde_json::to_string(&notification) // Already fixed
        .unwrap_or_else(|e| {
            log::error!("Failed to format notification: {}", e);
            // Return a default error JSON string
            r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Failed to format notification"}}"#
                .to_string()
        })
}

/// Initialize the notification system with a JSON-RPC sender
pub async fn init_notification_system(json_rpc_sender: mpsc::Sender<NotificationPayload>) {
    NOTIFICATION_REGISTRY
        .set_json_rpc_sender(json_rpc_sender)
        .await;
}

/// Handler for initialized notification
pub fn handle_initialized_notification() {
    log::info!("Received initialized notification");
}

/// Handler for cancelled notification
pub fn handle_cancelled_notification(params: CancelledNotification) {
    log::info!(
        "Received cancelled notification: request_id={}, reason={:?}",
        params.request_id,
        params.reason
    );
}

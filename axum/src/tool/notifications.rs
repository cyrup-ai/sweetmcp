use log::info;
use serde::{Deserialize, Serialize};

use crate::types::CancelledNotification;

/// Tool-specific notification types and logic (e.g., progress, completion, errors).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolProgressNotification {
    pub tool_name: String,
    pub progress: u32,
    pub total: u32,
    pub message: Option<String>,
}

/// Handler for notifications/initialized notification
pub fn notifications_initialized() {
    info!("Client initialized notification received");
}

/// Handler for notifications/cancelled notification
pub fn notifications_cancelled(params: CancelledNotification) {
    info!("Request cancelled: id={}", params.request_id);
}

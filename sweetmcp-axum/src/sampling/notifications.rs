use serde::{Deserialize, Serialize};

/// Sampling-specific notification types and logic (e.g., streaming tokens, progress).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingProgressNotification {
    pub request_id: String,
    pub progress: u32,
    pub total: u32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingTokenNotification {
    pub request_id: String,
    pub token: String,
    pub index: u32,
}

// ...other sampling-specific notification types can be added here

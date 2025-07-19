//! Utility modules for the memory system

pub mod config;
pub mod error;

pub use error::{Error, Result};

use chrono::Utc;

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> u64 {
    Utc::now().timestamp_millis() as u64
}

/// Generate a new unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Convert timestamp to ISO8601 string
pub fn timestamp_to_iso8601(timestamp_ms: u64) -> String {
    let datetime = chrono::DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64)
        .unwrap_or_else(|| Utc::now());
    datetime.to_rfc3339()
}

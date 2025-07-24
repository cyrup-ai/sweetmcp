//! Transaction logging functionality extracted from transaction manager

use std::sync::Arc;
use tokio::sync::Mutex;

/// Transaction log entry
#[derive(Debug, Clone)]
pub struct TransactionLogEntry {
    /// Transaction ID
    pub transaction_id: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Action
    pub action: TransactionAction,
}

/// Transaction actions for logging
#[derive(Debug, Clone)]
pub enum TransactionAction {
    Begin,
    Commit,
    Rollback,
    Abort(String),
}

/// Transaction logger for audit trail integrity
pub struct TransactionLogger {
    /// Transaction log entries
    log_entries: Arc<Mutex<Vec<TransactionLogEntry>>>,
}

impl TransactionLogger {
    /// Create a new transaction logger
    pub fn new() -> Self {
        Self {
            log_entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Log a transaction action
    pub async fn log_action(&self, transaction_id: String, action: TransactionAction) {
        let entry = TransactionLogEntry {
            transaction_id,
            timestamp: chrono::Utc::now(),
            action,
        };

        self.log_entries.lock().await.push(entry);
    }

    /// Get all log entries (for audit purposes)
    pub async fn get_log_entries(&self) -> Vec<TransactionLogEntry> {
        self.log_entries.lock().await.clone()
    }

    /// Get log entries for a specific transaction
    pub async fn get_transaction_log(&self, transaction_id: &str) -> Vec<TransactionLogEntry> {
        self.log_entries
            .lock()
            .await
            .iter()
            .filter(|entry| entry.transaction_id == transaction_id)
            .cloned()
            .collect()
    }

    /// Get log entries within a time range
    pub async fn get_log_entries_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<TransactionLogEntry> {
        self.log_entries
            .lock()
            .await
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get the log reference for direct access
    pub fn log_reference(&self) -> Arc<Mutex<Vec<TransactionLogEntry>>> {
        self.log_entries.clone()
    }
}

impl Default for TransactionLogger {
    fn default() -> Self {
        Self::new()
    }
}
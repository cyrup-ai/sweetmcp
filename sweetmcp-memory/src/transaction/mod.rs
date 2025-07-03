//! Transaction management module for mem0-rs
//! 
//! This module provides ACID transaction support for memory operations,
//! ensuring data consistency and isolation.

pub mod transaction_manager;

#[cfg(test)]
pub mod tests;

// Re-export main types
pub use transaction_manager::*;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Transaction result type
pub type Result<T> = std::result::Result<T, TransactionError>;

/// Transaction error types
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Transaction aborted: {0}")]
    Aborted(String),
    
    #[error("Deadlock detected")]
    Deadlock,
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Transaction timeout")]
    Timeout,
    
    #[error("Invalid transaction state: {0}")]
    InvalidState(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Transaction isolation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    /// Read uncommitted - lowest isolation
    ReadUncommitted,
    /// Read committed - prevents dirty reads
    ReadCommitted,
    /// Repeatable read - prevents dirty and non-repeatable reads
    RepeatableRead,
    /// Serializable - highest isolation
    Serializable,
}

/// Transaction state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// Transaction is active
    Active,
    /// Transaction is being committed
    Committing,
    /// Transaction is committed
    Committed,
    /// Transaction is being rolled back
    RollingBack,
    /// Transaction is aborted
    Aborted,
}

/// A pending commit operation that can be awaited
pub struct PendingCommit {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingCommit {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingCommit {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(TransactionError::DatabaseError(
                "Commit task failed".to_string()
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A pending rollback operation that can be awaited
pub struct PendingRollback {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingRollback {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingRollback {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(TransactionError::RollbackFailed(
                "Rollback task failed".to_string()
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Transaction trait
pub trait Transaction: Send + Sync {
    /// Get transaction ID
    fn id(&self) -> String;
    
    /// Get current state
    fn state(&self) -> TransactionState;
    
    /// Get isolation level
    fn isolation_level(&self) -> IsolationLevel;
    
    /// Commit the transaction
    fn commit(self) -> PendingCommit;
    
    /// Rollback the transaction
    fn rollback(self) -> PendingRollback;
    
    /// Check if transaction is active
    fn is_active(&self) -> bool {
        self.state() == TransactionState::Active
    }
}

/// Transaction context
#[derive(Debug, Clone)]
pub struct TransactionContext {
    /// Transaction ID
    pub id: String,
    /// Isolation level
    pub isolation_level: IsolationLevel,
    /// Start timestamp
    pub started_at: std::time::Instant,
    /// Timeout duration
    pub timeout: Option<std::time::Duration>,
}
//! Transaction management implementation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, RwLock};

use crate::transaction::{
    IsolationLevel, PendingCommit, PendingRollback, Result, Transaction, TransactionContext,
    TransactionError, TransactionState,
};
use crate::transaction::lock_manager::{Lock, LockManager, LockType};
use crate::transaction::logging::{TransactionAction, TransactionLogger};
use crate::transaction::operations::Operation;

/// Transaction manager for coordinating transactions
pub struct TransactionManager {
    /// Active transactions
    active_transactions: Arc<RwLock<HashMap<String, Arc<Mutex<TransactionImpl>>>>>,

    /// Transaction logger
    logger: Arc<TransactionLogger>,

    /// Lock manager
    lock_manager: Arc<LockManager>,
}

/// Transaction implementation
pub struct TransactionImpl {
    /// Transaction context
    context: TransactionContext,

    /// Transaction state
    state: TransactionState,

    /// Operations performed in this transaction
    operations: Vec<Operation>,

    /// Locks held by this transaction
    locks: Vec<Lock>,
}


impl TransactionManager {
    /// Create a new transaction manager
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            logger: Arc::new(TransactionLogger::new()),
            lock_manager: Arc::new(LockManager::new()),
        }
    }

    /// Begin a new transaction
    pub async fn begin_transaction(
        &self,
        isolation_level: IsolationLevel,
        timeout: Option<std::time::Duration>,
    ) -> Result<Arc<Mutex<TransactionImpl>>> {
        let context = TransactionContext {
            id: uuid::Uuid::new_v4().to_string(),
            isolation_level,
            started_at: std::time::Instant::now(),
            timeout,
        };

        let transaction = Arc::new(Mutex::new(TransactionImpl {
            context: context.clone(),
            state: TransactionState::Active,
            operations: Vec::new(),
            locks: Vec::new(),
        }));

        // Add to active transactions
        self.active_transactions
            .write()
            .await
            .insert(context.id.clone(), transaction.clone());

        // Log transaction begin
        self.logger.log_action(context.id, TransactionAction::Begin).await;

        Ok(transaction)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: &str) -> Option<Arc<Mutex<TransactionImpl>>> {
        self.active_transactions.read().await.get(id).cloned()
    }

    /// Commit a transaction
    pub async fn commit_transaction(&self, id: String) -> Result<()> {
        let transaction = self.active_transactions.write().await.remove(&id).ok_or(
            TransactionError::InvalidState("Transaction not found".to_string()),
        )?;

        let mut tx = transaction.lock().await;

        // Check state
        if tx.state != TransactionState::Active {
            return Err(TransactionError::InvalidState(format!(
                "Cannot commit transaction in state {:?}",
                tx.state
            )));
        }

        // Change state
        tx.state = TransactionState::Committing;

        // Apply all operations (in a real implementation)
        // This is where we would persist changes to the database

        // Release all locks
        for lock in &tx.locks {
            self.lock_manager
                .release_lock(&lock.resource, id.clone())
                .await?;
        }

        // Mark as committed
        tx.state = TransactionState::Committed;

        // Log commit
        self.logger.log_action(id, TransactionAction::Commit).await;

        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback_transaction(&self, id: String) -> Result<()> {
        let transaction = self.active_transactions.write().await.remove(&id).ok_or(
            TransactionError::InvalidState("Transaction not found".to_string()),
        )?;

        let mut tx = transaction.lock().await;

        // Check state
        if tx.state != TransactionState::Active {
            return Err(TransactionError::InvalidState(format!(
                "Cannot rollback transaction in state {:?}",
                tx.state
            )));
        }

        // Change state
        tx.state = TransactionState::RollingBack;

        // Undo all operations (in a real implementation)
        // This is where we would revert changes

        // Release all locks
        for lock in &tx.locks {
            self.lock_manager
                .release_lock(&lock.resource, id.clone())
                .await?;
        }

        // Mark as aborted
        tx.state = TransactionState::Aborted;

        // Log rollback
        self.logger.log_action(id, TransactionAction::Rollback).await;

        Ok(())
    }

    /// Acquire a lock for a transaction
    pub async fn acquire_lock(
        &self,
        transaction_id: String,
        resource: String,
        lock_type: LockType,
    ) -> Result<()> {
        self.lock_manager
            .acquire_lock(resource.clone(), lock_type, transaction_id.clone())
            .await?;

        // Add to transaction's lock list
        if let Some(transaction) = self.get_transaction(&transaction_id).await {
            let mut tx = transaction.lock().await;
            tx.locks.push(Lock {
                resource,
                lock_type,
                transaction_id: transaction_id.clone(),
            });
        }

        Ok(())
    }

}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}


impl Transaction for TransactionImpl {
    fn id(&self) -> String {
        self.context.id.clone()
    }

    fn state(&self) -> TransactionState {
        self.state
    }

    fn isolation_level(&self) -> IsolationLevel {
        self.context.isolation_level
    }

    fn commit(self) -> PendingCommit {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // This would be handled by the TransactionManager
            let _ = tx.send(Ok(()));
        });

        PendingCommit::new(rx)
    }

    fn rollback(self) -> PendingRollback {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // This would be handled by the TransactionManager
            let _ = tx.send(Ok(()));
        });

        PendingRollback::new(rx)
    }
}

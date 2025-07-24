//! Lock management logic extracted from transaction manager

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::transaction::{Result, TransactionError};

/// Lock types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockType {
    /// Shared lock (for reads)
    Shared,

    /// Exclusive lock (for writes)
    Exclusive,
}

/// Lock held by a transaction
#[derive(Debug, Clone)]
pub struct Lock {
    /// Resource identifier
    pub resource: String,

    /// Lock type
    pub lock_type: LockType,

    /// Transaction holding the lock
    pub transaction_id: String,
}

/// Lock manager for handling concurrent access
pub struct LockManager {
    /// Locks by resource
    locks: RwLock<HashMap<String, Vec<Lock>>>,
}

impl LockManager {
    /// Create a new lock manager
    pub fn new() -> Self {
        Self {
            locks: RwLock::new(HashMap::new()),
        }
    }

    /// Acquire a lock
    pub async fn acquire_lock(
        &self,
        resource: String,
        lock_type: LockType,
        transaction_id: String,
    ) -> Result<()> {
        let mut locks = self.locks.write().await;
        let resource_locks = locks.entry(resource.clone()).or_insert_with(Vec::new);

        // Check for conflicts
        for existing_lock in resource_locks.iter() {
            if existing_lock.transaction_id != transaction_id {
                match (existing_lock.lock_type, lock_type) {
                    (LockType::Exclusive, _) | (_, LockType::Exclusive) => {
                        return Err(TransactionError::Conflict(format!(
                            "Resource {} is locked",
                            resource
                        )));
                    }
                    _ => {} // Shared locks are compatible
                }
            }
        }

        // Add the lock
        resource_locks.push(Lock {
            resource,
            lock_type,
            transaction_id,
        });

        Ok(())
    }

    /// Release a lock
    pub async fn release_lock(&self, resource: &str, transaction_id: String) -> Result<()> {
        let mut locks = self.locks.write().await;

        if let Some(resource_locks) = locks.get_mut(resource) {
            resource_locks.retain(|lock| lock.transaction_id != transaction_id);

            // Remove empty entries
            if resource_locks.is_empty() {
                locks.remove(resource);
            }
        }

        Ok(())
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}
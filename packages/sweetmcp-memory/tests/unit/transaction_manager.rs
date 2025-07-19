//! Unit tests for transaction manager

use sweetmcp_memory::transaction::*;

#[tokio::test]
async fn test_transaction_lifecycle() {
    let manager = TransactionManager::new();
    
    // Begin transaction
    let tx = manager
        .begin_transaction(IsolationLevel::ReadCommitted, None)
        .await
        .expect("Failed to begin transaction");
    
    let tx_impl = tx.lock().await;
    let tx_id = tx_impl.id();
    assert_eq!(tx_impl.state(), TransactionState::Active);
    drop(tx_impl);
    
    // Commit transaction
    manager.commit_transaction(tx_id).await.expect("Failed to commit transaction");
    
    // Transaction should no longer be active
    assert!(manager.get_transaction(&tx_id).await.is_none());
}

#[tokio::test]
async fn test_transaction_rollback() {
    let manager = TransactionManager::new();
    
    // Begin transaction
    let tx = manager
        .begin_transaction(IsolationLevel::ReadCommitted, None)
        .await
        .expect("Failed to begin transaction");
    
    let tx_impl = tx.lock().await;
    let tx_id = tx_impl.id();
    drop(tx_impl);
    
    // Rollback transaction
    manager.rollback_transaction(tx_id).await.expect("Failed to rollback transaction");
    
    // Transaction should no longer be active
    assert!(manager.get_transaction(&tx_id).await.is_none());
}
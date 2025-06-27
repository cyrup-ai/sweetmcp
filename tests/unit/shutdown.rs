use sweetmcp::shutdown::*;
use std::sync::Arc;
use tempfile::tempdir;
use std::sync::atomic::Ordering;

#[tokio::test]
async fn test_shutdown_coordinator() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let coordinator = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    // Should not be shutting down initially
    assert!(!coordinator.is_shutting_down());

    // Start some requests
    let _guard1 = coordinator.request_start();
    let _guard2 = coordinator.request_start();
    assert_eq!(coordinator.active_request_count(), 2);

    // Drop one guard
    drop(_guard1);
    assert_eq!(coordinator.active_request_count(), 1);
}

#[tokio::test]
async fn test_state_persistence() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let coordinator = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    // Update state
    coordinator.update_state(|state| {
        state.peers = vec!["peer1:8080".to_string(), "peer2:8080".to_string()];
    }).await;

    // Save state
    coordinator.save_state().await.expect("Failed to save state");

    // Create new coordinator and load state
    let coordinator2 = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    let loaded_state = coordinator2.load_state().await.expect("Failed to load state").expect("State should exist");
    assert_eq!(loaded_state.peers.len(), 2);
    assert_eq!(loaded_state.build_id, sweetmcp::peer_discovery::BUILD_ID);
}

#[tokio::test]
async fn test_request_guard() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let coordinator = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    {
        let _guard = coordinator.request_start();
        assert_eq!(coordinator.active_request_count(), 1);
    }
    // Guard dropped, count should be 0
    assert_eq!(coordinator.active_request_count(), 0);
}

#[tokio::test]
async fn test_shutdown_blocks_new_requests() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let coordinator = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    // Initiate shutdown
    coordinator.shutting_down.store(true, Ordering::SeqCst);

    // New requests should not be tracked
    let guard = coordinator.request_start();
    assert!(!guard.active);
    assert_eq!(coordinator.active_request_count(), 0);
}

#[tokio::test]
async fn test_shutdown_signal_subscription() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let coordinator = Arc::new(ShutdownCoordinator::new(
        temp_dir.path().to_path_buf()
    ));

    let mut receiver = coordinator.subscribe();

    // Send shutdown signal
    let _ = coordinator.shutdown_tx.send(());

    // Should receive signal
    assert!(receiver.recv().await.is_ok());
}
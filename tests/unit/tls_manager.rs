//! Unit tests for TLS manager module
//! 
//! Tests for production-grade TLS and mTLS configuration functionality

use sweetmcp::tls::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_tls_manager_creation() {
    let temp_dir = tempdir().unwrap();
    let manager = TlsManager::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Verify files were created
    assert!(temp_dir.path().join("ca.crt").exists());
    assert!(temp_dir.path().join("ca.key").exists());
    assert!(temp_dir.path().join("server.crt").exists());
    assert!(temp_dir.path().join("server.key").exists());
}

#[tokio::test] 
async fn test_server_client_configs() {
    let temp_dir = tempdir().unwrap();
    let manager = TlsManager::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Should create valid configs
    let _server_config = manager.server_config().unwrap();
    let _client_config = manager.client_config().unwrap();
}
//! Unit tests for TLS manager module
//! 
//! Tests for production-grade TLS and mTLS configuration functionality

use sweetmcp::tls::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_tls_manager_creation() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let manager = TlsManager::new(temp_dir.path().to_path_buf()).await.expect("Failed to create TlsManager");
    
    // Verify files were created
    assert!(temp_dir.path().join("ca.crt").exists(), "CA certificate file was not created");
    assert!(temp_dir.path().join("ca.key").exists(), "CA private key file was not created");
    assert!(temp_dir.path().join("server.crt").exists(), "Server certificate file was not created");
    assert!(temp_dir.path().join("server.key").exists(), "Server private key file was not created");
}

#[tokio::test] 
async fn test_server_client_configs() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let manager = TlsManager::new(temp_dir.path().to_path_buf()).await.expect("Failed to create TlsManager");
    
    // Should create valid configs
    let _server_config = manager.server_config().expect("Failed to create server config");
    let _client_config = manager.client_config().expect("Failed to create client config");
}
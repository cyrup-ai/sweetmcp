//! Unit tests for crypto module

use anyhow::Result;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sweetmcp::crypto::*;

const TOKEN_VALIDITY_HOURS: u64 = 48;

#[tokio::test]
async fn test_token_encryption_decryption() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TokenManager::new()?);
    
    let original_token = "test-discovery-token-12345";
    
    // Encrypt
    let encrypted = manager.encrypt_token(original_token).await?;
    assert!(!encrypted.ciphertext.is_empty());
    assert!(!encrypted.key_id.is_empty());
    
    // Decrypt
    let decrypted = manager.decrypt_token(&encrypted).await?;
    assert_eq!(decrypted, original_token);
    
    Ok(())
}

#[tokio::test]
async fn test_token_rotation() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TokenManager::new()?);
    
    // Get initial key ID
    let initial_key_id = {
        let keypair = manager.current_keypair.read().await;
        keypair.key_id.clone()
    };
    
    // Rotate
    manager.rotate_keypair().await?;
    
    // Verify key changed
    let new_key_id = {
        let keypair = manager.current_keypair.read().await;
        keypair.key_id.clone()
    };
    
    assert_ne!(initial_key_id, new_key_id);
    
    // Verify previous key is stored
    let previous = manager.previous_keypair.read().await;
    assert!(previous.is_some());
    assert_eq!(previous.as_ref().expect("Previous keypair should exist").key_id, initial_key_id);
    
    Ok(())
}

#[tokio::test]
async fn test_token_revocation() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TokenManager::new()?);
    
    let token = "revocable-token";
    let encrypted = manager.encrypt_token(token).await?;
    
    // Should decrypt successfully
    let decrypted = manager.decrypt_token(&encrypted).await?;
    assert_eq!(decrypted, token);
    
    // Extract token data to get the nonce
    let token_data = manager.extract_token_data(&encrypted).await?;
    
    // Revoke the token
    manager.revoke_token(token_data.nonce).await;
    
    // Should fail to decrypt now
    let result = manager.decrypt_token(&encrypted).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_expired_token() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TokenManager::new()?);
    
    let token = "test-token";
    let mut encrypted = manager.encrypt_token(token).await?;
    
    // Set created_at to past
    encrypted.created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() - (TOKEN_VALIDITY_HOURS + 1) * 3600;
    
    // Should fail due to expiry
    let result = manager.decrypt_token(&encrypted).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("expired"));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_secure_token_wrapper() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(TokenManager::new()?);
    
    let mut token1 = SecureDiscoveryToken::new(manager.clone());
    token1.set("my-secret-token".to_string());
    
    // Encrypt
    let encrypted_str = token1.to_encrypted().await?;
    
    // Decrypt with new token instance
    let mut token2 = SecureDiscoveryToken::new(manager);
    token2.from_encrypted(&encrypted_str).await?;
    
    assert_eq!(token2.get_raw(), Some("my-secret-token"));
    
    Ok(())
}
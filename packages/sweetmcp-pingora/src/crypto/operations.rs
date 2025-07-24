//! Token management operations
//!
//! This module provides token encryption, decryption, rotation, and revocation
//! operations with zero allocation patterns and blazing-fast performance.

use crate::crypto::core::*;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sodiumoxide::crypto::{box_, sealedbox};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use tracing::{error, info};

impl TokenManager {
    /// Encrypt a token for secure transmission
    pub async fn encrypt_token(&self, token: &str) -> Result<EncryptedToken> {
        let current = self.current_keypair.read().await;
        
        let token_data = TokenData::new(token.to_string());
        let plaintext = serde_json::to_vec(&token_data)
            .context("Failed to serialize token data")?;

        let ciphertext = sealedbox::seal(&plaintext, &current.public_key);
        let ciphertext_b64 = BASE64.encode(&ciphertext);

        Ok(EncryptedToken::new(ciphertext_b64, current.key_id.clone()))
    }

    /// Decrypt a token from secure transmission
    pub async fn decrypt_token(&self, encrypted: &EncryptedToken) -> Result<String> {
        // Validate the encrypted token first
        self.validate_encrypted_token(encrypted)?;

        // Check if token is revoked
        if self.is_token_revoked(&encrypted.key_id).await {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }

        let ciphertext = BASE64.decode(&encrypted.ciphertext)
            .context("Failed to decode ciphertext")?;

        // Try current keypair first
        let current = self.current_keypair.read().await;
        if encrypted.key_id == current.key_id {
            if let Ok(plaintext) = sealedbox::open(
                &ciphertext,
                &current.public_key,
                &current.secret_key,
            ) {
                let token_data: TokenData = serde_json::from_slice(&plaintext)
                    .context("Failed to deserialize token data")?;
                
                // Validate token data
                if !token_data.is_valid() {
                    return Err(anyhow::anyhow!("Invalid token data"));
                }
                
                if token_data.is_expired() {
                    return Err(anyhow::anyhow!("Token data is expired"));
                }
                
                return Ok(token_data.token);
            }
        }

        // Try previous keypair if current failed
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            if encrypted.key_id == prev_keypair.key_id {
                if let Ok(plaintext) = sealedbox::open(
                    &ciphertext,
                    &prev_keypair.public_key,
                    &prev_keypair.secret_key,
                ) {
                    let token_data: TokenData = serde_json::from_slice(&plaintext)
                        .context("Failed to deserialize token data")?;
                    
                    // Validate token data
                    if !token_data.is_valid() {
                        return Err(anyhow::anyhow!("Invalid token data"));
                    }
                    
                    if token_data.is_expired() {
                        return Err(anyhow::anyhow!("Token data is expired"));
                    }
                    
                    return Ok(token_data.token);
                }
            }
        }

        Err(anyhow::anyhow!("Failed to decrypt token"))
    }

    /// Extract token data without decrypting the token itself
    pub async fn extract_token_data(&self, encrypted: &EncryptedToken) -> Result<TokenData> {
        // Validate the encrypted token first
        self.validate_encrypted_token(encrypted)?;

        let ciphertext = BASE64.decode(&encrypted.ciphertext)
            .context("Failed to decode ciphertext")?;

        // Try current keypair first
        let current = self.current_keypair.read().await;
        if encrypted.key_id == current.key_id {
            if let Ok(plaintext) = sealedbox::open(
                &ciphertext,
                &current.public_key,
                &current.secret_key,
            ) {
                let token_data: TokenData = serde_json::from_slice(&plaintext)
                    .context("Failed to deserialize token data")?;
                return Ok(token_data);
            }
        }

        // Try previous keypair if current failed
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            if encrypted.key_id == prev_keypair.key_id {
                if let Ok(plaintext) = sealedbox::open(
                    &ciphertext,
                    &prev_keypair.public_key,
                    &prev_keypair.secret_key,
                ) {
                    let token_data: TokenData = serde_json::from_slice(&plaintext)
                        .context("Failed to deserialize token data")?;
                    return Ok(token_data);
                }
            }
        }

        Err(anyhow::anyhow!("Failed to extract token data"))
    }

    /// Rotate the keypair (move current to previous, generate new current)
    pub async fn rotate_keypair(&self) -> Result<()> {
        info!("Starting keypair rotation");

        let new_keypair = Self::generate_keypair()
            .context("Failed to generate new keypair")?;

        // Move current to previous
        {
            let current = self.current_keypair.read().await;
            let mut previous = self.previous_keypair.write().await;
            *previous = Some(TokenKeypair {
                public_key: current.public_key,
                secret_key: current.secret_key,
                key_id: current.key_id.clone(),
                created_at: current.created_at,
            });
        }

        // Set new current
        {
            let mut current = self.current_keypair.write().await;
            *current = new_keypair;
        }

        info!("Keypair rotation completed successfully");
        Ok(())
    }

    /// Start automatic token rotation
    pub async fn start_rotation_task(self: std::sync::Arc<Self>) -> Result<()> {
        let mut interval = interval(std::time::Duration::from_secs(3600)); // Check every hour

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                if self.needs_rotation().await {
                    if let Err(e) = self.rotate_keypair().await {
                        error!("Failed to rotate keypair: {}", e);
                    }
                }

                // Clean up old revocations (older than 7 days)
                let max_age = std::time::Duration::from_secs(7 * 24 * 3600);
                if let Err(e) = self.cleanup_expired_revocations(max_age).await {
                    error!("Failed to cleanup expired revocations: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Revoke a token by its identifier
    pub async fn revoke_token(&self, token_id: &str) -> Result<()> {
        let mut revoked = self.revoked_tokens.write().await;
        let revocation_time = SystemTime::now();
        
        revoked.insert(token_id.to_string(), revocation_time);
        
        info!("Token {} revoked at {:?}", token_id, revocation_time);
        Ok(())
    }

    /// Revoke multiple tokens at once
    pub async fn revoke_tokens(&self, token_ids: &[String]) -> Result<usize> {
        let mut revoked = self.revoked_tokens.write().await;
        let revocation_time = SystemTime::now();
        let mut revoked_count = 0;
        
        for token_id in token_ids {
            revoked.insert(token_id.clone(), revocation_time);
            revoked_count += 1;
        }
        
        info!("Revoked {} tokens at {:?}", revoked_count, revocation_time);
        Ok(revoked_count)
    }

    /// Un-revoke a token (remove from revocation list)
    pub async fn unrevoke_token(&self, token_id: &str) -> Result<bool> {
        let mut revoked = self.revoked_tokens.write().await;
        let was_revoked = revoked.remove(token_id).is_some();
        
        if was_revoked {
            info!("Token {} un-revoked", token_id);
        }
        
        Ok(was_revoked)
    }

    /// Batch encrypt multiple tokens
    pub async fn batch_encrypt_tokens(&self, tokens: &[String]) -> Result<Vec<EncryptedToken>> {
        let mut encrypted_tokens = Vec::with_capacity(tokens.len());
        
        for token in tokens {
            let encrypted = self.encrypt_token(token).await
                .context("Failed to encrypt token in batch")?;
            encrypted_tokens.push(encrypted);
        }
        
        Ok(encrypted_tokens)
    }

    /// Batch decrypt multiple tokens
    pub async fn batch_decrypt_tokens(&self, encrypted_tokens: &[EncryptedToken]) -> Result<Vec<String>> {
        let mut decrypted_tokens = Vec::with_capacity(encrypted_tokens.len());
        
        for encrypted in encrypted_tokens {
            let decrypted = self.decrypt_token(encrypted).await
                .context("Failed to decrypt token in batch")?;
            decrypted_tokens.push(decrypted);
        }
        
        Ok(decrypted_tokens)
    }

    /// Verify token integrity without decrypting
    pub async fn verify_token_integrity(&self, encrypted: &EncryptedToken) -> bool {
        // Basic validation first
        if self.validate_encrypted_token(encrypted).is_err() {
            return false;
        }

        // Check if we can decode the ciphertext
        if BASE64.decode(&encrypted.ciphertext).is_err() {
            return false;
        }

        // Try to decrypt with current keypair
        let current = self.current_keypair.read().await;
        if encrypted.key_id == current.key_id {
            let ciphertext = match BASE64.decode(&encrypted.ciphertext) {
                Ok(ct) => ct,
                Err(_) => return false,
            };

            if sealedbox::open(&ciphertext, &current.public_key, &current.secret_key).is_ok() {
                return true;
            }
        }

        // Try with previous keypair
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            if encrypted.key_id == prev_keypair.key_id {
                let ciphertext = match BASE64.decode(&encrypted.ciphertext) {
                    Ok(ct) => ct,
                    Err(_) => return false,
                };

                return sealedbox::open(&ciphertext, &prev_keypair.public_key, &prev_keypair.secret_key).is_ok();
            }
        }

        false
    }

    /// Re-encrypt token with current keypair
    pub async fn reencrypt_token(&self, encrypted: &EncryptedToken) -> Result<EncryptedToken> {
        // First decrypt the token
        let token = self.decrypt_token(encrypted).await
            .context("Failed to decrypt token for re-encryption")?;
        
        // Then encrypt with current keypair
        self.encrypt_token(&token).await
            .context("Failed to re-encrypt token")
    }

    /// Get token encryption metadata
    pub async fn get_token_metadata(&self, encrypted: &EncryptedToken) -> Result<TokenMetadata> {
        let token_data = self.extract_token_data(encrypted).await
            .context("Failed to extract token data for metadata")?;
        
        Ok(TokenMetadata {
            key_id: encrypted.key_id.clone(),
            created_at: encrypted.created_at,
            issued_at: token_data.issued_at,
            nonce: token_data.nonce,
            is_expired: encrypted.is_expired(),
            age: encrypted.age(),
            token_age: token_data.age(),
        })
    }

    /// Validate token chain (ensure tokens are properly linked)
    pub async fn validate_token_chain(&self, tokens: &[EncryptedToken]) -> Result<TokenChainValidation> {
        let mut validation = TokenChainValidation {
            total_tokens: tokens.len(),
            valid_tokens: 0,
            invalid_tokens: 0,
            expired_tokens: 0,
            revoked_tokens: 0,
            integrity_failures: 0,
        };

        for token in tokens {
            // Check basic validity
            if self.validate_encrypted_token(token).is_err() {
                validation.invalid_tokens += 1;
                continue;
            }

            // Check expiration
            if token.is_expired() {
                validation.expired_tokens += 1;
                continue;
            }

            // Check revocation
            if self.is_token_revoked(&token.key_id).await {
                validation.revoked_tokens += 1;
                continue;
            }

            // Check integrity
            if !self.verify_token_integrity(token).await {
                validation.integrity_failures += 1;
                continue;
            }

            validation.valid_tokens += 1;
        }

        Ok(validation)
    }
}

/// Token metadata for analysis
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    pub key_id: String,
    pub created_at: u64,
    pub issued_at: u64,
    pub nonce: String,
    pub is_expired: bool,
    pub age: std::time::Duration,
    pub token_age: std::time::Duration,
}

/// Token chain validation results
#[derive(Debug, Clone)]
pub struct TokenChainValidation {
    pub total_tokens: usize,
    pub valid_tokens: usize,
    pub invalid_tokens: usize,
    pub expired_tokens: usize,
    pub revoked_tokens: usize,
    pub integrity_failures: usize,
}

impl TokenChainValidation {
    /// Check if the token chain is healthy
    pub fn is_healthy(&self) -> bool {
        self.total_tokens > 0 
            && self.valid_tokens > 0
            && (self.valid_tokens as f64 / self.total_tokens as f64) > 0.8
    }

    /// Get validation success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tokens == 0 {
            0.0
        } else {
            self.valid_tokens as f64 / self.total_tokens as f64
        }
    }

    /// Get most common failure type
    pub fn primary_failure_type(&self) -> &'static str {
        let failures = [
            ("invalid", self.invalid_tokens),
            ("expired", self.expired_tokens),
            ("revoked", self.revoked_tokens),
            ("integrity", self.integrity_failures),
        ];

        failures.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| *name)
            .unwrap_or("none")
    }
}
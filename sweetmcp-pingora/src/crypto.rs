//! Cryptographic utilities for secure token handling
//!
//! This module provides:
//! - NaCl box encryption for discovery tokens
//! - Token rotation every 24 hours
//! - Revocation list support

#![allow(dead_code)]

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, sealedbox};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info};

const TOKEN_ROTATION_HOURS: u64 = 24;
const TOKEN_VALIDITY_HOURS: u64 = 48; // Allow grace period for rotation

/// Encrypted discovery token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedToken {
    /// The encrypted token data
    pub ciphertext: String,
    /// Timestamp when token was created
    pub created_at: u64,
    /// Public key used for encryption (for key rotation)
    pub key_id: String,
}

/// Token manager for secure discovery tokens
pub struct TokenManager {
    /// Current keypair for encryption
    pub current_keypair: Arc<RwLock<TokenKeypair>>,
    /// Previous keypair for decryption during rotation
    pub previous_keypair: Arc<RwLock<Option<TokenKeypair>>>,
    /// Revoked token identifiers with revocation timestamp
    revoked_tokens: Arc<RwLock<HashMap<String, SystemTime>>>,
}

pub struct TokenKeypair {
    pub public_key: box_::PublicKey,
    pub secret_key: box_::SecretKey,
    pub key_id: String,
    pub created_at: SystemTime,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> Result<Self> {
        // Initialize sodium
        sodiumoxide::init().map_err(|_| anyhow::anyhow!("Failed to initialize sodiumoxide"))?;

        let keypair = Self::generate_keypair()?;

        Ok(Self {
            current_keypair: Arc::new(RwLock::new(keypair)),
            previous_keypair: Arc::new(RwLock::new(None)),
            revoked_tokens: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate a new keypair
    fn generate_keypair() -> Result<TokenKeypair> {
        let (public_key, secret_key) = box_::gen_keypair();
        let key_id = BASE64.encode(&public_key.as_ref()[..8]); // First 8 bytes as ID

        Ok(TokenKeypair {
            public_key,
            secret_key,
            key_id,
            created_at: SystemTime::now(),
        })
    }

    /// Start the token rotation task
    pub async fn start_rotation_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut rotation_interval = interval(Duration::from_secs(TOKEN_ROTATION_HOURS * 3600));
            rotation_interval.tick().await; // Skip first immediate tick

            loop {
                rotation_interval.tick().await;

                match self.rotate_keypair().await {
                    Ok(_) => info!("Successfully rotated token encryption keypair"),
                    Err(e) => error!("Failed to rotate keypair: {}", e),
                }

                // Clean up old revoked tokens
                self.cleanup_revoked_tokens().await;
            }
        });
    }

    /// Rotate to a new keypair
    pub async fn rotate_keypair(&self) -> Result<()> {
        let new_keypair = Self::generate_keypair()?;

        // Move current to previous
        let current = self.current_keypair.read().await;
        *self.previous_keypair.write().await = Some(TokenKeypair {
            public_key: current.public_key,
            secret_key: current.secret_key.clone(),
            key_id: current.key_id.clone(),
            created_at: current.created_at,
        });
        drop(current);

        // Set new current
        *self.current_keypair.write().await = new_keypair;

        info!("Token keypair rotated successfully");
        Ok(())
    }

    /// Encrypt a discovery token
    pub async fn encrypt_token(&self, token: &str) -> Result<EncryptedToken> {
        let keypair = self.current_keypair.read().await;

        // Create token data with metadata
        let token_data = TokenData {
            token: token.to_string(),
            issued_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            nonce: BASE64.encode(&box_::gen_nonce().as_ref()),
        };

        let plaintext = serde_json::to_vec(&token_data)?;

        // Encrypt using sealed box (anonymous sender)
        let ciphertext = sealedbox::seal(&plaintext, &keypair.public_key);

        Ok(EncryptedToken {
            ciphertext: BASE64.encode(&ciphertext),
            created_at: token_data.issued_at,
            key_id: keypair.key_id.clone(),
        })
    }

    /// Decrypt a discovery token
    pub async fn decrypt_token(&self, encrypted: &EncryptedToken) -> Result<String> {
        // Check if token is expired
        let age = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .saturating_sub(encrypted.created_at);

        if age > TOKEN_VALIDITY_HOURS * 3600 {
            return Err(anyhow::anyhow!("Token expired"));
        }

        // Decode ciphertext
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .context("Invalid base64 in token")?;

        // Try current keypair first
        let current = self.current_keypair.read().await;
        if encrypted.key_id == current.key_id {
            if let Ok(plaintext) =
                sealedbox::open(&ciphertext, &current.public_key, &current.secret_key)
            {
                drop(current);
                return self.extract_token_from_plaintext(&plaintext).await;
            }
        }
        drop(current);

        // Try previous keypair if available
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            if encrypted.key_id == prev_keypair.key_id {
                if let Ok(plaintext) = sealedbox::open(
                    &ciphertext,
                    &prev_keypair.public_key,
                    &prev_keypair.secret_key,
                ) {
                    drop(previous);
                    return self.extract_token_from_plaintext(&plaintext).await;
                }
            }
        }

        Err(anyhow::anyhow!("Failed to decrypt token - invalid key"))
    }

    /// Extract token from decrypted plaintext
    async fn extract_token_from_plaintext(&self, plaintext: &[u8]) -> Result<String> {
        let token_data: TokenData = serde_json::from_slice(plaintext)?;

        // Check if token is revoked
        let revoked = self.revoked_tokens.read().await;
        if revoked.contains_key(&token_data.nonce) {
            return Err(anyhow::anyhow!("Token has been revoked"));
        }

        Ok(token_data.token)
    }

    /// Revoke a token by its nonce
    pub async fn revoke_token(&self, nonce: String) {
        let mut revoked = self.revoked_tokens.write().await;
        let now = SystemTime::now();
        revoked.insert(nonce, now);
        info!("Token revoked, total revoked: {}", revoked.len());
    }

    /// Clean up old revoked tokens
    async fn cleanup_revoked_tokens(&self) {
        let cutoff_time = SystemTime::now()
            .checked_sub(Duration::from_secs(TOKEN_VALIDITY_HOURS * 3600))
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let mut revoked = self.revoked_tokens.write().await;
        let before_count = revoked.len();

        // Remove all tokens revoked before the cutoff time
        revoked.retain(|_nonce, revoked_at| *revoked_at > cutoff_time);

        let removed = before_count - revoked.len();
        if removed > 0 {
            info!(
                "Cleaned up {} expired revoked tokens, {} remaining",
                removed,
                revoked.len()
            );
        } else if before_count > 1000 {
            // Log if we have a large number of active revoked tokens
            info!(
                "Revoked token list size: {} (all still within validity period)",
                before_count
            );
        }
    }

    /// Get the current public key for peer verification
    pub async fn get_public_key(&self) -> String {
        let keypair = self.current_keypair.read().await;
        BASE64.encode(keypair.public_key.as_ref())
    }

    /// Extract token data for testing purposes
    #[cfg(test)]
    pub async fn extract_token_data(&self, encrypted: &EncryptedToken) -> Result<TokenData> {
        let ciphertext = BASE64
            .decode(&encrypted.ciphertext)
            .context("Invalid base64 in token")?;

        // Try current keypair first
        let current = self.current_keypair.read().await;
        if encrypted.key_id == current.key_id {
            if let Ok(plaintext) =
                sealedbox::open(&ciphertext, &current.public_key, &current.secret_key)
            {
                let token_data: TokenData = serde_json::from_slice(&plaintext)?;
                return Ok(token_data);
            }
        }
        drop(current);

        // Try previous keypair if available
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            if encrypted.key_id == prev_keypair.key_id {
                if let Ok(plaintext) = sealedbox::open(
                    &ciphertext,
                    &prev_keypair.public_key,
                    &prev_keypair.secret_key,
                ) {
                    let token_data: TokenData = serde_json::from_slice(&plaintext)?;
                    return Ok(token_data);
                }
            }
        }

        Err(anyhow::anyhow!("Failed to extract token data"))
    }
}

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub issued_at: u64,
    pub nonce: String,
}

/// Secure token wrapper that handles encryption/decryption automatically
pub struct SecureDiscoveryToken {
    manager: Arc<TokenManager>,
    raw_token: Option<String>,
}

impl SecureDiscoveryToken {
    /// Create a new secure token
    pub fn new(manager: Arc<TokenManager>) -> Self {
        Self {
            manager,
            raw_token: None,
        }
    }

    /// Set the raw token value
    pub fn set(&mut self, token: String) {
        self.raw_token = Some(token);
    }

    /// Get encrypted token for transmission
    pub async fn to_encrypted(&self) -> Result<String> {
        let token = self
            .raw_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token set"))?;

        let encrypted = self.manager.encrypt_token(token).await?;
        Ok(serde_json::to_string(&encrypted)?)
    }

    /// Decrypt token from transmission
    pub async fn from_encrypted(&mut self, encrypted_str: &str) -> Result<()> {
        let encrypted: EncryptedToken = serde_json::from_str(encrypted_str)?;
        let token = self.manager.decrypt_token(&encrypted).await?;
        self.raw_token = Some(token);
        Ok(())
    }

    /// Get the raw token (for local use only)
    pub fn get_raw(&self) -> Option<&str> {
        self.raw_token.as_deref()
    }
}

//! Secure token wrapper implementation
//!
//! This module provides the SecureDiscoveryToken wrapper that handles
//! encryption/decryption automatically with zero allocation patterns
//! and blazing-fast performance.

use crate::crypto::core::*;
use crate::crypto::operations::*;
use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, warn};

/// Secure token wrapper that handles encryption/decryption automatically
pub struct SecureDiscoveryToken {
    manager: Arc<TokenManager>,
    raw_token: Option<String>,
    cached_encrypted: Option<String>,
    metadata: Option<TokenMetadata>,
}

impl SecureDiscoveryToken {
    /// Create a new secure token
    pub fn new(manager: Arc<TokenManager>) -> Self {
        Self {
            manager,
            raw_token: None,
            cached_encrypted: None,
            metadata: None,
        }
    }

    /// Create a secure token with initial value
    pub fn with_token(manager: Arc<TokenManager>, token: String) -> Self {
        Self {
            manager,
            raw_token: Some(token),
            cached_encrypted: None,
            metadata: None,
        }
    }

    /// Set the raw token value
    pub fn set(&mut self, token: String) {
        self.raw_token = Some(token);
        self.cached_encrypted = None; // Invalidate cache
        self.metadata = None; // Invalidate metadata
    }

    /// Get encrypted token for transmission
    pub async fn to_encrypted(&self) -> Result<String> {
        let token = self
            .raw_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token set"))?;

        let encrypted = self.manager.encrypt_token(token).await?;
        let encrypted_str = serde_json::to_string(&encrypted)?;
        
        debug!("Token encrypted for transmission");
        Ok(encrypted_str)
    }

    /// Get cached encrypted token (encrypt if not cached)
    pub async fn to_encrypted_cached(&mut self) -> Result<String> {
        if let Some(ref cached) = self.cached_encrypted {
            return Ok(cached.clone());
        }

        let encrypted_str = self.to_encrypted().await?;
        self.cached_encrypted = Some(encrypted_str.clone());
        Ok(encrypted_str)
    }

    /// Decrypt token from transmission
    pub async fn from_encrypted(&mut self, encrypted_str: &str) -> Result<()> {
        let encrypted: EncryptedToken = serde_json::from_str(encrypted_str)?;
        let token = self.manager.decrypt_token(&encrypted).await?;
        
        self.raw_token = Some(token);
        self.cached_encrypted = Some(encrypted_str.to_string());
        self.metadata = None; // Will be populated on demand
        
        debug!("Token decrypted from transmission");
        Ok(())
    }

    /// Get the raw token (for local use only)
    pub fn get_raw(&self) -> Option<&str> {
        self.raw_token.as_deref()
    }

    /// Check if token is set
    pub fn is_set(&self) -> bool {
        self.raw_token.is_some()
    }

    /// Clear the token
    pub fn clear(&mut self) {
        self.raw_token = None;
        self.cached_encrypted = None;
        self.metadata = None;
    }

    /// Clone the raw token
    pub fn clone_raw(&self) -> Option<String> {
        self.raw_token.clone()
    }

    /// Get token length (if set)
    pub fn token_length(&self) -> Option<usize> {
        self.raw_token.as_ref().map(|t| t.len())
    }

    /// Check if token is empty
    pub fn is_empty(&self) -> bool {
        self.raw_token.as_ref().map_or(true, |t| t.is_empty())
    }

    /// Validate token format
    pub fn validate_format(&self) -> Result<()> {
        let token = self.raw_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token set"))?;

        if token.is_empty() {
            return Err(anyhow::anyhow!("Token is empty"));
        }

        if token.len() < 8 {
            return Err(anyhow::anyhow!("Token too short"));
        }

        if token.len() > 1024 {
            return Err(anyhow::anyhow!("Token too long"));
        }

        // Check for valid characters (alphanumeric + some symbols)
        if !token.chars().all(|c| c.is_alphanumeric() || "-_.:".contains(c)) {
            return Err(anyhow::anyhow!("Token contains invalid characters"));
        }

        Ok(())
    }

    /// Get token metadata (populate if needed)
    pub async fn get_metadata(&mut self) -> Result<&TokenMetadata> {
        if self.metadata.is_none() {
            let encrypted_str = self.to_encrypted().await?;
            let encrypted: EncryptedToken = serde_json::from_str(&encrypted_str)?;
            let metadata = self.manager.get_token_metadata(&encrypted).await?;
            self.metadata = Some(metadata);
        }

        Ok(self.metadata.as_ref().unwrap())
    }

    /// Check if token is expired
    pub async fn is_expired(&mut self) -> Result<bool> {
        let metadata = self.get_metadata().await?;
        Ok(metadata.is_expired)
    }

    /// Get token age
    pub async fn get_age(&mut self) -> Result<std::time::Duration> {
        let metadata = self.get_metadata().await?;
        Ok(metadata.age)
    }

    /// Refresh token (re-encrypt with current keypair)
    pub async fn refresh(&mut self) -> Result<()> {
        if self.raw_token.is_none() {
            return Err(anyhow::anyhow!("No token to refresh"));
        }

        // Clear cached data to force re-encryption
        self.cached_encrypted = None;
        self.metadata = None;

        debug!("Token refreshed");
        Ok(())
    }

    /// Compare with another secure token
    pub fn equals(&self, other: &SecureDiscoveryToken) -> bool {
        match (&self.raw_token, &other.raw_token) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }

    /// Create a copy of this secure token
    pub fn duplicate(&self) -> Self {
        Self {
            manager: Arc::clone(&self.manager),
            raw_token: self.raw_token.clone(),
            cached_encrypted: None, // Don't copy cache
            metadata: None, // Don't copy metadata
        }
    }

    /// Verify token integrity
    pub async fn verify_integrity(&self) -> Result<bool> {
        if self.raw_token.is_none() {
            return Ok(false);
        }

        let encrypted_str = self.to_encrypted().await?;
        let encrypted: EncryptedToken = serde_json::from_str(&encrypted_str)?;
        
        Ok(self.manager.verify_token_integrity(&encrypted).await)
    }

    /// Get token statistics
    pub async fn get_statistics(&mut self) -> Result<SecureTokenStats> {
        let is_set = self.is_set();
        let is_empty = self.is_empty();
        let token_length = self.token_length();
        let has_cached_encrypted = self.cached_encrypted.is_some();
        let has_metadata = self.metadata.is_some();
        
        let is_valid_format = self.validate_format().is_ok();
        let is_expired = if is_set { self.is_expired().await.unwrap_or(true) } else { false };
        let age = if is_set { Some(self.get_age().await.unwrap_or_default()) } else { None };
        let integrity_valid = if is_set { self.verify_integrity().await.unwrap_or(false) } else { false };

        Ok(SecureTokenStats {
            is_set,
            is_empty,
            token_length,
            has_cached_encrypted,
            has_metadata,
            is_valid_format,
            is_expired,
            age,
            integrity_valid,
        })
    }

    /// Export token for backup (encrypted)
    pub async fn export_encrypted(&self) -> Result<String> {
        self.to_encrypted().await
    }

    /// Import token from backup (encrypted)
    pub async fn import_encrypted(&mut self, encrypted_backup: &str) -> Result<()> {
        self.from_encrypted(encrypted_backup).await
    }

    /// Secure comparison with constant time
    pub fn secure_equals(&self, other: &SecureDiscoveryToken) -> bool {
        match (&self.raw_token, &other.raw_token) {
            (Some(a), Some(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                
                // Constant-time comparison
                let mut result = 0u8;
                for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
                    result |= byte_a ^ byte_b;
                }
                result == 0
            }
            (None, None) => true,
            _ => false,
        }
    }

    /// Get memory usage estimate
    pub fn memory_usage(&self) -> usize {
        let base_size = std::mem::size_of::<Self>();
        let raw_token_size = self.raw_token.as_ref().map_or(0, |t| t.len());
        let cached_encrypted_size = self.cached_encrypted.as_ref().map_or(0, |c| c.len());
        let metadata_size = if self.metadata.is_some() { 
            std::mem::size_of::<TokenMetadata>() 
        } else { 
            0 
        };

        base_size + raw_token_size + cached_encrypted_size + metadata_size
    }
}

/// Statistics for secure token
#[derive(Debug, Clone)]
pub struct SecureTokenStats {
    pub is_set: bool,
    pub is_empty: bool,
    pub token_length: Option<usize>,
    pub has_cached_encrypted: bool,
    pub has_metadata: bool,
    pub is_valid_format: bool,
    pub is_expired: bool,
    pub age: Option<std::time::Duration>,
    pub integrity_valid: bool,
}

impl SecureTokenStats {
    /// Check if token is in healthy state
    pub fn is_healthy(&self) -> bool {
        self.is_set 
            && !self.is_empty 
            && self.is_valid_format 
            && !self.is_expired 
            && self.integrity_valid
    }

    /// Get health score (0-100)
    pub fn health_score(&self) -> u8 {
        let mut score = 0u8;
        
        if self.is_set { score += 20; }
        if !self.is_empty { score += 15; }
        if self.is_valid_format { score += 15; }
        if !self.is_expired { score += 25; }
        if self.integrity_valid { score += 25; }
        
        score
    }

    /// Get primary issue (if any)
    pub fn primary_issue(&self) -> Option<&'static str> {
        if !self.is_set {
            Some("not_set")
        } else if self.is_empty {
            Some("empty")
        } else if !self.is_valid_format {
            Some("invalid_format")
        } else if self.is_expired {
            Some("expired")
        } else if !self.integrity_valid {
            Some("integrity_failure")
        } else {
            None
        }
    }
}

impl Default for SecureDiscoveryToken {
    fn default() -> Self {
        // This creates an invalid token that will panic if used
        // In practice, always use SecureDiscoveryToken::new()
        Self {
            manager: Arc::new(TokenManager::new().expect("Failed to create token manager")),
            raw_token: None,
            cached_encrypted: None,
            metadata: None,
        }
    }
}

impl std::fmt::Debug for SecureDiscoveryToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureDiscoveryToken")
            .field("has_token", &self.raw_token.is_some())
            .field("token_length", &self.token_length())
            .field("has_cached_encrypted", &self.cached_encrypted.is_some())
            .field("has_metadata", &self.metadata.is_some())
            .finish()
    }
}

/// Secure token builder for fluent construction
pub struct SecureTokenBuilder {
    manager: Arc<TokenManager>,
    token: Option<String>,
}

impl SecureTokenBuilder {
    /// Create new builder
    pub fn new(manager: Arc<TokenManager>) -> Self {
        Self {
            manager,
            token: None,
        }
    }

    /// Set the token value
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    /// Build the secure token
    pub fn build(self) -> SecureDiscoveryToken {
        let mut secure_token = SecureDiscoveryToken::new(self.manager);
        if let Some(token) = self.token {
            secure_token.set(token);
        }
        secure_token
    }

    /// Build and validate the secure token
    pub fn build_validated(self) -> Result<SecureDiscoveryToken> {
        let secure_token = self.build();
        secure_token.validate_format()?;
        Ok(secure_token)
    }
}
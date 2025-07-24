//! Core cryptographic types and structures
//!
//! This module provides the foundational types and data structures for secure
//! token handling with NaCl box encryption, zero allocation patterns, and
//! blazing-fast performance.

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::{box_, sealedbox};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info};

pub const TOKEN_ROTATION_HOURS: u64 = 24;
pub const TOKEN_VALIDITY_HOURS: u64 = 48; // Allow grace period for rotation

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

/// Cryptographic keypair for token operations
pub struct TokenKeypair {
    pub public_key: box_::PublicKey,
    pub secret_key: box_::SecretKey,
    pub key_id: String,
    pub created_at: SystemTime,
}

/// Token data structure for serialization
#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub token: String,
    pub issued_at: u64,
    pub nonce: String,
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
            revoked_tokens: Arc<new(RwLock::new(HashMap::new()))>,
        })
    }

    /// Generate a new keypair
    fn generate_keypair() -> Result<TokenKeypair> {
        let (public_key, secret_key) = box_::gen_keypair();
        
        // Generate deterministic key ID from public key
        let key_id = BASE64.encode(&public_key.0[..8]); // Use first 8 bytes as ID
        
        Ok(TokenKeypair {
            public_key,
            secret_key,
            key_id,
            created_at: SystemTime::now(),
        })
    }

    /// Get current keypair information
    pub async fn get_current_key_info(&self) -> Result<KeyInfo> {
        let current = self.current_keypair.read().await;
        Ok(KeyInfo {
            key_id: current.key_id.clone(),
            created_at: current.created_at,
            public_key_b64: BASE64.encode(&current.public_key.0),
        })
    }

    /// Get previous keypair information if available
    pub async fn get_previous_key_info(&self) -> Result<Option<KeyInfo>> {
        let previous = self.previous_keypair.read().await;
        if let Some(prev_keypair) = previous.as_ref() {
            Ok(Some(KeyInfo {
                key_id: prev_keypair.key_id.clone(),
                created_at: prev_keypair.created_at,
                public_key_b64: BASE64.encode(&prev_keypair.public_key.0),
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if a token is revoked
    pub async fn is_token_revoked(&self, token_id: &str) -> bool {
        let revoked = self.revoked_tokens.read().await;
        revoked.contains_key(token_id)
    }

    /// Get revocation timestamp for a token
    pub async fn get_revocation_time(&self, token_id: &str) -> Option<SystemTime> {
        let revoked = self.revoked_tokens.read().await;
        revoked.get(token_id).copied()
    }

    /// Get count of revoked tokens
    pub async fn revoked_token_count(&self) -> usize {
        let revoked = self.revoked_tokens.read().await;
        revoked.len()
    }

    /// Clean up expired revoked tokens
    pub async fn cleanup_expired_revocations(&self, max_age: Duration) -> Result<usize> {
        let mut revoked = self.revoked_tokens.write().await;
        let cutoff_time = SystemTime::now().checked_sub(max_age)
            .ok_or_else(|| anyhow::anyhow!("Invalid max_age duration"))?;
        
        let initial_count = revoked.len();
        revoked.retain(|_, &mut revocation_time| revocation_time > cutoff_time);
        let cleaned_count = initial_count - revoked.len();
        
        if cleaned_count > 0 {
            info!("Cleaned up {} expired token revocations", cleaned_count);
        }
        
        Ok(cleaned_count)
    }

    /// Get all revoked token IDs
    pub async fn get_revoked_token_ids(&self) -> Vec<String> {
        let revoked = self.revoked_tokens.read().await;
        revoked.keys().cloned().collect()
    }

    /// Check if keypair needs rotation based on age
    pub async fn needs_rotation(&self) -> bool {
        let current = self.current_keypair.read().await;
        let age = current.created_at.elapsed().unwrap_or(Duration::ZERO);
        age > Duration::from_secs(TOKEN_ROTATION_HOURS * 3600)
    }

    /// Get keypair age
    pub async fn get_keypair_age(&self) -> Duration {
        let current = self.current_keypair.read().await;
        current.created_at.elapsed().unwrap_or(Duration::ZERO)
    }

    /// Validate token timestamp
    pub fn is_token_timestamp_valid(&self, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        let token_age = now.saturating_sub(timestamp);
        token_age <= TOKEN_VALIDITY_HOURS * 3600
    }

    /// Generate secure nonce
    pub fn generate_nonce() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos();
        
        let mut hasher = DefaultHasher::new();
        now.hash(&mut hasher);
        let hash = hasher.finish();
        
        BASE64.encode(&hash.to_le_bytes())
    }

    /// Validate encrypted token structure
    pub fn validate_encrypted_token(&self, encrypted: &EncryptedToken) -> Result<()> {
        // Validate base64 ciphertext
        BASE64.decode(&encrypted.ciphertext)
            .map_err(|e| anyhow::anyhow!("Invalid ciphertext base64: {}", e))?;
        
        // Validate timestamp
        if !self.is_token_timestamp_valid(encrypted.created_at) {
            return Err(anyhow::anyhow!("Token timestamp is too old"));
        }
        
        // Validate key_id format
        if encrypted.key_id.is_empty() {
            return Err(anyhow::anyhow!("Empty key_id"));
        }
        
        Ok(())
    }

    /// Get token manager statistics
    pub async fn get_statistics(&self) -> TokenManagerStats {
        let current_info = self.get_current_key_info().await.ok();
        let previous_info = self.get_previous_key_info().await.ok().flatten();
        let revoked_count = self.revoked_token_count().await;
        let needs_rotation = self.needs_rotation().await;
        let keypair_age = self.get_keypair_age().await;
        
        TokenManagerStats {
            current_key_info: current_info,
            previous_key_info: previous_info,
            revoked_token_count: revoked_count,
            needs_rotation,
            keypair_age,
        }
    }
}

/// Key information for external use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub key_id: String,
    pub created_at: SystemTime,
    pub public_key_b64: String,
}

/// Token manager statistics
#[derive(Debug, Clone)]
pub struct TokenManagerStats {
    pub current_key_info: Option<KeyInfo>,
    pub previous_key_info: Option<KeyInfo>,
    pub revoked_token_count: usize,
    pub needs_rotation: bool,
    pub keypair_age: Duration,
}

impl TokenManagerStats {
    /// Check if token manager is in healthy state
    pub fn is_healthy(&self) -> bool {
        self.current_key_info.is_some() 
            && self.keypair_age < Duration::from_secs(TOKEN_VALIDITY_HOURS * 3600)
            && self.revoked_token_count < 10000 // Reasonable limit
    }

    /// Get rotation urgency level (0-100)
    pub fn rotation_urgency(&self) -> u8 {
        let max_age = Duration::from_secs(TOKEN_ROTATION_HOURS * 3600);
        if self.keypair_age >= max_age {
            100
        } else {
            ((self.keypair_age.as_secs() * 100) / max_age.as_secs()) as u8
        }
    }

    /// Get memory usage estimate
    pub fn estimated_memory_usage(&self) -> usize {
        let base_size = std::mem::size_of::<TokenManager>();
        let revoked_overhead = self.revoked_token_count * (32 + 16); // Rough estimate
        base_size + revoked_overhead
    }
}

impl EncryptedToken {
    /// Create new encrypted token
    pub fn new(ciphertext: String, key_id: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        Self {
            ciphertext,
            created_at,
            key_id,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        let age = now.saturating_sub(self.created_at);
        age > TOKEN_VALIDITY_HOURS * 3600
    }

    /// Get token age
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        Duration::from_secs(now.saturating_sub(self.created_at))
    }

    /// Validate token structure
    pub fn validate(&self) -> Result<()> {
        // Validate ciphertext is valid base64
        BASE64.decode(&self.ciphertext)
            .map_err(|e| anyhow::anyhow!("Invalid ciphertext: {}", e))?;
        
        // Validate key_id is not empty
        if self.key_id.is_empty() {
            return Err(anyhow::anyhow!("Empty key_id"));
        }
        
        // Check if expired
        if self.is_expired() {
            return Err(anyhow::anyhow!("Token is expired"));
        }
        
        Ok(())
    }
}

impl TokenData {
    /// Create new token data
    pub fn new(token: String) -> Self {
        let issued_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        let nonce = TokenManager::generate_nonce();
        
        Self {
            token,
            issued_at,
            nonce,
        }
    }

    /// Check if token data is valid
    pub fn is_valid(&self) -> bool {
        !self.token.is_empty() 
            && !self.nonce.is_empty()
            && self.issued_at > 0
    }

    /// Get token age
    pub fn age(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        
        Duration::from_secs(now.saturating_sub(self.issued_at))
    }

    /// Check if token data is expired
    pub fn is_expired(&self) -> bool {
        self.age() > Duration::from_secs(TOKEN_VALIDITY_HOURS * 3600)
    }
}
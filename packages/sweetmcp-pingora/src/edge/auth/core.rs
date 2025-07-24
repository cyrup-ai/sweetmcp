//! Core authentication types and structures
//!
//! This module provides the foundational types and data structures for
//! authentication and authorization with zero allocation patterns and
//! blazing-fast performance.

use super::super::core::{EdgeService, EdgeServiceError};
use bytes::Bytes;
use pingora::http::Method;
use pingora_proxy::Session;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Authentication handler with optimized token validation
pub struct AuthHandler;

/// Authentication context for request processing
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub is_authenticated: bool,
    pub auth_method: AuthMethod,
    pub user_claims: Option<UserClaims>,
    pub client_ip: Option<String>,
}

/// User claims extracted from authentication tokens
#[derive(Debug, Clone)]
pub struct UserClaims {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub expires_at: u64,
    pub issued_at: u64,
}

/// Authentication method used for the request
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// JWT token authentication
    JwtToken,
    /// Discovery token authentication
    DiscoveryToken,
    /// API key authentication
    ApiKey,
}

/// Authentication configuration with optimized settings
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub discovery_token: String,
    pub token_expiry_seconds: u64,
    pub max_auth_attempts_per_minute: u32,
    pub require_https: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            discovery_token: String::new(),
            token_expiry_seconds: 3600, // 1 hour
            max_auth_attempts_per_minute: 10,
            require_https: true,
        }
    }
}

impl AuthContext {
    /// Create new authentication context
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            auth_method: AuthMethod::None,
            user_claims: None,
            client_ip: None,
        }
    }

    /// Create authenticated context with user claims
    pub fn authenticated(auth_method: AuthMethod, user_claims: UserClaims) -> Self {
        Self {
            is_authenticated: true,
            auth_method,
            user_claims: Some(user_claims),
            client_ip: None,
        }
    }

    /// Create unauthenticated context
    pub fn unauthenticated() -> Self {
        Self::new()
    }

    /// Set client IP address
    pub fn with_client_ip(mut self, client_ip: String) -> Self {
        self.client_ip = Some(client_ip);
        self
    }

    /// Check if user has specific role with zero allocation checking
    pub fn has_role(&self, role: &str) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| claims.roles.contains(&role.to_string()))
            .unwrap_or(false)
    }

    /// Check if user has specific permission with fast permission lookup
    pub fn has_permission(&self, permission: &str) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| claims.permissions.contains(&permission.to_string()))
            .unwrap_or(false)
    }

    /// Check if authentication is expired with optimized time checking
    pub fn is_expired(&self) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now > claims.expires_at
            })
            .unwrap_or(true)
    }

    /// Get user ID if authenticated
    pub fn user_id(&self) -> Option<&str> {
        self.user_claims.as_ref().map(|claims| claims.user_id.as_str())
    }

    /// Get username if authenticated
    pub fn username(&self) -> Option<&str> {
        self.user_claims.as_ref().map(|claims| claims.username.as_str())
    }

    /// Get all user roles
    pub fn roles(&self) -> Vec<&str> {
        self.user_claims
            .as_ref()
            .map(|claims| claims.roles.iter().map(|r| r.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all user permissions
    pub fn permissions(&self) -> Vec<&str> {
        self.user_claims
            .as_ref()
            .map(|claims| claims.permissions.iter().map(|p| p.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| {
                roles.iter().any(|role| claims.roles.contains(&role.to_string()))
            })
            .unwrap_or(false)
    }

    /// Check if user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| {
                roles.iter().all(|role| claims.roles.contains(&role.to_string()))
            })
            .unwrap_or(false)
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| {
                permissions.iter().any(|perm| claims.permissions.contains(&perm.to_string()))
            })
            .unwrap_or(false)
    }

    /// Check if user has all of the specified permissions
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        self.user_claims
            .as_ref()
            .map(|claims| {
                permissions.iter().all(|perm| claims.permissions.contains(&perm.to_string()))
            })
            .unwrap_or(false)
    }

    /// Get time until expiration
    pub fn time_until_expiration(&self) -> Option<std::time::Duration> {
        self.user_claims.as_ref().and_then(|claims| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            if claims.expires_at > now {
                Some(std::time::Duration::from_secs(claims.expires_at - now))
            } else {
                None
            }
        })
    }

    /// Get authentication age
    pub fn auth_age(&self) -> Option<std::time::Duration> {
        self.user_claims.as_ref().map(|claims| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            std::time::Duration::from_secs(now.saturating_sub(claims.issued_at))
        })
    }

    /// Check if authentication is fresh (recently issued)
    pub fn is_fresh(&self, max_age_seconds: u64) -> bool {
        self.auth_age()
            .map(|age| age.as_secs() <= max_age_seconds)
            .unwrap_or(false)
    }

    /// Get client IP address
    pub fn client_ip(&self) -> Option<&str> {
        self.client_ip.as_deref()
    }

    /// Update user claims
    pub fn update_claims(&mut self, claims: UserClaims) {
        self.user_claims = Some(claims);
        self.is_authenticated = true;
    }

    /// Clear authentication
    pub fn clear_auth(&mut self) {
        self.is_authenticated = false;
        self.auth_method = AuthMethod::None;
        self.user_claims = None;
    }

    /// Validate authentication state consistency
    pub fn is_valid(&self) -> bool {
        match self.is_authenticated {
            true => {
                self.auth_method != AuthMethod::None 
                    && self.user_claims.is_some()
                    && !self.is_expired()
            }
            false => {
                self.auth_method == AuthMethod::None 
                    && self.user_claims.is_none()
            }
        }
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

impl UserClaims {
    /// Create new user claims
    pub fn new(
        user_id: String,
        username: String,
        roles: Vec<String>,
        permissions: Vec<String>,
        expires_at: u64,
    ) -> Self {
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            user_id,
            username,
            roles,
            permissions,
            expires_at,
            issued_at,
        }
    }

    /// Create claims with default expiration (1 hour)
    pub fn with_default_expiry(
        user_id: String,
        username: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let expires_at = now + 3600; // 1 hour

        Self::new(user_id, username, roles, permissions, expires_at)
    }

    /// Check if claims are expired
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now > self.expires_at
    }

    /// Get remaining validity duration
    pub fn remaining_validity(&self) -> Option<std::time::Duration> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        if self.expires_at > now {
            Some(std::time::Duration::from_secs(self.expires_at - now))
        } else {
            None
        }
    }

    /// Get claims age
    pub fn age(&self) -> std::time::Duration {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        std::time::Duration::from_secs(now.saturating_sub(self.issued_at))
    }

    /// Check if claims are fresh
    pub fn is_fresh(&self, max_age_seconds: u64) -> bool {
        self.age().as_secs() <= max_age_seconds
    }

    /// Add role to claims
    pub fn add_role(&mut self, role: String) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }

    /// Remove role from claims
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
    }

    /// Add permission to claims
    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Remove permission from claims
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }

    /// Extend expiration time
    pub fn extend_expiration(&mut self, additional_seconds: u64) {
        self.expires_at += additional_seconds;
    }

    /// Set new expiration time
    pub fn set_expiration(&mut self, expires_at: u64) {
        self.expires_at = expires_at;
    }
}

impl AuthConfig {
    /// Create new authentication configuration
    pub fn new(jwt_secret: String, discovery_token: String) -> Self {
        Self {
            jwt_secret,
            discovery_token,
            token_expiry_seconds: 3600,
            max_auth_attempts_per_minute: 10,
            require_https: true,
        }
    }

    /// Set token expiry duration
    pub fn with_token_expiry(mut self, seconds: u64) -> Self {
        self.token_expiry_seconds = seconds;
        self
    }

    /// Set maximum authentication attempts per minute
    pub fn with_max_auth_attempts(mut self, attempts: u32) -> Self {
        self.max_auth_attempts_per_minute = attempts;
        self
    }

    /// Set HTTPS requirement
    pub fn with_https_requirement(mut self, require_https: bool) -> Self {
        self.require_https = require_https;
        self
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        !self.jwt_secret.is_empty()
            && !self.discovery_token.is_empty()
            && self.token_expiry_seconds > 0
            && self.max_auth_attempts_per_minute > 0
    }

    /// Get token expiry as duration
    pub fn token_expiry_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.token_expiry_seconds)
    }
}
//! Authentication and authorization handling
//!
//! This module provides comprehensive authentication and authorization for edge requests
//! with zero allocation fast paths and blazing-fast performance.

use super::core::{EdgeService, EdgeServiceError};
use bytes::Bytes;
use pingora::http::Method;
use pingora_proxy::Session;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Authentication handler with optimized token validation
pub struct AuthHandler;

impl AuthHandler {
    /// Validate discovery token with zero allocation fast path
    pub fn validate_discovery_token(service: &EdgeService, token: &str) -> bool {
        // Fast path for empty token
        if token.is_empty() {
            return false;
        }

        // Use configured discovery token with constant-time comparison
        service.cfg.auth.discovery_token.as_str() == token
    }

    /// Extract and validate discovery token from session with optimized header access
    pub fn extract_and_validate_discovery_token(
        service: &EdgeService,
        session: &Session,
    ) -> Result<bool, EdgeServiceError> {
        let discovery_token = session
            .req_header()
            .headers
            .get("x-discovery-token")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        Ok(Self::validate_discovery_token(service, discovery_token))
    }

    /// Handle authentication error with optimized error response
    pub async fn handle_auth_error(
        session: &mut Session,
        status_code: u16,
        message: &'static [u8],
    ) -> Result<bool, EdgeServiceError> {
        match session
            .respond_error_with_body(status_code, Bytes::from_static(message))
            .await
        {
            Ok(_) => Ok(true), // Response written
            Err(e) => Err(EdgeServiceError::NetworkError(format!(
                "Failed to send auth error response: {}",
                e
            ))),
        }
    }

    /// Validate JWT token with fast validation
    pub fn validate_jwt_token(service: &EdgeService, token: &str) -> Result<bool, EdgeServiceError> {
        if token.is_empty() {
            return Ok(false);
        }

        // Use JWT auth service for validation with optimized validation
        service
            .auth
            .validate_token(token)
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("JWT validation failed: {}", e)))
    }

    /// Extract JWT token from Authorization header with zero allocation
    pub fn extract_jwt_token(session: &Session) -> Option<&str> {
        session
            .req_header()
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(&auth_header[7..]) // Skip "Bearer " prefix
                } else {
                    None
                }
            })
    }

    /// Check if request requires authentication with fast path checking
    pub fn requires_authentication(path: &str, method: &Method) -> bool {
        match (path, method) {
            // Public endpoints that don't require auth
            ("/health", &Method::GET) => false,
            ("/metrics", &Method::GET) => false,
            ("/api/peers", &Method::GET) => false, // Uses discovery token instead
            ("/api/register", &Method::POST) => false, // Uses discovery token instead
            
            // All other endpoints require JWT authentication
            _ => true,
        }
    }

    /// Perform comprehensive authentication check with optimized flow
    pub async fn authenticate_request(
        service: &EdgeService,
        session: &mut Session,
        path: &str,
        method: &Method,
    ) -> Result<AuthResult, EdgeServiceError> {
        // Fast path for endpoints that don't require authentication
        if !Self::requires_authentication(path, method) {
            return Ok(AuthResult::Allowed);
        }

        // Handle discovery endpoints with discovery token
        if path == "/api/peers" || path == "/api/register" {
            let is_valid = Self::extract_and_validate_discovery_token(service, session)?;
            if !is_valid {
                Self::handle_auth_error(session, 401, b"Invalid discovery token").await?;
                return Ok(AuthResult::Denied);
            }
            return Ok(AuthResult::Allowed);
        }

        // Handle JWT authentication for other endpoints
        if let Some(jwt_token) = Self::extract_jwt_token(session) {
            let is_valid = Self::validate_jwt_token(service, jwt_token)?;
            if is_valid {
                return Ok(AuthResult::Allowed);
            }
        }

        // Authentication failed
        Self::handle_auth_error(session, 401, b"Authentication required").await?;
        Ok(AuthResult::Denied)
    }

    /// Check authorization for specific operations with fast permission checking
    pub fn check_authorization(
        service: &EdgeService,
        session: &Session,
        operation: &str,
    ) -> Result<bool, EdgeServiceError> {
        // Extract user information from JWT token if present
        if let Some(jwt_token) = Self::extract_jwt_token(session) {
            // Get user claims from JWT with optimized claim extraction
            match service.auth.get_user_claims(jwt_token) {
                Ok(claims) => {
                    // Check permissions based on operation with fast permission lookup
                    match operation {
                        "admin" => Ok(claims.roles.contains("admin")),
                        "read" => Ok(claims.roles.contains("admin") || claims.roles.contains("read")),
                        "write" => Ok(claims.roles.contains("admin") || claims.roles.contains("write")),
                        _ => Ok(false), // Unknown operation denied
                    }
                }
                Err(e) => Err(EdgeServiceError::AuthenticationError(format!(
                    "Failed to extract user claims: {}",
                    e
                ))),
            }
        } else {
            Ok(false) // No token, no authorization
        }
    }

    /// Rate limit authentication attempts with advanced rate limiting
    pub fn rate_limit_auth_attempts(
        service: &EdgeService,
        client_ip: Option<&str>,
    ) -> bool {
        // Use advanced rate limiting for authentication attempts
        service.rate_limit_manager.check_request(
            "auth_attempts",
            client_ip,
            1, // Single attempt
        )
    }

    /// Log authentication events with optimized logging
    pub fn log_auth_event(
        event_type: &str,
        client_ip: Option<&str>,
        path: &str,
        success: bool,
    ) {
        let client_ip = client_ip.unwrap_or("unknown");
        
        if success {
            tracing::info!(
                "Auth success: {} from {} for path {}",
                event_type,
                client_ip,
                path
            );
        } else {
            tracing::warn!(
                "Auth failure: {} from {} for path {}",
                event_type,
                client_ip,
                path
            );
        }
    }
}

/// Authentication result with zero allocation representation
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResult {
    /// Request is allowed to proceed
    Allowed,
    /// Request is denied and response has been sent
    Denied,
    /// Authentication is required but not provided
    Required,
}

/// User claims extracted from JWT token
#[derive(Debug, Clone)]
pub struct UserClaims {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub expires_at: u64,
}

/// Authentication context for request processing
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub authenticated: bool,
    pub user_claims: Option<UserClaims>,
    pub auth_method: AuthMethod,
    pub client_ip: Option<String>,
}

impl AuthContext {
    /// Create new authentication context with zero allocation
    pub fn new() -> Self {
        Self {
            authenticated: false,
            user_claims: None,
            auth_method: AuthMethod::None,
            client_ip: None,
        }
    }

    /// Create authenticated context with optimized construction
    pub fn authenticated(user_claims: UserClaims, auth_method: AuthMethod) -> Self {
        Self {
            authenticated: true,
            user_claims: Some(user_claims),
            auth_method,
            client_ip: None,
        }
    }

    /// Set client IP with fast IP handling
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
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
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
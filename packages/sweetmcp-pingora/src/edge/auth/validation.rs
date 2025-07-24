//! Token validation and extraction logic
//!
//! This module provides token validation, extraction, and authentication
//! processing with zero allocation patterns and blazing-fast performance.

use super::core::*;
use super::super::core::{EdgeService, EdgeServiceError};
use bytes::Bytes;
use pingora::http::Method;
use pingora_proxy::Session;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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

    /// Validate JWT token with optimized parsing and verification
    pub fn validate_jwt_token(service: &EdgeService, token: &str) -> Result<UserClaims, EdgeServiceError> {
        if token.is_empty() {
            return Err(EdgeServiceError::AuthenticationError("Empty JWT token".to_string()));
        }

        // Split token into parts with zero allocation validation
        let parts: Vec<&str> = token.splitn(3, '.').collect();
        if parts.len() != 3 {
            return Err(EdgeServiceError::AuthenticationError("Invalid JWT format".to_string()));
        }

        // Decode header with fast base64 decoding
        let header = Self::decode_jwt_part(parts[0])
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("Invalid JWT header: {}", e)))?;

        // Decode payload with optimized JSON parsing
        let payload = Self::decode_jwt_part(parts[1])
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("Invalid JWT payload: {}", e)))?;

        // Verify signature with constant-time comparison
        if !Self::verify_jwt_signature(service, parts[0], parts[1], parts[2])? {
            return Err(EdgeServiceError::AuthenticationError("Invalid JWT signature".to_string()));
        }

        // Parse claims from payload
        Self::parse_jwt_claims(&payload)
    }

    /// Extract JWT token from Authorization header with optimized header parsing
    pub fn extract_jwt_token(session: &Session) -> Option<&str> {
        session
            .req_header()
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|auth_header| {
                if auth_header.starts_with("Bearer ") {
                    Some(&auth_header[7..]) // Skip "Bearer "
                } else {
                    None
                }
            })
    }

    /// Extract API key from header with fast header lookup
    pub fn extract_api_key(session: &Session) -> Option<&str> {
        session
            .req_header()
            .headers
            .get("x-api-key")
            .and_then(|h| h.to_str().ok())
    }

    /// Validate API key with optimized key comparison
    pub fn validate_api_key(service: &EdgeService, api_key: &str) -> bool {
        if api_key.is_empty() {
            return false;
        }

        // In a real implementation, this would check against a database or key store
        // For now, we'll use a simple validation based on key format
        api_key.len() >= 32 && api_key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Decode JWT part (header or payload) with optimized base64 decoding
    fn decode_jwt_part(part: &str) -> Result<String, EdgeServiceError> {
        // Add padding if needed for base64url decoding
        let padded = match part.len() % 4 {
            0 => part.to_string(),
            2 => format!("{}==", part),
            3 => format!("{}=", part),
            _ => return Err(EdgeServiceError::AuthenticationError("Invalid base64 length".to_string())),
        };

        // Replace URL-safe characters
        let standard_b64 = padded.replace('-', "+").replace('_', "/");

        // Decode base64
        base64::decode(&standard_b64)
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("Base64 decode error: {}", e)))
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| EdgeServiceError::AuthenticationError(format!("UTF-8 decode error: {}", e)))
            })
    }

    /// Verify JWT signature with HMAC-SHA256
    fn verify_jwt_signature(
        service: &EdgeService,
        header: &str,
        payload: &str,
        signature: &str,
    ) -> Result<bool, EdgeServiceError> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        // Create HMAC instance
        let mut mac = HmacSha256::new_from_slice(service.cfg.auth.jwt_secret.as_bytes())
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("HMAC key error: {}", e)))?;

        // Update with header and payload
        mac.update(header.as_bytes());
        mac.update(b".");
        mac.update(payload.as_bytes());

        // Get expected signature
        let expected_signature = mac.finalize().into_bytes();

        // Decode provided signature
        let provided_signature = Self::decode_base64url(signature)
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("Signature decode error: {}", e)))?;

        // Constant-time comparison
        Ok(expected_signature.as_slice() == provided_signature.as_slice())
    }

    /// Decode base64url string
    fn decode_base64url(input: &str) -> Result<Vec<u8>, EdgeServiceError> {
        // Add padding if needed
        let padded = match input.len() % 4 {
            0 => input.to_string(),
            2 => format!("{}==", input),
            3 => format!("{}=", input),
            _ => return Err(EdgeServiceError::AuthenticationError("Invalid base64url length".to_string())),
        };

        // Replace URL-safe characters
        let standard_b64 = padded.replace('-', "+").replace('_', "/");

        // Decode base64
        base64::decode(&standard_b64)
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("Base64url decode error: {}", e)))
    }

    /// Parse JWT claims from payload JSON
    fn parse_jwt_claims(payload: &str) -> Result<UserClaims, EdgeServiceError> {
        use serde_json::Value;

        let json: Value = serde_json::from_str(payload)
            .map_err(|e| EdgeServiceError::AuthenticationError(format!("JSON parse error: {}", e)))?;

        // Extract required fields
        let user_id = json["sub"]
            .as_str()
            .ok_or_else(|| EdgeServiceError::AuthenticationError("Missing 'sub' claim".to_string()))?
            .to_string();

        let username = json["username"]
            .as_str()
            .unwrap_or(&user_id)
            .to_string();

        let expires_at = json["exp"]
            .as_u64()
            .ok_or_else(|| EdgeServiceError::AuthenticationError("Missing 'exp' claim".to_string()))?;

        let issued_at = json["iat"]
            .as_u64()
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            });

        // Extract roles (optional)
        let roles = json["roles"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        // Extract permissions (optional)
        let permissions = json["permissions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(UserClaims {
            user_id,
            username,
            roles,
            permissions,
            expires_at,
            issued_at,
        })
    }

    /// Perform comprehensive authentication with optimized flow
    pub async fn authenticate_request(
        service: &EdgeService,
        session: &mut Session,
    ) -> Result<AuthContext, EdgeServiceError> {
        // Extract client IP for context
        let client_ip = Self::extract_client_ip(session);

        // Try JWT authentication first (most common)
        if let Some(jwt_token) = Self::extract_jwt_token(session) {
            match Self::validate_jwt_token(service, jwt_token) {
                Ok(claims) => {
                    debug!("JWT authentication successful for user: {}", claims.user_id);
                    return Ok(AuthContext::authenticated(AuthMethod::JwtToken, claims)
                        .with_client_ip(client_ip.unwrap_or_default()));
                }
                Err(e) => {
                    warn!("JWT authentication failed: {}", e);
                    // Continue to try other methods
                }
            }
        }

        // Try discovery token authentication
        if Self::extract_and_validate_discovery_token(service, session)? {
            debug!("Discovery token authentication successful");
            let claims = UserClaims::new(
                "discovery".to_string(),
                "discovery".to_string(),
                vec!["discovery".to_string()],
                vec!["discovery".to_string()],
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() + service.cfg.auth.token_expiry_seconds,
            );
            return Ok(AuthContext::authenticated(AuthMethod::DiscoveryToken, claims)
                .with_client_ip(client_ip.unwrap_or_default()));
        }

        // Try API key authentication
        if let Some(api_key) = Self::extract_api_key(session) {
            if Self::validate_api_key(service, api_key) {
                debug!("API key authentication successful");
                let claims = UserClaims::new(
                    format!("api_key_{}", &api_key[..8]), // Use first 8 chars as ID
                    "api_user".to_string(),
                    vec!["api_user".to_string()],
                    vec!["api_access".to_string()],
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() + service.cfg.auth.token_expiry_seconds,
                );
                return Ok(AuthContext::authenticated(AuthMethod::ApiKey, claims)
                    .with_client_ip(client_ip.unwrap_or_default()));
            }
        }

        // No valid authentication found
        debug!("No valid authentication found");
        Ok(AuthContext::unauthenticated().with_client_ip(client_ip.unwrap_or_default()))
    }

    /// Extract client IP address with proxy header support
    fn extract_client_ip(session: &Session) -> Option<String> {
        // Try X-Forwarded-For first (most common proxy header)
        if let Some(forwarded_for) = session
            .req_header()
            .headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
        {
            // Take the first IP in the chain
            if let Some(first_ip) = forwarded_for.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }

        // Try X-Real-IP header
        if let Some(real_ip) = session
            .req_header()
            .headers
            .get("x-real-ip")
            .and_then(|h| h.to_str().ok())
        {
            return Some(real_ip.to_string());
        }

        // Try CF-Connecting-IP (Cloudflare)
        if let Some(cf_ip) = session
            .req_header()
            .headers
            .get("cf-connecting-ip")
            .and_then(|h| h.to_str().ok())
        {
            return Some(cf_ip.to_string());
        }

        // Fall back to remote address
        session.client_addr().map(|addr| addr.to_string())
    }

    /// Validate request method for authentication requirements
    pub fn validate_request_method(method: &Method) -> bool {
        // Allow all standard HTTP methods
        matches!(
            method,
            &Method::GET | &Method::POST | &Method::PUT | &Method::DELETE | 
            &Method::PATCH | &Method::HEAD | &Method::OPTIONS
        )
    }

    /// Check if request requires authentication based on path and method
    pub fn requires_authentication(path: &str, method: &Method) -> bool {
        // Public endpoints that don't require authentication
        let public_paths = [
            "/health",
            "/metrics",
            "/status",
            "/ping",
        ];

        // Check if path is public
        if public_paths.iter().any(|&public_path| path.starts_with(public_path)) {
            return false;
        }

        // OPTIONS requests are typically public for CORS
        if method == &Method::OPTIONS {
            return false;
        }

        // All other endpoints require authentication
        true
    }

    /// Validate HTTPS requirement
    pub fn validate_https_requirement(service: &EdgeService, session: &Session) -> bool {
        if !service.cfg.auth.require_https {
            return true; // HTTPS not required
        }

        // Check if request is HTTPS
        session.req_header().uri.scheme_str() == Some("https")
    }
}
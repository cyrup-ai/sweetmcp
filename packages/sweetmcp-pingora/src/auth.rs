//! JWT Authentication and RBAC for SweetMCP Server

use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Expiration timestamp
    pub exp: i64,

    /// Issued at timestamp
    pub iat: i64,

    /// JWT ID for tracking
    pub jti: String,

    /// User roles for RBAC
    pub roles: Vec<String>,

    /// Additional permissions
    pub permissions: Vec<String>,

    /// Session metadata
    pub session_id: String,
}

/// Available roles in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Role {
    /// Administrator with full access
    Admin,

    /// User with standard MCP access
    User,

    /// Service account for automated systems
    Service,

    /// Read-only access
    ReadOnly,
}

#[allow(dead_code)]
impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
            Role::Service => "service",
            Role::ReadOnly => "readonly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "user" => Some(Role::User),
            "service" => Some(Role::Service),
            "readonly" => Some(Role::ReadOnly),
            _ => None,
        }
    }
}

/// Available permissions for fine-grained access control
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum Permission {
    /// Access to MCP tools
    ToolsAccess,

    /// Access to MCP resources
    ResourcesAccess,

    /// Access to MCP prompts
    PromptsAccess,

    /// Admin operations
    AdminAccess,

    /// Metrics access
    MetricsAccess,

    /// Health check access
    HealthAccess,
}

#[allow(dead_code)]
impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ToolsAccess => "tools:access",
            Permission::ResourcesAccess => "resources:access",
            Permission::PromptsAccess => "prompts:access",
            Permission::AdminAccess => "admin:access",
            Permission::MetricsAccess => "metrics:access",
            Permission::HealthAccess => "health:access",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tools:access" => Some(Permission::ToolsAccess),
            "resources:access" => Some(Permission::ResourcesAccess),
            "prompts:access" => Some(Permission::PromptsAccess),
            "admin:access" => Some(Permission::AdminAccess),
            "metrics:access" => Some(Permission::MetricsAccess),
            "health:access" => Some(Permission::HealthAccess),
            _ => None,
        }
    }
}

/// JWT authentication handler
#[allow(dead_code)]
pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    expiry_duration: Duration,
}

#[allow(dead_code)]
impl JwtAuth {
    /// Create a new JWT authentication handler
    pub fn new(secret: Arc<[u8; 32]>, expiry_duration: Duration) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 60; // 60 seconds leeway for clock skew

        Self {
            encoding_key,
            decoding_key,
            validation,
            expiry_duration,
        }
    }

    /// Generate a new JWT token for a user
    pub fn generate_token(
        &self,
        user_id: &str,
        roles: Vec<Role>,
        permissions: Vec<Permission>,
    ) -> Result<String> {
        let now = OffsetDateTime::now_utc();
        let exp = now + self.expiry_duration;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
            jti: Uuid::new_v4().to_string(),
            roles: roles.into_iter().map(|r| r.as_str().to_string()).collect(),
            permissions: permissions
                .into_iter()
                .map(|p| p.as_str().to_string())
                .collect(),
            session_id: Uuid::new_v4().to_string(),
        };

        let header = Header::new(Algorithm::HS256);

        encode(&header, &claims, &self.encoding_key).context("Failed to encode JWT token")
    }

    /// Verify and decode a JWT token from Authorization header
    pub fn verify(&self, auth_header: &str) -> Result<Claims> {
        let token = auth_header
            .strip_prefix("Bearer ")
            .context("Authorization header must start with 'Bearer '")?;

        debug!("Verifying JWT token");

        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .context("Invalid JWT token")?;

        debug!("JWT token verified for user: {}", token_data.claims.sub);

        Ok(token_data.claims)
    }

    /// Check if claims have required role
    pub fn has_role(&self, claims: &Claims, required_role: &Role) -> bool {
        claims
            .roles
            .iter()
            .any(|role| Role::from_str(role).map_or(false, |r| r == *required_role))
    }

    /// Check if claims have required permission
    pub fn has_permission(&self, claims: &Claims, required_permission: &Permission) -> bool {
        claims
            .permissions
            .iter()
            .any(|perm| Permission::from_str(perm).map_or(false, |p| p == *required_permission))
    }

    /// Check if claims have any of the required permissions
    pub fn has_any_permission(&self, claims: &Claims, required_permissions: &[Permission]) -> bool {
        required_permissions
            .iter()
            .any(|req_perm| self.has_permission(claims, req_perm))
    }

    /// Get default permissions for a role
    pub fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        match role {
            Role::Admin => vec![
                Permission::ToolsAccess,
                Permission::ResourcesAccess,
                Permission::PromptsAccess,
                Permission::AdminAccess,
                Permission::MetricsAccess,
                Permission::HealthAccess,
            ],
            Role::User => vec![
                Permission::ToolsAccess,
                Permission::ResourcesAccess,
                Permission::PromptsAccess,
                Permission::HealthAccess,
            ],
            Role::Service => vec![
                Permission::ToolsAccess,
                Permission::ResourcesAccess,
                Permission::MetricsAccess,
                Permission::HealthAccess,
            ],
            Role::ReadOnly => vec![Permission::HealthAccess, Permission::MetricsAccess],
        }
    }
}

/// Authorization context for request processing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub roles: HashSet<Role>,
    pub permissions: HashSet<Permission>,
    pub jwt_id: String,
}

#[allow(dead_code)]
impl AuthContext {
    pub fn from_claims(claims: Claims) -> Self {
        let roles: HashSet<Role> = claims
            .roles
            .iter()
            .filter_map(|r| Role::from_str(r))
            .collect();

        let permissions: HashSet<Permission> = claims
            .permissions
            .iter()
            .filter_map(|p| Permission::from_str(p))
            .collect();

        Self {
            user_id: claims.sub,
            session_id: claims.session_id,
            roles,
            permissions,
            jwt_id: claims.jti,
        }
    }

    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.permissions.contains(p))
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }
}

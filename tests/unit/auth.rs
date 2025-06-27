use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use sweetmcp::auth::{JwtAuth, Claims, AuthContext, Role, Permission};

#[test]
fn test_jwt_auth_comprehensive() -> Result<()> {
    let secret = Arc::new([0u8; 32]);
    let auth = JwtAuth::new(secret, Duration::from_secs(3600));
    
    let token = auth.generate_token(
        "test_user",
        vec![Role::User],
        vec![Permission::ToolsAccess],
    )?;
    
    let claims = auth.verify(&format!("Bearer {}", token))?;
    assert_eq!(claims.sub, "test_user");
    assert!(auth.has_role(&claims, &Role::User));
    assert!(auth.has_permission(&claims, &Permission::ToolsAccess));
    
    Ok(())
}

#[test]
fn test_role_permissions() {
    let secret = Arc::new([0u8; 32]);
    let auth = JwtAuth::new(secret, Duration::from_secs(3600));
    
    let admin_perms = auth.get_role_permissions(&Role::Admin);
    assert!(admin_perms.contains(&Permission::AdminAccess));
    
    let user_perms = auth.get_role_permissions(&Role::User);
    assert!(!user_perms.contains(&Permission::AdminAccess));
    assert!(user_perms.contains(&Permission::ToolsAccess));
}

#[test]
fn test_auth_context() {
    let claims = Claims {
        sub: "test_user".to_string(),
        exp: (OffsetDateTime::now_utc() + Duration::from_secs(3600)).unix_timestamp(),
        iat: OffsetDateTime::now_utc().unix_timestamp(),
        jti: "test_jti".to_string(),
        roles: vec!["admin".to_string()],
        permissions: vec!["admin:access".to_string()],
        session_id: "test_session".to_string(),
    };
    
    let auth_ctx = AuthContext::from_claims(claims);
    assert!(auth_ctx.is_admin());
    assert!(auth_ctx.has_permission(&Permission::AdminAccess));
}
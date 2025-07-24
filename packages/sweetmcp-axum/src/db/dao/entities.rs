//! Base entities and example implementations
//!
//! This module provides base entity structures and example implementations
//! with zero allocation patterns and blazing-fast performance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::core::{Entity, utc_now};

/// Common fields for database entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    /// Entity ID
    pub id: Option<String>,

    /// Creation timestamp
    #[serde(default = "utc_now")]
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

impl BaseEntity {
    /// Create a new entity
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a new entity with a specific ID
    pub fn with_id(id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Some(id),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if the entity is new (no ID)
    pub fn is_new(&self) -> bool {
        self.id.is_none()
    }

    /// Check if the entity is persisted (has ID)
    pub fn is_persisted(&self) -> bool {
        self.id.is_some()
    }

    /// Get the age of the entity in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get the time since last update in seconds
    pub fn seconds_since_update(&self) -> i64 {
        (Utc::now() - self.updated_at).num_seconds()
    }

    /// Check if the entity was recently created (within specified seconds)
    pub fn is_recently_created(&self, seconds: i64) -> bool {
        self.age_seconds() <= seconds
    }

    /// Check if the entity was recently updated (within specified seconds)
    pub fn is_recently_updated(&self, seconds: i64) -> bool {
        self.seconds_since_update() <= seconds
    }

    /// Clone the entity and mark it as new (remove ID and update timestamps)
    pub fn clone_as_new(&self) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Prepare for update (update the updated_at timestamp)
    pub fn prepare_for_update(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl Default for BaseEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Example user entity implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(flatten)]
    pub base: BaseEntity,

    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub profile: UserProfile,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
}

impl Entity for User {
    fn table_name() -> &'static str {
        "users"
    }

    fn id(&self) -> Option<String> {
        self.base.id.clone()
    }

    fn set_id(&mut self, id: String) {
        self.base.id = Some(id);
    }
}

impl User {
    /// Create a new user
    pub fn new(username: String, email: String) -> Self {
        Self {
            base: BaseEntity::new(),
            username,
            email,
            password_hash: None,
            is_active: true,
            last_login: None,
            profile: UserProfile::default(),
        }
    }

    /// Create a new user with password
    pub fn with_password(username: String, email: String, password_hash: String) -> Self {
        Self {
            base: BaseEntity::new(),
            username,
            email,
            password_hash: Some(password_hash),
            is_active: true,
            last_login: None,
            profile: UserProfile::default(),
        }
    }

    /// Set the password hash
    pub fn set_password_hash(&mut self, password_hash: String) {
        self.password_hash = Some(password_hash);
        self.base.prepare_for_update();
    }

    /// Clear the password hash
    pub fn clear_password(&mut self) {
        self.password_hash = None;
        self.base.prepare_for_update();
    }

    /// Check if user has a password set
    pub fn has_password(&self) -> bool {
        self.password_hash.is_some()
    }

    /// Activate the user
    pub fn activate(&mut self) {
        self.is_active = true;
        self.base.prepare_for_update();
    }

    /// Deactivate the user
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.base.prepare_for_update();
    }

    /// Record a login
    pub fn record_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.base.prepare_for_update();
    }

    /// Check if user has ever logged in
    pub fn has_logged_in(&self) -> bool {
        self.last_login.is_some()
    }

    /// Get days since last login
    pub fn days_since_last_login(&self) -> Option<i64> {
        self.last_login.map(|login_time| {
            (Utc::now() - login_time).num_days()
        })
    }

    /// Check if user is recently active (logged in within specified days)
    pub fn is_recently_active(&self, days: i64) -> bool {
        match self.days_since_last_login() {
            Some(days_since) => days_since <= days,
            None => false, // Never logged in
        }
    }

    /// Update user profile
    pub fn update_profile(&mut self, profile: UserProfile) {
        self.profile = profile;
        self.base.prepare_for_update();
    }

    /// Set display name
    pub fn set_display_name(&mut self, display_name: Option<String>) {
        self.profile.display_name = display_name;
        self.base.prepare_for_update();
    }

    /// Get the user's display name or username as fallback
    pub fn get_display_name(&self) -> &str {
        self.profile.display_name.as_deref().unwrap_or(&self.username)
    }

    /// Get the user's full name if available
    pub fn get_full_name(&self) -> Option<String> {
        match (&self.profile.first_name, &self.profile.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        }
    }

    /// Check if user profile is complete
    pub fn is_profile_complete(&self) -> bool {
        self.profile.first_name.is_some() 
            && self.profile.last_name.is_some()
            && self.profile.display_name.is_some()
    }

    /// Validate user data
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.username.trim().is_empty() {
            errors.push("Username cannot be empty".to_string());
        }

        if self.username.len() < 3 {
            errors.push("Username must be at least 3 characters long".to_string());
        }

        if self.email.trim().is_empty() {
            errors.push("Email cannot be empty".to_string());
        }

        if !self.email.contains('@') {
            errors.push("Email must be a valid email address".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Create a sanitized version for public display (remove sensitive data)
    pub fn sanitize(&self) -> PublicUser {
        PublicUser {
            id: self.base.id.clone(),
            username: self.username.clone(),
            display_name: self.get_display_name().to_string(),
            is_active: self.is_active,
            created_at: self.base.created_at,
            last_login: self.last_login,
            profile: PublicUserProfile {
                display_name: self.profile.display_name.clone(),
                bio: self.profile.bio.clone(),
                avatar_url: self.profile.avatar_url.clone(),
            },
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            base: BaseEntity::new(),
            username: String::new(),
            email: String::new(),
            password_hash: None,
            is_active: true,
            last_login: None,
            profile: UserProfile::default(),
        }
    }
}

impl UserProfile {
    /// Create a new empty profile
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a profile with basic information
    pub fn with_names(first_name: String, last_name: String) -> Self {
        Self {
            first_name: Some(first_name.clone()),
            last_name: Some(last_name.clone()),
            display_name: Some(format!("{} {}", first_name, last_name)),
            bio: None,
            avatar_url: None,
            timezone: None,
            locale: None,
        }
    }

    /// Check if the profile has any information
    pub fn is_empty(&self) -> bool {
        self.first_name.is_none()
            && self.last_name.is_none()
            && self.display_name.is_none()
            && self.bio.is_none()
            && self.avatar_url.is_none()
            && self.timezone.is_none()
            && self.locale.is_none()
    }

    /// Get completion percentage (0.0 to 1.0)
    pub fn completion_percentage(&self) -> f32 {
        let total_fields = 7.0;
        let mut filled_fields = 0.0;

        if self.first_name.is_some() { filled_fields += 1.0; }
        if self.last_name.is_some() { filled_fields += 1.0; }
        if self.display_name.is_some() { filled_fields += 1.0; }
        if self.bio.is_some() { filled_fields += 1.0; }
        if self.avatar_url.is_some() { filled_fields += 1.0; }
        if self.timezone.is_some() { filled_fields += 1.0; }
        if self.locale.is_some() { filled_fields += 1.0; }

        filled_fields / total_fields
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            first_name: None,
            last_name: None,
            display_name: None,
            bio: None,
            avatar_url: None,
            timezone: None,
            locale: None,
        }
    }
}

/// Public user representation (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: Option<String>,
    pub username: String,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub profile: PublicUserProfile,
}

/// Public user profile (without sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUserProfile {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}
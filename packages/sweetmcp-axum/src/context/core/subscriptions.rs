//! Context subscription management
//!
//! This module provides context subscription functionality for managing
//! context change notifications and subscriptions with zero allocation
//! patterns and blazing-fast performance.

use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

/// Context subscription information
#[derive(Debug, Clone)]
pub struct ContextSubscription {
    /// Unique subscription identifier
    pub id: String,
    /// Scopes that this subscription monitors
    pub scopes: Vec<String>,
    /// When the subscription was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Global context subscription storage
pub static CONTEXT_SUBSCRIPTIONS: Lazy<Arc<RwLock<HashMap<String, ContextSubscription>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

impl ContextSubscription {
    /// Create a new context subscription
    pub fn new(id: String, scopes: Vec<String>) -> Self {
        Self {
            id,
            scopes,
            created_at: chrono::Utc::now(),
        }
    }

    /// Check if subscription matches a scope
    pub fn matches_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope || scope.starts_with(&format!("{}.", s)))
    }

    /// Check if subscription matches any of the provided scopes
    pub fn matches_any_scope(&self, scopes: &[String]) -> bool {
        scopes.iter().any(|scope| self.matches_scope(scope))
    }

    /// Get subscription age in seconds
    pub fn age_seconds(&self) -> i64 {
        (chrono::Utc::now() - self.created_at).num_seconds()
    }

    /// Check if subscription is expired (older than specified duration)
    pub fn is_expired(&self, max_age_seconds: i64) -> bool {
        self.age_seconds() > max_age_seconds
    }

    /// Update scopes for this subscription
    pub fn update_scopes(&mut self, new_scopes: Vec<String>) {
        self.scopes = new_scopes;
    }

    /// Add scope to subscription
    pub fn add_scope(&mut self, scope: String) {
        if !self.scopes.contains(&scope) {
            self.scopes.push(scope);
        }
    }

    /// Remove scope from subscription
    pub fn remove_scope(&mut self, scope: &str) -> bool {
        if let Some(pos) = self.scopes.iter().position(|s| s == scope) {
            self.scopes.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if subscription has any scopes
    pub fn has_scopes(&self) -> bool {
        !self.scopes.is_empty()
    }

    /// Get scope count
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }
}

/// Context subscription manager
pub struct ContextSubscriptionManager;

impl ContextSubscriptionManager {
    /// Add a new context subscription
    pub async fn add_subscription(subscription: ContextSubscription) -> Result<(), String> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        
        // Check if subscription already exists
        if subscriptions.contains_key(&subscription.id) {
            return Err(format!("Subscription with ID '{}' already exists", subscription.id));
        }
        
        subscriptions.insert(subscription.id.clone(), subscription);
        Ok(())
    }

    /// Remove a context subscription
    pub async fn remove_subscription(subscription_id: &str) -> Option<ContextSubscription> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        subscriptions.remove(subscription_id)
    }

    /// Get a context subscription by ID
    pub async fn get_subscription(subscription_id: &str) -> Option<ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions.get(subscription_id).cloned()
    }

    /// Get all context subscriptions
    pub async fn get_all_subscriptions() -> HashMap<String, ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions.clone()
    }

    /// Get subscriptions that match a specific scope
    pub async fn get_subscriptions_for_scope(scope: &str) -> Vec<ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.matches_scope(scope))
            .cloned()
            .collect()
    }

    /// Get subscriptions that match any of the provided scopes
    pub async fn get_subscriptions_for_scopes(scopes: &[String]) -> Vec<ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.matches_any_scope(scopes))
            .cloned()
            .collect()
    }

    /// Update subscription scopes
    pub async fn update_subscription_scopes(
        subscription_id: &str,
        new_scopes: Vec<String>,
    ) -> Result<(), String> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        
        if let Some(subscription) = subscriptions.get_mut(subscription_id) {
            subscription.update_scopes(new_scopes);
            Ok(())
        } else {
            Err(format!("Subscription with ID '{}' not found", subscription_id))
        }
    }

    /// Add scope to existing subscription
    pub async fn add_scope_to_subscription(
        subscription_id: &str,
        scope: String,
    ) -> Result<(), String> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        
        if let Some(subscription) = subscriptions.get_mut(subscription_id) {
            subscription.add_scope(scope);
            Ok(())
        } else {
            Err(format!("Subscription with ID '{}' not found", subscription_id))
        }
    }

    /// Remove scope from existing subscription
    pub async fn remove_scope_from_subscription(
        subscription_id: &str,
        scope: &str,
    ) -> Result<bool, String> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        
        if let Some(subscription) = subscriptions.get_mut(subscription_id) {
            Ok(subscription.remove_scope(scope))
        } else {
            Err(format!("Subscription with ID '{}' not found", subscription_id))
        }
    }

    /// Check if subscription exists
    pub async fn subscription_exists(subscription_id: &str) -> bool {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions.contains_key(subscription_id)
    }

    /// Get subscription count
    pub async fn subscription_count() -> usize {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions.len()
    }

    /// Clear all subscriptions
    pub async fn clear_all_subscriptions() -> usize {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        let count = subscriptions.len();
        subscriptions.clear();
        count
    }

    /// Remove expired subscriptions
    pub async fn remove_expired_subscriptions(max_age_seconds: i64) -> Vec<String> {
        let mut subscriptions = CONTEXT_SUBSCRIPTIONS.write().await;
        let mut removed_ids = Vec::new();
        
        subscriptions.retain(|id, subscription| {
            if subscription.is_expired(max_age_seconds) {
                removed_ids.push(id.clone());
                false
            } else {
                true
            }
        });
        
        removed_ids
    }

    /// Get subscription statistics
    pub async fn get_subscription_stats() -> SubscriptionStats {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        
        let mut scope_counts = HashMap::new();
        let mut total_scopes = 0;
        let mut oldest_subscription = None;
        let mut newest_subscription = None;
        
        for subscription in subscriptions.values() {
            total_scopes += subscription.scopes.len();
            
            // Count scopes
            for scope in &subscription.scopes {
                *scope_counts.entry(scope.clone()).or_insert(0) += 1;
            }
            
            // Track oldest and newest
            if oldest_subscription.is_none() || subscription.created_at < oldest_subscription.unwrap() {
                oldest_subscription = Some(subscription.created_at);
            }
            
            if newest_subscription.is_none() || subscription.created_at > newest_subscription.unwrap() {
                newest_subscription = Some(subscription.created_at);
            }
        }
        
        SubscriptionStats {
            total_subscriptions: subscriptions.len(),
            total_scopes,
            unique_scopes: scope_counts.len(),
            scope_counts,
            oldest_subscription,
            newest_subscription,
        }
    }

    /// Validate all subscriptions
    pub async fn validate_subscriptions() -> Vec<String> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        let mut errors = Vec::new();
        
        for (id, subscription) in subscriptions.iter() {
            // Check for empty ID
            if id.trim().is_empty() {
                errors.push("Found subscription with empty ID".to_string());
            }
            
            // Check for empty scopes
            if subscription.scopes.is_empty() {
                errors.push(format!("Subscription '{}' has no scopes", id));
            }
            
            // Check for invalid scope names
            for scope in &subscription.scopes {
                if scope.trim().is_empty() {
                    errors.push(format!("Subscription '{}' has empty scope", id));
                }
                
                if scope.contains("..") {
                    errors.push(format!("Subscription '{}' has invalid scope: '{}'", id, scope));
                }
            }
            
            // Check for future creation time
            if subscription.created_at > chrono::Utc::now() {
                errors.push(format!("Subscription '{}' has future creation time", id));
            }
        }
        
        errors
    }

    /// Get subscriptions created within a time range
    pub async fn get_subscriptions_in_time_range(
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.created_at >= start && sub.created_at <= end)
            .cloned()
            .collect()
    }

    /// Get subscriptions older than specified age
    pub async fn get_old_subscriptions(max_age_seconds: i64) -> Vec<ContextSubscription> {
        let subscriptions = CONTEXT_SUBSCRIPTIONS.read().await;
        subscriptions
            .values()
            .filter(|sub| sub.is_expired(max_age_seconds))
            .cloned()
            .collect()
    }
}

/// Subscription statistics
#[derive(Debug, Clone)]
pub struct SubscriptionStats {
    /// Total number of subscriptions
    pub total_subscriptions: usize,
    /// Total number of scopes across all subscriptions
    pub total_scopes: usize,
    /// Number of unique scopes
    pub unique_scopes: usize,
    /// Count of subscriptions per scope
    pub scope_counts: HashMap<String, usize>,
    /// Creation time of oldest subscription
    pub oldest_subscription: Option<chrono::DateTime<chrono::Utc>>,
    /// Creation time of newest subscription
    pub newest_subscription: Option<chrono::DateTime<chrono::Utc>>,
}
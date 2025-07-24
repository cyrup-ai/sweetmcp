//! Global context management
//!
//! This module provides global context initialization and management
//! functionality with zero allocation patterns and blazing-fast performance.

use std::sync::Arc;
use once_cell::sync::OnceCell;

use super::types::{ApplicationContext, SamplingContext};

/// Global application context initialized at runtime
pub static APPLICATION_CONTEXT: OnceCell<ApplicationContext> = OnceCell::new();

/// Global sampling context initialized at runtime
pub static SAMPLING_CONTEXT: OnceCell<SamplingContext> = OnceCell::new();

/// Global context manager for initialization and access
pub struct GlobalContextManager;

impl GlobalContextManager {
    /// Initialize the global application context
    /// Must be called once during application startup
    pub async fn initialize_global_context(
        config_path: &std::path::Path,
        log_level: &str,
        plugin_configs: &[crate::config::PluginConfig],
    ) -> Result<(), anyhow::Error> {
        let config_path = config_path.to_path_buf();
        let log_level = log_level.to_string();
        let plugin_configs = plugin_configs.to_vec();

        // Initialize the application context
        let app_context = ApplicationContext::initialize(&config_path, &log_level, &plugin_configs)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to initialize application context"))?;

        // Set the global instance
        APPLICATION_CONTEXT
            .set(app_context)
            .map_err(|_| anyhow::anyhow!("Global application context already initialized"))?;

        // Get the app context after it's been initialized
        let app_context = APPLICATION_CONTEXT
            .get()
            .ok_or_else(|| anyhow::anyhow!("Application context not initialized"))?;

        // Initialize the sampling context
        let sampling_context = SamplingContext::new(app_context.clone());
        SAMPLING_CONTEXT
            .set(sampling_context)
            .map_err(|_| anyhow::anyhow!("Global sampling context already initialized"))?;

        Ok(())
    }

    /// Get the global application context
    pub fn get_application_context() -> Option<&'static ApplicationContext> {
        APPLICATION_CONTEXT.get()
    }

    /// Get the global sampling context
    pub fn get_sampling_context() -> Option<&'static SamplingContext> {
        SAMPLING_CONTEXT.get()
    }

    /// Check if global contexts are initialized
    pub fn are_contexts_initialized() -> bool {
        APPLICATION_CONTEXT.get().is_some() && SAMPLING_CONTEXT.get().is_some()
    }

    /// Check if application context is initialized
    pub fn is_application_context_initialized() -> bool {
        APPLICATION_CONTEXT.get().is_some()
    }

    /// Check if sampling context is initialized
    pub fn is_sampling_context_initialized() -> bool {
        SAMPLING_CONTEXT.get().is_some()
    }

    /// Get application context with error handling
    pub fn require_application_context() -> Result<&'static ApplicationContext, anyhow::Error> {
        APPLICATION_CONTEXT
            .get()
            .ok_or_else(|| anyhow::anyhow!("Application context not initialized"))
    }

    /// Get sampling context with error handling
    pub fn require_sampling_context() -> Result<&'static SamplingContext, anyhow::Error> {
        SAMPLING_CONTEXT
            .get()
            .ok_or_else(|| anyhow::anyhow!("Sampling context not initialized"))
    }

    /// Validate global context integrity
    pub async fn validate_global_contexts() -> Result<Vec<String>, anyhow::Error> {
        let mut errors = Vec::new();

        // Validate application context
        if let Some(app_context) = APPLICATION_CONTEXT.get() {
            let app_errors = app_context.validate().await;
            errors.extend(app_errors);
        } else {
            errors.push("Application context not initialized".to_string());
        }

        // Validate sampling context
        if SAMPLING_CONTEXT.get().is_none() {
            errors.push("Sampling context not initialized".to_string());
        }

        Ok(errors)
    }

    /// Get global context statistics
    pub async fn get_global_context_stats() -> Result<GlobalContextStats, anyhow::Error> {
        let app_context = Self::require_application_context()?;
        let sampling_context = Self::require_sampling_context()?;

        let app_stats = app_context.get_stats().await;
        let sampling_stats = sampling_context.get_stats().await;

        Ok(GlobalContextStats {
            application_initialized: true,
            sampling_initialized: true,
            app_stats,
            sampling_stats,
        })
    }

    /// Reinitialize contexts (for testing or recovery)
    pub async fn reinitialize_contexts(
        config_path: &std::path::Path,
        log_level: &str,
        plugin_configs: &[crate::config::PluginConfig],
    ) -> Result<(), anyhow::Error> {
        // Note: OnceCell doesn't support reinitialization in production
        // This would require using a different synchronization primitive
        // For now, this is a placeholder for potential future functionality
        Err(anyhow::anyhow!(
            "Context reinitialization not supported with OnceCell. Restart application instead."
        ))
    }

    /// Shutdown contexts gracefully (cleanup)
    pub async fn shutdown_contexts() -> Result<(), anyhow::Error> {
        // Perform cleanup operations before shutdown
        if let Some(app_context) = APPLICATION_CONTEXT.get() {
            // Cleanup plugin manager
            if let Err(e) = app_context.plugin_manager().shutdown().await {
                log::warn!("Plugin manager shutdown error: {}", e);
            }

            // Cleanup memory adapter
            if let Err(e) = app_context.memory_adapter().shutdown().await {
                log::warn!("Memory adapter shutdown error: {}", e);
            }

            log::info!("Application context shutdown completed");
        }

        if let Some(sampling_context) = SAMPLING_CONTEXT.get() {
            // Clear active sessions
            sampling_context.clear_active_sessions().await;
            
            // Clear sampling configuration
            sampling_context.clear_sampling_config().await;

            log::info!("Sampling context shutdown completed");
        }

        Ok(())
    }
}

/// Convenience functions for global context access
pub mod context_access {
    use super::*;

    /// Get application context (panics if not initialized)
    pub fn app_context() -> &'static ApplicationContext {
        APPLICATION_CONTEXT
            .get()
            .expect("Application context not initialized")
    }

    /// Get sampling context (panics if not initialized)
    pub fn sampling_context() -> &'static SamplingContext {
        SAMPLING_CONTEXT
            .get()
            .expect("Sampling context not initialized")
    }

    /// Try to get application context (returns None if not initialized)
    pub fn try_app_context() -> Option<&'static ApplicationContext> {
        APPLICATION_CONTEXT.get()
    }

    /// Try to get sampling context (returns None if not initialized)
    pub fn try_sampling_context() -> Option<&'static SamplingContext> {
        SAMPLING_CONTEXT.get()
    }

    /// Get logger from application context
    pub fn logger() -> Arc<crate::context::logger::ConsoleLogger> {
        app_context().logger().clone()
    }

    /// Get plugin manager from application context
    pub fn plugin_manager() -> Arc<crate::plugin::PluginManager> {
        app_context().plugin_manager().clone()
    }

    /// Get memory adapter from application context
    pub fn memory_adapter() -> Arc<crate::context::MemoryContextAdapter> {
        app_context().memory_adapter().clone()
    }

    /// Check if database is available
    pub fn is_database_available() -> bool {
        try_app_context()
            .map(|ctx| ctx.is_database_initialized())
            .unwrap_or(false)
    }
}

/// Global context statistics
#[derive(Debug, Clone)]
pub struct GlobalContextStats {
    /// Whether application context is initialized
    pub application_initialized: bool,
    /// Whether sampling context is initialized
    pub sampling_initialized: bool,
    /// Application context statistics
    pub app_stats: super::types::ContextStats,
    /// Sampling context statistics
    pub sampling_stats: super::types::SamplingStats,
}

/// Initialize global context (convenience function)
pub async fn initialize_global_context(
    config_path: &std::path::Path,
    log_level: &str,
    plugin_configs: &[crate::config::PluginConfig],
) -> Result<(), anyhow::Error> {
    GlobalContextManager::initialize_global_context(config_path, log_level, plugin_configs).await
}
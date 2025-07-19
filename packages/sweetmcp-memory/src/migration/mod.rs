//! Migration module for mem0-rs
//!
//! This module provides functionality for data migration, import/export,
//! and schema evolution for the memory system.

pub mod converter;
pub mod exporter;
pub mod importer;
pub mod schema_migrations;
pub mod validator;

// Re-export main types
pub use converter::*;
pub use exporter::*;
pub use importer::*;
pub use schema_migrations::*;
pub use validator::*;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

/// Result type for migration operations
pub type Result<T> = std::result::Result<T, MigrationError>;

/// Migration error types
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Schema mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Migration direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    Up,
    Down,
}

/// A pending migration operation that can be awaited
pub struct PendingMigration {
    rx: oneshot::Receiver<Result<()>>,
}

impl PendingMigration {
    pub fn new(rx: oneshot::Receiver<Result<()>>) -> Self {
        Self { rx }
    }
}

impl Future for PendingMigration {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll(cx) {
            Poll::Ready(Ok(result)) => Poll::Ready(result),
            Poll::Ready(Err(_)) => Poll::Ready(Err(MigrationError::DatabaseError(
                "Migration task failed".to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Migration trait
pub trait Migration: Send + Sync {
    /// Get the migration version
    fn version(&self) -> u32;

    /// Get the migration name
    fn name(&self) -> &str;

    /// Apply the migration
    fn up(&self) -> PendingMigration;

    /// Rollback the migration
    fn down(&self) -> PendingMigration;
}

/// Migration manager
pub struct MigrationManager {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    /// Add a migration
    pub fn add_migration(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
    }

    /// Run pending migrations
    pub fn migrate(&self) -> PendingMigration {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Implementation would check current version and apply pending migrations
            let _ = tx.send(Ok(()));
        });

        PendingMigration::new(rx)
    }

    /// Rollback to a specific version
    pub fn rollback_to(&self, _version: u32) -> PendingMigration {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            // Implementation would rollback migrations to reach target version
            let _ = tx.send(Ok(()));
        });

        PendingMigration::new(rx)
    }
}

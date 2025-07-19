use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Configuration error
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}

impl Error {
    /// Create a validation error
    pub fn validation<T: Into<String>>(message: T) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

/// Storage engine for SurrealDB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageEngine {
    /// SurrealKV storage (recommended for local apps)
    SurrealKv,
    /// HTTP connection to a remote SurrealDB server
    Http,
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::SurrealKv
    }
}

impl std::fmt::Display for StorageEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SurrealKv => write!(f, "surrealkv"),
            Self::Http => write!(f, "http"),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Storage engine
    pub engine: StorageEngine,

    /// Path to database (for file-based storage)
    pub path: Option<String>,

    /// URL for remote connections
    pub url: Option<String>,

    /// Namespace
    pub namespace: Option<String>,

    /// Database
    pub database: Option<String>,

    /// Username
    pub username: Option<String>,

    /// Password
    pub password: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            engine: StorageEngine::default(),
            path: Some("./data/surreal.db".to_string()),
            url: None,
            namespace: default_namespace(),
            database: default_database(),
            username: None,
            password: None,
        }
    }
}

/// Default namespace
fn default_namespace() -> Option<String> {
    Some("test".to_string())
}

/// Default database
fn default_database() -> Option<String> {
    Some("test".to_string())
}

impl DatabaseConfig {
    /// Create a new database configuration for local development
    pub fn local<P: Into<String>>(path: P) -> Self {
        Self {
            engine: StorageEngine::SurrealKv,
            path: Some(path.into()),
            url: None,
            namespace: default_namespace(),
            database: default_database(),
            username: None,
            password: None,
        }
    }

    /// Create a new database configuration for SurrealKV
    pub fn surrealkv<P, U, PW>(path: P, username: Option<U>, password: Option<PW>) -> Self
    where
        P: Into<String>,
        U: Into<String>,
        PW: Into<String>,
    {
        Self {
            engine: StorageEngine::SurrealKv,
            path: Some(path.into()),
            url: None,
            namespace: default_namespace(),
            database: default_database(),
            username: username.map(|u| u.into()),
            password: password.map(|p| p.into()),
        }
    }

    /// Create a new database configuration for HTTP
    pub fn http<U, UN, PW>(url: U, username: Option<UN>, password: Option<PW>) -> Self
    where
        U: Into<String>,
        UN: Into<String>,
        PW: Into<String>,
    {
        Self {
            engine: StorageEngine::Http,
            path: None,
            url: Some(url.into()),
            namespace: default_namespace(),
            database: default_database(),
            username: username.map(|u| u.into()),
            password: password.map(|p| p.into()),
        }
    }

    /// Set the namespace
    pub fn with_namespace<T: Into<String>>(mut self, namespace: T) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Set the database
    pub fn with_database<T: Into<String>>(mut self, database: T) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Set the credentials
    pub fn with_credentials<U, P>(mut self, username: U, password: P) -> Self
    where
        U: Into<String>,
        P: Into<String>,
    {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }
}

/// Metrics configuration for the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to collect metrics
    pub enabled: bool,

    /// Prefix for metric names
    pub prefix: String,

    /// Whether to include query execution time
    pub query_timing: bool,

    /// Whether to collect table-level metrics
    pub table_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "surrealdb".to_string(),
            query_timing: true,
            table_metrics: true,
        }
    }
}

impl DatabaseConfig {
    /// Ensures the database directory exists (for file-based storage)
    pub fn ensure_db_dir(&self) -> std::io::Result<()> {
        if self.engine == StorageEngine::SurrealKv {
            if let Some(path_str) = &self.path {
                if let Some(parent) = Path::new(path_str).parent() {
                    debug!("Ensuring database directory exists: {}", parent.display());
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        Ok(())
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), Error> {
        // Check fields based on storage engine type
        match self.engine {
            StorageEngine::SurrealKv => {
                // Local file-based engine requires path
                if self.path.is_none() || self.path.as_ref().unwrap().is_empty() {
                    let err_msg = format!("Path is required for {} storage engine", self.engine);
                    return Err(Error::validation(err_msg));
                }

                // Ensure directory exists since SurrealKV requires a valid directory
                if let Some(path_str) = &self.path {
                    if let Some(parent) = Path::new(path_str).parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            let err_msg =
                                format!("Failed to create directory for SurrealKV: {}", e);
                            return Err(Error::validation(err_msg));
                        }
                    }
                }
            }
            StorageEngine::Http => {
                if self.url.is_none() || self.url.as_ref().unwrap().is_empty() {
                    return Err(Error::validation("URL is required for HTTP storage engine"));
                }

                // Validate HTTP URL format
                let url_str = self.url.as_ref().unwrap();
                if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                    return Err(Error::validation(
                        "HTTP URL must start with 'http://' or 'https://'",
                    ));
                }
            }
        }

        // Validate namespace and database names
        if let Some(ns) = &self.namespace {
            if ns.is_empty() {
                return Err(Error::validation("Namespace cannot be empty"));
            }
        }

        if let Some(db) = &self.database {
            if db.is_empty() {
                return Err(Error::validation("Database cannot be empty"));
            }
        }

        Ok(())
    }
}

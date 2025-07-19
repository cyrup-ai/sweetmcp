use serde::{Serialize, de::DeserializeOwned};
use surrealdb::{
    Surreal,
    engine::{
        local::{Db, SurrealKv},
        remote::http,
    },
    opt::auth::Root,
};
use tracing::{debug, info, warn};

use crate::db::config::{DatabaseConfig, StorageEngine};
use crate::db::error::{SurrealdbError, SurrealdbErrorContext};
use crate::db::result::Result;

/// Unified client for different SurrealDB storage engines
#[derive(Debug)]
pub enum DatabaseClient {
    /// SurrealKV embedded key-value store
    SurrealKv(Surreal<Db>),
    /// HTTP connection to remote SurrealDB instance
    RemoteHttp(Surreal<http::Client>),
}

impl Clone for DatabaseClient {
    fn clone(&self) -> Self {
        match self {
            Self::SurrealKv(db) => Self::SurrealKv(db.clone()),
            Self::RemoteHttp(db) => Self::RemoteHttp(db.clone()),
        }
    }
}

impl DatabaseClient {
    /// Extract result from a query using a specific extraction strategy
    async fn extract_result<T>(&self, query: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // First, try to get results as Vec<T>
        let response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).await?,
        };

        // Check for query errors
        if let Err(e) = response.check().map_err(SurrealdbError::Database) {
            return Err(e);
        }

        let mut response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).await?,
        };

        // Try to extract as Vec<T> first
        if let Ok(mut results) = response.take::<Vec<T>>(0_usize) {
            if !results.is_empty() {
                return Ok(results.remove(0));
            }
        }

        // Try to extract as Option<T>
        let mut response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).await?,
        };

        match response.take::<Option<T>>(0_usize) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(SurrealdbError::NotFound(SurrealdbErrorContext::new(
                "No result found",
            ))),
            Err(_) => {
                // Try once more for a bare value
                let mut response = match self {
                    DatabaseClient::SurrealKv(db) => db.query(query).await?,
                    DatabaseClient::RemoteHttp(db) => db.query(query).await?,
                };

                // Convert to a generic result
                let value = response
                    .take::<Option<surrealdb::sql::Value>>(0_usize)
                    .map_err(SurrealdbError::from)?;

                match value {
                    Some(val) => {
                        // Convert the value to our target type
                        let json_val = serde_json::Value::from(val);
                        serde_json::from_value::<T>(json_val).map_err(SurrealdbError::Serialization)
                    }
                    None => Err(SurrealdbError::NotFound(SurrealdbErrorContext::new(
                        "Empty result",
                    ))),
                }
            }
        }
    }

    /// Run a SQL query directly
    pub fn query<T>(&self, query: &str) -> crate::types::AsyncTask<Result<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let self_clone = self.clone();
        let query = query.to_string();
        crate::types::AsyncTask::from_future(
            async move { self_clone.extract_result::<T>(&query).await },
        )
    }
    /// Extract result from a query with parameters
    async fn extract_result_with_params<T>(
        &self,
        query: &str,
        params: impl Serialize + Clone + Send + 'static,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Run the query with parameters
        let response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).bind(params.clone()).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).bind(params.clone()).await?,
        };
        // Check for query errors
        response.check().map_err(SurrealdbError::Database)?;

        // Extract result using same strategy as extract_result
        let mut response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).bind(params.clone()).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).bind(params.clone()).await?,
        };

        // Try to extract as Vec<T> first
        if let Ok(mut results) = response.take::<Vec<T>>(0_usize) {
            if !results.is_empty() {
                return Ok(results.remove(0));
            }
        }

        // Try to extract as Option<T>
        let mut response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).bind(params.clone()).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).bind(params.clone()).await?,
        };

        match response.take::<Option<T>>(0_usize) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(SurrealdbError::NotFound(SurrealdbErrorContext::new(
                "No result found",
            ))),
            Err(_) => {
                // Try once more for a bare value
                let mut response = match self {
                    DatabaseClient::SurrealKv(db) => db.query(query).bind(params.clone()).await?,
                    DatabaseClient::RemoteHttp(db) => db.query(query).bind(params.clone()).await?,
                };

                // Convert to a generic result
                let value = response
                    .take::<Option<surrealdb::sql::Value>>(0_usize)
                    .map_err(SurrealdbError::from)?;

                match value {
                    Some(val) => {
                        // Convert the value to our target type
                        let json_val = serde_json::Value::from(val);
                        serde_json::from_value::<T>(json_val).map_err(SurrealdbError::Serialization)
                    }
                    None => Err(SurrealdbError::NotFound(SurrealdbErrorContext::new(
                        "Empty result",
                    ))),
                }
            }
        }
    }

    /// Run a SQL query with parameters
    pub async fn query_with_params<T>(
        &self,
        query: &str,
        params: impl Serialize + Clone + Send + 'static,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let start = std::time::Instant::now();
        let result = self.extract_result_with_params::<T>(query, params).await;
        let _duration = start.elapsed();
        result
    }

    /// Create a new record
    pub fn create<T>(
        &self,
        table: &str,
        data: impl Serialize + Clone + Send + 'static,
    ) -> crate::types::AsyncTask<Result<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let table = table.to_string();
        let client = self.clone();
        crate::types::AsyncTask::from_future(async move {
            let result: Option<T> = match &client {
                DatabaseClient::SurrealKv(db) => db.create(&table).content(data).await?,
                DatabaseClient::RemoteHttp(db) => db.create(&table).content(data).await?,
            };

            result.ok_or_else(|| {
                SurrealdbError::NotFound(SurrealdbErrorContext::new("Failed to create record"))
            })
        })
    }

    /// Select records by query - returns a stream
    pub fn select<T>(&self, table: &str) -> crate::types::AsyncTask<crate::types::AsyncStream<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let table = table.to_string();
        let client = self.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let results: Vec<T> = match &client {
                DatabaseClient::SurrealKv(db) => db.select(&table).await.unwrap_or_default(),
                DatabaseClient::RemoteHttp(db) => db.select(&table).await.unwrap_or_default(),
            };

            for item in results {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        crate::types::AsyncTask::from_future(async move { crate::types::AsyncStream::new(rx) })
    }

    /// Get a single record by ID
    pub fn find_by_id<T>(&self, table: &str, id: &str) -> crate::types::AsyncTask<Result<Option<T>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let table = table.to_string();
        let id = id.to_string();
        let client = self.clone();
        crate::types::AsyncTask::from_future(async move {
            let result: Option<T> = match &client {
                DatabaseClient::SurrealKv(db) => db.select((&table, &id)).await?,
                DatabaseClient::RemoteHttp(db) => db.select((&table, &id)).await?,
            };
            Ok(result)
        })
    }

    /// Find all records as a stream
    pub fn find<T>(&self, table: &str) -> crate::types::AsyncTask<crate::types::AsyncStream<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        self.select(table)
    }

    /// Update a record
    pub fn update<T>(
        &self,
        table: &str,
        id: &str,
        data: impl Serialize + Clone + Send + 'static,
    ) -> crate::types::AsyncTask<Result<Option<T>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let table = table.to_string();
        let id = id.to_string();
        let client = self.clone();
        crate::types::AsyncTask::from_future(async move {
            let result: Option<T> = match &client {
                DatabaseClient::SurrealKv(db) => db.update((&table, &id)).content(data).await?,
                DatabaseClient::RemoteHttp(db) => db.update((&table, &id)).content(data).await?,
            };
            Ok(result)
        })
    }

    /// Delete a record
    pub fn delete<T>(&self, table: &str, id: &str) -> crate::types::AsyncTask<Result<Option<T>>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let table = table.to_string();
        let id = id.to_string();
        let client = self.clone();
        crate::types::AsyncTask::from_future(async move {
            let result: Option<T> = match &client {
                DatabaseClient::SurrealKv(db) => db.delete((&table, &id)).await?,
                DatabaseClient::RemoteHttp(db) => db.delete((&table, &id)).await?,
            };
            Ok(result)
        })
    }

    /// Execute a transaction operation
    async fn execute_transaction(&self, operation: &str) -> Result<()> {
        let query = match operation {
            "begin" => "BEGIN TRANSACTION",
            "commit" => "COMMIT TRANSACTION",
            "rollback" => "ROLLBACK TRANSACTION",
            _ => {
                return Err(SurrealdbError::other(format!(
                    "Invalid transaction operation: {}",
                    operation
                )));
            }
        };

        let response = match self {
            DatabaseClient::SurrealKv(db) => db.query(query).await?,
            DatabaseClient::RemoteHttp(db) => db.query(query).await?,
        };

        response.check().map_err(SurrealdbError::Database)?;
        Ok(())
    }

    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<()> {
        self.execute_transaction("begin").await
    }

    /// Commit a transaction
    pub async fn commit_transaction(&self) -> Result<()> {
        self.execute_transaction("commit").await
    }

    /// Rollback a transaction
    pub async fn rollback_transaction(&self) -> Result<()> {
        self.execute_transaction("rollback").await
    }

    /// Check if the database is healthy
    pub async fn health_check(&self) -> Result<bool> {
        // Use a simple boolean query to check database health
        let response = match self {
            DatabaseClient::SurrealKv(db) => db.query("RETURN true").await?,
            DatabaseClient::RemoteHttp(db) => db.query("RETURN true").await?,
        };

        Ok(response.check().is_ok())
    }
}

/// Connect to SurrealDB using the provided configuration
pub async fn connect_database(config: DatabaseConfig) -> Result<DatabaseClient> {
    // Validate configuration
    config.validate()?;

    // Ensure database directory exists for file-based storage
    if let Err(e) = config.ensure_db_dir() {
        warn!("Failed to create database directory: {}", e);
    }

    info!("Connecting to SurrealDB using {:?} engine", config.engine);

    let client = match config.engine {
        StorageEngine::SurrealKv => {
            debug!("Using SurrealKV storage at {:?}", config.path);

            // Validate path for SurrealKV
            let path_str = config.path.as_ref().ok_or_else(|| {
                SurrealdbError::configuration(SurrealdbErrorContext::new(
                    "Path is required for SurrealKv storage engine",
                ))
            })?;

            // Connect to SurrealKV
            let db = Surreal::new::<SurrealKv>(path_str.as_str()).await?;

            if let (Some(ns), Some(db_name)) = (&config.namespace, &config.database) {
                if !ns.is_empty() && !db_name.is_empty() {
                    db.use_ns(ns).use_db(db_name).await?;
                }
            }

            // Add authentication if provided
            if let (Some(user), Some(pass)) = (&config.username, &config.password) {
                if !user.is_empty() && !pass.is_empty() {
                    db.signin(Root {
                        username: user,
                        password: pass,
                    })
                    .await
                    .map_err(|e| {
                        SurrealdbError::authentication(SurrealdbErrorContext::new(format!(
                            "Authentication failed: {}",
                            e
                        )))
                    })?;
                }
            }

            DatabaseClient::SurrealKv(db)
        }
        StorageEngine::Http => {
            debug!("Using HTTP connection at {:?}", config.url);

            // Validate URL format for HTTP
            let url_str = config
                .url
                .as_ref()
                .or(config.path.as_ref())
                .ok_or_else(|| {
                    SurrealdbError::configuration(SurrealdbErrorContext::new(
                        "URL is required for HTTP storage engine",
                    ))
                })?;

            if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                warn!("HTTP URL should start with http:// or https:// prefix");
                return Err(SurrealdbError::configuration(SurrealdbErrorContext::new(
                    "HTTP URL must start with 'http://' or 'https://'",
                )));
            }

            // Connect to remote SurrealDB instance via HTTP
            let db = Surreal::new::<http::Http>(url_str.as_str()).await?;

            // Authenticate if credentials are provided
            if let (Some(user), Some(pass)) = (&config.username, &config.password) {
                if !user.is_empty() && !pass.is_empty() {
                    db.signin(Root {
                        username: user,
                        password: pass,
                    })
                    .await
                    .map_err(|e| {
                        SurrealdbError::authentication(SurrealdbErrorContext::new(format!(
                            "Authentication failed: {}",
                            e
                        )))
                    })?;
                }
            }

            if let (Some(ns), Some(db_name)) = (&config.namespace, &config.database) {
                if !ns.is_empty() && !db_name.is_empty() {
                    db.use_ns(ns).use_db(db_name).await?;
                }
            }

            DatabaseClient::RemoteHttp(db)
        }
    };

    // Connect to the database complete
    info!("Database connected successfully");

    Ok(client)
}

/// Create a new database connection from a configuration
#[allow(dead_code)]
pub async fn new(config: DatabaseConfig) -> Result<DatabaseClient> {
    debug!("Creating database client from config: {:?}", config);
    connect_database(config).await
}

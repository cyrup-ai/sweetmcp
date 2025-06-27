use std::{
    any::Any,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Result, anyhow};
use futures::Stream;
use log::{debug, error};
use once_cell::sync::OnceCell;
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::{db::DatabaseClient, types::*};

// ----- Public API -----
// Non-async functions that return domain-specific types

/// List all resources
/// Returns a stream of resources that can be consumed asynchronously
pub fn resources_list(request: Option<ListResourcesRequest>) -> ResourceStream {
    match get_cms_dao() {
        Ok(dao) => dao.resources_list(request),
        Err(e) => {
            error!("Failed to get CmsDao in resources_list: {}", e);
            // Return an empty stream on error
            let (tx, rx) = mpsc::channel(1);
            // Close the sender immediately to signal empty stream
            drop(tx);
            ResourceStream::new(rx)
        }
    }
}

/// Get a specific resource by URI
/// Returns a future that resolves to the resource or an error
pub fn resource_read(request: ReadResourceRequest) -> AsyncResource {
    match get_cms_dao() {
        Ok(dao) => dao.resource_read(request),
        Err(e) => {
            error!("Failed to get CmsDao in resource_read: {}", e);
            // Return a future that resolves to an error
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                "CmsDao not initialized: {}",
                e
            ))));
            AsyncResource { rx }
        }
    }
}

/// Find resources by tags
pub fn find_by_tags(tags: Vec<String>) -> ResourceStream {
    match get_cms_dao() {
        Ok(dao) => dao.find_by_tags(tags),
        Err(e) => {
            error!("Failed to get CmsDao in find_by_tags: {}", e);
            // Return an empty stream on error
            let (tx, rx) = mpsc::channel(1);
            // Close the sender immediately to signal empty stream
            drop(tx);
            ResourceStream::new(rx)
        }
    }
}

/// Find resource by slug
pub fn find_by_slug(slug: String) -> AsyncResource {
    match get_cms_dao() {
        Ok(dao) => dao.find_by_slug(slug),
        Err(e) => {
            error!("Failed to get CmsDao in find_by_slug: {}", e);
            // Return a future that resolves to an error
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                "CmsDao not initialized: {}",
                e
            ))));
            AsyncResource { rx }
        }
    }
}

// ----- Stream and Future Types -----

/// Stream type for resources
pub struct ResourceStream {
    inner: ReceiverStream<HandlerResult<Resource>>,
}

impl ResourceStream {
    pub fn new(rx: mpsc::Receiver<HandlerResult<Resource>>) -> Self {
        Self {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl Stream for ResourceStream {
    type Item = HandlerResult<Resource>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

/// Future type for a single resource read
pub struct AsyncResource {
    rx: oneshot::Receiver<HandlerResult<ReadResourceResult>>,
}

impl Future for AsyncResource {
    type Output = HandlerResult<ReadResourceResult>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| {
                Err(rpc_router::HandlerError::new(
                    "AsyncResource channel closed unexpectedly",
                ))
            })
        })
    }
}

// ----- Resource Manager Trait -----

/// Resource manager interface
pub trait ResourceManager: Send + Sync + std::fmt::Debug {
    /// Get the name of this manager
    fn name(&self) -> &str;

    /// Get this manager as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// List resources with optional filtering
    fn list_resources(&self, request: Option<ListResourcesRequest>) -> ResourceStream;

    /// Read a specific resource by URI
    fn read_resource(&self, request: ReadResourceRequest) -> AsyncResource;

    /// Find resources by tags
    fn find_by_tags(&self, tags: Vec<String>) -> ResourceStream;

    /// Find a resource by slug
    fn find_by_slug(&self, slug: String) -> AsyncResource;
}

// ----- SurrealDB Implementation -----

/// Concrete implementation for SurrealDB
#[derive(Debug, Clone)]
pub struct SurrealDbManager {
    client: DatabaseClient,
}

impl SurrealDbManager {
    pub fn new(client: DatabaseClient) -> Self {
        Self { client }
    }

    // Helper method to run a resources query
    fn resources_query_stream(&self, query: &str, bindings: serde_json::Value) -> ResourceStream {
        let (tx, rx) = mpsc::channel(32);
        let client = self.client.clone();

        // Clone the query string to own it in the async task
        let query_owned = query.to_string();

        tokio::spawn(async move {
            debug!("Executing query: {}", query_owned);

            // Execute the query with bindings
            let response = client
                .query_with_params::<Vec<ResourceRow>>(&query_owned, bindings)
                .await;

            match response {
                Ok(rows) => {
                    for row in rows {
                        // Convert the database row to a Resource
                        let resource = map_row_to_resource(row);
                        if tx.send(Ok(resource)).await.is_err() {
                            debug!("Receiver dropped for resources query stream");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error executing resources query: {}", e);
                    let _ = tx
                        .send(Err(rpc_router::HandlerError::new(format!(
                            "Database query error: {}",
                            e
                        ))))
                        .await;
                }
            }
        });

        ResourceStream::new(rx)
    }
}

impl ResourceManager for SurrealDbManager {
    fn name(&self) -> &str {
        "SurrealDbManager"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn list_resources(&self, request: Option<ListResourcesRequest>) -> ResourceStream {
        // Base query for resources
        let mut query = String::from(
            "SELECT id, type, title, description, content, category, \
                                     tags, mime_type, parent, children, links \
                                     FROM resource",
        );

        // Create bindings object for parameterized query
        let mut bindings = serde_json::Map::new();

        // Add filters if specified
        if let Some(req) = &request {
            let mut conditions = Vec::new();

            if let Some(category) = &req.category {
                conditions.push("category = $category".to_string());
                bindings.insert(
                    "category".to_string(),
                    serde_json::Value::String(category.clone()),
                );
            }

            if let Some(tags) = &req.tags {
                if !tags.is_empty() {
                    conditions.push("$tags CONTAINSANY tags".to_string());
                    bindings.insert(
                        "tags".to_string(),
                        serde_json::Value::Array(
                            tags.iter()
                                .map(|t| serde_json::Value::String(t.clone()))
                                .collect(),
                        ),
                    );
                }
            }

            if !conditions.is_empty() {
                query.push_str(" WHERE ");
                query.push_str(&conditions.join(" AND "));
            }

            // Add pagination
            if let Some(limit) = req.limit {
                query.push_str(&format!(" LIMIT {}", limit));
            }

            if let Some(offset) = req.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        }

        // Convert bindings to Value
        let bindings_value = serde_json::Value::Object(bindings);

        // Execute the query as a stream
        self.resources_query_stream(&query, bindings_value)
    }

    fn read_resource(&self, request: ReadResourceRequest) -> AsyncResource {
        let (tx, rx) = oneshot::channel();
        let client = self.client.clone();
        let uri = request.uri;

        tokio::spawn(async move {
            // Extract ID from URI - convert URL to string and extract parts
            let uri_str = uri.to_string();
            // Handle both cms:// scheme or path-based URIs
            let id = if uri_str.starts_with("cms://") {
                uri_str.replace("cms://", "")
            } else if uri_str.starts_with("/") {
                uri_str[1..].to_string()
            } else {
                uri_str
            };

            debug!("Reading resource with ID: {}", id);

            // Query for a single resource
            let query = "SELECT id, type, title, description, content, category, \
                        tags, mime_type, parent, children, links \
                        FROM resource WHERE id = $id";

            let bindings = serde_json::json!({
                "id": id
            });

            match client
                .query_with_params::<Vec<ResourceRow>>(&query, bindings)
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                            "Resource not found: {}",
                            uri
                        ))));
                        return;
                    }

                    let row = &rows[0];
                    let resource = map_row_to_resource(row.clone());

                    // Create ResourceContent from the resource
                    let content = ResourceContent {
                        uri: resource.uri.clone(),
                        mime_type: resource.mime_type.clone(),
                        text: row.content.clone(),
                        blob: None,
                    };

                    let result = ReadResourceResult { content };
                    let _ = tx.send(Ok(result));
                }
                Err(e) => {
                    error!("Error reading resource: {}", e);
                    let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                        "Database query error: {}",
                        e
                    ))));
                }
            }
        });

        AsyncResource { rx }
    }

    fn find_by_tags(&self, tags: Vec<String>) -> ResourceStream {
        // Query for resources with matching tags
        let query = "SELECT id, type, title, description, content, category, \
                    tags, mime_type, parent, children, links \
                    FROM resource \
                    WHERE $tags CONTAINSANY tags";

        let bindings = serde_json::json!({
            "tags": tags
        });

        self.resources_query_stream(&query, bindings)
    }

    fn find_by_slug(&self, slug: String) -> AsyncResource {
        let (tx, rx) = oneshot::channel();
        let client = self.client.clone();

        tokio::spawn(async move {
            debug!("Finding resource by slug: {}", slug);

            // We already have a String for the slug parameter
            let query = "SELECT id, type, title, description, content, category, \
                        tags, mime_type, parent, children, links \
                        FROM resource \
                        WHERE slug = $slug";

            let bindings = serde_json::json!({
                "slug": slug
            });

            match client
                .query_with_params::<Vec<ResourceRow>>(&query, bindings)
                .await
            {
                Ok(rows) => {
                    if rows.is_empty() {
                        let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                            "Resource not found with slug: {}",
                            slug
                        ))));
                        return;
                    }

                    let row = &rows[0];
                    let resource = map_row_to_resource(row.clone());

                    // Create ResourceContent from the resource
                    let content = ResourceContent {
                        uri: resource.uri.clone(),
                        mime_type: resource.mime_type.clone(),
                        text: row.content.clone(),
                        blob: None,
                    };

                    let result = ReadResourceResult { content };
                    let _ = tx.send(Ok(result));
                }
                Err(e) => {
                    error!("Error finding resource by slug: {}", e);
                    let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                        "Database query error: {}",
                        e
                    ))));
                }
            }
        });

        AsyncResource { rx }
    }
}

// ----- CMS DAO -----

/// Row type from SurrealDB for resources
#[derive(serde::Deserialize, Debug, Clone)]
struct ResourceRow {
    // SurrealDB specific ID format
    id: surrealdb::sql::Thing,
    #[serde(rename = "type")]
    type_: String,
    title: String,
    description: Option<String>,
    content: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    mime_type: Option<String>,
    // References to other resources
    parent: Option<surrealdb::sql::Thing>,
    children: Option<Vec<surrealdb::sql::Thing>>,
    links: Option<Vec<surrealdb::sql::Thing>>,
}

/// Helper function to map a database row to a Resource
fn map_row_to_resource(row: ResourceRow) -> Resource {
    // Convert SurrealDB Thing to a URL
    let uri_str = format!("cms://{}", row.id);
    let uri = url::Url::parse(&uri_str).unwrap_or_else(|_| {
        // Fallback URL if parsing fails
        url::Url::parse(&format!("cms://resource/{}", row.id.id))
            .unwrap_or_else(|_| url::Url::parse("cms://resource/unknown").unwrap())
    });

    // Helper to convert SurrealDB Thing to URL
    let thing_to_url = |thing: Option<surrealdb::sql::Thing>| -> Option<url::Url> {
        thing.and_then(|t| url::Url::parse(&format!("cms://{}", t)).ok())
    };

    // Helper to convert Option<Vec<Thing>> to Option<Vec<Url>>
    let vec_thing_to_vec_url =
        |things: Option<Vec<surrealdb::sql::Thing>>| -> Option<Vec<url::Url>> {
            things.map(|ts| {
                ts.into_iter()
                    .filter_map(|t| url::Url::parse(&format!("cms://{}", t)).ok())
                    .collect()
            })
        };

    // Use category and type to create node_type
    let node_type = match &row.category {
        Some(category) => Some(format!("{}/{}", row.type_, category)),
        None => Some(row.type_.clone()),
    };

    // Create metadata from tags if present
    let metadata = if let Some(tags) = &row.tags {
        if !tags.is_empty() {
            Some(serde_json::json!({ "tags": tags }))
        } else {
            None
        }
    } else {
        None
    };

    Resource {
        uri,
        name: row.title,
        description: row.description,
        mime_type: row.mime_type,
        node_type,
        parent: thing_to_url(row.parent),
        children: vec_thing_to_vec_url(row.children),
        links: vec_thing_to_vec_url(row.links),
        metadata,
    }
}

/// Data Access Object for CMS resources
#[derive(Debug)]
pub struct CmsDao {
    manager: Box<dyn ResourceManager>,
}

impl CmsDao {
    /// Create a new CmsDao with the given resource manager
    pub fn new(manager: Box<dyn ResourceManager>) -> Self {
        Self { manager }
    }

    /// Create a CmsDao with a SurrealDB backend
    pub fn with_surrealdb(client: DatabaseClient) -> Self {
        Self {
            manager: Box::new(SurrealDbManager::new(client)),
        }
    }

    /// Get the underlying database client if available
    pub fn client(&self) -> Option<&DatabaseClient> {
        self.manager
            .as_any()
            .downcast_ref::<SurrealDbManager>()
            .map(|m| &m.client)
    }

    /// List resources with optional filtering
    pub fn resources_list(&self, request: Option<ListResourcesRequest>) -> ResourceStream {
        self.manager.list_resources(request)
    }

    /// Read a specific resource by URI
    pub fn resource_read(&self, request: ReadResourceRequest) -> AsyncResource {
        self.manager.read_resource(request)
    }

    /// Find resources by tags
    pub fn find_by_tags(&self, tags: Vec<String>) -> ResourceStream {
        self.manager.find_by_tags(tags)
    }

    /// Find a resource by slug
    pub fn find_by_slug(&self, slug: String) -> AsyncResource {
        self.manager.find_by_slug(slug)
    }
}

// ----- Global CMS DAO Instance -----

static CMS_DAO: OnceCell<CmsDao> = OnceCell::new();

/// Initialize the CMS DAO with the given instance
pub fn init_cms_dao(dao: CmsDao) -> Result<()> {
    CMS_DAO
        .set(dao)
        .map_err(|_| anyhow!("CmsDao already initialized"))
}

/// Get a reference to the global CMS DAO
fn get_cms_dao() -> Result<&'static CmsDao> {
    CMS_DAO
        .get()
        .ok_or_else(|| anyhow!("CmsDao not initialized"))
}

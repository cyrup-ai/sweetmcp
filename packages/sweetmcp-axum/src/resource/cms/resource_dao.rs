use std::any::Any;
use std::{
    collections::BTreeMap, // Import BTreeMap for bindings
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    fmt::Write, // For zero-allocation string formatting
};

// Zero-allocation string formatting imports
use arrayvec::ArrayString;

use futures::Stream;
use rpc_router::HandlerResult;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

use crate::db::{
    DatabaseClient, // Keep DatabaseClient
};
use crates::types::*;

// Stream type for resources
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
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        std::pin::Pin::new(&mut self.inner).poll_next(cx)
    }
}

// Future type for a single resource read
pub struct AsyncResource {
    rx: oneshot::Receiver<HandlerResult<ReadResourceResult>>,
}

impl Future for AsyncResource {
    type Output = HandlerResult<ReadResourceResult>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|res| {
            res.unwrap_or_else(|_| Err(rpc_router::HandlerError::new("oneshot cancelled")))
        })
    }
}

// --- Stream-based resources_list ---
pub fn resources_list_stream(request: Option<ListResourcesRequest>) -> ResourceStream {
    let (tx, rx) = mpsc::channel(16);

    // Clone the request for the async task
    let request_clone = request.clone(); // Keep clone

    tokio::spawn(async move {
        // let db = get_db().await; // Use the imported get_db // Removed

        // Build the query based on request parameters
        let mut query = "SELECT * FROM node".to_string();
        let mut where_clauses = Vec::new();
        // Use BTreeMap for bindings as required by SurrealDB client
        let mut bindings: BTreeMap<String, surrealdb::sql::Value> = BTreeMap::new();

        // Default filter for pages
        where_clauses.push("type = 'page'".to_string()); // Assuming 'node' table has 'page' type

        // Add category filter if specified
        if let Some(req) = &request_clone {
            // Use cloned request
            if let Some(category) = &req.category {
                where_clauses.push("category = $category".to_string());
                bindings.insert("category".to_string(), category.clone().into());
            }

            // Add tags filter if specified
            if let Some(tags) = &req.tags {
                if !tags.is_empty() {
                    // Ensure tags are converted to Value::Array for CONTAINS ALL
                    let tag_values: Vec<surrealdb::sql::Value> =
                        tags.iter().map(|t| t.clone().into()).collect();
                    where_clauses.push("tags CONTAINSALL $tags".to_string()); // Use CONTAINSALL
                    bindings.insert("tags".to_string(), tag_values.into());
                }
            }
        }

        // Add WHERE clause if we have conditions
        if !where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clauses.join(" AND "));
        }

        // Add limit if specified (using zero-allocation string formatting)
        if let Some(req) = &request_clone {
            if let Some(limit) = req.limit {
                use arrayvec::ArrayString;
                use std::fmt::Write;
                let mut limit_str: ArrayString<32> = ArrayString::new(); // 32 chars is more than enough for numbers
                write!(&mut limit_str, " LIMIT {}", limit)
                    .expect("Numeric formatting should not fail");
                query.push_str(&limit_str);
            }

            // Add offset if specified (using zero-allocation string formatting)
            if let Some(offset) = req.offset {
                use arrayvec::ArrayString;
                use std::fmt::Write;
                let mut offset_str: ArrayString<32> = ArrayString::new();
                write!(&mut offset_str, " START {}", offset)
                    .expect("Numeric formatting should not fail");
                query.push_str(&offset_str);
            }
        }

        // Run the query with bindings
        // Get the database client via the ResourceDao
        let db_client = match get_resource_dao().and_then(|dao| {
            dao.client()
                .cloned()
                .ok_or_else(|| anyhow!("ResourceDao manager is not SurrealDbManager"))
        }) {
            // Adjusted error message
            Ok(client) => client,
            Err(e) => {
                log::error!("Failed to get DatabaseClient for query: {}", e);
                // Send error and exit task
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "ResourceDao not initialized: {}",
                    e
                ))));
                return;
            },
        };

        let response = db_client
            .query_with_params::<Vec<NodeRow>>(&query, bindings)
            .await; // Ensure type is Vec<NodeRow>

        #[derive(serde::Deserialize, Debug)]
        struct NodeRow {
            // Assuming id is Thing format like "node:xxxx"
            id: surrealdb::sql::Thing,
            #[serde(rename = "type")]
            type_: String,
            title: String,
            description: Option<String>,
            content: Option<String>,
            mime_type: Option<String>,
            // Assuming parent/children/links are stored as Thing or Vec<Thing>
            parent: Option<surrealdb::sql::Thing>,
            children: Option<Vec<surrealdb::sql::Thing>>,
            links: Option<Vec<surrealdb::sql::Thing>>,
            metadata: Option<serde_json::Value>,
            #[serde(default)]
            category: Option<String>,
            #[serde(default)]
            tags: Option<Vec<String>>, // Add tags field
        }

        match response {
            Ok(node_rows) => {
                // Ensure handling Vec<NodeRow>
                for row in node_rows {
                    // Construct URI from Thing id using zero-allocation string formatting
                    let mut uri_str: ArrayString<64> = ArrayString::new();
                    if write!(&mut uri_str, "cms://{}", row.id).is_err() {
                        log::error!("Failed to format URI for node id: {}", row.id);
                        continue; // Skip this row if URI formatting fails
                    }
                    let uri = match url::Url::parse(&uri_str) {
                        Ok(u) => u,
                        Err(e) => {
                            log::error!("Failed to parse node URI '{}': {}", uri_str, e);
                            continue; // Skip this row if URI is invalid
                        },
                    };

                    // Helper to convert Option<Thing> to Option<Url> using zero-allocation formatting
                    let thing_to_url = |thing: Option<surrealdb::sql::Thing>| -> Option<Url> {
                        thing.and_then(|t| {
                            let mut url_str: ArrayString<64> = ArrayString::new();
                            if write!(&mut url_str, "cms://{}", t).is_ok() {
                                Url::parse(&url_str).ok()
                            } else {
                                None
                            }
                        })
                    };

                    // Convert content to a string if present using zero-allocation formatting
                    let content_snippet = row.content.as_ref().map(|content| {
                        // If content is longer than 100 chars, truncate it for the description
                        if content.len() > 100 {
                            let mut truncated: ArrayString<128> = ArrayString::new();
                            if write!(&mut truncated, "{}...", &content[..97]).is_ok() {
                                truncated.to_string()
                            } else {
                                content[..97].to_string()
                            }
                        } else {
                            content.clone()
                        }
                    });

                    // Use category for resource node_type if available using zero-allocation formatting
                    let node_type = match &row.category {
                        Some(category) => {
                            let mut type_str: ArrayString<64> = ArrayString::new();
                            if write!(&mut type_str, "{}/{}", row.type_, category).is_ok() {
                                type_str.to_string()
                            } else {
                                row.type_.clone()
                            }
                        },
                        None => row.type_
                    };

                    // Use tags if available for metadata
                    let metadata = if let Some(tags) = &row.tags {
                        if !tags.is_empty() {
                            Some(serde_json::json!({ "tags": tags }))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Build URI using zero-allocation string formatting
                    let mut uri_str: ArrayString<256> = ArrayString::new(); // URIs are typically short
                    if write!(&mut uri_str, "cms://{}", row.id).is_err() {
                        log::error!("Failed to format URI for row id: {}", row.id);
                        continue; // Skip this row if URI formatting fails
                    }
                    let uri = match Url::parse(&uri_str) {
                        Ok(url) => url,
                        Err(e) => {
                            log::error!("Failed to parse resource URI: {}", e);
                            // Fallback to a basic URL using zero-allocation formatting
                            let mut fallback_uri: ArrayString<64> = ArrayString::new();
                            if write!(&mut fallback_uri, "cms://resource/{}", row.id.id).is_ok() {
                                Url::parse(&fallback_uri).unwrap_or_else(|_| {
                                    Url::parse("cms://resource/unknown").unwrap()
                                })
                            } else {
                                Url::parse("cms://resource/unknown").unwrap()
                            }
                        }
                    };

                    // Helper to convert SurrealDB Thing to URL using zero-allocation formatting
                    let thing_to_url = |thing: Option<surrealdb::sql::Thing>| -> Option<Url> {
                        thing.and_then(|t| {
                            let mut url_str: ArrayString<64> = ArrayString::new();
                            if write!(&mut url_str, "cms://{}", t).is_ok() {
                                Url::parse(&url_str).ok()
                            } else {
                                None
                            }
                        })
                    };

                    // Helper to convert Option<Vec<Thing>> to Option<Vec<Url>> using zero-allocation formatting
                    let vec_thing_to_vec_url = |things: Option<Vec<surrealdb::sql::Thing>>| -> Option<Vec<Url>> {
                        things.map(|ts| {
                            ts.into_iter()
                                .filter_map(|t| {
                                    let mut url_str: ArrayString<64> = ArrayString::new();
                                    if write!(&mut url_str, "cms://{}", t).is_ok() {
                                        Url::parse(&url_str).ok()
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        })
                    };

                    let resource = Resource {
                        uri,
                        name: row.title,
                        description: row.description.or(content_snippet),
                        mime_type: row.mime_type,
                        node_type: Some(node_type),
                        parent: thing_to_url(row.parent),
                        children: vec_thing_to_vec_url(row.children),
                        links: vec_thing_to_vec_url(row.links),
                        metadata,
                    };
                    if tx.send(Ok(resource)).await.is_err() {
                        log::warn!("Receiver dropped for resources_list_stream");
                        break; // Stop sending if receiver is gone
                    }
                }
            },
            Err(e) => {
                log::error!("Failed to query resources: {}", e);
                // Send an error through the channel if the query failed
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "Database query failed: {}",
                    e
                ))));
            },
        }
    });

    ResourceStream::new(rx)
}

// --- Future-based resource_read ---
pub fn resource_read_async(request: ReadResourceRequest) -> AsyncResource {
    let (tx, rx) = oneshot::channel();
    let uri = request.uri.clone(); // Clone URI for the task

    tokio::spawn(async move {
        // Parse the Thing ID from the URI
        let thing_id = match surrealdb::sql::Thing::try_from(uri.as_str()) {
            // Assuming URI format is like "cms:node:some_id" or "node:some_id"
            // Adjust parsing based on actual URI scheme and Thing format
            Ok(id) if id.tb == "node" => id, // Ensure it's a node table
            _ => {
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "Invalid or non-node URI: {}",
                    uri
                ))));
                return; // Early return if URI is invalid
            },
        };

        // let db = get_db().await; // Get DB client // Removed

        // Define the NodeRow struct for deserialization
        #[derive(serde::Deserialize, Debug)]
        struct NodeRow {
            // id: surrealdb::sql::Thing, // ID is known, not needed in result usually
            #[serde(rename = "type")]
            type_: String,
            title: String,
            description: Option<String>,
            content: Option<String>,
            mime_type: Option<String>,
            parent: Option<surrealdb::sql::Thing>,
            children: Option<Vec<surrealdb::sql::Thing>>,
            links: Option<Vec<surrealdb::sql::Thing>>,
            metadata: Option<serde_json::Value>,
            #[serde(default)]
            category: Option<String>,
            #[serde(default)]
            tags: Option<Vec<String>>,
        }

        // Execute the SELECT query for the specific node ID
        // Get the database client
        let db_client = match get_resource_dao().and_then(|dao| {
            dao.client()
                .cloned()
                .ok_or_else(|| anyhow!("ResourceDao manager is not SurrealDbManager"))
        }) {
            // Adjusted error message
            Ok(client) => client,
            Err(e) => {
                log::error!("Failed to get DatabaseClient for read: {}", e);
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "ResourceDao not initialized: {}",
                    e
                ))));
                return;
            },
        };

        let response = db_client
            .get::<NodeRow>("node", &thing_id.to_string())
            .await; // Use client.get

        match response {
            Ok(Some(row)) => {
                // Successfully fetched the node
                let content = ResourceContent {
                    uri: uri.clone(), // Use the original valid URI
                    mime_type: row.mime_type.or_else(|| Some("text/markdown".to_string())), /* Default mime type */
                    text: row.content,
                    blob: None, // Assuming blob is not fetched here
                };
                let _ = tx.send(Ok(ReadResourceResult { content }));
            },
            Ok(None) => {
                // Node not found
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "Resource not found: {}",
                    uri
                ))));
            },
            Err(e) => {
                // Database error during fetch
                log::error!("Failed to read resource {}: {}", uri, e);
                let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                    "Database error fetching resource: {}",
                    e
                ))));
            },
        }
    });

    AsyncResource { rx } // Return the future immediately
}

// --- ResourceManager trait ---
// Trait with sync methods that return domain-specific types

pub trait ResourceManager: Send + Sync + Any + 'static {
    // Add Any bound
    // A human-readable name
    fn name(&self) -> &str;

    // Add method to allow downcasting
    fn as_any(&self) -> &dyn Any;

    // Returns a stream of resources
    fn list_resources(&self, request: Option<ListResourcesRequest>) -> ResourceStream;

    // Returns a future for a single resource
    fn read_resource(&self, request: ReadResourceRequest) -> AsyncResource;
}

// Concrete implementation for SurrealDB
#[derive(Debug, Clone)] // Add derive Debug and Clone
pub struct SurrealDbManager {
    client: DatabaseClient, // Add client field
}

impl SurrealDbManager {
    // Add a constructor
    pub fn new(client: DatabaseClient) -> Self {
        Self { client }
    }
}

impl ResourceManager for SurrealDbManager {
    fn name(&self) -> &str {
        "SurrealDbManager"
    }

    fn as_any(&self) -> &dyn Any {
        // Implement as_any
        self
    }

    fn list_resources(&self, request: Option<ListResourcesRequest>) -> ResourceStream {
        resources_list_stream(request)
    }

    fn read_resource(&self, request: ReadResourceRequest) -> AsyncResource {
        resource_read_async(request)
    }
}

// --- ResourceDao ---
// Data Access Object for resources

// Keep Debug removed
pub struct ResourceDao {
    manager: Box<dyn ResourceManager>,
}

impl ResourceDao {
    pub fn new(manager: Box<dyn ResourceManager>) -> Self {
        Self { manager }
    }

    // Modify to accept a DatabaseClient
    pub fn with_surrealdb(client: DatabaseClient) -> Self {
        Self {
            manager: Box::new(SurrealDbManager::new(client)),
        }
    }

    // Use Any trait for downcasting
    pub fn client(&self) -> Option<&DatabaseClient> {
        self.manager
            .as_any()
            .downcast_ref::<SurrealDbManager>()
            .map(|m| &m.client)
    }

    pub fn resources_list(&self, request: Option<ListResourcesRequest>) -> ResourceStream {
        self.manager.list_resources(request)
    }

    pub fn resource_read(&self, request: ReadResourceRequest) -> AsyncResource {
        self.manager.read_resource(request)
    }
}

use once_cell::sync::OnceCell;

static RESOURCE_DAO: OnceCell<ResourceDao> = OnceCell::new();

use anyhow::{Result, anyhow}; // Add anyhow Result

pub fn init_resource_dao(dao: ResourceDao) -> Result<(), anyhow::Error> {
    RESOURCE_DAO
        .set(dao)
        .map_err(|_| anyhow!("ResourceDao already initialized"))
}

// Changed to return Result<&'static ResourceDao, Error>
fn get_resource_dao() -> Result<&'static ResourceDao, anyhow::Error> {
    RESOURCE_DAO
        .get()
        .ok_or_else(|| anyhow!("ResourceDao not initialized"))
}

// --- Public API ---
// Non-async functions that return domain-specific types

// List all resources
// Note: This function now needs error handling if get_resource_dao fails.
// Returning the stream directly might hide the initialization error.
// Consider returning Result<ResourceStream, Error> or handling the error here.
// For now, keeping the signature but logging the error if DAO is not initialized.
pub fn resources_list(request: Option<ListResourcesRequest>) -> ResourceStream {
    match get_resource_dao() {
        Ok(dao) => dao.resources_list(request),
        Err(e) => {
            log::error!("Failed to get ResourceDao in resources_list: {}", e);
            // Return an empty stream on error
            let (tx, rx) = mpsc::channel(1);
            // Close the sender immediately to signal empty stream
            drop(tx);
            ResourceStream::new(rx)
        },
    }
}

// Get a specific resource by URI
// Similar error handling consideration as above.
pub fn resource_read(request: ReadResourceRequest) -> AsyncResource {
    match get_resource_dao() {
        Ok(dao) => dao.resource_read(request),
        Err(e) => {
            log::error!("Failed to get ResourceDao in resource_read: {}", e);
            // Return a future that immediately resolves to an error
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                "ResourceDao not initialized: {}",
                e
            ))));
            AsyncResource { rx }
        },
    }
}

// Find resources by category
// Similar error handling consideration as above.
pub fn find_by_category(category: &str) -> ResourceStream {
    match get_resource_dao() {
        Ok(dao) => {
            let mut request = ListResourcesRequest::default();
            request.category = Some(category.to_string());
            dao.resources_list(Some(request))
        },
        Err(e) => {
            log::error!("Failed to get ResourceDao in find_by_category: {}", e);
            // Return an empty stream on error
            let (tx, rx) = mpsc::channel(1);
            drop(tx);
            ResourceStream::new(rx)
        },
    }
}

// Find a resource by slug (assuming slug corresponds to the ID part of the Thing)
// Similar error handling consideration as above.
pub fn find_by_slug(slug: &str) -> AsyncResource {
    match get_resource_dao() {
        Ok(dao) => {
            // Construct the Thing ID string using zero-allocation formatting
            let mut thing_id_str: ArrayString<64> = ArrayString::new();
            if write!(&mut thing_id_str, "node:{}", slug).is_err() {
                let (tx, rx) = oneshot::channel();
                let _ = tx.send(Err(rpc_router::HandlerError::new(
                    "Failed to format thing ID string".to_string()
                )));
                return AsyncResource { rx };
            }
            // Attempt to parse the Thing ID string into a URL using zero-allocation formatting
            let mut uri_str: ArrayString<64> = ArrayString::new();
            if write!(&mut uri_str, "cms://{}", thing_id_str).is_err() {
                let (tx, rx) = oneshot::channel();
                let _ = tx.send(Err(rpc_router::HandlerError::new(
                    "Failed to format URI string".to_string()
                )));
                return AsyncResource { rx };
            }
            match Url::parse(&uri_str) {
                Ok(uri) => dao.resource_read(ReadResourceRequest { uri, meta: None }),
                Err(e) => {
                    log::error!("Failed to create valid URI for slug '{}': {}", slug, e);
                    // Return a future that immediately resolves to an error
                    let (tx, rx) = oneshot::channel();
                    let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                        "Invalid slug format: {}",
                        slug
                    ))));
                    AsyncResource { rx }
                },
            }
        },
        Err(e) => {
            log::error!("Failed to get ResourceDao in find_by_slug: {}", e);
            // Return a future that immediately resolves to an error
            let (tx, rx) = oneshot::channel();
            let _ = tx.send(Err(rpc_router::HandlerError::new(format!(
                "Invalid slug format: {}",
                slug
            ))));
            AsyncResource { rx }
        },
    }
}

// Find resources by tags
// Similar error handling consideration as above.
pub fn find_by_tags(tags: &[String]) -> ResourceStream {
    match get_resource_dao() {
        Ok(dao) => {
            let mut request = ListResourcesRequest::default();
            request.tags = Some(tags.to_vec());
            dao.resources_list(Some(request))
        },
        Err(e) => {
            log::error!("Failed to get ResourceDao in find_by_tags: {}", e);
            // Return an empty stream on error
            let (tx, rx) = mpsc::channel(1);
            drop(tx);
            ResourceStream::new(rx)
        },
    }
}

//! API module for exposing memory system functionality
//! This module is feature-gated with the "api" feature

// TODO: Implement these modules
// #[cfg(feature = "api")]
// pub mod routes;
// #[cfg(feature = "api")]
// pub mod handlers;
// #[cfg(feature = "api")]
// pub mod middleware;
// #[cfg(feature = "api")]
// pub mod models;

#[cfg(feature = "api")]
use axum::Router;
#[cfg(feature = "api")]
use std::net::SocketAddr;
#[cfg(feature = "api")]
use std::sync::Arc;

#[cfg(feature = "api")]
use crate::memory::MemoryManager;
#[cfg(feature = "api")]
use crate::utils::config::APIConfig;

/// API server for the memory system
#[cfg(feature = "api")]
pub struct APIServer<M>
where
    M: MemoryManager + 'static,
{
    /// Memory manager
    memory_manager: Arc<M>,
    /// API configuration
    config: APIConfig,
    /// Router
    router: Router,
}

#[cfg(feature = "api")]
impl<M> APIServer<M>
where
    M: MemoryManager + 'static,
{
    /// Create a new API server
    pub fn new(memory_manager: Arc<M>, config: APIConfig) -> Self {
        // TODO: Implement routes module
        // let router = routes::create_router(memory_manager.clone(), &config);
        let router = Router::new();

        Self {
            memory_manager,
            config,
            router,
        }
    }

    /// Start the API server
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port).parse::<SocketAddr>()?;

        tracing::info!("API server listening on {}", addr);

        // Updated to use tokio::net::TcpListener with axum 0.8.x
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, self.router.clone()).await?;

        Ok(())
    }
}

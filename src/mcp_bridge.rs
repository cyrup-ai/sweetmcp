use mcp_rust_sdk::{Response, Request};
use mcp_rust_sdk::server::ServerHandler;
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

pub type BridgeMsg = (Request, oneshot::Sender<Response>);

// Simple embedded MCP server handler
pub struct EmbeddedHandler;

impl mcp_rust_sdk::server::ServerHandler for EmbeddedHandler {
    fn initialize<'life0, 'async_trait>(
        &'life0 self,
        _implementation: mcp_rust_sdk::types::Implementation,
        _capabilities: mcp_rust_sdk::types::ClientCapabilities,
    ) -> Pin<Box<dyn Future<Output = Result<mcp_rust_sdk::types::ServerCapabilities, mcp_rust_sdk::Error>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            Ok(mcp_rust_sdk::types::ServerCapabilities::default())
        })
    }

    fn shutdown<'life0, 'async_trait>(
        &'life0 self,
    ) -> Pin<Box<dyn Future<Output = Result<(), mcp_rust_sdk::Error>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            Ok(())
        })
    }

    fn handle_method<'life0, 'life1, 'async_trait>(
        &'life0 self,
        method: &'life1 str,
        params: Option<serde_json::Value>,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, mcp_rust_sdk::Error>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            // Simple echo handler for now
            Ok(serde_json::json!({
                "method": method,
                "params": params,
                "status": "handled"
            }))
        })
    }
}

pub async fn run(mut rx: mpsc::Receiver<BridgeMsg>) {
    let handler = Arc::new(EmbeddedHandler);
    
    while let Some((req, tx)) = rx.recv().await {
        // Handle the request directly since we don't have a full transport layer
        let result = match req.method.as_str() {
            "initialize" => {
                let default_impl = mcp_rust_sdk::types::Implementation {
                    name: "SweetMCP Server".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                };
                let default_caps = mcp_rust_sdk::types::ClientCapabilities::default();
                match handler.initialize(default_impl, default_caps).await {
                    Ok(caps) => Response::success(req.id, Some(serde_json::to_value(caps).unwrap())),
                    Err(e) => Response::error(req.id, mcp_rust_sdk::protocol::ResponseError::from(e)),
                }
            }
            "shutdown" => {
                match handler.shutdown().await {
                    Ok(()) => Response::success(req.id, None),
                    Err(e) => Response::error(req.id, mcp_rust_sdk::protocol::ResponseError::from(e)),
                }
            }
            _ => {
                match handler.handle_method(&req.method, req.params).await {
                    Ok(result) => Response::success(req.id, Some(result)),
                    Err(e) => Response::error(req.id, mcp_rust_sdk::protocol::ResponseError::from(e)),
                }
            }
        };
        
        let _ = tx.send(result);
    }
}
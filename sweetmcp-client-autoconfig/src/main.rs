mod clients;
mod watcher;

use anyhow::Result;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

use crate::watcher::ClientWatcher;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
    
    info!("🍯 SweetMCP Client Auto-Configuration Service");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("Zero-friction MCP integration for all your AI tools!");
    
    // Create watcher with all client plugins
    let clients = clients::all_clients();
    let watcher = Arc::new(ClientWatcher::new(clients));
    
    // Start watching for installations
    watcher.start().await?;
    
    Ok(())
}
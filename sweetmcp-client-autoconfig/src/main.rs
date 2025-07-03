use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

use sweetmcp_client_autoconfig::{clients, watcher::AutoConfigWatcher};

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
    let watcher = AutoConfigWatcher::new(clients)?;

    // Start watching for installations
    watcher.run().await?;

    Ok(())
}

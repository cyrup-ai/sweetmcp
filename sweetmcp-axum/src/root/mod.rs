use log::debug;
use rpc_router::HandlerResult;

use crates::types::{ListRootsRequest, ListRootsResult, Root};

/// Handler for the roots/list MCP method
/// Lists all available workspace roots
pub async fn roots_list(_request: Option<ListRootsRequest>) -> HandlerResult<ListRootsResult> {
    debug!("Listing available roots");

    // In a real-world implementation, these would be dynamically loaded
    // from configuration or scanned from the filesystem
    let response = ListRootsResult {
        roots: vec![
            Root {
                name: "workspace".to_string(),
                url: "file:///workspace".to_string(),
            },
            // Add additional roots as needed
        ],
    };

    Ok(response)
}

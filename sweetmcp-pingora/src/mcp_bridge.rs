use serde_json::Value;
use sweetmcp_axum::JSONRPC_VERSION;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info};

// Bridge message type for communication between Pingora and MCP handler
pub type BridgeMsg = (
    Value,
    crate::normalize::ProtocolContext,
    oneshot::Sender<Value>,
);

// Run the MCP bridge that processes incoming messages
pub async fn run(mut rx: mpsc::Receiver<BridgeMsg>) {
    info!("MCP bridge started and ready to process messages");

    while let Some((request, _protocol_ctx, tx)) = rx.recv().await {
        // Forward JSON-RPC request to sweetmcp-axum via HTTP
        let client = reqwest::Client::new();

        let response = match client
            .post("http://localhost:8080/rpc")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
        {
            Ok(http_response) => match http_response.json::<Value>().await {
                Ok(json_response) => json_response,
                Err(e) => {
                    error!("Failed to parse JSON response from Axum: {:?}", e);
                    serde_json::json!({
                        "jsonrpc": JSONRPC_VERSION,
                        "error": {
                            "code": -32603,
                            "message": "Internal error: invalid response from backend"
                        },
                        "id": request.get("id").cloned().unwrap_or(Value::Null)
                    })
                }
            },
            Err(e) => {
                error!("Failed to forward request to Axum: {:?}", e);
                serde_json::json!({
                    "jsonrpc": JSONRPC_VERSION,
                    "error": {
                        "code": -32603,
                        "message": "Internal error: backend unavailable"
                    },
                    "id": request.get("id").cloned().unwrap_or(Value::Null)
                })
            }
        };

        if let Err(e) = tx.send(response) {
            error!("Failed to send response back through bridge: {:?}", e);
        }
    }

    info!("MCP bridge shutting down");
}

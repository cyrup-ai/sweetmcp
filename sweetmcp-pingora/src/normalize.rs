//! Converts GraphQL, JSON-RPC, or Cap'n Proto payloads into standard JSON-RPC for MCP.

use anyhow::{bail, Context, Result};
use async_graphql::parser::{parse_query, types::*};
use async_graphql::{Name, Positioned};
use async_graphql_value::Value;
use capnp::message::ReaderOptions;
use serde_json::json;
use sweetmcp_axum::JSONRPC_VERSION;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Proto {
    GraphQL,
    JsonRpc,
    Capnp,
    McpStreamableHttp,
}

/// Context for tracking protocol conversion
#[derive(Debug, Clone)]
pub struct ProtocolContext {
    pub protocol: Proto,
    pub original_query: Option<String>, // For GraphQL response shaping
    pub request_id: String,
}

/// Normalize incoming protocol to JSON-RPC for cyrup-mcp-api
pub fn to_json_rpc(_user: &str, body: &[u8]) -> Result<(ProtocolContext, serde_json::Value)> {
    to_json_rpc_with_headers(_user, body, None)
}

/// Normalize incoming protocol to JSON-RPC with optional header context
pub fn to_json_rpc_with_headers(
    _user: &str,
    body: &[u8],
    req_header: Option<&pingora::http::RequestHeader>,
) -> Result<(ProtocolContext, serde_json::Value)> {
    // Try JSON-RPC first (most specific)
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) {
        if v.get("jsonrpc").is_some() {
            // Validate it's proper JSON-RPC
            let _method = v
                .get("method")
                .and_then(|m| m.as_str())
                .ok_or_else(|| anyhow::anyhow!("JSON-RPC missing method"))?;

            let id = v
                .get("id")
                .cloned()
                .unwrap_or_else(|| json!(Uuid::new_v4().to_string()));

            let ctx = ProtocolContext {
                protocol: Proto::JsonRpc,
                original_query: None,
                request_id: id.to_string(),
            };

            // Pass through valid JSON-RPC unchanged
            return Ok((ctx, v));
        }

        // Check if it's MCP Streamable HTTP JSON-RPC (without explicit jsonrpc field)
        if let Some(header) = req_header {
            let uri_path = header.uri.path();
            if uri_path == "/mcp" || uri_path.starts_with("/mcp/") {
                // MCP Streamable HTTP can have JSON-RPC without the version field
                if v.get("method").is_some() {
                    let id = v
                        .get("id")
                        .cloned()
                        .unwrap_or_else(|| json!(Uuid::new_v4().to_string()));

                    // Add jsonrpc version if missing
                    let mut mcp_json = v;
                    if mcp_json.get("jsonrpc").is_none() {
                        mcp_json["jsonrpc"] = json!(JSONRPC_VERSION);
                    }

                    let ctx = ProtocolContext {
                        protocol: Proto::McpStreamableHttp,
                        original_query: None,
                        request_id: id.to_string(),
                    };

                    return Ok((ctx, mcp_json));
                }
            }
        }
    }

    // Try GraphQL
    if let Ok(query_str) = std::str::from_utf8(body) {
        if let Ok(doc) = parse_query(query_str) {
            return graphql_to_json_rpc(query_str, doc);
        }
    }

    // Try Cap'n Proto
    let mut body_slice = body;
    if capnp::serialize::read_message_from_flat_slice(&mut body_slice, ReaderOptions::new()).is_ok()
    {
        return capnp_to_json_rpc(body);
    }

    bail!("Unknown protocol - expected JSON-RPC, GraphQL, or Cap'n Proto")
}

/// Convert GraphQL query to JSON-RPC
fn graphql_to_json_rpc(
    query_str: &str,
    doc: ExecutableDocument,
) -> Result<(ProtocolContext, serde_json::Value)> {
    // Find the first operation (query or mutation)
    let operation = doc
        .operations
        .iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No GraphQL operation found"))?;

    let (method, params) = match &operation.1.node.ty {
        OperationType::Query => {
            extract_graphql_method_and_params(&operation.1.node.selection_set.node)?
        }
        OperationType::Mutation => {
            extract_graphql_method_and_params(&operation.1.node.selection_set.node)?
        }
        OperationType::Subscription => {
            bail!("GraphQL subscriptions not supported")
        }
    };

    let request_id = Uuid::new_v4().to_string();

    let ctx = ProtocolContext {
        protocol: Proto::GraphQL,
        original_query: Some(query_str.to_string()),
        request_id: request_id.clone(),
    };

    let json_rpc = json!({
        "jsonrpc": JSONRPC_VERSION,
        "method": method,
        "params": params,
        "id": request_id
    });

    Ok((ctx, json_rpc))
}

/// Extract MCP method and params from GraphQL selection set
fn extract_graphql_method_and_params(
    selection_set: &SelectionSet,
) -> Result<(String, serde_json::Value)> {
    // Handle queries like: { tools { list } } or { resources { read(uri: "file.txt") } }
    if selection_set.items.is_empty() {
        bail!("Empty GraphQL selection set");
    }

    // Get the first field selection
    let field = match &selection_set.items[0].node {
        Selection::Field(field) => &field.node,
        _ => bail!("Expected field selection in GraphQL query"),
    };

    let namespace = field.name.node.as_str();

    // Check if this is a nested selection (e.g., tools { list })
    if !field.selection_set.node.items.is_empty() {
        let subfield = match &field.selection_set.node.items[0].node {
            Selection::Field(subfield) => &subfield.node,
            _ => bail!("Expected field selection in GraphQL query"),
        };

        let method_name = format!("{}/{}", namespace, subfield.name.node);
        let params = extract_graphql_arguments(&subfield.arguments);

        Ok((method_name, params))
    } else {
        // Direct method call (e.g., callTool)
        let method_name = namespace.to_string();
        let params = extract_graphql_arguments(&field.arguments);

        Ok((method_name, params))
    }
}

/// Extract arguments from GraphQL field as JSON params
fn extract_graphql_arguments(
    arguments: &Vec<(Positioned<Name>, Positioned<Value>)>,
) -> serde_json::Value {
    if arguments.is_empty() {
        return json!({});
    }

    let mut params = serde_json::Map::new();

    for (name, value) in arguments {
        let key = name.node.as_str();
        let val = graphql_value_to_json(&value.node);
        params.insert(key.to_string(), val);
    }

    json!(params)
}

/// Convert GraphQL Value to JSON Value  
fn graphql_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => json!(null),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!(i)
            } else if let Some(f) = n.as_f64() {
                json!(f)
            } else {
                json!(n.to_string())
            }
        }
        Value::String(s) => json!(s),
        Value::Boolean(b) => json!(b),
        Value::Binary(bytes) => {
            // Convert binary to base64 string
            use base64::Engine;
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            json!(encoded)
        }
        Value::Enum(e) => json!(e.as_str()),
        Value::List(list) => {
            let items: Vec<_> = list.iter().map(|v| graphql_value_to_json(v)).collect();
            json!(items)
        }
        Value::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (key, val) in obj {
                map.insert(key.as_str().to_string(), graphql_value_to_json(val));
            }
            json!(map)
        }
        Value::Variable(_) => {
            // Variables should be resolved before this point
            json!(null)
        }
    }
}

/// Convert Cap'n Proto to JSON-RPC
fn capnp_to_json_rpc(_body: &[u8]) -> Result<(ProtocolContext, serde_json::Value)> {
    // TODO: Implement Cap'n Proto parsing based on MCP schema
    // For now, return error
    bail!("Cap'n Proto support not yet implemented")
}

/// Convert JSON-RPC response back to original protocol format
pub fn from_json_rpc(
    ctx: &ProtocolContext,
    json_rpc_response: &serde_json::Value,
) -> Result<Vec<u8>> {
    match ctx.protocol {
        Proto::JsonRpc => {
            // Pass through unchanged
            serde_json::to_vec(json_rpc_response).context("Failed to serialize JSON-RPC response")
        }
        Proto::McpStreamableHttp => {
            // MCP Streamable HTTP uses standard JSON-RPC format
            serde_json::to_vec(json_rpc_response)
                .context("Failed to serialize MCP Streamable HTTP response")
        }
        Proto::GraphQL => graphql_from_json_rpc(ctx, json_rpc_response),
        Proto::Capnp => capnp_from_json_rpc(ctx, json_rpc_response),
    }
}

/// Convert JSON-RPC response to GraphQL response
fn graphql_from_json_rpc(_ctx: &ProtocolContext, response: &serde_json::Value) -> Result<Vec<u8>> {
    let mut graphql_response = json!({
        "data": null
    });

    // Check for JSON-RPC error
    if let Some(error) = response.get("error") {
        let error_message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");

        graphql_response["errors"] = json!([{
            "message": error_message,
            "extensions": {
                "code": error.get("code")
            }
        }]);
    } else if let Some(result) = response.get("result") {
        // Build GraphQL response based on original query structure
        // For now, simple passthrough - would need to parse original query for proper shaping
        graphql_response["data"] = result.clone();
    }

    serde_json::to_vec(&graphql_response).context("Failed to serialize GraphQL response")
}

/// Convert JSON-RPC response to Cap'n Proto
fn capnp_from_json_rpc(_ctx: &ProtocolContext, _response: &serde_json::Value) -> Result<Vec<u8>> {
    bail!("Cap'n Proto response conversion not yet implemented")
}

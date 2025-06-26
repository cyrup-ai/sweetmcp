//! Converts GraphQL, JSON-RPC, or Cap'n Proto payloads into MCP Request.

use anyhow::{bail, Result};
use async_graphql::parser::parse_query;
use capnp::message::ReaderOptions;
use mcp_rust_sdk::{Request, protocol::RequestId};
use serde_json::{json, Value};
use uuid::Uuid;

pub enum Proto {
    GraphQL,
    JsonRpc,
    Capnp,
}

pub fn to_mcp(_user: &str, body: &[u8]) -> Result<(Proto, Request)> {
    if let Ok(v) = serde_json::from_slice::<Value>(body) {
        if v.get("jsonrpc").is_some() {
            return Ok((Proto::JsonRpc, serde_json::from_value(v)?));
        }
    }
    if std::str::from_utf8(body).map(parse_query).is_ok() {
        return Ok((
            Proto::GraphQL,
            Request::new(
                "graphql/execute",
                Some(json!({ "query": std::str::from_utf8(body)? })),
                RequestId::String(Uuid::new_v4().to_string()),
            )
        ));
    }
    let mut body_slice = body;
    if capnp::serialize::read_message_from_flat_slice(&mut body_slice, ReaderOptions::new()).is_ok() {
        return Ok((
            Proto::Capnp,
            Request::new(
                "capnp/execute",
                Some(json!({ "capnp": base64_url::encode(body) })),
                RequestId::String(Uuid::new_v4().to_string()),
            )
        ));
    }
    bail!("unknown protocol")
}


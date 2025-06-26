# SweetMCP Server

<p align="center">
  <img src="./assets/sweetmcp.png" alt="SweetMCP" style="max-width: 100%; width: 600px;">
</p>

A multi-protocol edge proxy that normalizes GraphQL, JSON-RPC 2.0, and Cap'n Proto requests into Model Context Protocol (MCP) format.

## What It Does

The server accepts requests in three protocol formats and converts them to MCP:
- **GraphQL** queries 
- **JSON-RPC 2.0** method calls
- **Cap'n Proto** binary messages

It routes requests based on server load:
- Handles requests locally when not overloaded
- Forwards to the peer with lowest `node_load1` metric when overloaded

## Configuration

### Required Environment Variable
```bash
export SWEETMCP_JWT_SECRET=$(openssl rand -base64 32)
```

### Optional Configuration
```bash
export SWEETMCP_UPSTREAMS="https://peer1:8443,https://peer2:8443"
export SWEETMCP_INFLIGHT_MAX=400
export SWEETMCP_TCP_BIND="0.0.0.0:8443"
export SWEETMCP_UDS_PATH="/run/sugora.sock"
export SWEETMCP_METRICS_BIND="127.0.0.1:9090"
```

## Running

```bash
cargo run --release
```

## Endpoints

- TCP: `0.0.0.0:8443` - Main service endpoint
- Unix Socket: `/run/sugora.sock` - Local access
- Metrics: `http://127.0.0.1:9090/metrics` - Prometheus metrics

## Authentication

Uses JWT tokens with HS256 signing. Include in requests:
```
Authorization: Bearer <token>
```

## Protocol Examples

### GraphQL
```bash
curl -X POST http://localhost:8443/graphql \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/graphql" \
  -d 'query { method(params: {}) }'
```

### JSON-RPC 2.0
```bash
curl -X POST http://localhost:8443/ \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

### Cap'n Proto
Send binary Cap'n Proto messages to the same endpoints.

## License

Dual licensed under MIT OR Apache-2.0.
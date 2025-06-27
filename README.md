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

## Auto-Discovery

SweetMCP nodes automatically discover each other using industry-standard approaches:

- **DNS-based**: Primary discovery via SRV records with secure DoH (DNS over HTTPS)
- **Local Networks**: mDNS fallback for zero-config local discovery
- **Peer Exchange**: HTTP-based peer sharing for mesh formation

### DNS Discovery Configuration

```bash
# Option 1: Explicit service name
export SWEETMCP_DNS_SERVICE="_sweetmcp._tcp.example.com"

# Option 2: Auto-generated from domain
export SWEETMCP_DOMAIN="example.com"  # Creates: _sweetmcp._tcp.example.com
```

### DNS SRV Record Example

```dns
_sweetmcp._tcp.example.com. 300 IN SRV 10 50 8443 node1.example.com.
_sweetmcp._tcp.example.com. 300 IN SRV 10 50 8443 node2.example.com.
```

Nodes verify build compatibility using a git-hash based build ID to ensure only identical builds form a cluster.

## Configuration

### Required Environment Variable
```bash
export SWEETMCP_JWT_SECRET=$(openssl rand -base64 32)
```

### Optional Configuration
```bash
export SWEETMCP_INFLIGHT_MAX=400
export SWEETMCP_TCP_BIND="0.0.0.0:8443"
export SWEETMCP_UDS_PATH="/run/sugora.sock"
export SWEETMCP_METRICS_BIND="127.0.0.1:9090"

# Discovery Security (recommended for production)
export SWEETMCP_DISCOVERY_TOKEN="your-shared-secret-token"

# DNS Discovery (recommended)
export SWEETMCP_DNS_SERVICE="_sweetmcp._tcp.example.com"
# or
export SWEETMCP_DOMAIN="example.com"

# Static upstreams (fallback if DNS not available)
export SWEETMCP_UPSTREAMS="https://peer1:8443,https://peer2:8443"
```

### Production Security

When deploying to production, always set:
- `SWEETMCP_DISCOVERY_TOKEN`: Shared secret for discovery endpoints
- Rate limiting: Built-in 10 req/min per endpoint
- Health checks: Automatic TCP health checks every 10s
- Metrics: Available at `/metrics` endpoint

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
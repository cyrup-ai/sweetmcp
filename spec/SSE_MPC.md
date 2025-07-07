# MCP SSE Transport Protocol: Wire-Level Specification and Implementation

The Model Context Protocol (MCP) SSE transport provides a bidirectional communication channel using Server-Sent Events for server-to-client messages and HTTP POST for client-to-server messages. **Note: SSE transport was deprecated in protocol version 2025-03-26 in favor of Streamable HTTP transport, but remains supported for backwards compatibility.**

## Complete wire-level protocol for MCP SSE servers

The MCP SSE transport operates through a **dual-endpoint architecture** that separates the persistent SSE connection from message submission:

### Required HTTP Endpoints

**SSE Endpoint (GET /sse)**
```
GET /sse HTTP/1.1
Host: localhost:8080
Accept: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
```

**Server Response Headers:**
```
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache
Connection: keep-alive
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, x-api-key
```

**Messages Endpoint (POST /messages)**
```
POST /messages?session_id=unique-session-id HTTP/1.1
Host: localhost:8080
Content-Type: application/json
Content-Length: 158

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}
```

### SSE Event Stream Format

The server communicates through **standard SSE events** following RFC 6455:

```
event: endpoint
data: /messages?session_id=9bb7cf474d1e4e24832ee7cce54993f3

event: message
data: {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":true}}}}

event: ping
data: 2025-01-07T12:00:00Z

: keep-alive comment
```

Each event consists of:
- **event**: Event type identifier (endpoint, message, ping, error)
- **data**: JSON-encoded payload or plain text
- **id**: Optional event ID for resumability

### JSON-RPC Message Format

All messages follow **JSON-RPC 2.0 specification** with this structure:

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "method": "string",
  "params": {
    "_meta": { "optional": "metadata" },
    "additional": "parameters"
  }
}
```

**Response Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "result": {
    "_meta": { "optional": "metadata" },
    "data": "response_data"
  }
}
```

**Error Format:**
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "error": {
    "code": -32000,
    "message": "Error description",
    "data": { "additional": "error_details" }
  }
}
```

## The exact sequence of messages Claude sends to discover tools

### 1. Initialize Handshake

**Client → Server: Initialize Request**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {}
    },
    "clientInfo": {
      "name": "claude-ai",
      "version": "0.1.0"
    }
  }
}
```

**Server → Client: Initialize Response**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "logging": {},
      "prompts": { "listChanged": true },
      "resources": { "subscribe": true, "listChanged": true },
      "tools": { "listChanged": true }
    },
    "serverInfo": {
      "name": "ExampleServer",
      "version": "1.0.0"
    }
  }
}
```

**Client → Server: Initialized Notification**
```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### 2. Tools Discovery

**Client → Server: List Tools Request**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}
```

**Server → Client: Tools List Response**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "generate-image",
        "description": "Generate an image using Stable Diffusion based on a text prompt",
        "inputSchema": {
          "type": "object",
          "properties": {
            "prompt": {
              "type": "string",
              "description": "The text prompt for image generation"
            },
            "width": {
              "type": "integer",
              "description": "Image width in pixels",
              "default": 512
            },
            "height": {
              "type": "integer",
              "description": "Image height in pixels",
              "default": 512
            }
          },
          "required": ["prompt"]
        }
      }
    ]
  }
}
```

### 3. Tool Invocation

**Client → Server: Tool Call Request**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "generate-image",
    "arguments": {
      "prompt": "A sunset over mountains",
      "width": 1024,
      "height": 768
    }
  }
}
```

**Server → Client: Tool Result Response**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Image generated successfully!"
      },
      {
        "type": "image",
        "data": "base64_encoded_image_data",
        "mimeType": "image/png"
      }
    ]
  }
}
```

## Raw examples of actual HTTP requests and responses

### Complete Session Example with curl

**1. Establish SSE Connection:**
```bash
# Start SSE connection and keep it open
curl -N -H "Accept: text/event-stream" http://localhost:8080/sse

# Response stream:
event: endpoint
data: /messages?session_id=abc123def456

event: ping
data: Server is alive!
```

**2. Initialize MCP Session:**
```bash
curl -X POST http://localhost:8080/messages?session_id=abc123def456 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "curl-client",
        "version": "1.0.0"
      }
    }
  }'
```

**3. Discover Available Tools:**
```bash
curl -X POST http://localhost:8080/messages?session_id=abc123def456 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list",
    "params": {}
  }'
```

**4. Invoke a Tool:**
```bash
curl -X POST http://localhost:8080/messages?session_id=abc123def456 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {
        "message": "Hello, MCP!"
      }
    }
  }'
```

### Raw Network Trace Example

**Complete HTTP trace of initialization:**
```
POST /messages HTTP/1.1
Host: localhost:8080
Content-Length: 158
Content-Type: application/json
User-Agent: curl/7.68.0
Accept: */*

{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}

HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 245
Date: Wed, 02 Apr 2025 19:03:55 GMT
Connection: keep-alive

{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"enabled":true},"prompts":{"enabled":true}},"serverInfo":{"name":"example-server","version":"1.0.0"}}}
```

### Debugging with tcpdump

```bash
# Capture MCP traffic on localhost
sudo tcpdump -i lo0 -A -s 0 'port 8080'

# Example output showing SSE stream:
12:34:56.789012 IP localhost.8080 > localhost.54321: Flags [P.], seq 1:156
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-cache

event: endpoint
data: /messages?session_id=xyz789

event: message
data: {"jsonrpc":"2.0","id":1,"result":{"tools":[{"name":"echo"}]}}
```

## Implementation details for --transport sse servers

### Required Server Implementation Components

**1. Express.js/Node.js Example:**
```javascript
import express from "express";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { SSEServerTransport } from "@modelcontextprotocol/sdk/server/sse.js";

const server = new McpServer({
  name: "Example SSE Server",
  version: "1.0.0",
});

const app = express();
app.use(express.json());

const transports = {};

// SSE endpoint
app.get("/sse", async (req, res) => {
  const transport = new SSEServerTransport("/messages", res);
  transports[transport.sessionId] = transport;
  
  transport.onclose = () => {
    delete transports[transport.sessionId];
  };
  
  await server.connect(transport);
});

// Messages endpoint
app.post("/messages", async (req, res) => {
  const sessionId = req.query.sessionId;
  const transport = transports[sessionId];
  
  if (transport) {
    await transport.handlePostMessage(req, res, req.body);
  } else {
    res.status(400).send('Invalid session ID');
  }
});

app.listen(3001);
```

**2. Python FastAPI Implementation:**
```python
from mcp.server.sse import SseServerTransport
from starlette.applications import Starlette
from starlette.routing import Route, Mount

sse = SseServerTransport("/messages/")

async def handle_sse(request):
    async with sse.connect_sse(
        request.scope, request.receive, request._send
    ) as streams:
        await server.run(
            streams[0], streams[1], 
            server.create_initialization_options()
        )

async def handle_messages(request):
    await sse.handle_post_message(
        request.scope, request.receive, request._send
    )

app = Starlette(routes=[
    Route("/sse", endpoint=handle_sse, methods=["GET"]),
    Mount("/messages/", app=sse.handle_post_message),
])
```

### CORS Headers and Security Requirements

**Required CORS Configuration:**
```javascript
// CORS middleware
app.use((req, res, next) => {
  res.header("Access-Control-Allow-Origin", "*");
  res.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
  res.header("Access-Control-Allow-Headers", "Content-Type, Authorization");
  next();
});
```

**Security Best Practices:**
- **Bind to localhost only** (127.0.0.1) for local servers, never 0.0.0.0
- **Validate Origin header** to prevent DNS rebinding attacks
- **Implement proper authentication** (bearer tokens, API keys)
- **Set message size limits** to prevent DoS attacks
- **Use HTTPS in production** environments

### Error Handling and Response Formats

**Standard Error Codes:**
- `-32700`: Parse error
- `-32600`: Invalid Request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32000` to `-32099`: Server-specific errors

**Error Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid parameters",
    "data": {
      "details": "Missing required field 'prompt'",
      "field": "prompt"
    }
  }
}
```

### Session Management

Sessions are maintained through unique IDs passed in the query string:
- Generated when SSE connection is established
- Used to route POST messages to correct transport
- Cleaned up when connection closes
- No built-in persistence mechanism

The MCP SSE transport provides a robust but complex protocol for bidirectional communication between Claude and MCP servers. While deprecated in favor of the simpler Streamable HTTP transport, understanding its wire-level details remains valuable for maintaining existing implementations and debugging legacy systems.
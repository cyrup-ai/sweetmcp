# SweetMCP Architecture - THE ACTUAL TRUTH

## THE REAL ARCHITECTURE: 3-Binary Architecture

**BINARIES THAT SHOULD EXIST:**
1. `sweetmcp-daemon` - Service manager daemon with clap subcommands 
2. `sweetmcp_server` - Pingora proxy/load balancer (managed by daemon)
3. **`sweet` - Standalone MCP server** (managed by daemon, runs on localhost:8080)

## How Everything Actually Fucking Works

### 1. How pingora loads MCP functionality
**FROM `sweetmcp-pingora/src/mcp_bridge.rs`:**
- Pingora receives requests (GraphQL, JSON-RPC, Cap'n Proto) 
- `mcp_bridge::run()` forwards ALL MCP requests to **separate MCP server via HTTP**:
  ```rust
  let response = client
      .post("http://localhost:8080/rpc")  // ← CALLS SEPARATE AXUM SERVER!
      .json(&request)
      .send()
      .await
  ```
- Pingora is a PROXY, not an MCP server itself
- The axum/sweetmcp-axum runs as SEPARATE PROCESS on port 8080

### 2. What axum main.rs actually does
- Creates standalone MCP server that runs on localhost:8080
- Loads plugins via PluginManager
- Handles actual MCP JSON-RPC requests
- Pingora forwards requests TO this server

### 3. How daemon manages everything
- Daemon spawns pingora binary (`sweetmcp_server`) 
- Daemon ALSO needs to spawn axum binary (`sweet`) on port 8080
- Daemon handles install/uninstall of BOTH binaries
- Autoconfig runs internally in daemon (no separate binary needed)

## MY FUCK-UP ANALYSIS

### sweetmcp-axum/src/main.rs - **CRITICAL BINARY DELETED** 
- This creates the `sweet` binary that runs the actual MCP server
- Pingora depends on this running on localhost:8080
- **I BROKE THE ARCHITECTURE BY DELETING THIS**

### sweetmcp-client-autoconfig/src/main.rs - CORRECTLY DELETED
- Autoconfig runs internally in daemon via "internal:autoconfig"
- No separate binary needed

## IMMEDIATE FIXES NEEDED

1. **RESTORE** `sweetmcp-axum/src/main.rs` 
2. **UPDATE** installer to create service for `sweet` binary on port 8080
3. **VERIFY** pingora expects MCP server on localhost:8080

## THE TRUTH: Pingora = Proxy, Axum = MCP Server
#!/bin/bash
# SweetMCP Claude Desktop Integration Wrapper
# This script provides the full SweetMCP experience through the Pingora gateway

set -euo pipefail

# Configuration
SWEETMCP_HOST="mcp.cyrup.dev"
SWEETMCP_PORT="8443"
DAEMON_NAME="cyrupd"
SOCKET_PATH="/run/sugora.sock"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[SweetMCP]${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}[SweetMCP]${NC} $1" >&2
}

error() {
    echo -e "${RED}[SweetMCP]${NC} $1" >&2
}

# Check if daemon is running
check_daemon_status() {
    if command -v launchctl >/dev/null 2>&1; then
        # macOS
        if launchctl list | grep -q "$DAEMON_NAME"; then
            return 0
        fi
    elif command -v systemctl >/dev/null 2>&1; then
        # Linux with systemd
        if systemctl is-active --quiet "$DAEMON_NAME"; then
            return 0
        fi
    fi
    return 1
}

# Start daemon if not running
ensure_daemon_running() {
    if ! check_daemon_status; then
        warn "SweetMCP daemon not running, attempting to start..."
        
        if command -v launchctl >/dev/null 2>&1; then
            # macOS
            sudo launchctl load -w "/Library/LaunchDaemons/${DAEMON_NAME}.plist" 2>/dev/null || true
        elif command -v systemctl >/dev/null 2>&1; then
            # Linux
            sudo systemctl start "$DAEMON_NAME" 2>/dev/null || true
        fi
        
        # Wait a moment for startup
        sleep 2
        
        if ! check_daemon_status; then
            error "Failed to start SweetMCP daemon"
            exit 1
        fi
    fi
    
    log "SweetMCP daemon is running"
}

# Check if Pingora gateway is responding
check_gateway_health() {
    local max_attempts=30
    local attempt=1
    
    log "Checking Pingora gateway health..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -k -s --connect-timeout 2 "https://${SWEETMCP_HOST}:${SWEETMCP_PORT}/health" >/dev/null 2>&1; then
            log "Pingora gateway is healthy"
            return 0
        fi
        
        warn "Gateway not ready (attempt $attempt/$max_attempts), waiting..."
        sleep 1
        ((attempt++))
    done
    
    error "Pingora gateway failed to respond after $max_attempts attempts"
    return 1
}

# Bridge MCP protocol through Pingora gateway
bridge_mcp_protocol() {
    log "Starting MCP protocol bridge through Pingora gateway"
    
    # Use socat to bridge stdin/stdout to HTTPS endpoint
    # This maintains the MCP JSON-RPC protocol while routing through Pingora
    exec socat STDIO SSL:${SWEETMCP_HOST}:${SWEETMCP_PORT},verify=0
}

# Main execution
main() {
    log "SweetMCP Claude Desktop Integration starting..."
    
    # Ensure daemon is running
    ensure_daemon_running
    
    # Check gateway health
    if ! check_gateway_health; then
        error "Gateway health check failed"
        exit 1
    fi
    
    # Bridge the MCP protocol
    bridge_mcp_protocol
}

# Handle cleanup on exit
trap 'log "MCP bridge session ended"' EXIT

# Check dependencies
if ! command -v socat >/dev/null 2>&1; then
    error "socat is required but not installed"
    error "Install with: brew install socat (macOS) or apt-get install socat (Linux)"
    exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
    error "curl is required but not installed"
    exit 1
fi

# Run main function
main "$@"
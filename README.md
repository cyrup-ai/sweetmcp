# SweetMCP 🍯

<p align="center">
  <img src="./assets/sweetmcp.png" alt="SweetMCP" style="max-width: 100%; width: 600px;">
</p>

<p align="center">
  <strong>The sweetest Model Context Protocol integration platform</strong>
</p>

<p align="center">
  Zero-configuration AI tool integration with automatic SSL certificates and local DNS
</p>

---

## 🚀 One-Line Installation

### macOS & Linux
```bash
curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash
```

### Windows (PowerShell as Administrator)
```powershell
iex (iwr -UseBasicParsing https://get.cyrup.ai/sweetmcp.ps1).Content
```

<details>
<summary>🖱️ <strong>What does this do?</strong></summary>

The installer performs a complete soup-to-nuts setup:
1. ✅ **System Check** - Verifies requirements (git, curl, sudo)
2. ✅ **Rust Installation** - Auto-installs Rust toolchain if missing  
3. ✅ **Repository Clone** - Pulls latest code from GitHub
4. ✅ **Release Build** - Compiles optimized binaries
5. ✅ **SSL Certificates** - Generates wildcard certs for *.cyrup.{dev,ai,cloud,pro}
6. ✅ **Local DNS** - Adds host entries for sweetmcp.cyrup.* domains
7. ✅ **Trust Store** - Imports certificates to OS trust store
8. ✅ **Service Install** - Registers system daemon (systemd/launchd)
9. ✅ **Verification** - Tests all components

</details>

### Platform Support

| Platform | Status | Architecture |
|----------|--------|--------------|
| 🍎 **macOS** | ✅ Ready | Intel + Apple Silicon |
| 🐧 **Linux** | ✅ Ready | x86_64 + aarch64 |
| 🪟 **Windows** | ✅ Ready | x86_64 + i686 |

---

## 🎯 What You Get

After installation, these endpoints are instantly available:

- **https://sweetmcp.cyrup.dev:8443** - Primary endpoint
- **https://sweetmcp.cyrup.ai:8443** - AI-focused endpoint  
- **https://sweetmcp.cyrup.cloud:8443** - Cloud services
- **https://sweetmcp.cyrup.pro:8443** - Professional tools

### 🔧 Core Components

| Component | Description | Location |
|-----------|-------------|----------|
| **SweetMCP Daemon** | Service manager | `/usr/local/bin/cyrupd` |
| **Pingora Gateway** | High-performance proxy | Managed by daemon |
| **SSL Certificates** | Wildcard *.cyrup.* certs | `~/.config/sweetmcp/` |
| **Local DNS** | Host file entries | `/etc/hosts` |
| **Configuration** | Service definitions | `~/.config/cyrupd/` |

---

## 🔌 AI Tool Integration

SweetMCP automatically detects and configures popular AI development tools:

### Supported Tools
- **Claude Desktop** - Auto-configures MCP servers
- **Windsurf** - IDE integration 
- **VSCode** - Extension support (planned)
- **Zed** - Native integration (planned)
- **Cursor** - AI pair programming (planned)

### Auto-Detection
The daemon scans every 15 minutes and automatically:
- Detects installed AI tools
- Configures MCP server connections
- Updates tool-specific settings
- Validates connectivity

---

## 🛠️ Manual Installation

If you prefer to review the code first:

```bash
# Clone repository
git clone git@github.com:cyrup-ai/sweetmcp.git
cd sweetmcp

# Build release binaries
cargo build --release --package sweetmcp-daemon

# Install with sudo privileges
sudo ./target/release/sweetmcp-daemon install
```

---

## 🎮 Service Management

### Linux (systemd)
```bash
# Start service
sudo systemctl start cyrupd

# Enable auto-start
sudo systemctl enable cyrupd

# Check status
sudo systemctl status cyrupd

# View logs
journalctl -u cyrupd -f
```

### macOS (launchd)
```bash
# Service auto-starts after installation

# Check status
sudo launchctl list | grep cyrupd

# View logs
tail -f /var/log/cyrupd.log

# Manual start/stop
sudo launchctl load /Library/LaunchDaemons/com.cyrup.cyrupd.plist
sudo launchctl unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist
```

---

## 🏗️ Architecture

SweetMCP is a multi-protocol edge proxy that normalizes different request formats into Model Context Protocol (MCP):

### Supported Protocols
- **GraphQL** queries → MCP
- **JSON-RPC 2.0** method calls → MCP  
- **Cap'n Proto** binary messages → MCP

### Load Balancing
- Handles requests locally when not overloaded
- Forwards to peer with lowest `node_load1` metric when overloaded
- Auto-discovery via DNS SRV records and mDNS

### Security Features
- JWT authentication with HS256 signing
- TLS/mTLS support with automatic certificate management
- Rate limiting (10 req/min per endpoint)
- Automatic health checks every 10s

---

## ⚙️ Configuration

### Environment Variables
```bash
# Required
export SWEETMCP_JWT_SECRET=$(openssl rand -base64 32)

# Optional
export SWEETMCP_TCP_BIND="0.0.0.0:8443"
export SWEETMCP_UDS_PATH="/run/sugora.sock"
export SWEETMCP_METRICS_BIND="127.0.0.1:9090"
export SWEETMCP_INFLIGHT_MAX=400

# Discovery (recommended for production)
export SWEETMCP_DNS_SERVICE="_sweetmcp._tcp.example.com"
export SWEETMCP_DISCOVERY_TOKEN="your-shared-secret"
```

### DNS SRV Records
```dns
_sweetmcp._tcp.example.com. 300 IN SRV 10 50 8443 node1.example.com.
_sweetmcp._tcp.example.com. 300 IN SRV 10 50 8443 node2.example.com.
```

---

## 🔗 Protocol Examples

### GraphQL
```bash
curl -X POST https://sweetmcp.cyrup.dev:8443/graphql \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/graphql" \
  -d 'query { method(params: {}) }'
```

### JSON-RPC 2.0  
```bash
curl -X POST https://sweetmcp.cyrup.dev:8443/ \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
```

### Cap'n Proto
Send binary Cap'n Proto messages to any endpoint.

---

## 🚨 Troubleshooting

### Installation Issues
```bash
# Check requirements
curl --version && git --version && sudo --version

# Manual Rust install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Service Issues  
```bash
# Check daemon status
sudo systemctl status cyrupd  # Linux
sudo launchctl list | grep cyrupd  # macOS

# Regenerate certificates
sudo rm ~/.config/sweetmcp/wildcard.cyrup.pem
sudo cyrupd install
```

### DNS Issues
```bash
# Test host resolution
ping -c 1 sweetmcp.cyrup.dev
nslookup sweetmcp.cyrup.ai

# Check host entries
grep sweetmcp /etc/hosts
```

---

## 🗑️ Uninstallation

```bash
# Stop service
sudo systemctl stop cyrupd && sudo systemctl disable cyrupd  # Linux
sudo launchctl unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist  # macOS

# Remove daemon
sudo cyrupd uninstall

# Clean up files (optional)
rm -rf ~/.config/cyrupd ~/.config/sweetmcp
sudo sed -i '/# SweetMCP Auto-Integration/,+4d' /etc/hosts
```

---

## 📚 Documentation

- **[Installation Guide](./INSTALL.md)** - Detailed installation instructions
- **[API Documentation](https://docs.cyrup.ai/sweetmcp)** - Complete API reference
- **[Tool Integration](https://docs.cyrup.ai/sweetmcp/tools)** - AI tool setup guides
- **[Configuration](https://docs.cyrup.ai/sweetmcp/config)** - Advanced configuration

---

## 🤝 Community & Support

- **[GitHub Issues](https://github.com/cyrup-ai/sweetmcp/issues)** - Bug reports & feature requests
- **[Discord Community](https://discord.gg/cyrup-ai)** - Chat with the community  
- **[Documentation](https://docs.cyrup.ai/sweetmcp)** - Complete guides & tutorials

---

## 📄 License

Dual licensed under **MIT** OR **Apache-2.0** - choose whichever works best for your project.

---

<p align="center">
  <strong>Made with 🍯 by <a href="https://cyrup.ai">Cyrup.ai</a></strong>
</p>
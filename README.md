# SweetMCP 🍯

<p align="center">
  <img src="./docs/assets/sweetmcp.png" alt="SweetMCP" style="max-width: 100%; width: 600px;">
</p>

<p align="center">
  <strong>The ultimate Model Context Protocol integration platform</strong><br>
  <em>Zero-configuration AI agent infrastructure with enterprise-grade security</em>
</p>

---

## ⚡ One-Line Installation

```bash
curl -fsSL https://get.cyrup.ai/install | sh
```

<details>
<summary>🔍 <strong>Installation Details</strong></summary>

**Cross-Platform Support**: Auto-detects OS and architecture
- **macOS**: Intel + Apple Silicon (Darwin)
- **Linux**: x86_64 + aarch64 (systemd)  
- **Windows**: x86_64 + i686 (services)

**What it installs**:
1. Rust toolchain (if missing)
2. Clones `git@github.com:cyrup-ai/sweetmcp`
3. Builds native binaries with Cargo
4. Generates wildcard SSL certificates (`*.cyrup.{dev,ai,cloud,pro}`)
5. Configures OS trust stores and DNS resolution
6. Deploys systemd/launchd services
7. Starts daemon with proper privileges

**Result**: Production-ready MCP infrastructure in ~60 seconds
</details>

---

## 🏗️ Core Architecture

SweetMCP provides six foundational AI infrastructure components:

### 🗄️ SURREAL MEMORY
> *Context-augmentation memory system (vector & graph relationships)*

- **Backend**: [SurrealDB](https://surrealdb.com/) multi-model database
- **Vector Search**: Embedded similarity matching  
- **Graph Relations**: Knowledge graph with entity linking
- **Retention**: Global + project-local scoping
- **Integration**: Full [Obsidian](https://obsidian.md/) knowledge-base support
- **Visibility**: Transparent agent context inspection

```rust
// Example: Memory API
sweetmcp::memory::store_context("project_id", &embedding, &metadata).await?;
let related = sweetmcp::memory::semantic_search("query", 0.8, 10).await?;
```

### 🔐 CRYYPT
> *[Post-quantum](https://en.wikipedia.org/wiki/Post-quantum_cryptography) encryption vault and secret management*

- **Cryptography**: Military-grade, quantum-resistant algorithms
- **Zero-Knowledge**: Agents never access raw credentials
- **Architecture**: Pure Rust implementation with timing-safe operations
- **Access Control**: Time-based expiration and role-based permissions
- **Storage**: Encrypted vault with secure key derivation

```rust
// Example: Secret access without exposure
let result = cryypt::execute_with_secret("aws_key", |secret| {
    aws_client.authenticate(secret).call_api()
}).await?;
// Secret never leaves secure context
```

### 🎤 FLUENT VOICE
> *Pure-Rust AI voice system with predictive capabilities*

- **STT**: [faster-whisper](https://github.com/SYSTRAN/faster-whisper) speech-to-text
- **Wake Words**: Native wake word detection
- **Prediction**: Anticipatory text completion during speech
- **TTS**: Pure-Rust Candle port of [Dia Voice](https://diaai.org/)
- **Performance**: Low-latency, on-device processing
- **Quality**: Expressive voice synthesis with emotional range

```rust
// Example: Voice interaction
let transcription = fluent_voice::listen_with_prediction().await?;
let response = fluent_voice::synthesize(&text, Voice::Expressive).await?;
```

### ⚡ LOCAL AI
> *Autonomous ["ambient agents"](https://arxiv.org/abs/2407.01502) for background optimization*

- **Architecture**: Self-configuring agent swarm
- **Autonomy**: Zero direct user interaction required
- **Capabilities**: 
  - Prompt optimization and auto-tuning
  - Task decomposition and parallelization  
  - Documentation procurement and indexing
  - Preference learning and memorization
- **Integration**: Assists primary AI models transparently

```rust
// Example: Ambient agent coordination
ambient::spawn_optimizer_for(primary_model_id).await?;
ambient::background_task_decomposer::register(complex_task).await?;
```

### 🧠 CODE GENERATION
> *Advanced reasoning and autocoding capabilities*

- **Reasoning**: Deep reasoning chains with iterative refinement
- **Search**: [MCTS (Monte Carlo Tree Search)](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search) for solution exploration
- **Research**: Deep research agents with multi-source synthesis
- **Context**: Entire codebase understanding and analysis
- **Generation**: Specialized models for different programming domains

```rust
// Example: Code generation with reasoning
let solution = code_gen::reason_and_generate(CodeRequest {
    context: codebase_context,
    requirements: user_spec,
    search_strategy: SearchStrategy::MCTS,
    reasoning_depth: 5
}).await?;
```

### ∅ ZERO FRICTION
> *Universal tool integration without configuration*

**Supported Platforms**:
- [Claude Desktop](https://claude.ai/desktop) | [Claude Code](https://claude.ai/code)
- [Windsurf](https://codeium.com/windsurf) | [Cursor.AI](https://cursor.sh/)
- [VSCode](https://code.visualstudio.com/) | [Zed](https://zed.dev/)
- [Raycast](https://raycast.com/) | [Cline](https://github.com/cline/cline) | [Roo Code](https://github.com/RooVetGit/Roo-Cline)
- **+ thousands more via MCP standard**

**Operating Systems**: Linux, macOS, Windows (all architectures)
**Setup Time**: Literally zero - auto-discovery and configuration

```json
// Auto-generated MCP configuration
{
  "mcpServers": {
    "sweetmcp": {
      "command": "sweetmcp",
      "args": ["--stdio"],
      "env": {}
    }
  }
}
```

---

## 🔧 Technical Specifications

### Protocol Support
- **MCP (Model Context Protocol)** - Native first-class support
- **JSON-RPC 2.0** - Full specification compliance  
- **GraphQL** - Query-based tool invocation
- **Cap'n Proto** - High-performance binary protocol

### Performance Characteristics
- **Latency**: Sub-millisecond tool routing
- **Throughput**: 10K+ requests/second per core
- **Memory**: Minimal footprint with smart caching
- **Scaling**: Horizontal with service mesh discovery

### Security Model
- **Authentication**: JWT with configurable signing algorithms
- **Transport**: TLS 1.3 with optional mTLS
- **Rate Limiting**: Configurable per-endpoint throttling
- **Isolation**: Process-level separation for tool execution

### Service Endpoints
```
https://sweetmcp.cyrup.dev:8443   # Primary development endpoint
https://sweetmcp.cyrup.ai:8443    # AI-optimized routing
https://sweetmcp.cyrup.cloud:8443 # Cloud services integration  
https://sweetmcp.cyrup.pro:8443   # Professional tooling
```

---

## 🚀 Advanced Configuration

### Environment Variables
```bash
# Core Configuration
export SWEETMCP_JWT_SECRET="$(openssl rand -base64 32)"
export SWEETMCP_TCP_BIND="0.0.0.0:8443"
export SWEETMCP_METRICS_BIND="127.0.0.1:9090"

# Performance Tuning
export SWEETMCP_INFLIGHT_MAX=1000
export SWEETMCP_WORKER_THREADS=8
export SWEETMCP_CONNECTION_POOL_SIZE=100

# Discovery & Clustering
export SWEETMCP_DNS_SERVICE="_sweetmcp._tcp.example.com"
export SWEETMCP_DISCOVERY_TOKEN="cluster-shared-secret"
export SWEETMCP_NODE_ID="$(hostname)"
```

### Clustering with DNS SRV
```dns
_sweetmcp._tcp.example.com. 300 IN SRV 10 50 8443 node1.example.com.
_sweetmcp._tcp.example.com. 300 IN SRV 10 30 8443 node2.example.com.
_sweetmcp._tcp.example.com. 300 IN SRV 20 20 8443 node3.example.com.
```

### Service Management
```bash
# Linux (systemd)
sudo systemctl start|stop|restart cyrupd
sudo systemctl enable cyrupd  # Auto-start on boot
journalctl -u cyrupd -f       # Live logs

# macOS (launchd)  
sudo launchctl load|unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist
tail -f /var/log/cyrupd.log

# Windows (sc)
sc start|stop cyrupd
sc config cyrupd start=auto
```

---

## 🧪 API Examples

### MCP Tool Invocation
```bash
curl -X POST https://sweetmcp.cyrup.dev:8443/ \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call", 
    "params": {
      "name": "file_operations/read",
      "arguments": {"path": "/etc/hosts"}
    }
  }'
```

### GraphQL Query
```bash
curl -X POST https://sweetmcp.cyrup.dev:8443/graphql \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/graphql" \
  -d 'query {
    tools {
      list {
        name
        description
        inputSchema
      }
    }
  }'
```

### Memory Operations
```bash
# Store context
curl -X POST https://sweetmcp.cyrup.dev:8443/ \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "memory/store",
    "params": {
      "content": "User prefers TypeScript over JavaScript",
      "metadata": {"type": "preference", "user_id": "123"}
    }
  }'

# Semantic search
curl -X POST https://sweetmcp.cyrup.dev:8443/ \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "jsonrpc": "2.0", 
    "method": "memory/search",
    "params": {
      "query": "programming language preferences",
      "threshold": 0.8,
      "limit": 5
    }
  }'
```

---

## 🔬 Development & Debugging

### Build from Source
```bash
git clone git@github.com:cyrup-ai/sweetmcp.git
cd sweetmcp
cargo build --release
cargo test --all-features
```

### Debug Mode
```bash
RUST_LOG=debug,sweetmcp=trace cargo run -- daemon --debug
```

### Health Checks
```bash
# Service health
curl -f https://sweetmcp.cyrup.dev:8443/health

# Metrics (Prometheus format)
curl http://127.0.0.1:9090/metrics

# Tool registry status
curl https://sweetmcp.cyrup.dev:8443/tools/list
```

---

## 🗑️ Uninstallation

```bash
# Stop services
sudo systemctl stop cyrupd && sudo systemctl disable cyrupd  # Linux
sudo launchctl unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist  # macOS

# Remove daemon and certificates
sudo cyrupd uninstall

# Clean configuration (optional)
rm -rf ~/.config/{cyrupd,sweetmcp}
sudo sed -i '/# SweetMCP Auto-Integration/,+5d' /etc/hosts
```

---

## 📚 Resources

- **[API Documentation](https://docs.cyrup.ai/sweetmcp)** - Complete technical reference
- **[MCP Specification](https://spec.modelcontextprotocol.io/)** - Protocol standards
- **[GitHub Issues](https://github.com/cyrup-ai/sweetmcp/issues)** - Bug reports & features
- **[Discord Community](https://discord.gg/cyrup-ai)** - Developer chat

---

## 📄 License

Dual licensed under **MIT** OR **Apache-2.0** - choose what works for your use case.

---

<p align="center">
  <strong>Made with 🍯 by <a href="https://github.com/cyrup-ai">Cyrup.ai</a></strong>
</p>
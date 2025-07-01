# SweetMCP One-Line Installer

## Quick Install

```bash
curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash
```

## What It Does

The installer performs a complete soup-to-nuts installation:

1. **System Check** - Verifies requirements (git, curl, sudo)
2. **Rust Installation** - Installs Rust toolchain if missing
3. **Repository Clone** - Clones from `git@github.com:cyrup-ai/sweetmcp`
4. **Build** - Compiles SweetMCP in release mode
5. **Installation** - Runs daemon installer with sudo
6. **Verification** - Tests all components

## What Gets Installed

### Core Components
- **SweetMCP Daemon** (`cyrupd`) - System service manager
- **SweetMCP Pingora** - High-performance gateway server
- **Wildcard Certificate** - Self-signed cert for all *.cyrup.* domains
- **Host Entries** - Local DNS for sweetmcp.cyrup.{dev,ai,cloud,pro}
- **Trust Store Integration** - OS-level certificate trust

### File Locations

| Component | Location |
|-----------|----------|
| Daemon Binary | `/usr/local/bin/cyrupd` |
| Configuration | `~/.config/cyrupd/cyrupd.toml` |
| Certificate | `~/.config/sweetmcp/wildcard.cyrup.pem` |
| Service Definition | Platform-specific (systemd/launchd) |
| Host Entries | `/etc/hosts` |

### Network Endpoints

After installation, these domains resolve to `127.0.0.1:8443`:

- `https://sweetmcp.cyrup.dev:8443`
- `https://sweetmcp.cyrup.ai:8443` 
- `https://sweetmcp.cyrup.cloud:8443`
- `https://sweetmcp.cyrup.pro:8443`

## Platform Support

| Platform | Status | Package Manager |
|----------|--------|-----------------|
| macOS (Intel) | ✅ Full | Homebrew compatible |
| macOS (Apple Silicon) | ✅ Full | Homebrew compatible |
| Linux (x86_64) | ✅ Full | systemd |
| Linux (aarch64) | ✅ Full | systemd |
| Windows | 🚧 Planned | Services |

## Manual Installation

If you prefer to review the code first:

```bash
# Clone repository
git clone git@github.com:cyrup-ai/sweetmcp.git
cd sweetmcp

# Build project
cargo build --release --package sweetmcp-daemon

# Install with sudo
sudo ./target/release/sweetmcp-daemon install
```

## Service Management

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
# Start service (auto-loaded on install)
sudo launchctl load /Library/LaunchDaemons/com.cyrup.cyrupd.plist

# Check status
sudo launchctl list | grep cyrupd

# View logs
tail -f /var/log/cyrupd.log
```

## Configuration

Default configuration is created at `~/.config/cyrupd/cyrupd.toml`:

```toml
[services]
# Service definitions go here

[logging]
level = "info"

[network]
bind_address = "0.0.0.0:8443"
metrics_bind = "127.0.0.1:9090"
```

## Troubleshooting

### Installation Fails
```bash
# Check system requirements
curl --version
git --version
sudo --version

# Check Rust installation
rustc --version
cargo --version

# Manual Rust install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Service Won't Start
```bash
# Check daemon status
sudo systemctl status cyrupd  # Linux
sudo launchctl list | grep cyrupd  # macOS

# Check configuration
cat ~/.config/cyrupd/cyrupd.toml

# Check logs
journalctl -u cyrupd -n 50  # Linux
tail -50 /var/log/cyrupd.log  # macOS
```

### Certificate Issues
```bash
# Regenerate certificate
sudo rm ~/.config/sweetmcp/wildcard.cyrup.pem
sudo cyrupd install

# Test certificate
openssl x509 -in ~/.config/sweetmcp/wildcard.cyrup.pem -text -noout
```

### Host Resolution Issues
```bash
# Check host entries
grep sweetmcp /etc/hosts

# Test resolution
ping -c 1 sweetmcp.cyrup.dev
nslookup sweetmcp.cyrup.ai
```

## Uninstallation

```bash
# Stop and disable service
sudo systemctl stop cyrupd && sudo systemctl disable cyrupd  # Linux
sudo launchctl unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist  # macOS

# Remove daemon
sudo cyrupd uninstall

# Clean up files (optional)
rm -rf ~/.config/cyrupd
rm -rf ~/.config/sweetmcp
sudo sed -i '/# SweetMCP Auto-Integration/,+4d' /etc/hosts
```

## Security

- All operations requiring elevated privileges use `sudo`
- Private keys are stored with `0600` permissions
- Certificates are self-signed for local development only
- Network binding defaults to localhost for security

## Support

- **Issues**: https://github.com/cyrup-ai/sweetmcp/issues
- **Documentation**: https://docs.cyrup.ai/sweetmcp
- **Community**: https://discord.gg/cyrup-ai
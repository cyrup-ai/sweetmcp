# SweetMCP Installer & Daemon Architecture Guide

## Overview

SweetMCP uses a sophisticated 3-binary architecture with cross-platform installation support:

1. **`sweetmcp-daemon`** - Service manager daemon with clap subcommands
2. **`sweetmcp_server`** - Pingora proxy/load balancer (managed by daemon)  
3. **`sweet`** - Standalone MCP server (managed by daemon, runs on localhost:8080)

## Installation Flow

### 1. Initial Installer Script
- **Windows**: `install.ps1` (PowerShell)
- **macOS/Linux**: `install.sh` (Bash)

The installer performs:
1. Auto-installs dependencies (Git, Rust, build tools, OpenSSL)
2. Clones the repository
3. Builds the daemon binary (`cargo build --release --package sweetmcp-daemon`)
4. Executes `sweetmcp-daemon install` to perform system installation

### 2. Daemon Installation Process

The daemon's `install` command (`sweetmcp-daemon/src/installer.rs`) performs:

1. **Binary Installation**
   - Copies daemon to `/usr/local/bin/cyrupd` (macOS/Linux) or `C:\Program Files\Cyrupd\cyrupd.exe` (Windows)
   - Creates config directory at `~/.config/cyrupd/`
   - Generates default configuration

2. **Certificate Generation**
   - Creates wildcard SSL certificate for `*.cyrup.{dev,ai,cloud,pro}`
   - Self-signed with 100-year validity
   - Includes private key in combined PEM format
   - Stores at `~/.config/sweetmcp/wildcard.cyrup.pem`

3. **System Trust Store Import**
   - **macOS**: Uses `security add-trusted-cert` to add to System keychain
   - **Linux**: Copies to `/usr/local/share/ca-certificates/` and runs `update-ca-certificates`
   - **Windows**: Currently not implemented

4. **Host Entries**
   - Adds entries to `/etc/hosts` (Unix) or `C:\Windows\System32\drivers\etc\hosts` (Windows)
   - Maps `sweetmcp.cyrup.{dev,ai,cloud,pro}` to `127.0.0.1`

5. **Service Registration**
   - **macOS**: Creates launchd plist at `/Library/LaunchDaemons/`
   - **Linux**: Creates systemd unit at `/etc/systemd/system/`
   - **Windows**: Registers Windows service

6. **Additional Components**
   - Attempts to install fluent-voice components at `/opt/sweetmcp/fluent-voice`
   - Creates service definition files

## Platform-Specific Implementation

### macOS (`install/macos.rs`)
- Uses osascript for GUI privilege escalation
- Extracts embedded helper app from ZIP data
- Verifies code signature on helper
- Creates launchd plist with comprehensive configuration
- Runs as root with wheel group

### Linux (`install/linux.rs`)  
- Creates systemd unit with advanced features
- Supports both system and user services
- Configures journal integration
- Sets up drop-in directories for overrides
- Handles capabilities and security contexts

### Windows (`install/windows.rs`)
- Uses Service Control Manager API
- Configures failure actions and recovery
- Sets up Windows Event Log integration
- Handles UAC elevation requirements

## Common Installer Errors & Solutions

### 1. Permission/Privilege Errors

**Error**: "Permission denied. Please provide administrator credentials."
- **Cause**: Installer requires admin/root privileges
- **Solution**: 
  - Windows: Run PowerShell as Administrator
  - macOS/Linux: Script will prompt for sudo password
  - If GUI prompt fails on macOS, try running from Terminal

### 2. Build Failures

**Error**: "Build failed"
- **Common Causes**:
  - Missing Rust toolchain
  - Insufficient disk space (need 2GB+)
  - Missing build dependencies
- **Solutions**:
  - Ensure Rust is installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  - Free up disk space
  - Install build tools:
    - macOS: `xcode-select --install`
    - Linux: `sudo apt install build-essential` (or equivalent)
    - Windows: Visual Studio Build Tools

### 3. Certificate Errors

**Error**: "Failed to generate wildcard certificate and import"
- **Causes**:
  - Existing invalid certificate
  - Trust store access issues
  - Missing OpenSSL
- **Solutions**:
  - Remove existing cert: `rm ~/.config/sweetmcp/wildcard.cyrup.pem`
  - Ensure OpenSSL is installed
  - Check keychain/trust store permissions

### 4. Host File Errors

**Error**: "Failed to add SweetMCP host entries"
- **Cause**: Cannot write to hosts file
- **Solutions**:
  - Ensure admin privileges
  - Check if hosts file is read-only
  - Manually add entries if needed:
    ```
    127.0.0.1 sweetmcp.cyrup.dev
    127.0.0.1 sweetmcp.cyrup.ai
    127.0.0.1 sweetmcp.cyrup.cloud
    127.0.0.1 sweetmcp.cyrup.pro
    ```

### 5. Service Registration Failures

**Error**: "Failed to create service"
- **Common Causes**:
  - Service already exists
  - Invalid service configuration
  - System service manager issues
- **Solutions**:
  - Check if service exists: 
    - macOS: `sudo launchctl list | grep cyrupd`
    - Linux: `sudo systemctl status cyrupd`
    - Windows: `sc query cyrupd`
  - Uninstall first: `sudo cyrupd uninstall`
  - Check system logs for details

### 6. Port Conflicts

**Error**: Service fails to start - port 8443 already in use
- **Diagnosis**: `sudo lsof -i :8443` or `netstat -an | grep 8443`
- **Solution**: 
  - Stop conflicting service
  - Or configure different port in `~/.config/cyrupd/config.toml`

### 7. Missing Dependencies

**Error**: Various dependency-related failures
- **Auto-installed dependencies**:
  - Git
  - Rust toolchain
  - C/C++ compilers
  - OpenSSL development libraries
- **Manual installation if auto-install fails**:
  - macOS: Use Homebrew
  - Linux: Use system package manager
  - Windows: Use winget or manual downloads

## Architecture Insights

### Critical Understanding
- **Pingora is a PROXY**, not the MCP server
- Pingora forwards all MCP requests to Axum server on `localhost:8080`
- The Axum server (`sweet` binary) is the actual MCP implementation
- Both binaries must be running for the system to work

### Service Management

**Start/Stop/Status Commands:**
```bash
# Linux
sudo systemctl start cyrupd
sudo systemctl stop cyrupd
sudo systemctl status cyrupd
journalctl -u cyrupd -f  # View logs

# macOS  
sudo launchctl load /Library/LaunchDaemons/com.cyrup.cyrupd.plist
sudo launchctl unload /Library/LaunchDaemons/com.cyrup.cyrupd.plist
tail -f /var/log/cyrupd.log

# Windows
sc start cyrupd
sc stop cyrupd
sc query cyrupd
```

## Troubleshooting Checklist

1. **Verify installation completed:**
   - Check daemon binary exists: `/usr/local/bin/cyrupd`
   - Check config exists: `~/.config/cyrupd/config.toml`
   - Check certificate exists: `~/.config/sweetmcp/wildcard.cyrup.pem`

2. **Verify services are running:**
   - Daemon service is active
   - Can reach `https://sweetmcp.cyrup.dev:8443`

3. **Check logs for errors:**
   - Service logs (see commands above)
   - Installation output

4. **Common recovery steps:**
   - Full uninstall and reinstall
   - Manual service restart
   - Certificate regeneration
   - Host file verification

## Binary Signing (Optional)

The installer supports code signing:
- macOS: Developer ID certificates
- Windows: Authenticode certificates  
- Linux: GPG signatures

Use `--sign` flag during installation to enable signing verification.
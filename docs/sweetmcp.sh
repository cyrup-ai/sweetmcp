#!/bin/bash
set -euo pipefail

# SweetMCP One-Line Installer
# Usage: curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash

# Color output functions
red() { echo -e "\033[0;31m$1\033[0m"; }
green() { echo -e "\033[0;32m$1\033[0m"; }
yellow() { echo -e "\033[0;33m$1\033[0m"; }
blue() { echo -e "\033[0;34m$1\033[0m"; }

# Logging functions
info() { blue "[INFO] $1"; }
warn() { yellow "[WARN] $1"; }
error() { red "[ERROR] $1"; }
success() { green "[SUCCESS] $1"; }

# Cleanup function
cleanup() {
    if [[ -n "${TEMP_DIR:-}" && -d "$TEMP_DIR" ]]; then
        info "Cleaning up temporary directory: $TEMP_DIR"
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# Platform detection
detect_platform() {
    local os arch
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" && exit 1 ;;
    esac
    
    case "$os" in
        linux) PLATFORM="$arch-unknown-linux-gnu" ;;
        darwin) PLATFORM="$arch-apple-darwin" ;;
        *) error "Unsupported operating system: $os" && exit 1 ;;
    esac
    
    info "Detected platform: $PLATFORM"
}

# Check system requirements
check_requirements() {
    info "Checking system requirements..."
    
    # Check for required commands
    local required_commands=("git" "curl" "sudo")
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            error "Required command not found: $cmd"
            case "$cmd" in
                git) info "Install git: https://git-scm.com/downloads" ;;
                curl) info "Install curl: https://curl.se/download.html" ;;
                sudo) error "sudo is required for system installation" ;;
            esac
            exit 1
        fi
    done
    
    # Check for Rust toolchain
    if ! command -v rustc >/dev/null 2>&1; then
        warn "Rust toolchain not found. Installing..."
        install_rust
    else
        info "Rust toolchain found: $(rustc --version)"
    fi
    
    success "System requirements satisfied"
}

# Install Rust toolchain
install_rust() {
    info "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    
    # Source the cargo environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    
    # Verify installation
    if command -v rustc >/dev/null 2>&1; then
        success "Rust installed: $(rustc --version)"
    else
        error "Failed to install Rust toolchain"
        exit 1
    fi
}

# Clone the repository
clone_repository() {
    info "Cloning SweetMCP repository..."
    
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Try SSH first, fallback to HTTPS
    if git clone git@github.com:cyrup-ai/sweetmcp.git 2>/dev/null; then
        info "Cloned via SSH"
    elif git clone https://github.com/cyrup-ai/sweetmcp.git 2>/dev/null; then
        info "Cloned via HTTPS"
    else
        error "Failed to clone repository"
        exit 1
    fi
    
    cd sweetmcp
    success "Repository cloned successfully"
}

# Build the project
build_project() {
    info "Building SweetMCP..."
    
    # Build in release mode for performance
    if cargo build --release --package sweetmcp-daemon; then
        success "Build completed successfully"
    else
        error "Build failed"
        exit 1
    fi
}

# Install the daemon
install_daemon() {
    info "Installing SweetMCP daemon..."
    
    # Run the daemon installer with sudo
    if sudo ./target/release/sweetmcp-daemon install; then
        success "SweetMCP daemon installed successfully"
    else
        error "Daemon installation failed"
        exit 1
    fi
}

# Verify installation
verify_installation() {
    info "Verifying installation..."
    
    # Check if daemon is installed
    if command -v cyrupd >/dev/null 2>&1; then
        success "Daemon binary installed: $(which cyrupd)"
    else
        warn "Daemon binary not found in PATH"
    fi
    
    # Check certificate
    local cert_path
    if [[ -n "${XDG_CONFIG_HOME:-}" ]]; then
        cert_path="$XDG_CONFIG_HOME/sweetmcp/wildcard.cyrup.pem"
    else
        cert_path="$HOME/.config/sweetmcp/wildcard.cyrup.pem"
    fi
    
    if [[ -f "$cert_path" ]]; then
        success "Wildcard certificate installed: $cert_path"
    else
        warn "Wildcard certificate not found at: $cert_path"
    fi
    
    # Test host entries
    local test_domains=("sweetmcp.cyrup.dev" "sweetmcp.cyrup.ai" "sweetmcp.cyrup.cloud" "sweetmcp.cyrup.pro")
    local hosts_working=true
    
    for domain in "${test_domains[@]}"; do
        if ping -c 1 -W 1 "$domain" >/dev/null 2>&1; then
            success "Host entry working: $domain"
        else
            warn "Host entry not working: $domain"
            hosts_working=false
        fi
    done
    
    if $hosts_working; then
        success "All host entries are working"
    else
        warn "Some host entries may need manual verification"
    fi
}

# Main installation function
main() {
    info "Starting SweetMCP installation..."
    info "============================================"
    
    detect_platform
    check_requirements
    clone_repository
    build_project
    install_daemon
    verify_installation
    
    info "============================================"
    success "SweetMCP installation completed!"
    info ""
    info "Next steps:"
    info "  1. Start the daemon: sudo systemctl start cyrupd (Linux) or sudo launchctl load /Library/LaunchDaemons/com.cyrup.cyrupd.plist (macOS)"
    info "  2. Enable auto-start: sudo systemctl enable cyrupd (Linux) or it's already enabled (macOS)"
    info "  3. Check status: sudo systemctl status cyrupd (Linux) or sudo launchctl list | grep cyrupd (macOS)"
    info "  4. View logs: journalctl -u cyrupd -f (Linux) or tail -f /var/log/cyrupd.log (macOS)"
    info ""
    info "Configuration:"
    info "  - Config file: ~/.config/cyrupd/cyrupd.toml"
    info "  - Certificate: ~/.config/sweetmcp/wildcard.cyrup.pem"
    info "  - Host entries: /etc/hosts"
    info ""
    info "Domains available:"
    info "  - https://sweetmcp.cyrup.dev:8443"
    info "  - https://sweetmcp.cyrup.ai:8443"
    info "  - https://sweetmcp.cyrup.cloud:8443"
    info "  - https://sweetmcp.cyrup.pro:8443"
    info ""
    success "Welcome to SweetMCP! 🍯"
}

# Run main function
main "$@"
#!/bin/bash
set -euo pipefail

# SweetMCP Ultimate Installer - The One True Installer! 🍯
# Usage: curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash
# OR run directly from the project directory: ./docs/sweetmcp.sh

# Color output functions (merged from docs version - superior logging)
red() { echo -e "\033[0;31m$1\033[0m"; }
green() { echo -e "\033[0;32m$1\033[0m"; }
yellow() { echo -e "\033[0;33m$1\033[0m"; }
blue() { echo -e "\033[0;34m$1\033[0m"; }

# Enhanced logging functions (from docs version)
info() { blue "[INFO] $1"; }
warn() { yellow "[WARN] $1"; }
error() { red "[ERROR] $1"; }
success() { green "[SUCCESS] $1"; }

# Get sudo access upfront (from install.sh - superior sudo management)
info "🍯 SweetMCP Ultimate Installer - We Do It All!"
info "This installer needs administrator privileges to:"
echo "  • Install system dependencies"
echo "  • Install the SweetMCP daemon"
echo "  • Configure certificates and services"
echo ""

# Get sudo privileges and keep them alive (from install.sh)
sudo -v
# Keep sudo alive in background
while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &
SUDO_PID=$!

# Enhanced cleanup function - ONLY cleans $XDG_CONFIG_HOME/sweetmcp/*
cleanup() {
    # Kill the sudo keepalive process
    if [[ -n "${SUDO_PID:-}" ]]; then
        kill $SUDO_PID 2>/dev/null || true
    fi
    # Clean up any build artifacts in sweetmcp directory only
    if [[ -n "${PROJECT_DIR:-}" && -d "$PROJECT_DIR" ]]; then
        info "Cleaning up build artifacts..."
        cd "$PROJECT_DIR"
        # Remove cargo build artifacts to save space
        if [[ -d "target" ]]; then
            rm -rf target/debug 2>/dev/null || true
            rm -rf target/*/deps 2>/dev/null || true
            rm -rf target/*/incremental 2>/dev/null || true
        fi
    fi
}
trap cleanup EXIT

# Get config directory - NEVER use temp dirs!
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
SWEETMCP_HOME="$CONFIG_HOME/sweetmcp"

# Enhanced OS detection (from install.sh - more comprehensive)
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$ID
        OS_LIKE="${ID_LIKE:-}"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        OS_LIKE=""
    else
        OS="unknown"
        OS_LIKE=""
    fi
    info "Detected OS: $OS"
}

# Platform detection (from docs version - more sophisticated)
detect_platform() {
    local arch
    arch=$(uname -m)
    
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" && exit 1 ;;
    esac
    
    case "$OS" in
        linux) PLATFORM="$arch-unknown-linux-gnu" ;;
        darwin|macos) PLATFORM="$arch-apple-darwin" ;;
        *) error "Unsupported operating system: $OS" && exit 1 ;;
    esac
    
    info "Detected platform: $PLATFORM"
}

# Enhanced dependency installation (from install.sh - comprehensive OS support)
install_deps() {
    info "Installing dependencies..."
    
    if [[ "$OS" == "ubuntu" ]] || [[ "$OS" == "debian" ]] || [[ "$OS_LIKE" == *"debian"* ]]; then
        # Update package list
        sudo apt-get update -qq
        # Install all build dependencies
        sudo apt-get install -y git curl build-essential pkg-config libssl-dev
    elif [[ "$OS" == "fedora" ]] || [[ "$OS" == "rhel" ]] || [[ "$OS" == "centos" ]]; then
        sudo dnf install -y git curl gcc gcc-c++ make pkgconfig openssl-devel
    elif [[ "$OS" == "arch" ]] || [[ "$OS" == "manjaro" ]]; then
        sudo pacman -S --needed --noconfirm git curl base-devel openssl
    elif [[ "$OS" == "opensuse"* ]]; then
        sudo zypper install -y git curl gcc gcc-c++ make pkg-config libopenssl-devel
    elif [[ "$OS" == "alpine" ]]; then
        sudo apk add --no-cache git curl build-base pkgconfig openssl-dev
    elif [[ "$OS" == "macos" ]]; then
        # Install Xcode Command Line Tools if needed
        if ! xcode-select -p >/dev/null 2>&1; then
            warn "Installing Xcode Command Line Tools..."
            xcode-select --install 2>/dev/null || true
            # Wait for installation
            until xcode-select -p >/dev/null 2>&1; do
                sleep 5
            done
        fi
        # Install Homebrew if needed
        if ! command -v brew >/dev/null 2>&1; then
            warn "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            if [[ -f /opt/homebrew/bin/brew ]]; then
                eval "$(/opt/homebrew/bin/brew shellenv)"
            fi
        fi
        # Install dependencies via Homebrew
        brew install git curl openssl pkg-config
    else
        warn "Unknown OS, attempting generic install..."
        # Try to find and use available package manager
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update -qq && sudo apt-get install -y git curl build-essential pkg-config libssl-dev
        elif command -v yum >/dev/null 2>&1; then
            sudo yum install -y git curl gcc gcc-c++ make pkgconfig openssl-devel
        elif command -v apk >/dev/null 2>&1; then
            sudo apk add --no-cache git curl build-base pkgconfig openssl-dev
        else
            error "Could not detect package manager. Please install: git, curl, gcc, make, pkg-config, openssl-dev"
            exit 1
        fi
    fi
    
    success "Dependencies installed!"
}

# Enhanced Rust installation (merged best practices)
install_rust() {
    if ! command -v rustc >/dev/null 2>&1; then
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
    else
        info "Rust toolchain found: $(rustc --version)"
    fi
}

# CRITICAL FIX: Smart repository management with proper updates - ONLY in $XDG_CONFIG_HOME!
manage_repository() {
    info "Managing SweetMCP repository..."
    
    # Check if we're running from inside the project directory
    if [[ -f "Cargo.toml" ]] && grep -q "sweetmcp" "Cargo.toml" 2>/dev/null; then
        info "Running from project directory..."
        PROJECT_DIR="$(pwd)"
        
        # If we're in a git repo, update it
        if [[ -d ".git" ]]; then
            info "Updating existing git repository..."
            if git fetch origin && git pull origin main; then
                success "Repository updated successfully"
            else
                warn "Git update failed, continuing with current version"
            fi
        fi
    else
        # We're running from curl - ONLY use $XDG_CONFIG_HOME/sweetmcp!
        info "Setting up SweetMCP home directory..."
        mkdir -p "$SWEETMCP_HOME"
        cd "$SWEETMCP_HOME"
        
        # FIXED: Smart handling of existing installations - NO temp dirs!
        if [[ -d "sweetmcp" ]]; then
            cd sweetmcp
            if [[ -d ".git" ]]; then
                info "Updating existing SweetMCP installation..."
                if git fetch origin && git pull origin main; then
                    success "SweetMCP updated to latest version"
                else
                    warn "Git update failed, reinstalling..."
                    cd ..
                    rm -rf sweetmcp
                    if ! git clone https://github.com/cyrup-ai/sweetmcp.git; then
                        error "Failed to clone repository"
                        exit 1
                    fi
                    cd sweetmcp
                fi
            else
                warn "Existing directory is not a git repository, reinstalling..."
                cd ..
                rm -rf sweetmcp
                if ! git clone https://github.com/cyrup-ai/sweetmcp.git; then
                    error "Failed to clone repository"
                    exit 1
                fi
                cd sweetmcp
            fi
        else
            info "Cloning SweetMCP repository..."
            if ! git clone https://github.com/cyrup-ai/sweetmcp.git; then
                error "Failed to clone repository"
                exit 1
            fi
            cd sweetmcp
        fi
        
        PROJECT_DIR="$(pwd)"
        success "Repository ready at: $PROJECT_DIR"
    fi
}

# Enhanced build process (merged best practices)
build_project() {
    info "Building SweetMCP (this may take a few minutes)..."
    
    # Build in release mode for performance
    if cargo build --release --package sweetmcp-daemon; then
        success "Build completed successfully"
    else
        error "Build failed"
        exit 1
    fi
}

# Enhanced daemon installation (from install.sh)
install_daemon() {
    info "Installing SweetMCP daemon..."
    
    # Install with the sudo privileges we already have
    if sudo ./target/release/sweetmcp-daemon install; then
        success "SweetMCP daemon installed successfully"
    else
        error "Installation failed"
        exit 1
    fi
}

# Enhanced verification (from docs version - comprehensive checks)
verify_installation() {
    info "Verifying installation..."
    
    # Check if daemon is installed
    if command -v cyrupd >/dev/null 2>&1; then
        success "Daemon binary installed: $(which cyrupd)"
    else
        warn "Daemon binary not found in PATH"
    fi
    
    # Check certificate
    local cert_path="$CONFIG_HOME/sweetmcp/wildcard.cyrup.pem"
    if [[ -f "$cert_path" ]]; then
        success "Wildcard certificate installed: $cert_path"
    else
        warn "Wildcard certificate not found at: $cert_path"
    fi
    
    # Test host entries with timeout
    local test_domains=("sweetmcp.cyrup.dev" "sweetmcp.cyrup.ai" "sweetmcp.cyrup.cloud" "sweetmcp.cyrup.pro")
    local hosts_working=true
    
    for domain in "${test_domains[@]}"; do
        if timeout 2 ping -c 1 "$domain" >/dev/null 2>&1; then
            success "Host entry working: $domain"
        else
            warn "Host entry may need verification: $domain"
            hosts_working=false
        fi
    done
    
    if $hosts_working; then
        success "All host entries are working"
    else
        warn "Some host entries may need manual verification"
    fi
}

# Main installation process
main() {
    info "Starting SweetMCP installation..."
    info "=========================================="
    
    detect_os
    detect_platform
    install_deps
    install_rust
    manage_repository
    build_project
    install_daemon
    verify_installation
    
    info "=========================================="
    success "SweetMCP installation completed! 🍯"
    info ""
    info "Project location: $PROJECT_DIR"
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
    info "Available at:"
    info "  • https://sweetmcp.cyrup.dev:8443"
    info "  • https://sweetmcp.cyrup.ai:8443"
    info "  • https://sweetmcp.cyrup.cloud:8443"
    info "  • https://sweetmcp.cyrup.pro:8443"
    info ""
    success "Go have fun! 🚀"
}

# Run main function
main "$@"